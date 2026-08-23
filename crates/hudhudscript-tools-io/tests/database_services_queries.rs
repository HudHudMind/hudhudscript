#![cfg(feature = "db")]

mod database_test_support;

use database_test_support::{
    live_database_config, live_test_guard, open_live_database, unique_suffix, LiveBackend,
};
use hudhudscript_tools_io::database::{
    DatabaseError, DatabaseService, ExecuteOptions, TransactionOptions,
};
use serde_json::{json, Value};

type TestResult<T = ()> = Result<T, String>;

struct Names {
    parents: String,
    children: String,
}

impl Names {
    fn unique() -> Self {
        let suffix = unique_suffix();
        Self {
            parents: format!("hudhud_parents_{suffix}"),
            children: format!("hudhud_children_{suffix}"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires PostgreSQL credentials in .hudhud/database.toml"]
async fn postgres_crud_joins_filters_pagination_and_transactions() {
    run_query_contract(LiveBackend::Postgres).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires MySQL credentials in .hudhud/database.toml"]
async fn mysql_crud_joins_filters_pagination_and_transactions() {
    run_query_contract(LiveBackend::Mysql).await;
}

async fn run_query_contract(backend: LiveBackend) {
    let _guard = live_test_guard().await;
    let connection = open_live_database(live_database_config(backend))
        .await
        .expect("open live database for query contract");
    let names = Names::unique();
    let outcome = query_contract(&connection.handle, backend, &names).await;
    cleanup(&connection.handle, backend, &names).await;
    let close = DatabaseService.close(&connection.handle).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
    close.expect("close live query contract database");
}

async fn query_contract(handle: &str, backend: LiveBackend, names: &Names) -> TestResult {
    checked(DatabaseService.health(handle).await, "health check")?;
    create_schema(handle, backend, names).await?;
    for (id, code) in [(1, "Ada"), (2, "Linus"), (3, "Grace")] {
        insert_parent(handle, backend, names, id, code).await?;
    }

    let p1 = backend.parameter(1);
    let p2 = backend.parameter(2);
    let selected = query(
        handle,
        &format!("SELECT code FROM {} WHERE id = {p1}", names.parents),
        &[json!(1)],
    )
    .await?;
    ensure(
        selected.rows[0]["code"] == json!("Ada"),
        "SELECT/WHERE returned wrong row",
    )?;

    let updated = execute(
        handle,
        &format!("UPDATE {} SET code = {p1} WHERE id = {p2}", names.parents),
        &[json!("Linus Torvalds"), json!(2)],
    )
    .await?;
    ensure(
        updated.rows_affected == 1,
        "UPDATE affected the wrong row count",
    )?;

    for (id, parent, label) in [(10, 1, "compiler"), (11, 1, "runtime"), (12, 2, "kernel")] {
        let sql = format!(
            "INSERT INTO {} (id, parent_id, label) VALUES ({}, {}, {})",
            names.children,
            backend.parameter(1),
            backend.parameter(2),
            backend.parameter(3)
        );
        execute(handle, &sql, &[json!(id), json!(parent), json!(label)]).await?;
    }

    let joined = query(
        handle,
        &format!(
            "SELECT p.code AS parent_code, c.label AS child_label FROM {} p INNER JOIN {} c ON c.parent_id = p.id WHERE p.id = {p1} ORDER BY c.id",
            names.parents, names.children
        ),
        &[json!(1)],
    )
    .await?;
    ensure(
        joined.rows.len() == 2,
        "INNER JOIN returned wrong row count",
    )?;
    ensure(
        joined.rows[1]["child_label"] == json!("runtime"),
        "INNER JOIN ordering failed",
    )?;

    let left = query(
        handle,
        &format!(
            "SELECT p.code, c.label FROM {} p LEFT JOIN {} c ON c.parent_id = p.id WHERE p.id = {p1}",
            names.parents, names.children
        ),
        &[json!(3)],
    )
    .await?;
    ensure(
        left.rows.len() == 1 && left.rows[0]["label"] == Value::Null,
        "LEFT JOIN null row failed",
    )?;

    let page = query(
        handle,
        &format!(
            "SELECT code FROM {} ORDER BY id LIMIT {p1} OFFSET {p2}",
            names.parents
        ),
        &[json!(1), json!(1)],
    )
    .await?;
    ensure(
        page.rows.len() == 1 && page.rows[0]["code"] == json!("Linus Torvalds"),
        "LIMIT/OFFSET failed",
    )?;

    let bounded = checked(
        DatabaseService
            .query(
                handle,
                &format!("SELECT code FROM {} ORDER BY id", names.parents),
                &[],
                ExecuteOptions {
                    timeout_ms: None,
                    max_rows: Some(2),
                },
            )
            .await,
        "bounded SELECT",
    )?;
    ensure(
        bounded.rows.len() == 2 && bounded.truncated,
        "max_rows truncation failed",
    )?;

    let duplicate = insert_parent(handle, backend, names, 20, "Ada").await;
    ensure(duplicate.is_err(), "UNIQUE constraint accepted a duplicate")?;
    let foreign_key = execute(
        handle,
        &format!(
            "INSERT INTO {} (id, parent_id, label) VALUES ({}, {}, {})",
            names.children,
            backend.parameter(1),
            backend.parameter(2),
            backend.parameter(3)
        ),
        &[json!(21), json!(999), json!("orphan")],
    )
    .await;
    ensure(foreign_key.is_err(), "foreign key accepted an orphan")?;

    transaction_contract(handle, backend, names).await?;
    let deleted = execute(
        handle,
        &format!("DELETE FROM {} WHERE id = {p1}", names.parents),
        &[json!(4)],
    )
    .await?;
    ensure(
        deleted.rows_affected == 1,
        "DELETE affected the wrong row count",
    )?;
    let absent = query(
        handle,
        &format!("SELECT id FROM {} WHERE id = {p1}", names.parents),
        &[json!(4)],
    )
    .await?;
    ensure(absent.rows.is_empty(), "DELETE did not remove the row")
}

async fn create_schema(handle: &str, backend: LiveBackend, names: &Names) -> TestResult {
    execute(
        handle,
        &format!(
            "CREATE TABLE {} (id BIGINT PRIMARY KEY, code VARCHAR(100) NOT NULL UNIQUE)",
            names.parents
        ),
        &[],
    )
    .await?;
    let suffix = if backend == LiveBackend::Mysql {
        " ENGINE=InnoDB"
    } else {
        ""
    };
    execute(
        handle,
        &format!(
            "CREATE TABLE {} (id BIGINT PRIMARY KEY, parent_id BIGINT NOT NULL, label VARCHAR(100) NOT NULL, FOREIGN KEY (parent_id) REFERENCES {}(id)){suffix}",
            names.children, names.parents
        ),
        &[],
    )
    .await?;
    Ok(())
}

async fn insert_parent(
    handle: &str,
    backend: LiveBackend,
    names: &Names,
    id: i64,
    code: &str,
) -> TestResult {
    execute(
        handle,
        &format!(
            "INSERT INTO {} (id, code) VALUES ({}, {})",
            names.parents,
            backend.parameter(1),
            backend.parameter(2)
        ),
        &[json!(id), json!(code)],
    )
    .await
    .map(|_| ())
}

async fn transaction_contract(handle: &str, backend: LiveBackend, names: &Names) -> TestResult {
    let insert = format!(
        "INSERT INTO {} (id, code) VALUES ({}, {})",
        names.parents,
        backend.parameter(1),
        backend.parameter(2)
    );
    let committed = checked(
        DatabaseService
            .begin(handle, TransactionOptions::default())
            .await,
        "begin commit transaction",
    )?;
    checked(
        DatabaseService
            .transaction_execute(
                &committed.transaction,
                &insert,
                &[json!(4), json!("Committed")],
                ExecuteOptions::default(),
            )
            .await,
        "insert committed row",
    )?;
    checked(
        DatabaseService.commit(&committed.transaction).await,
        "commit transaction",
    )?;

    let rolled_back = checked(
        DatabaseService
            .begin(handle, TransactionOptions::default())
            .await,
        "begin rollback transaction",
    )?;
    checked(
        DatabaseService
            .transaction_execute(
                &rolled_back.transaction,
                &insert,
                &[json!(5), json!("Rolled Back")],
                ExecuteOptions::default(),
            )
            .await,
        "insert rolled-back row",
    )?;
    checked(
        DatabaseService.rollback(&rolled_back.transaction).await,
        "rollback transaction",
    )?;
    let count = query(
        handle,
        &format!(
            "SELECT COUNT(*) AS total FROM {} WHERE id IN (4, 5)",
            names.parents
        ),
        &[],
    )
    .await?;
    ensure(
        count.rows[0]["total"] == json!(1),
        "commit/rollback visibility failed",
    )
}

async fn cleanup(handle: &str, backend: LiveBackend, names: &Names) {
    let _ = backend;
    for table in [&names.children, &names.parents] {
        let _ = DatabaseService
            .execute(
                handle,
                &format!("DROP TABLE IF EXISTS {table}"),
                &[],
                ExecuteOptions::default(),
            )
            .await;
    }
}

async fn execute(
    handle: &str,
    sql: &str,
    params: &[Value],
) -> TestResult<hudhudscript_tools_io::database::QueryResult> {
    checked(
        DatabaseService
            .execute(handle, sql, params, ExecuteOptions::default())
            .await,
        "execute SQL",
    )
}

async fn query(
    handle: &str,
    sql: &str,
    params: &[Value],
) -> TestResult<hudhudscript_tools_io::database::QueryResult> {
    checked(
        DatabaseService
            .query(handle, sql, params, ExecuteOptions::default())
            .await,
        "query SQL",
    )
}

fn checked<T>(result: Result<T, DatabaseError>, context: &str) -> TestResult<T> {
    result.map_err(|error| format!("{context}: {error}"))
}

fn ensure(condition: bool, message: &str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
