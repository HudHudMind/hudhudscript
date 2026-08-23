use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::{HostAccessDecision, HostAccessPolicy, VM};

#[test]
fn database_module_executes_real_sqlite_queries() {
    let source = r#"
        let db = database.open({
            backend: "sqlite",
            url: "sqlite::memory:",
            max_connections: 1,
            sqlite_create_if_missing: true
        });
        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)");
        db.execute("INSERT INTO users (name) VALUES (?)", ["Ada"]);
        let rows = db.query("SELECT name FROM users");
        let database_result = rows.rows[0].name;
        db.close();
    "#;
    let ast = parse(source).expect("parse database script");
    let bytecode = Compiler::new()
        .compile(&ast)
        .expect("compile database script");
    let mut policy = HostAccessPolicy::permissive();
    policy.modules.database = Some(HostAccessDecision::Allow);
    let mut vm = VM::new();
    vm.set_host_access_policy(policy);
    vm.execute(&bytecode).expect("execute database script");
    assert_eq!(
        vm.get_variable("database_result")
            .and_then(|value| value.as_str()),
        Some("Ada")
    );
}

#[test]
fn database_module_is_denied_without_explicit_host_access() {
    let source = r#"database.open({ backend: "sqlite", url: "sqlite::memory:" });"#;
    let ast = parse(source).expect("parse denied script");
    let bytecode = Compiler::new()
        .compile(&ast)
        .expect("compile denied script");
    let error = VM::new()
        .execute(&bytecode)
        .expect_err("database must be opt-in");
    assert!(error
        .to_string()
        .contains("Host access denied: module 'database'"));
}
