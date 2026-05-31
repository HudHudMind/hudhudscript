//! Action execution for skills
//!
//! Defines the `ActionExecutor` trait, `ActionResult`, and `ActionChain` for
//! sequential execution with output piping between steps.

use crate::skill::SkillAction;
use std::collections::HashMap;

/// Outcome of a single action execution
#[derive(Debug, Clone, PartialEq)]
pub enum ActionStatus {
    Success,
    Failure,
    Skipped,
}

/// Result produced by executing one action
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// Which tool was invoked
    pub tool_name: String,
    /// Execution status
    pub status: ActionStatus,
    /// Output key-value pairs (available as input to the next action in a chain)
    pub output: HashMap<String, String>,
    /// Human-readable message or error description
    pub message: String,
}

/// Trait for dispatching a single action to a plugin/tool
pub trait ActionExecutor: Send + Sync {
    /// Execute a skill action with the given input context.
    ///
    /// `input` contains merged outputs from previous actions in the chain
    /// plus any event payload variables.
    fn execute(&self, action: &SkillAction, input: &HashMap<String, String>) -> ActionResult;
}

/// Executes a sequence of `SkillAction`s, piping the output of each into
/// the input of the next.
pub struct ActionChain<'a> {
    executor: &'a dyn ActionExecutor,
}

impl<'a> ActionChain<'a> {
    /// Create a new chain backed by the given executor.
    pub fn new(executor: &'a dyn ActionExecutor) -> Self {
        Self { executor }
    }

    /// Run all actions sequentially. Stops on the first failure unless
    /// `continue_on_failure` is true.
    ///
    /// Template variables in action args of the form `{{key}}` are replaced
    /// with values from the accumulated context before dispatch.
    pub fn run(
        &self,
        actions: &[SkillAction],
        initial_context: &HashMap<String, String>,
        continue_on_failure: bool,
    ) -> Vec<ActionResult> {
        let mut results = Vec::with_capacity(actions.len());
        let mut ctx = initial_context.clone();

        for action in actions {
            // Resolve template variables in args
            let resolved = resolve_templates(&action.args, &ctx);
            let resolved_action = SkillAction {
                tool: action.tool.clone(),
                args: resolved,
                timeout: action.timeout,
            };

            let result = self.executor.execute(&resolved_action, &ctx);

            // Merge outputs into context for the next action
            if result.status == ActionStatus::Success {
                for (k, v) in &result.output {
                    ctx.insert(k.clone(), v.clone());
                }
            }

            let failed = result.status == ActionStatus::Failure;
            results.push(result);

            if failed && !continue_on_failure {
                break;
            }
        }

        results
    }
}

/// Replace `{{key}}` placeholders in values with entries from `ctx`.
pub fn resolve_templates(
    args: &HashMap<String, String>,
    ctx: &HashMap<String, String>,
) -> HashMap<String, String> {
    args.iter()
        .map(|(k, v)| {
            let mut resolved = v.clone();
            for (ck, cv) in ctx {
                let token = format!("{{{{{}}}}}", ck);
                resolved = resolved.replace(&token, cv);
            }
            (k.clone(), resolved)
        })
        .collect()
}

// ── Default no-op executor (useful for testing) ────────────────────────────

/// A default executor that records invocations but does not actually run tools.
pub struct NoopExecutor;

impl ActionExecutor for NoopExecutor {
    fn execute(&self, action: &SkillAction, _input: &HashMap<String, String>) -> ActionResult {
        ActionResult {
            tool_name: action.tool.clone(),
            status: ActionStatus::Success,
            output: action.args.clone(),
            message: format!("noop: executed tool '{}'", action.tool),
        }
    }
}
