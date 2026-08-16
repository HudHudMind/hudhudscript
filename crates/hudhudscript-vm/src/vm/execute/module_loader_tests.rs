//! G08 acceptance tests: circular module import guard.

use crate::vm::VM;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("hudhud-g08-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir must be created");
    dir
}

fn write_module(dir: &std::path::Path, name: &str, source: &str) {
    std::fs::write(dir.join(name), source).expect("module file must be written");
}

fn run_main(dir: &std::path::Path, source: &str) -> Result<VM, hudhudscript_errors::Error> {
    let ast = parse(source).expect("main source must parse");
    let mut compiler = Compiler::new();
    compiler.set_module_base_dir(dir.to_path_buf());
    let bytecode = compiler.compile(&ast).expect("main source must compile");
    let mut vm = VM::new();
    vm.execute(&bytecode)?;
    Ok(vm)
}

#[test]
fn module_self_cycle_returns_clean_error() {
    let dir = temp_dir("self");
    write_module(
        &dir,
        "self.hud",
        "use \"self.hud\" as inner\nlet value = 1\n",
    );
    let source = "use \"self.hud\" as m\nlet value = 2\n";
    let result = run_main(&dir, source);
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("self-import must fail"),
    };
    assert!(
        error.message.contains("Circular module import"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn module_two_node_cycle_returns_full_chain() {
    let dir = temp_dir("two-node");
    write_module(&dir, "left.hud", "use \"right.hud\" as r\nlet value = 1\n");
    write_module(&dir, "right.hud", "use \"left.hud\" as l\nlet value = 2\n");
    let source = "use \"left.hud\" as m\nlet value = 3\n";
    let result = run_main(&dir, source);
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("two-node cycle must fail"),
    };
    assert!(
        error.message.contains("Circular module import"),
        "{}",
        error.message
    );
    assert!(error.message.contains("left.hud"), "{}", error.message);
    assert!(error.message.contains("right.hud"), "{}", error.message);
    assert!(
        error.message.contains("->"),
        "chain must be reported: {}",
        error.message
    );
}

#[test]
fn module_diamond_is_not_reported_as_cycle() {
    let dir = temp_dir("diamond");
    write_module(&dir, "top.hud", "let shared = 42\n");
    write_module(
        &dir,
        "left.hud",
        "use \"top.hud\" as t\nlet lv = t.shared + 1\n",
    );
    write_module(
        &dir,
        "right.hud",
        "use \"top.hud\" as t\nlet rv = t.shared + 2\n",
    );
    write_module(
        &dir,
        "main_mod.hud",
        "use \"left.hud\" as l\nuse \"right.hud\" as r\nlet total = l.lv + r.rv\n",
    );
    let source = "use \"main_mod.hud\" as m\nlet result = m.total\n";
    let vm = run_main(&dir, source).expect("diamond must load");
    let value = vm
        .get_variable_owned("result")
        .expect("result must be published");
    assert_eq!(value.as_number(), Some(87.0));
}

#[test]
fn module_guard_cleans_up_after_failed_load() {
    let dir = temp_dir("failed");
    write_module(&dir, "broken.hud", "let x = \n");
    let source = "use \"broken.hud\" as m\nlet value = 1\n";
    let error = match run_main(&dir, source) {
        Err(error) => error,
        Ok(_) => panic!("broken module must fail"),
    };
    assert!(error.message.contains("broken.hud"), "{}", error.message);

    // The failed module must not linger as an active identity: fix the file
    // and load again — this must NOT report a false cycle.
    write_module(&dir, "broken.hud", "let value = 7\n");
    let vm = run_main(&dir, "use \"broken.hud\" as m\nlet value = m.value\n")
        .expect("retry after fix must succeed");
    let value = vm.get_variable_owned("value").expect("value published");
    assert_eq!(value.as_number(), Some(7.0));
}

#[test]
fn module_load_retry_after_error_is_not_false_cycle() {
    let dir = temp_dir("retry");
    write_module(
        &dir,
        "a.hud",
        "use \"missing_dep.hud\" as d\nlet value = 1\n",
    );
    let error = match run_main(
        &dir,
        "use \"a.hud\" as m
let value = 2
",
    ) {
        Err(error) => error,
        Ok(_) => panic!("missing dep must fail"),
    };
    assert!(
        !error.message.contains("Circular module import"),
        "{}",
        error.message
    );

    write_module(&dir, "a.hud", "let value = 5\n");
    let vm = run_main(&dir, "use \"a.hud\" as m\nlet value = m.value\n")
        .expect("fixed module must load without a false cycle");
    let value = vm.get_variable_owned("value").expect("value published");
    assert_eq!(value.as_number(), Some(5.0));
}
