//! Promise combinators (all, race)

use crate::promise::{Promise, PromiseError};
use futures::future::select_all;

/// Wait for all promises to resolve concurrently.
/// Returns Ok with all values (in input order) if all succeed, or Err with the first error.
///
/// Audit v3 F11.1 / PERF-27: the extra `Sync` bound comes from the
/// `OnceLock<Result<T, String>>` the promise now carries internally —
/// `OnceLock<T>` is `Sync` only when `T: Sync`.
pub async fn promise_all<T: Clone + Send + Sync + 'static>(
    promises: Vec<Promise<T>>,
) -> Result<Vec<T>, PromiseError> {
    let n = promises.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    // Spawn all awaits concurrently, carrying their original index so we can
    // reassemble results in the correct order.
    let mut handles: Vec<tokio::task::JoinHandle<(usize, Result<T, PromiseError>)>> = promises
        .into_iter()
        .enumerate()
        .map(|(idx, promise)| tokio::spawn(async move { (idx, promise.await_result().await) }))
        .collect();

    let mut results: Vec<Option<T>> = (0..n).map(|_| None).collect();

    // Drain all handles, collecting results.
    while !handles.is_empty() {
        // select_all awaits the first handle to finish, returns (output, index, remaining).
        let (join_result, _done_idx, remaining) = select_all(handles).await;
        handles = remaining;

        match join_result {
            Err(e) => {
                // JoinError - abort remaining tasks and propagate.
                for h in handles {
                    h.abort();
                }
                return Err(PromiseError::Rejected(format!("Task join error: {}", e)));
            }
            Ok((_promise_idx, Err(e))) => {
                // One promise rejected - abort remaining tasks and propagate.
                for h in handles {
                    h.abort();
                }
                return Err(e);
            }
            Ok((promise_idx, Ok(value))) => {
                results[promise_idx] = Some(value);
            }
        }
    }

    Ok(results
        .into_iter()
        .map(|v| v.expect("all slots filled"))
        .collect())
}

/// Race multiple promises - return the result of the first one to complete.
/// Losing tasks are aborted to prevent resource leaks.
pub async fn promise_race<T: Clone + Send + Sync + 'static>(
    promises: Vec<Promise<T>>,
) -> Result<T, PromiseError> {
    if promises.is_empty() {
        return Err(PromiseError::Rejected("No promises to race".to_string()));
    }

    // Spawn each promise await as an independent task.
    let handles: Vec<tokio::task::JoinHandle<Result<T, PromiseError>>> = promises
        .into_iter()
        .map(|promise| tokio::spawn(async move { promise.await_result().await }))
        .collect();

    // select_all awaits the first handle to finish.
    let (join_result, _winner_idx, remaining) = select_all(handles).await;

    // Abort all losing tasks to prevent leaks.
    for h in remaining {
        h.abort();
    }

    match join_result {
        Ok(result) => result,
        Err(e) => Err(PromiseError::Rejected(format!("Task join error: {}", e))),
    }
}

// Tests moved to hudhud-script-tests/tests/async_test_inline.rs
