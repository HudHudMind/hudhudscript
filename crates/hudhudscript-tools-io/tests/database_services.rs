#![cfg(feature = "db")]

mod database_test_support;

use database_test_support::{
    live_database_config, live_test_guard, open_live_database, LiveBackend,
};
use hudhudscript_tools_io::database::{
    DatabaseConfig, DatabaseService, ExecuteOptions, TransactionOptions,
};
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the PostgreSQL URL in the Git-ignored HudHud secrets config"]
async fn postgres_contract() {
    let _guard = live_test_guard().await;
    contract(live_database_config(LiveBackend::Postgres), "$1", "$2").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires the MySQL URL in the Git-ignored HudHud secrets config"]
async fn mysql_contract() {
    let _guard = live_test_guard().await;
    contract(live_database_config(LiveBackend::Mysql), "?", "?").await;
}

async fn contract(config: DatabaseConfig, first: &str, second: &str) {
    let connection = open_live_database(config)
        .await
        .expect("open service database");
    let table = format!("hudhud_contract_{}", uuid::Uuid::new_v4().simple());
    DatabaseService
        .execute(
            &connection.handle,
            &format!("CREATE TABLE {table} (id BIGINT PRIMARY KEY, name VARCHAR(100) NOT NULL)"),
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("create contract table");
    let insert = format!("INSERT INTO {table} (id, name) VALUES ({first}, {second})");
    DatabaseService
        .execute(
            &connection.handle,
            &insert,
            &[json!(1), json!("Ada")],
            ExecuteOptions::default(),
        )
        .await
        .expect("insert contract row");
    let result = DatabaseService
        .query(
            &connection.handle,
            &format!("SELECT id, name FROM {table}"),
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("query contract row");
    assert_eq!(result.rows[0]["name"], json!("Ada"));

    let transaction = DatabaseService
        .begin(&connection.handle, TransactionOptions::default())
        .await
        .expect("begin contract transaction");
    DatabaseService
        .transaction_execute(
            &transaction.transaction,
            &insert,
            &[json!(2), json!("rollback")],
            ExecuteOptions::default(),
        )
        .await
        .expect("transaction insert");
    DatabaseService
        .rollback(&transaction.transaction)
        .await
        .expect("rollback contract transaction");
    let count = DatabaseService
        .query(
            &connection.handle,
            &format!("SELECT COUNT(*) AS total FROM {table}"),
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("count contract rows");
    assert_eq!(count.rows[0]["total"], json!(1));

    DatabaseService
        .execute(
            &connection.handle,
            &format!("DROP TABLE {table}"),
            &[],
            ExecuteOptions::default(),
        )
        .await
        .expect("drop contract table");
    DatabaseService
        .close(&connection.handle)
        .await
        .expect("close service database");
}
