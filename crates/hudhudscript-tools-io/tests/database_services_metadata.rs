#![cfg(feature = "db")]

mod database_test_support;

use database_test_support::{
    live_database_config, live_test_guard, open_live_database, unique_migration_version,
    unique_suffix, LiveBackend,
};
use hudhudscript_tools_io::database::{DatabaseService, ExecuteOptions, Migration};
use serde_json::json;

struct Names {
    described: String,
    migrated: String,
    version: i64,
}

impl Names {
    fn unique() -> Self {
        let suffix = unique_suffix();
        Self {
            described: format!("hudhud_metadata_{suffix}"),
            migrated: format!("hudhud_migrated_{suffix}"),
            version: unique_migration_version(),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires PostgreSQL credentials in .hudhud/database.toml"]
async fn postgres_metadata_and_migrations() {
    metadata_contract(LiveBackend::Postgres).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires MySQL credentials in .hudhud/database.toml"]
async fn mysql_metadata_and_migrations() {
    metadata_contract(LiveBackend::Mysql).await;
}

async fn metadata_contract(backend: LiveBackend) {
    let _guard = live_test_guard().await;
    let connection = open_live_database(live_database_config(backend))
        .await
        .expect("open live metadata database");
    let names = Names::unique();
    let outcome = metadata_contract_inner(&connection.handle, &names).await;
    cleanup(&connection.handle, backend, &names).await;
    let close = DatabaseService.close(&connection.handle).await;
    if let Err(message) = outcome {
        panic!("{message}");
    }
    close.expect("close live metadata database");
}

async fn metadata_contract_inner(handle: &str, names: &Names) -> Result<(), String> {
    execute(
        handle,
        &format!(
            "CREATE TABLE {} (id BIGINT PRIMARY KEY, label VARCHAR(100) NOT NULL)",
            names.described
        ),
        &[],
    )
    .await?;

    let tables = DatabaseService
        .list_tables(handle, None)
        .await
        .map_err(|error| format!("list tables: {error}"))?;
    if !tables.contains(&names.described) {
        return Err(format!(
            "list_tables omitted {}; returned {tables:?}",
            names.described
        ));
    }

    let description = DatabaseService
        .describe_table(handle, &names.described, None)
        .await
        .map_err(|error| format!("describe table: {error}"))?;
    let id = description
        .rows
        .iter()
        .find(|row| row["name"] == json!("id"))
        .ok_or_else(|| "describe_table omitted id".to_string())?;
    if id["primary_key"] != json!(true) || id["nullable"] != json!(false) {
        return Err(format!(
            "describe_table returned invalid id metadata: {id:?}"
        ));
    }
    if !description
        .rows
        .iter()
        .any(|row| row["name"] == json!("label"))
    {
        return Err("describe_table omitted label".into());
    }

    let migration = Migration {
        version: names.version,
        name: format!("create {}", names.migrated),
        sql: format!("CREATE TABLE {} (id BIGINT PRIMARY KEY)", names.migrated),
    };
    let applied = DatabaseService
        .migrate(handle, vec![migration.clone()])
        .await
        .map_err(|error| format!("apply migration: {error}"))?;
    if applied.applied != vec![names.version] {
        return Err(format!("migration was not applied: {applied:?}"));
    }
    let skipped = DatabaseService
        .migrate(handle, vec![migration.clone()])
        .await
        .map_err(|error| format!("skip migration: {error}"))?;
    if skipped.skipped != vec![names.version] {
        return Err(format!("migration was not skipped: {skipped:?}"));
    }
    let changed = Migration {
        sql: format!(
            "CREATE TABLE {} (id BIGINT PRIMARY KEY, changed BIGINT)",
            names.migrated
        ),
        ..migration
    };
    if DatabaseService.migrate(handle, vec![changed]).await.is_ok() {
        return Err("changed migration checksum was accepted".into());
    }

    let tables = DatabaseService
        .list_tables(handle, None)
        .await
        .map_err(|error| format!("list migrated table: {error}"))?;
    if !tables.contains(&names.migrated) {
        return Err("migration table was not created".into());
    }
    Ok(())
}

async fn cleanup(handle: &str, backend: LiveBackend, names: &Names) {
    let _ = DatabaseService
        .execute(
            handle,
            &format!(
                "DELETE FROM __hudhud_migrations WHERE version = {}",
                backend.parameter(1)
            ),
            &[json!(names.version)],
            ExecuteOptions::default(),
        )
        .await;
    for table in [&names.described, &names.migrated] {
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

async fn execute(handle: &str, sql: &str, params: &[serde_json::Value]) -> Result<(), String> {
    DatabaseService
        .execute(handle, sql, params, ExecuteOptions::default())
        .await
        .map(|_| ())
        .map_err(|error| format!("execute SQL: {error}"))
}
