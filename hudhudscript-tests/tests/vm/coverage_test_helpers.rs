fn assert_number(vm: &VM, name: &str, expected: f64) {
    let n = vm.get_variable(name)
        .and_then(|v| v.as_number())
        .unwrap_or_else(|| panic!("expected number for '{}', got {:?}", name, vm.get_variable(name)));
    assert!(
        (n - expected).abs() < 1e-10,
        "expected {} = {}, got {}",
        name,
        expected,
        n
    );
}

/// Assert a VM variable holds a specific string.
fn assert_string(vm: &VM, name: &str, expected: &str) {
    let s = vm.get_variable(name)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("expected string for '{}', got {:?}", name, vm.get_variable(name)));
    assert_eq!(
        s,
        expected,
        "expected '{}' = {:?}, got {:?}",
        name,
        expected,
        s
    );
}

/// Extract error message from a Result<VM, E> without requiring VM: Debug.
fn run_err_msg(result: Result<VM, hudhudscript_bytecode::error::CompileError>) -> String {
    match result {
        Ok(_) => panic!("expected error but got Ok"),
        Err(e) => format!("{:?}", e),
}
}

/// Assert a VM variable holds a specific boolean.
fn assert_bool(vm: &VM, name: &str, expected: bool) {
    let b = vm.get_variable(name)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("expected boolean for '{}', got {:?}", name, vm.get_variable(name)));
    assert_eq!(b, expected, "expected {} = {}", name, expected);
}

/// Assert a VM variable is null.
fn assert_null(vm: &VM, name: &str) {
    match vm.get_variable(name) {
        Some(v) if v.is_null() => {}
        other => panic!("expected null for '{}', got {:?}", name, other),
    }
}
