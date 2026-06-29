//! AST task execution helper.

use std::collections::HashMap;

use crate::agent::state::StateValue;
use crate::runtime::error::RuntimeError;

/// Execute an AST source string as a task body.
/// Binds input parameters into the interpreter environment before execution.
pub(crate) fn execute_ast_task(
    _source: &str,
    _agent_id: &str,
    task_name: &str,
    _input: HashMap<String, StateValue>,
) -> Result<StateValue, RuntimeError> {
    // AST execution is deprecated — use compile+VM pipeline instead.
    Err(RuntimeError::ExecutionFailed(format!(
        "Task '{}': AST execution deprecated. Compile to bytecode first.",
        task_name
    )))
}
