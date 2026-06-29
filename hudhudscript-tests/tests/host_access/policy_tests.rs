//! H7 — Host access policy integration tests.
//!
//! These tests verify that [host_access] config parsed from hudhud.toml is
//! converted into a VM policy and enforced at runtime for env() and Env.*.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;

fn run_with_host_access_toml(source: &str, toml: &str) -> Result<(), String> {
    let mut vm = VM::with_locale(hudhudscript_vm::OutputLocale::Default);
    let host_access: hudhudscript_cli::HostAccessConfig =
        toml::from_str(toml).map_err(|e| format!("TOML parse error: {}", e))?;
    let policy = host_access.to_policy();
    vm.set_host_access_policy(policy);

    let ast = parse(source).map_err(|e| format!("Parse error: {:?}", e))?;
    let bytecode = Compiler::new()
        .compile(&ast)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    vm.execute(&bytecode).map_err(|e| format!("{}", e))
}

fn assert_runtime_error(source: &str, toml: &str, expected_substring: &str) {
    let err = run_with_host_access_toml(source, toml).expect_err("expected runtime error");
    assert!(
        err.contains(expected_substring),
        "error '{}' did not contain '{}'",
        err,
        expected_substring
    );
}

fn assert_ok(source: &str, toml: &str) {
    run_with_host_access_toml(source, toml).expect("expected script success");
}

#[test]
fn no_config_is_permissive() {
    std::env::set_var("HUDHUD_H7_TEST_KEY", "visible");
    let mut vm = VM::with_locale(hudhudscript_vm::OutputLocale::Default);
    // No [host_access] config: VM keeps the default permissive policy.
    let ast = parse(r#"function get_test() { let key = "HUDHUD_H7_TEST_KEY"; return env(key); } return get_test();"#).expect("parse");
    let bytecode = Compiler::new().compile(&ast).expect("compile");
    vm.execute(&bytecode).expect("execute");
}

#[test]
fn deny_default_blocks_env_read() {
    let toml = r#"
default = "deny"
"#;
    assert_runtime_error(
        r#"function get_home() { let key = "HOME"; return env(key); } return get_home();"#,
        toml,
        "Host access denied: env('HOME') is not allowed",
    );
}

#[test]
fn explicit_env_allow_permits_read() {
    std::env::set_var("HUDHUD_H7_ALLOWED", "ok");
    let toml = r#"
default = "deny"

[env]
default = "deny"
allow = ["HUDHUD_H7_ALLOWED"]
"#;
    assert_ok(
        r#"function get_allowed() { let key = "HUDHUD_H7_ALLOWED"; return env(key); } return get_allowed();"#,
        toml,
    );
}

#[test]
fn env_deny_list_blocks_key() {
    std::env::set_var("HUDHUD_H7_SECRET", "x");
    let toml = r#"
default = "allow"

[env]
deny = ["HUDHUD_H7_SECRET"]
"#;
    assert_runtime_error(
        r#"function get_secret() { let key = "HUDHUD_H7_SECRET"; return env(key); } return get_secret();"#,
        toml,
        "Host access denied: env('HUDHUD_H7_SECRET')",
    );
}

#[test]
fn env_all_blocked_by_deny_default() {
    let toml = r#"
default = "deny"
"#;
    assert_runtime_error(
        r#"Env.all(); return null;"#,
        toml,
        "Host access denied: Env.all() is not allowed",
    );
}

#[test]
fn env_all_unfiltered_requires_open_policy() {
    let toml = r#"
default = "allow"

[env]
default = "allow"
allow = ["SOME_KEY"]
"#;
    assert_runtime_error(
        r#"Env.all_unfiltered(); return null;"#,
        toml,
        "Host access denied: Env.all_unfiltered() is not allowed",
    );
}

#[test]
fn exec_module_denied_by_default() {
    let toml = r#"
default = "deny"
"#;
    assert_runtime_error(
        r#"function run_python() { exec.run("python", ["-c", "print(1)"]); } run_python(); return null;"#,
        toml,
        "Host access denied: module 'exec' is not allowed",
    );
}

#[test]
fn exec_command_deny_list_blocks_command() {
    let toml = r#"
default = "allow"

[exec]
default = "allow"
deny = ["rm"]
"#;
    assert_runtime_error(
        r#"function run_rm() { exec.run("rm", ["-rf", "/"]); } run_rm(); return null;"#,
        toml,
        "Host access denied: command 'rm' is not allowed",
    );
}

#[test]
fn exec_allowed_command_works() {
    let toml = r#"
default = "deny"

[modules]
process = "allow"

[exec]
default = "allow"
allow = ["true"]
"#;
    assert_ok(
        r#"function run_true() { exec.run("true", []); } run_true(); return null;"#,
        toml,
    );
}

#[test]
fn toml_parse_env_allow() {
    let toml = r#"
default = "deny"

[env]
default = "deny"
allow = ["HUDHUD_H7_ALLOWED"]
"#;
    let cfg: hudhudscript_cli::HostAccessConfig = toml::from_str(toml).expect("parse");
    eprintln!("PARSE env.allow = {:?}", cfg.env.allow);
    assert_eq!(cfg.env.allow, vec!["HUDHUD_H7_ALLOWED".to_string()]);
}
