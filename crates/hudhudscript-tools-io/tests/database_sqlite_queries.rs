#![cfg(feature = "db")]

use hudhudscript_tools_io::database::{
    DatabaseConfig, DatabaseService, ExecuteOptions, TransactionOptions,
};
use serde_json::{json, Value};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sqlite_crud_joins_filters_pagination_and_constraints() {
    let connection = DatabaseService
        .open(DatabaseConfig::sqlite(":memory:"))
        .await
        .expect("open SQLite query contract");
    let handle = &connection.handle;
    execute(
        handle,
        "CREATE TABLE authors (id INTEGER PRIMARY KEY, email TEXT NOT NULL UNIQUE, name TEXT NOT NULL)",
        &[],
    )
    .await;
    execute(
        handle,
        "CREATE TABLE books (id INTEGER PRIMARY KEY, author_id INTEGER NOT NULL, title TEXT NOT NULL, FOREIGN KEY (author_id) REFERENCES authors(id))",
        &[],
    )
    .await;
    for (id, email, name) in [
        (1, "ada@example.test", "Ada"),
        (2, "linus@example.test", "Linus"),
        (3, "grace@example.test", "Grace"),
    ] {
        execute(
            handle,
            "INSERT INTO authors (id, email, name) VALUES (?, ?, ?)",
            &[json!(id), json!(email), json!(name)],
        )
        .await;
    }
    for (id, author, title) in [(10, 1, "Compiler"), (11, 1, "Runtime"), (12, 2, "Kernel")] {
        execute(
            handle,
            "INSERT INTO books (id, author_id, title) VALUES (?, ?, ?)",
            &[json!(id), json!(author), json!(title)],
        )
        .await;
    }

    let selected = query(
        handle,
        "SELECT name FROM authors WHERE email = ?",
        &[json!("ada@example.test")],
    )
    .await;
    assert_eq!(selected.rows[0]["name"], json!("Ada"));

    let updated = execute(
        handle,
        "UPDATE authors SET name = ? WHERE id = ?",
        &[json!("Linus Torvalds"), json!(2)],
    )
    .await;
    assert_eq!(updated.rows_affected, 1);

    let joined = query(
        handle,
        "SELECT a.name AS author_name, b.title FROM authors a INNER JOIN books b ON b.author_id = a.id WHERE a.id = ? ORDER BY b.id",
        &[json!(1)],
    )
    .await;
    assert_eq!(joined.rows.len(), 2);
    assert_eq!(joined.rows[1]["title"], json!("Runtime"));

    let left = query(
        handle,
        "SELECT a.name, b.title FROM authors a LEFT JOIN books b ON b.author_id = a.id WHERE a.id = ?",
        &[json!(3)],
    )
    .await;
    assert_eq!(left.rows.len(), 1);
    assert_eq!(left.rows[0]["title"], Value::Null);

    let page = query(
        handle,
        "SELECT name FROM authors ORDER BY id LIMIT ? OFFSET ?",
        &[json!(1), json!(1)],
    )
    .await;
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0]["name"], json!("Linus Torvalds"));

    let duplicate = DatabaseService
        .execute(
            handle,
            "INSERT INTO authors (id, email, name) VALUES (?, ?, ?)",
            &[json!(20), json!("ada@example.test"), json!("Duplicate")],
            ExecuteOptions::default(),
        )
        .await;
    assert!(duplicate.is_err());
    let orphan = DatabaseService
        .execute(
            handle,
            "INSERT INTO books (id, author_id, title) VALUES (?, ?, ?)",
            &[json!(21), json!(999), json!("Orphan")],
            ExecuteOptions::default(),
        )
        .await;
    assert!(orphan.is_err());

    let deleted = execute(handle, "DELETE FROM books WHERE id = ?", &[json!(12)]).await;
    assert_eq!(deleted.rows_affected, 1);
    let absent = query(handle, "SELECT id FROM books WHERE id = ?", &[json!(12)]).await;
    assert!(absent.rows.is_empty());

    DatabaseService
        .close(handle)
        .await
        .expect("close SQLite query contract");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sqlite_commit_rollback_health_close_and_reconnect_work() {
    let config = DatabaseConfig::sqlite(":memory:");
    let reconnect_config = config.clone();
    let connection = DatabaseService
        .open(config)
        .await
        .expect("open SQLite transaction contract");
    let handle = &connection.handle;
    let status = DatabaseService.health(handle).await.expect("SQLite health");
    assert!(!status.closed);
    execute(
        handle,
        "CREATE TABLE events (id INTEGER PRIMARY KEY, label TEXT NOT NULL)",
        &[],
    )
    .await;

    let committed = DatabaseService
        .begin(handle, TransactionOptions::default())
        .await
        .expect("begin committed SQLite transaction");
    DatabaseService
        .transaction_execute(
            &committed.transaction,
            "INSERT INTO events (id, label) VALUES (?, ?)",
            &[json!(1), json!("committed")],
            ExecuteOptions::default(),
        )
        .await
        .expect("insert committed SQLite row");
    DatabaseService
        .commit(&committed.transaction)
        .await
        .expect("commit SQLite transaction");

    let rolled_back = DatabaseService
        .begin(handle, TransactionOptions::default())
        .await
        .expect("begin rolled-back SQLite transaction");
    DatabaseService
        .transaction_execute(
            &rolled_back.transaction,
            "INSERT INTO events (id, label) VALUES (?, ?)",
            &[json!(2), json!("rolled back")],
            ExecuteOptions::default(),
        )
        .await
        .expect("insert rolled-back SQLite row");
    DatabaseService
        .rollback(&rolled_back.transaction)
        .await
        .expect("rollback SQLite transaction");

    let rows = query(handle, "SELECT id, label FROM events ORDER BY id", &[]).await;
    assert_eq!(rows.rows.len(), 1);
    assert_eq!(rows.rows[0]["label"], json!("committed"));
    DatabaseService.close(handle).await.expect("close SQLite");
    assert!(DatabaseService.status(handle).is_err());

    let reopened = DatabaseService
        .open(reconnect_config)
        .await
        .expect("reopen SQLite");
    DatabaseService
        .health(&reopened.handle)
        .await
        .expect("reopened SQLite health");
    DatabaseService
        .close(&reopened.handle)
        .await
        .expect("close reopened SQLite");
}

async fn execute(
    handle: &str,
    sql: &str,
    params: &[Value],
) -> hudhudscript_tools_io::database::QueryResult {
    DatabaseService
        .execute(handle, sql, params, ExecuteOptions::default())
        .await
        .expect("execute SQLite SQL")
}

async fn query(
    handle: &str,
    sql: &str,
    params: &[Value],
) -> hudhudscript_tools_io::database::QueryResult {
    DatabaseService
        .query(handle, sql, params, ExecuteOptions::default())
        .await
        .expect("query SQLite SQL")
}
