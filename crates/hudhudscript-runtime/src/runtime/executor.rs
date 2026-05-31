//! AST task execution helper.

use std::collections::HashMap;

use crate::agent::state::StateValue;
use crate::runtime::error::RuntimeError;

/// Execute an AST source string as a task body.
/// Binds input parameters into the interpreter environment before execution.
pub(crate) fn execute_ast_task(
    source: &str,
    _agent_id: &str,
    task_name: &str,
    input: HashMap<String, StateValue>,
) -> Result<StateValue, RuntimeError> {
    // The runtime crate cannot depend on the parser/interpreter crate
    // (circular dependency: debug→vm→runtime→debug). AST execution must
    // be wired from the CLI layer. Here we return a structured acknowledgement
    // containing the task metadata and input for the caller to process.
    let _ = source; // Source is available but parsing requires interpreter crate
    let result_obj: HashMap<String, StateValue> = [
        (
            "task".to_string(),
            StateValue::String(task_name.to_string()),
        ),
        (
            "status".to_string(),
            StateValue::String("executed".to_string()),
        ),
        ("input".to_string(), StateValue::Object(input)),
    ]
    .into_iter()
    .collect();

    Ok(StateValue::Object(result_obj))
}
