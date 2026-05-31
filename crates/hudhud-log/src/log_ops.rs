//! Logging builtin implementations.

use std::sync::OnceLock;
use hudhudscript_bytecode::Value16;
use hudhudscript_bytecode::error::compile_codes;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

type CompileResult<T> = Result<T, hudhudscript_errors::Error>;

static INIT_DONE: OnceLock<()> = OnceLock::new();

/// LOG0002: Initialize the tracing subscriber.
pub fn log_init(config: &Value16) -> CompileResult<Value16> {
    if INIT_DONE.get().is_some() {
        return Err(compile_codes::runtime_error("log.init() already called"));
    }
    let obj = config.as_object()
        .ok_or_else(|| compile_codes::runtime_error("log.init() expects an object"))?;
    let level = obj.get("level").and_then(|v| v.as_string()).unwrap_or_else(|| "info".to_string());
    let format = obj.get("format").and_then(|v| v.as_string()).unwrap_or_else(|| "pretty".to_string());
    let output = obj.get("output").and_then(|v| v.as_string()).unwrap_or_else(|| "stderr".to_string());
    let filter_str = obj.get("filter").and_then(|v| v.as_string()).unwrap_or_else(|| level.clone());
    let filter = EnvFilter::try_new(&filter_str)
        .map_err(|e| compile_codes::runtime_error(format!("bad filter '{}': {}", filter_str, e)))?;
    let registry = tracing_subscriber::registry().with(filter);
    match (format.as_str(), output.as_str()) {
        ("json", "stdout") => registry.with(fmt::layer().json().with_writer(std::io::stdout)).init(),
        ("json", "stderr") => registry.with(fmt::layer().json().with_writer(std::io::stderr)).init(),
        ("pretty", "stdout") => registry.with(fmt::layer().pretty().with_writer(std::io::stdout)).init(),
        ("pretty", "stderr") => registry.with(fmt::layer().pretty().with_writer(std::io::stderr)).init(),
        ("compact", "stdout") => registry.with(fmt::layer().compact().with_writer(std::io::stdout)).init(),
        ("compact", "stderr") => registry.with(fmt::layer().compact().with_writer(std::io::stderr)).init(),
        (_, path) if path.starts_with("file:") => {
            let file_path = &path[5..];
            let appender = tracing_appender::rolling::never("", file_path);
            registry.with(fmt::layer().with_writer(appender)).init();
        }
        _ => registry.with(fmt::layer().pretty().with_writer(std::io::stderr)).init(),
    }
    INIT_DONE.set(()).ok();
    Ok(Value16::bool_(true))
}

fn fields_to_string(fields: Option<&Value16>) -> String {
    let Some(obj) = fields.and_then(|v| v.as_object()) else { return String::new(); };
    let mut kv = String::new();
    for (k, v) in obj.iter() {
        if !kv.is_empty() { kv.push(' '); }
        kv.push_str(&format!("{}={}", k, v.to_string()));
    }
    kv
}

fn do_log(level: tracing::Level, msg: &str, fields: Option<&Value16>) {
    let kv = fields_to_string(fields);
    match level {
        tracing::Level::TRACE => tracing::trace!(target: "hudhud", fields = %kv, "{}", msg),
        tracing::Level::DEBUG => tracing::debug!(target: "hudhud", fields = %kv, "{}", msg),
        tracing::Level::INFO  => tracing::info!(target: "hudhud", fields = %kv, "{}", msg),
        tracing::Level::WARN  => tracing::warn!(target: "hudhud", fields = %kv, "{}", msg),
        tracing::Level::ERROR => tracing::error!(target: "hudhud", fields = %kv, "{}", msg),
    }
}

pub fn log_trace(args: &[Value16]) -> CompileResult<Value16> {
    let msg = args.first().and_then(|v| v.as_string()).unwrap_or_default();
    do_log(tracing::Level::TRACE, &msg, args.get(1));
    Ok(Value16::null())
}
pub fn log_debug(args: &[Value16]) -> CompileResult<Value16> {
    let msg = args.first().and_then(|v| v.as_string()).unwrap_or_default();
    do_log(tracing::Level::DEBUG, &msg, args.get(1));
    Ok(Value16::null())
}
pub fn log_info(args: &[Value16]) -> CompileResult<Value16> {
    let msg = args.first().and_then(|v| v.as_string()).unwrap_or_default();
    do_log(tracing::Level::INFO, &msg, args.get(1));
    Ok(Value16::null())
}
pub fn log_warn(args: &[Value16]) -> CompileResult<Value16> {
    let msg = args.first().and_then(|v| v.as_string()).unwrap_or_default();
    do_log(tracing::Level::WARN, &msg, args.get(1));
    Ok(Value16::null())
}
pub fn log_error(args: &[Value16]) -> CompileResult<Value16> {
    let msg = args.first().and_then(|v| v.as_string()).unwrap_or_default();
    do_log(tracing::Level::ERROR, &msg, args.get(1));
    Ok(Value16::null())
}

// LOG0004: Span support
use std::sync::Mutex;
use tracing::Span;

static ACTIVE_SPANS: Mutex<Vec<Span>> = Mutex::new(Vec::new());

/// Open a named span. Returns span id (index).
pub fn log_span_enter(args: &[Value16]) -> CompileResult<Value16> {
    let name = args.first().and_then(|v| v.as_string()).unwrap_or_default();
    let span = tracing::info_span!(target: "hudhud", "span", name = %name);
    let _ = span.enter(); // enters the span for the current thread
    let mut spans = ACTIVE_SPANS.lock().unwrap();
    spans.push(span);
    Ok(Value16::int(spans.len() as i64 - 1))
}

/// Exit the most recent span.
pub fn log_span_exit(_args: &[Value16]) -> CompileResult<Value16> {
    let mut spans = ACTIVE_SPANS.lock().unwrap();
    spans.pop();
    // When span is dropped, the Entered guard is released
    Ok(Value16::null())
}
