use std::future::Future;
use std::sync::OnceLock;

use super::DatabaseError;

fn runtime() -> Result<&'static tokio::runtime::Runtime, DatabaseError> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("hudhud-database")
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(|error| {
            DatabaseError::ConnectionFailed(format!("database runtime could not start: {error}"))
        })
}

/// Run a database future without nesting a Tokio runtime on the caller thread.
pub fn block_on<F, T>(future: F) -> Result<T, DatabaseError>
where
    F: Future<Output = Result<T, DatabaseError>> + Send,
    T: Send,
{
    let runtime = runtime()?;
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| runtime.block_on(future))
        }
        Ok(_) => scoped_block_on(runtime, future),
        Err(_) => runtime.block_on(future),
    }
}

fn scoped_block_on<F, T>(runtime: &tokio::runtime::Runtime, future: F) -> Result<T, DatabaseError>
where
    F: Future<Output = Result<T, DatabaseError>> + Send,
    T: Send,
{
    std::thread::scope(|scope| {
        scope
            .spawn(move || runtime.block_on(future))
            .join()
            .map_err(|_| {
                DatabaseError::ConnectionFailed("database runtime worker panicked".into())
            })?
    })
}
