//! G09: thread stack size resolution for the CLI startup decision.
//!
//! Precedence: `HUDHUD_THREAD_STACK_MB` env var over the TOML config's
//! `runtime.thread_stack_mb`. `0` disables the child thread entirely.
//!
//! Relocated verbatim from the `hudhud` binary crate (`src/startup.rs`) so
//! the external unit-test suite can exercise it; behavior is unchanged.

const DEFAULT_THREAD_STACK_MB: u32 = 64;
const MAX_THREAD_STACK_MB: u32 = 1024;

#[doc(hidden)]
pub fn resolve_thread_stack_mb(
    env_value: Option<&str>,
    config_mb: u32,
) -> Result<Option<u32>, String> {
    let selected = match env_value {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                DEFAULT_THREAD_STACK_MB
            } else {
                trimmed.parse::<u32>().unwrap_or(DEFAULT_THREAD_STACK_MB)
            }
        }
        None => config_mb,
    };
    if selected > MAX_THREAD_STACK_MB {
        return Err(format!(
            "thread stack size {} MB exceeds maximum {} MB",
            selected, MAX_THREAD_STACK_MB
        ));
    }
    if selected == 0 {
        Ok(None)
    } else {
        Ok(Some(selected))
    }
}

#[doc(hidden)]
pub fn stack_bytes(stack_mb: u32) -> Result<usize, String> {
    (stack_mb as usize)
        .checked_mul(1024 * 1024)
        .ok_or_else(|| "thread stack byte conversion overflowed".to_string())
}

#[doc(hidden)]
pub fn run_with_stack<F>(stack_mb: u32, function: F) -> Result<(), String>
where
    F: FnOnce() + Send + 'static,
{
    let handle = std::thread::Builder::new()
        .stack_size(stack_bytes(stack_mb)?)
        .name("hudhud-main".to_string())
        .spawn(function)
        .map_err(|error| format!("failed to spawn hudhud-main: {}", error))?;
    handle
        .join()
        .map_err(|payload| format!("hudhud-main panicked: {:?}", payload))
}
