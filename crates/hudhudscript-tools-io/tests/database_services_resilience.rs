#![cfg(feature = "db")]

mod database_test_support;

use database_test_support::{
    invalid_password_config, live_database_config, live_test_guard, open_live_database, LiveBackend,
};
use hudhudscript_tools_io::database::{
    DatabaseError, DatabaseService, ExecuteOptions, TransactionOptions,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires PostgreSQL credentials in .hudhud/database.toml"]
async fn postgres_rejects_bad_credentials_and_recovers_from_timeout_and_pool_exhaustion() {
    resilience_contract(LiveBackend::Postgres).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires MySQL credentials in .hudhud/database.toml"]
async fn mysql_rejects_bad_credentials_and_recovers_from_timeout_and_pool_exhaustion() {
    resilience_contract(LiveBackend::Mysql).await;
}

async fn resilience_contract(backend: LiveBackend) {
    let _guard = live_test_guard().await;
    invalid_credentials_are_redacted(backend).await;

    let mut config = live_database_config(backend);
    config.max_connections = 1;
    config.query_timeout_ms = 1_000;
    let reconnect_config = config.clone();
    let connection = open_live_database(config)
        .await
        .expect("open live database for resilience contract");

    let status = DatabaseService
        .health(&connection.handle)
        .await
        .expect("initial live database health check");
    assert!(!status.closed);
    assert_eq!(status.backend, connection.backend);

    let timed_out = DatabaseService
        .query(
            &connection.handle,
            backend.sleep_query(),
            &[],
            ExecuteOptions {
                timeout_ms: Some(20),
                max_rows: None,
            },
        )
        .await;
    assert!(matches!(timed_out, Err(DatabaseError::Timeout(_))));
    DatabaseService
        .health(&connection.handle)
        .await
        .expect("connection must recover after query cancellation");

    let transaction = DatabaseService
        .begin(&connection.handle, TransactionOptions::default())
        .await
        .expect("begin pool exhaustion transaction");
    let exhausted = DatabaseService
        .query(
            &connection.handle,
            "SELECT 1 AS available",
            &[],
            ExecuteOptions {
                timeout_ms: Some(100),
                max_rows: None,
            },
        )
        .await;
    assert!(matches!(exhausted, Err(DatabaseError::Timeout(_))));
    DatabaseService
        .rollback(&transaction.transaction)
        .await
        .expect("release exhausted pool transaction");
    DatabaseService
        .health(&connection.handle)
        .await
        .expect("pool must recover after connection release");

    DatabaseService
        .close(&connection.handle)
        .await
        .expect("close live resilience database");
    assert!(DatabaseService.status(&connection.handle).is_err());

    let reopened = open_live_database(reconnect_config)
        .await
        .expect("reopen live database");
    DatabaseService
        .health(&reopened.handle)
        .await
        .expect("reopened live database health check");
    DatabaseService
        .close(&reopened.handle)
        .await
        .expect("close reopened live database");
}

async fn invalid_credentials_are_redacted(backend: LiveBackend) {
    let (config, real_password) = invalid_password_config(backend);
    let error = open_live_database(config)
        .await
        .expect_err("invalid credentials must be rejected");
    assert!(matches!(error, DatabaseError::ConnectionFailed(_)));
    let rendered = error.to_string();
    assert!(
        rendered.contains("database rejected operation"),
        "invalid credential test did not reach the database server"
    );
    assert!(
        !rendered.contains(&real_password),
        "connection error exposed a database password"
    );
    assert!(
        !rendered.contains("hudhud-intentionally-invalid"),
        "connection error exposed the rejected password"
    );
}
