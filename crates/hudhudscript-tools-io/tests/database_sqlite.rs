#![cfg(feature = "db")]

use hudhudscript_tools_io::database::{
    register_database_tools, DatabaseConfig, DatabaseService, ExecuteOptions, Migration,
    TransactionOptions,
};
use hudhudscript_tools_schema::ToolRegistry;
use serde_json::json;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sqlite_pool_transactions_migrations_and_metadata_work() {
    let mut config = DatabaseConfig::sqlite(":memory:");
    config.max_rows = 2;
    let connection = DatabaseService.open(config).await.expect("open SQLite");

    DatabaseService
        .execute(
            &connection.handle,
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, payload BLOB)",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("create users");
    for (name, bytes) in [("Ada", "AQID"), ("Linus", "BAUG"), ("Grace", "BwgJ")] {
        DatabaseService
            .execute(
                &connection.handle,
                "INSERT INTO users (name, payload) VALUES (?, ?)",
                &[json!(name), json!({"$type": "bytes", "value": bytes})],
                ExecuteOptions::default(),
            )
            .await
            .expect("insert user");
    }

    let result = DatabaseService
        .query(
            &connection.handle,
            "SELECT id, name, payload FROM users ORDER BY id",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("query users");
    assert_eq!(result.rows.len(), 2);
    assert!(result.truncated);
    assert_eq!(result.rows[0]["name"], json!("Ada"));
    assert_eq!(result.rows[0]["payload"]["$type"], json!("bytes"));

    let transaction = DatabaseService
        .begin(
            &connection.handle,
            TransactionOptions {
                isolation: Some("immediate".into()),
                read_only: false,
                timeout_ms: None,
            },
        )
        .await
        .expect("begin transaction");
    DatabaseService
        .transaction_execute(
            &transaction.transaction,
            "INSERT INTO users (name) VALUES (?)",
            &[json!("Rolled Back")],
            ExecuteOptions::default(),
        )
        .await
        .expect("transaction insert");
    DatabaseService
        .rollback(&transaction.transaction)
        .await
        .expect("rollback");
    let count = DatabaseService
        .query(
            &connection.handle,
            "SELECT COUNT(*) AS count FROM users",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("count users");
    assert_eq!(count.rows[0]["count"], json!(3));

    let report = DatabaseService.migrate(&connection.handle, vec![Migration {
        version: 1,
        name: "create audit log".into(),
        sql: "CREATE TABLE audit_log (id INTEGER PRIMARY KEY, message TEXT); INSERT INTO audit_log (message) VALUES ('ready');".into(),
    }]).await.expect("apply migration");
    assert_eq!(report.applied, vec![1]);
    let skipped = DatabaseService.migrate(&connection.handle, vec![Migration {
        version: 1,
        name: "create audit log".into(),
        sql: "CREATE TABLE audit_log (id INTEGER PRIMARY KEY, message TEXT); INSERT INTO audit_log (message) VALUES ('ready');".into(),
    }]).await.expect("skip migration");
    assert_eq!(skipped.skipped, vec![1]);

    let tables = DatabaseService
        .list_tables(&connection.handle, None)
        .await
        .expect("list tables");
    assert!(tables.contains(&"users".to_string()));
    assert!(tables.contains(&"audit_log".to_string()));
    let columns = DatabaseService
        .describe_table(&connection.handle, "users", None)
        .await
        .expect("describe users");
    assert_eq!(columns.rows[0]["name"], json!("id"));

    let status = DatabaseService
        .health(&connection.handle)
        .await
        .expect("health");
    assert!(!status.closed);
    DatabaseService
        .close(&connection.handle)
        .await
        .expect("close pool");
    assert!(DatabaseService.status(&connection.handle).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn configured_database_agent_tools_are_executable() {
    let registry = ToolRegistry::new();
    register_database_tools(&registry, DatabaseConfig::sqlite(":memory:"))
        .expect("register database tools");
    registry
        .call_tool(
            "db_execute",
            json!({
                "sql": "CREATE TABLE notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL)"
            }),
        )
        .await
        .expect("create table through native tool");
    registry
        .call_tool(
            "db_execute",
            json!({
                "sql": "INSERT INTO notes (body) VALUES (?)", "params": ["hello"]
            }),
        )
        .await
        .expect("insert through native tool");
    let result = registry
        .call_tool(
            "db_query",
            json!({
                "sql": "SELECT body FROM notes"
            }),
        )
        .await
        .expect("query through native tool");
    assert_eq!(result["rows"][0]["body"], json!("hello"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn expired_transaction_is_rolled_back_and_releases_connection() {
    let mut config = DatabaseConfig::sqlite(":memory:");
    config.max_connections = 1;
    config.acquire_timeout_ms = 500;
    config.transaction_timeout_ms = 20;
    let connection = DatabaseService.open(config).await.expect("open SQLite");
    DatabaseService
        .execute(
            &connection.handle,
            "CREATE TABLE pending (id INTEGER PRIMARY KEY)",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("create pending table");
    let transaction = DatabaseService
        .begin(&connection.handle, TransactionOptions::default())
        .await
        .expect("begin expiring transaction");
    DatabaseService
        .transaction_execute(
            &transaction.transaction,
            "INSERT INTO pending (id) VALUES (1)",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("insert pending row");

    tokio::time::sleep(Duration::from_millis(80)).await;
    let result = DatabaseService
        .query(
            &connection.handle,
            "SELECT COUNT(*) AS count FROM pending",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("pool connection was released");
    assert_eq!(result.rows[0]["count"], json!(0));
    assert!(DatabaseService
        .transaction_query(
            &transaction.transaction,
            "SELECT 1",
            &[],
            ExecuteOptions::default(),
        )
        .await
        .is_err());
}
