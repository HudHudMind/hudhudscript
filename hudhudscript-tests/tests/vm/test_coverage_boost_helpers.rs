fn assert_number(vm: &VM, name: &str, expected: f64) {
    let n = vm.get_variable(name)
        .and_then(|v| v.as_number())
        .unwrap_or_else(|| panic!("expected number for '{}', got {:?}", name, vm.get_variable(name)));
    assert!(
        (n - expected).abs() < 1e-10,
        "expected {} = {}, got {}",
        name, expected, n
    );
}

fn assert_string(vm: &VM, name: &str, expected: &str) {
    let s = vm.get_variable(name)
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| panic!("expected string for '{}', got {:?}", name, vm.get_variable(name)));
    assert_eq!(s, expected, "var '{}'", name);
}

fn assert_bool(vm: &VM, name: &str, expected: bool) {
    let b = vm.get_variable(name)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("expected boolean for '{}', got {:?}", name, vm.get_variable(name)));
    assert_eq!(b, expected, "var '{}'", name);
}
