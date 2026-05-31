//! String-based dispatch router.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::utils::runtime_error;

/// Execute an exec method on the given arguments.
pub fn dispatch(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        "run" => super::run::exec_run(args),
        "output" => super::run::exec_output(args),
        "stream" => super::stream::exec_stream(args),
        "lines" => super::run::exec_lines(args),
        "spawn" => super::stream::exec_spawn(args),
        "timeout" => super::timeout::exec_timeout(args),
        "kill" => super::kill::exec_kill(args),
        "sudo" => Err(runtime_error(
            "exec.sudo() has been removed for security. Use exec.run() with proper sandbox configuration.".to_string(),
        )),
        _ => Err(runtime_error(format!("Unknown exec method: {}", method))),
    }
}
