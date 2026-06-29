const DEFAULT_THREAD_STACK_MB: u32 = 64;

/// Returns Some(stack_mb) if HUDHUD_THREAD_STACK_MB is set.
/// - Valid value: returns it (0 means disable)
/// - Empty string: default 64MB  
/// - Invalid: default 64MB (safe fallback)
/// - Not set at all: None (main thread, no spawn)
pub(crate) fn requested_thread_stack_mb() -> Option<u32> {
    match std::env::var("HUDHUD_THREAD_STACK_MB") {
        Ok(raw) => parse_thread_stack_mb(&raw),
        Err(_) => None,
    }
}

fn parse_thread_stack_mb(raw: &str) -> Option<u32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Some(DEFAULT_THREAD_STACK_MB);
    }
    match trimmed.parse::<u32>() {
        Ok(0) => None, // disable
        Ok(value) => Some(value),
        Err(_) => Some(DEFAULT_THREAD_STACK_MB),
    }
}

/// Run closure in a child thread with custom stack size.
/// Panic in child is propagated via process exit(1).
pub(crate) fn run_with_stack<F>(stack_mb: u32, f: F)
where
    F: FnOnce() + Send + 'static,
{
    let stack_size = (stack_mb as usize) * 1024 * 1024;
    let result = std::thread::Builder::new()
        .stack_size(stack_size)
        .name("hudhud-main".into())
        .spawn(f)
        .unwrap()
        .join();

    if let Err(e) = result {
        eprintln!("Thread panicked: {:?}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::parse_thread_stack_mb;

    #[test]
    fn parses_thread_stack_mb() {
        assert_eq!(parse_thread_stack_mb("64"), Some(64));
        assert_eq!(parse_thread_stack_mb("8"), Some(8));
        assert_eq!(parse_thread_stack_mb("0"), None);
        assert_eq!(parse_thread_stack_mb(""), Some(64));
        assert_eq!(parse_thread_stack_mb("abc"), Some(64));
    }
}
