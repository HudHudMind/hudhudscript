//! G09: thread stack size resolution for the CLI startup decision.
//!
//! Precedence: `HUDHUD_THREAD_STACK_MB` env var over the TOML config's
//! `runtime.thread_stack_mb`. `0` disables the child thread entirely.

const DEFAULT_THREAD_STACK_MB: u32 = 64;
const MAX_THREAD_STACK_MB: u32 = 1024;

pub(crate) fn resolve_thread_stack_mb(
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

pub(crate) fn stack_bytes(stack_mb: u32) -> Result<usize, String> {
    (stack_mb as usize)
        .checked_mul(1024 * 1024)
        .ok_or_else(|| "thread stack byte conversion overflowed".to_string())
}

pub(crate) fn run_with_stack<F>(stack_mb: u32, function: F) -> Result<(), String>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_env_overrides_toml() {
        assert_eq!(resolve_thread_stack_mb(Some("128"), 32).unwrap(), Some(128));
    }

    #[test]
    fn stack_toml_used_when_env_absent() {
        assert_eq!(resolve_thread_stack_mb(None, 32).unwrap(), Some(32));
    }

    #[test]
    fn stack_default_is_64_when_both_absent() {
        assert_eq!(resolve_thread_stack_mb(None, 64).unwrap(), Some(64));
    }

    #[test]
    fn stack_zero_disables_child_thread() {
        assert_eq!(resolve_thread_stack_mb(Some("0"), 64).unwrap(), None);
        assert_eq!(resolve_thread_stack_mb(None, 0).unwrap(), None);
    }

    #[test]
    fn stack_invalid_env_preserves_64mb_compatibility() {
        assert_eq!(
            resolve_thread_stack_mb(Some("not-a-number"), 64).unwrap(),
            Some(64)
        );
        assert_eq!(resolve_thread_stack_mb(Some("  "), 64).unwrap(), Some(64));
    }

    #[test]
    fn stack_upper_bound_is_rejected() {
        assert!(resolve_thread_stack_mb(Some("2048"), 64).is_err());
        assert!(resolve_thread_stack_mb(None, 1025).is_err());
        assert!(resolve_thread_stack_mb(Some("1024"), 64).is_ok());
    }

    #[test]
    fn stack_byte_conversion_is_checked() {
        assert_eq!(stack_bytes(64).unwrap(), 64 * 1024 * 1024);
        assert_eq!(stack_bytes(1024).unwrap(), 1024 * 1024 * 1024);
        // checked_mul only overflows on 32-bit targets; on 64-bit the
        // full u32 MB range converts exactly.
        if std::mem::size_of::<usize>() < 8 {
            assert_eq!(
                stack_bytes(u32::MAX).unwrap_err(),
                "thread stack byte conversion overflowed"
            );
        } else {
            assert_eq!(
                stack_bytes(u32::MAX).unwrap(),
                u32::MAX as usize * 1024 * 1024
            );
        }
    }

    #[test]
    fn spawned_thread_is_named_hudhud_main() {
        let name = std::sync::Arc::new(std::sync::Mutex::new(None));
        let observed = std::sync::Arc::clone(&name);
        run_with_stack(8, move || {
            *observed.lock().unwrap() = Some(std::thread::current().name().map(str::to_string));
        })
        .unwrap();
        assert_eq!(
            name.lock()
                .unwrap()
                .as_ref()
                .and_then(|inner| inner.as_deref()),
            Some("hudhud-main")
        );
    }
}
