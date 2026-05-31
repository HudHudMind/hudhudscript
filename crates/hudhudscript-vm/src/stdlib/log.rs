//! Logging module registration.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "log",
        Box::new(|method, args| match method {
            "init" => {
                let config = args.first().cloned().unwrap_or_default();
                hudhud_log::log_ops::log_init(&config)
            }
            "trace" => hudhud_log::log_ops::log_trace(&args),
            "debug" => hudhud_log::log_ops::log_debug(&args),
            "info" => hudhud_log::log_ops::log_info(&args),
            "warn" => hudhud_log::log_ops::log_warn(&args),
            "error" => hudhud_log::log_ops::log_error(&args),
            "span_enter" => hudhud_log::log_ops::log_span_enter(&args),
            "span_exit" => hudhud_log::log_ops::log_span_exit(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown log method: {}", method),
            )),
        }),
    );
}
