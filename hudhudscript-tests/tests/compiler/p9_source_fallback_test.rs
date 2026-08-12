use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;
use std::env;
use std::fs;

#[test]
fn test_source_import_fallback() {
    let temp_dir = tempfile::tempdir().unwrap();
    let module_path = temp_dir.path().join("my_module.hud");

    fs::write(
        &module_path,
        "function get_value(x) { return x }\nexport get_value",
    )
    .unwrap();

    // Format the import path correctly for the script
    let script_path = module_path.to_str().unwrap().replace("\\", "\\\\");

    // Test 1: Using 'use'
    let script_use = format!("use \"{}\"\nlet a = get_value(42)", script_path);
    let ast_use = parse(&script_use).expect("parse use failed");
    let mut compiler = Compiler::new();
    let bytecode_use = compiler.compile(&ast_use).expect("compile use failed");

    let mut vm_use = VM::new();
    vm_use
        .execute(&bytecode_use)
        .expect("vm execute use failed");
    assert_eq!(vm_use.get_variable("a").unwrap().to_string(), "42");

    // Test 2: Using 'import'
    let script_import = format!(
        "import my_module from \"{}\"\nlet f = my_module.get_value\nlet b = f(42)",
        script_path
    );
    let ast_import = parse(&script_import).expect("parse import failed");
    let mut compiler_import = Compiler::new();
    let bytecode_import = compiler_import
        .compile(&ast_import)
        .expect("compile import failed");

    let mut vm_import = VM::new();
    vm_import
        .execute(&bytecode_import)
        .expect("vm execute import failed");
    assert_eq!(vm_import.get_variable("b").unwrap().to_string(), "42");

    // Test 3 & 4: Relative path tests
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(temp_dir.path()).unwrap();

    let relative_path = "./my_module.hud";

    // Test 3: Relative 'use'
    let script_rel_use = format!("use \"{}\"\nlet c = get_value(42)", relative_path);
    let ast_rel_use = parse(&script_rel_use).expect("parse rel use failed");
    let mut comp_rel_use = Compiler::new();
    let bc_rel_use = comp_rel_use
        .compile(&ast_rel_use)
        .expect("compile rel use failed");
    let mut vm_rel_use = VM::new();
    vm_rel_use
        .execute(&bc_rel_use)
        .expect("vm execute rel use failed");
    assert_eq!(vm_rel_use.get_variable("c").unwrap().to_string(), "42");

    // Test 4: Relative 'import'
    let script_rel_import = format!(
        "import my_module from \"{}\"\nlet f2 = my_module.get_value\nlet d = f2(42)",
        relative_path
    );
    let ast_rel_import = parse(&script_rel_import).expect("parse rel import failed");
    let mut comp_rel_import = Compiler::new();
    let bc_rel_import = comp_rel_import
        .compile(&ast_rel_import)
        .expect("compile rel import failed");
    let mut vm_rel_import = VM::new();
    vm_rel_import
        .execute(&bc_rel_import)
        .expect("vm execute rel import failed");
    assert_eq!(vm_rel_import.get_variable("d").unwrap().to_string(), "42");

    env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_missing_explicit_source_errors_loudly() {
    let script = "use \"missing_agents.hudhud\" as agents;\nprint(agents);";
    let ast = parse(script).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = VM::new();
    let res = vm.execute(&bytecode);
    assert!(
        res.is_err(),
        "Expected error for missing explicit source module"
    );
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("Cannot read module") && err_str.contains("file not found"),
        "Error was: {}",
        err_str
    );
}
