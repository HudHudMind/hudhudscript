//! External tests for hudhudscript_rules::action

use hudhudscript_rules::action::{
    ActionChain, ActionExecutor, ActionResult, ActionStatus, NoopExecutor,
};
use hudhudscript_rules::resolve_templates;
use hudhudscript_rules::skill::SkillAction;
use std::collections::HashMap;

#[test]
fn test_resolve_templates() {
    let mut args = HashMap::new();
    args.insert("source".to_string(), "{{file.path}}".to_string());
    args.insert("dest".to_string(), "/backup/{{file.name}}".to_string());

    let mut ctx = HashMap::new();
    ctx.insert("file.path".to_string(), "/home/a/doc.txt".to_string());
    ctx.insert("file.name".to_string(), "doc.txt".to_string());

    let resolved = resolve_templates(&args, &ctx);
    assert_eq!(resolved.get("source").unwrap(), "/home/a/doc.txt");
    assert_eq!(resolved.get("dest").unwrap(), "/backup/doc.txt");
}

#[test]
fn test_resolve_templates_no_match() {
    let mut args = HashMap::new();
    args.insert("key".to_string(), "{{missing}}".to_string());

    let ctx = HashMap::new();
    let resolved = resolve_templates(&args, &ctx);
    assert_eq!(resolved.get("key").unwrap(), "{{missing}}");
}

#[test]
fn test_action_chain_sequential() {
    let executor = NoopExecutor;
    let chain = ActionChain::new(&executor);

    let actions = vec![
        SkillAction {
            tool: "step1".to_string(),
            args: {
                let mut m = HashMap::new();
                m.insert("out".to_string(), "value1".to_string());
                m
            },
            timeout: None,
        },
        SkillAction {
            tool: "step2".to_string(),
            args: {
                let mut m = HashMap::new();
                m.insert("input".to_string(), "{{out}}".to_string());
                m
            },
            timeout: None,
        },
    ];

    let results = chain.run(&actions, &HashMap::new(), false);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, ActionStatus::Success);
    assert_eq!(results[1].status, ActionStatus::Success);
    assert_eq!(results[1].tool_name, "step2");
}

#[test]
fn test_action_chain_stops_on_failure() {
    struct FailExecutor;
    impl ActionExecutor for FailExecutor {
        fn execute(&self, action: &SkillAction, _input: &HashMap<String, String>) -> ActionResult {
            ActionResult {
                tool_name: action.tool.clone(),
                status: ActionStatus::Failure,
                output: HashMap::new(),
                message: "fail".to_string(),
            }
        }
    }

    let executor = FailExecutor;
    let chain = ActionChain::new(&executor);

    let actions = vec![
        SkillAction {
            tool: "a".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
        SkillAction {
            tool: "b".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
    ];

    let results = chain.run(&actions, &HashMap::new(), false);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, ActionStatus::Failure);
}

#[test]
fn test_action_chain_continue_on_failure() {
    struct FailExecutor;
    impl ActionExecutor for FailExecutor {
        fn execute(&self, action: &SkillAction, _input: &HashMap<String, String>) -> ActionResult {
            ActionResult {
                tool_name: action.tool.clone(),
                status: ActionStatus::Failure,
                output: HashMap::new(),
                message: "fail".to_string(),
            }
        }
    }

    let executor = FailExecutor;
    let chain = ActionChain::new(&executor);

    let actions = vec![
        SkillAction {
            tool: "a".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
        SkillAction {
            tool: "b".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
    ];

    let results = chain.run(&actions, &HashMap::new(), true);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_action_chain_skipped_status_does_not_merge_output() {
    struct SkipExecutor;
    impl ActionExecutor for SkipExecutor {
        fn execute(&self, action: &SkillAction, _input: &HashMap<String, String>) -> ActionResult {
            ActionResult {
                tool_name: action.tool.clone(),
                status: ActionStatus::Skipped,
                output: {
                    let mut m = HashMap::new();
                    m.insert("key".to_string(), "val".to_string());
                    m
                },
                message: "skipped".to_string(),
            }
        }
    }

    let executor = SkipExecutor;
    let chain = ActionChain::new(&executor);

    let actions = vec![
        SkillAction {
            tool: "a".to_string(),
            args: HashMap::new(),
            timeout: None,
        },
        SkillAction {
            tool: "b".to_string(),
            args: {
                let mut m = HashMap::new();
                m.insert("ref".to_string(), "{{key}}".to_string());
                m
            },
            timeout: None,
        },
    ];

    let results = chain.run(&actions, &HashMap::new(), true);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].status, ActionStatus::Skipped);
    assert_eq!(results[1].output.get("key").unwrap(), "val");
    assert_eq!(results[1].status, ActionStatus::Skipped);
}

#[test]
fn test_noop_executor() {
    let executor = NoopExecutor;
    let action = SkillAction {
        tool: "backup".to_string(),
        args: {
            let mut m = HashMap::new();
            m.insert("source".to_string(), "/tmp/file".to_string());
            m
        },
        timeout: Some(30),
    };
    let result = executor.execute(&action, &HashMap::new());
    assert_eq!(result.status, ActionStatus::Success);
    assert_eq!(result.tool_name, "backup");
    assert!(result.message.contains("backup"));
}
