//! Coverage tests for `crates/hudhudscript-vm/src/vm/api.rs`.
//!
//! Exercises the VM's public API surface directly: call-depth limits,
//! `call_public`, global accessors, toml config lookup, swarm/council
//! dispatchers, constitution accessors, and the MCP tool-definition /
//! provider / tool registries. Every assertion pins an exact value or an
//! exact error message; no network, no child processes.
//!
//! Skipped api.rs surfaces (see report): the McpClient registry family
//! (register/get/unregister/with_max_mcp_servers/shutdown_mcp_clients —
//! constructing an McpClient requires a stdio child process or an SSE
//! endpoint, neither of which is hermetic), `set_provider` /
//! `set_toml_providers` / `allow_process` / `allow_insecure_http` /
//! `with_register_arena_kb` / `with_max_builtin_iter` /
//! `with_default_stack_bytes` / `with_provider_timeout_secs` (single-field
//! setters with no deterministic observable behavior without live
//! providers).

use hudhudscript_bytecode::{ObjMap, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_governance::enforcement::EvaluationContext;
use hudhudscript_mcp::Tool;
use hudhudscript_parser::parse;
use hudhudscript_runtime::provider::ProviderRegistry;
use hudhudscript_tools::ToolRegistry;
use hudhudscript_vm::vm::VM;
use serde_json::json;
use std::sync::Arc;

// ── helpers ────────────────────────────────────────────────────────────

fn compile(source: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(source).expect("source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("source must compile")
}

fn run(source: &str) -> VM {
    let bytecode = compile(source);
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("script must execute");
    vm
}

fn owned(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
}

// ── call-depth configuration ───────────────────────────────────────────

#[test]
fn with_max_call_depth_enforces_the_exact_limit() {
    // `+ 1` keeps the call out of tail position: a bare `return countdown(...)`
    // compiles to TailCall, which reuses the frame (CROSS-1 TCO) and never
    // grows call_depth, so only a non-tail call exercises the depth guard.
    let bytecode = compile("fn countdown(n) {\n    if (n == 0) { return 0 }\n    return countdown(n - 1) + 1\n}\nlet r = countdown(5000)");
    let mut vm = VM::new();
    vm.with_max_call_depth(16);
    let error = vm.execute(&bytecode).err().expect("depth limit must fire");
    assert!(
        error.message.contains("Maximum call depth exceeded (16)"),
        "unexpected error: {}",
        error.message
    );
    assert!(
        error.message.contains("'countdown'"),
        "error must name the recursive function: {}",
        error.message
    );
}

#[test]
fn with_max_call_depth_ceiling_caps_a_higher_limit() {
    // Non-tail recursion, see with_max_call_depth_enforces_the_exact_limit.
    let bytecode = compile("fn countdown(n) {\n    if (n == 0) { return 0 }\n    return countdown(n - 1) + 1\n}\nlet r = countdown(5000)");
    let mut vm = VM::new();
    vm.with_max_call_depth(3000);
    vm.with_max_call_depth_ceiling(24);
    // The ceiling must immediately clamp the previously-set limit.
    let error = vm.execute(&bytecode).err().expect("ceiling must clamp");
    assert!(
        error.message.contains("Maximum call depth exceeded (24)"),
        "unexpected error: {}",
        error.message
    );
}

// ── call_public ────────────────────────────────────────────────────────

#[test]
fn call_public_invokes_a_top_level_function_with_arguments() {
    let bytecode = compile("fn add(a, b) {\n    return a + b\n}\nlet sentinel = 1");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("script must execute");
    let sum = vm
        .call_public("add", &[Value16::int(2), Value16::int(3)], &bytecode)
        .expect("call_public must succeed");
    assert_eq!(sum.as_int(), Some(5));
}

#[test]
fn call_public_reports_missing_function_exactly() {
    let bytecode = compile("let sentinel = 1");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("script must execute");
    let error = vm.call_public("no_such_fn", &[], &bytecode).err().expect("missing fn must fail");
    // api.rs raises this via compile_codes::runtime_error → `Runtime error: ` prefix.
    assert!(error.message.contains("Function 'no_such_fn' not found"), "unexpected error: {}", error.message);
}

#[test]
fn call_public_reports_arity_mismatch_exactly() {
    let bytecode = compile("fn add(a, b) {\n    return a + b\n}");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("script must execute");
    let error = vm.call_public("add", &[Value16::int(1)], &bytecode).err().expect("arity must fail");
    assert!(error.message.contains("add expects 2 args, got 1"), "unexpected error: {}", error.message);
}

// ── globals accessors ──────────────────────────────────────────────────

#[test]
fn get_global_reads_host_defined_globals_and_missing_names_are_none() {
    let mut vm = VM::new();
    assert!(vm.get_global("cov_api_never_defined").is_none());
    vm.define_global("cov_api_host_var".to_string(), Value16::int(41));
    assert_eq!(vm.get_global("cov_api_host_var").unwrap().as_int(), Some(41));
    // The same binding is visible through the variable accessors.
    assert_eq!(owned(&vm, "cov_api_host_var").as_int(), Some(41));
}

#[test]
fn subject_intents_defaults_to_an_empty_list() {
    let vm = VM::new();
    assert!(vm.subject_intents("no-such-template").is_empty());
}

// ── toml config lookup ─────────────────────────────────────────────────

#[test]
fn toml_config_lookup_walks_nested_paths_and_misses_are_null() {
    let mut vm = VM::new();
    let mut srv = ObjMap::new();
    srv.insert("host".to_string(), Value16::string("h1"));
    let mut root = ObjMap::new();
    root.insert("srv".to_string(), Value16::object(srv));
    vm.set_toml_config_object(Value16::object(root));

    assert_eq!(vm.toml_config_lookup("srv.host").as_string(), Some("h1".to_string()));
    // Missing key inside an existing object.
    assert!(vm.toml_config_lookup("srv.port").is_null());
    // Walking through a non-object leaf must stop with null.
    assert!(vm.toml_config_lookup("srv.host.deeper").is_null());
    // Missing top-level section.
    assert!(vm.toml_config_lookup("missing").is_null());
}

// ── swarm dispatchers ──────────────────────────────────────────────────

#[test]
fn swarm_add_agent_appends_and_stringifies_non_string_agents() {
    let mut vm = run("let swarm = { agents: [] }");
    let bytecode = compile("let sentinel = 1");

    assert_eq!(
        vm.dispatch_swarm_add_agent("swarm", &Value16::string("alpha"), &bytecode).unwrap().as_bool(),
        Some(true)
    );
    // A non-string agent value is stored via its string form.
    assert_eq!(
        vm.dispatch_swarm_add_agent("swarm", &Value16::int(7), &bytecode).unwrap().as_bool(),
        Some(true)
    );
    let agents = owned(&vm, "swarm")
        .as_object()
        .expect("swarm must stay an object")
        .get("agents")
        .expect("agents field")
        .as_array()
        .expect("agents must be an array")
        .clone();
    assert_eq!(agents.len(), 2);
    assert_eq!(agents[0].as_string(), Some("alpha".to_string()));
    assert_eq!(agents[1].as_string(), Some("7".to_string()));
}

#[test]
fn swarm_remove_agent_filters_by_exact_name() {
    let mut vm = run("let swarm = { agents: [\"alpha\", \"beta\"] }");
    let bytecode = compile("let sentinel = 1");
    assert_eq!(
        vm.dispatch_swarm_remove_agent("swarm", "alpha", &bytecode)
            .unwrap()
            .as_bool(),
        Some(true)
    );
    let swarm = owned(&vm, "swarm");
    let agents = swarm
        .as_object()
        .expect("swarm must stay an object")
        .get("agents")
        .expect("agents field")
        .as_array()
        .expect("agents must be an array");
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].as_string(), Some("beta".to_string()));
}

#[test]
fn swarm_dispatchers_return_false_for_non_object_rosters() {
    let mut vm = run("let plain = 3");
    let bytecode = compile("let sentinel = 1");
    assert_eq!(
        vm.dispatch_swarm_add_agent("plain", &Value16::string("x"), &bytecode).unwrap().as_bool(),
        Some(false)
    );
    assert_eq!(
        vm.dispatch_swarm_remove_agent("plain", "x", &bytecode).unwrap().as_bool(),
        Some(false)
    );
    // A name that was never bound at all also reports false.
    assert_eq!(
        vm.dispatch_swarm_add_agent("unbound", &Value16::string("x"), &bytecode).unwrap().as_bool(),
        Some(false)
    );
}

#[test]
fn swarm_run_without_provider_agents_returns_an_empty_result_list() {
    // The named agent is not bound and a bound non-agent object has no
    // "provider" field, so no member can be dispatched: the run must
    // succeed with an empty array rather than error.
    let mut vm = run("let crew = { agents: [\"ghost\"] }\nlet simple = { name: \"n\" }");
    let bytecode = compile("let sentinel = 1");
    let result = vm
        .dispatch_swarm_run(
            "crew",
            &["ghost".to_string(), "simple".to_string()],
            &Value16::string("do things"),
            &bytecode,
        )
        .expect("swarm run must succeed");
    let items = result.as_array().expect("swarm run must return an array");
    assert_eq!(items.len(), 0);
}

// ── council dispatchers ────────────────────────────────────────────────

#[test]
fn council_add_member_wraps_the_agent_id_in_a_member_object() {
    let mut vm = run("let council = { members: [] }");
    let bytecode = compile("let sentinel = 1");
    assert_eq!(
        vm.dispatch_council_add_member("council", &Value16::string("gamma"), &bytecode)
            .unwrap()
            .as_bool(),
        Some(true)
    );
    let members = owned(&vm, "council")
        .as_object()
        .expect("council must stay an object")
        .get("members")
        .expect("members field")
        .as_array()
        .expect("members must be an array")
        .clone();
    assert_eq!(members.len(), 1);
    let member = members[0].as_object().expect("member must be an object");
    assert_eq!(
        member.get("agent_id").expect("agent_id field").as_string(),
        Some("gamma".to_string())
    );
}

#[test]
fn council_add_member_returns_false_for_non_object_councils() {
    let mut vm = run("let plain = \"nope\"");
    let bytecode = compile("let sentinel = 1");
    assert_eq!(
        vm.dispatch_council_add_member("plain", &Value16::string("x"), &bytecode)
            .unwrap()
            .as_bool(),
        Some(false)
    );
}

// ── constitution accessors ─────────────────────────────────────────────

#[test]
fn constitution_lifecycle_and_compliance_check() {
    let script = "let cfg = { laws: [ { name: \"risk-limit\", enforcement_level: \"mandatory\", rules: [\"risk < 10\"] } ] }\nlet reg = register_constitution(\"cov-const\", cfg)\nlet act = activate_constitution(\"cov-const\")";
    let mut vm = run(script);
    assert_eq!(owned(&vm, "reg").as_bool(), Some(true));
    assert_eq!(owned(&vm, "act").as_bool(), Some(true));

    assert!(vm.has_active_constitution());
    let constitution = vm.get_active_constitution().expect("constitution must be active");
    assert_eq!(constitution.id, "cov-const");
    assert_eq!(constitution.laws.len(), 1);

    // A compliant action context passes the mandatory risk bound.
    let mut ok_ctx = EvaluationContext::new();
    ok_ctx.insert("risk".to_string(), json!(3));
    vm.check_constitution_compliance(&ok_ctx)
        .expect("compliant context must pass");

    // A violating context is denied and carries a violation context entry.
    let mut bad_ctx = EvaluationContext::new();
    bad_ctx.insert("risk".to_string(), json!(15));
    let error = vm
        .check_constitution_compliance(&bad_ctx)
        .err()
        .expect("violating context must be denied");
    assert!(
        error.message.contains("Governance violation in constitution 'cov-const'"),
        "unexpected error: {}",
        error.message
    );
    assert!(
        error.context.iter().any(|(k, _)| k == "violation"),
        "error must carry a violation context entry"
    );
}

#[test]
fn advisory_laws_do_not_block_and_deactivation_clears_the_constitution() {
    let script = "let cfg = { laws: [ { name: \"soft\", enforcement_level: \"advisory\", rules: [\"risk < 10\"] } ] }\nlet reg = register_constitution(\"cov-soft\", cfg)\nlet act = activate_constitution(\"cov-soft\")";
    let mut vm = run(script);
    assert_eq!(owned(&vm, "act").as_bool(), Some(true));
    let mut bad_ctx = EvaluationContext::new();
    bad_ctx.insert("risk".to_string(), json!(99));
    vm.check_constitution_compliance(&bad_ctx)
        .expect("advisory violations must not block");

    // Deactivate on the same VM, then enforcement must be fully off.
    vm.execute(&compile("let off = deactivate_constitution(\"cov-soft\")"))
        .expect("deactivation must execute");
    assert!(!vm.has_active_constitution());
    assert!(vm.get_active_constitution().is_none());
    let mut ctx = EvaluationContext::new();
    ctx.insert("risk".to_string(), json!(9999));
    vm.check_constitution_compliance(&ctx)
        .expect("no constitution means no enforcement");
}

// ── provider / tool / MCP registries ───────────────────────────────────

#[test]
fn provider_and_tool_registries_default_to_none_and_roundtrip() {
    let mut vm = VM::new();
    assert!(vm.provider_registry().is_none());
    assert!(vm.tool_registry().is_none());

    let providers = Arc::new(ProviderRegistry::new());
    vm.set_provider_registry(providers.clone());
    assert!(Arc::ptr_eq(&vm.provider_registry().unwrap(), &providers));

    let tools = Arc::new(ToolRegistry::new());
    vm.set_tool_registry(tools.clone());
    assert!(Arc::ptr_eq(&vm.tool_registry().unwrap(), &tools));
}

#[test]
fn mcp_tool_definitions_store_query_and_miss() {
    let vm = VM::new();
    assert!(vm.get_mcp_tool_definitions("cov-srv").is_none());
    assert!(!vm.has_mcp_tool("cov-srv", "echo"));

    let defs = vec![
        Tool {
            name: "echo".to_string(),
            description: Some("returns its input".to_string()),
            input_schema: json!({"type": "object"}),
        },
        Tool {
            name: "dump".to_string(),
            description: None,
            input_schema: json!({"type": "object"}),
        },
    ];
    vm.set_mcp_tool_definitions("cov-srv".to_string(), defs);

    let stored = vm
        .get_mcp_tool_definitions("cov-srv")
        .expect("definitions must be stored");
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].name, "echo");
    assert_eq!(
        stored[0].description,
        Some("returns its input".to_string())
    );
    assert!(vm.has_mcp_tool("cov-srv", "echo"));
    assert!(vm.has_mcp_tool("cov-srv", "dump"));
    assert!(!vm.has_mcp_tool("cov-srv", "missing"));
    assert!(!vm.has_mcp_tool("other-srv", "echo"));
}
