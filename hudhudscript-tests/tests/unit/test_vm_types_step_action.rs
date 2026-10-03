//! VM types: `StepAction` must stay compact — it travels on the hot
//! dispatch path inside `Result<StepAction, CompileError>`.

use hudhudscript_vm::vm::types::StepAction;

#[test]
fn step_action_stays_compact() {
    assert!(std::mem::size_of::<StepAction>() <= 32);
}
