//! G06B acceptance tests for callback-based array operations.

use crate::vm::VM;
use hudhudscript_bytecode::{Bytecode, Value16};
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn compile_source(source: &str) -> Bytecode {
    let ast = parse(source).expect("test source must parse");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("test source must compile")
}

fn run_source(source: &str) -> VM {
    let bytecode = compile_source(source);
    let mut vm = VM::new();
    VM::reset_driver_entry_count_for_test();
    vm.execute(&bytecode).expect("test source must execute");
    assert_eq!(
        VM::driver_entry_count_for_test(),
        1,
        "array callbacks must stay on the canonical native driver"
    );
    vm
}

fn variable(vm: &VM, name: &str) -> Value16 {
    vm.get_variable_owned(name)
        .unwrap_or_else(|| panic!("{} must be published", name))
}

fn assert_numbers(vm: &VM, name: &str, expected: &[f64]) {
    let value = variable(vm, name);
    let array = value
        .as_array()
        .unwrap_or_else(|| panic!("{} must be an array", name));
    let actual: Vec<f64> = array
        .iter()
        .map(|item| {
            item.as_number()
                .unwrap_or_else(|| panic!("{} must contain only numbers", name))
        })
        .collect();
    assert_eq!(actual, expected);
}

fn assert_number(vm: &VM, name: &str, expected: f64) {
    assert_eq!(
        variable(vm, name)
            .as_number()
            .unwrap_or_else(|| panic!("{} must be a number", name)),
        expected
    );
}

fn assert_bool(vm: &VM, name: &str, expected: bool) {
    assert_eq!(
        variable(vm, name)
            .as_bool()
            .unwrap_or_else(|| panic!("{} must be a boolean", name)),
        expected
    );
}

#[test]
fn array_callback_sequence_preserves_all_method_operations() {
    let vm = run_source(
        r#"
let bonus = 10
let values = [1, 2, 3, 4]
let mapped = values.map((item, index) => { return item + index + bonus })
let filtered = values.filter((item, index) => { return item + index >= 5 })
let reduced = values.reduce((acc, item, index) => { return acc + item + index }, 10)
let each_result = values.forEach((item, index) => { return item + index })
let found = values.find((item, index) => { return item + index == 5 })
let missing = values.find((item, index) => { return item + index == 99 })
let some_true = values.some((item, index) => { return item + index == 5 })
let some_false = values.some((item, index) => { return item + index == 99 })
let every_true = values.every((item, index) => { return item + index > 0 })
let every_false = values.every((item, index) => { return item + index < 6 })
let empty = []
let empty_map = empty.map((item, index) => { return item + index })
let empty_every = empty.every((item, index) => { return item + index > 0 })
"#,
    );

    assert_numbers(&vm, "mapped", &[11.0, 13.0, 15.0, 17.0]);
    assert_numbers(&vm, "filtered", &[3.0, 4.0]);
    assert_number(&vm, "reduced", 26.0);
    assert!(variable(&vm, "each_result").is_null());
    assert_number(&vm, "found", 3.0);
    assert!(variable(&vm, "missing").is_null());
    assert_bool(&vm, "some_true", true);
    assert_bool(&vm, "some_false", false);
    assert_bool(&vm, "every_true", true);
    assert_bool(&vm, "every_false", false);
    assert_numbers(&vm, "empty_map", &[]);
    assert_bool(&vm, "empty_every", true);
}

#[test]
fn array_reduce_sequence_preserves_initial_value_and_indices() {
    let vm = run_source(
        r#"
let values = [2, 3, 4]
let without_initial = values.reduce((acc, item, index) => {
    return acc + item * index
})
let with_initial = values.reduce((acc, item, index) => {
    return acc + item * index
}, 10)
let empty = []
let empty_with_initial = empty.reduce((acc, item, index) => {
    return acc + item + index
}, 9)
"#,
    );

    assert_number(&vm, "without_initial", 13.0);
    assert_number(&vm, "with_initial", 21.0);
    assert_number(&vm, "empty_with_initial", 9.0);
}

#[test]
fn standalone_array_callbacks_use_the_same_continuation_lane() {
    let vm = run_source(
        r#"
let values = [1, 2, 3, 4]
let mapped = map(values, (item, index) => { return item + index })
let filtered = filter(values, (item, index) => { return item + index >= 5 })
let reduced = reduce(values, (acc, item, index) => { return acc + item + index }, 10)
let each_result = forEach(values, (item, index) => { return item + index })
let found = find(values, (item, index) => { return item + index == 5 })
let some_result = some(values, (item, index) => { return item + index == 5 })
let every_result = every(values, (item, index) => { return item + index > 0 })
"#,
    );

    assert_numbers(&vm, "mapped", &[1.0, 3.0, 5.0, 7.0]);
    assert_numbers(&vm, "filtered", &[3.0, 4.0]);
    assert_number(&vm, "reduced", 26.0);
    assert!(variable(&vm, "each_result").is_null());
    assert_number(&vm, "found", 3.0);
    assert_bool(&vm, "some_result", true);
    assert_bool(&vm, "every_result", true);
}

#[test]
fn deep_array_callback_sequence_does_not_open_nested_native_drivers() {
    let items = (0..2500)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"
let values = [{}]
let mapped = values.map((item, index) => {{ return item + index }})
"#,
        items
    );
    let vm = run_source(&source);
    let mapped = variable(&vm, "mapped");
    let array = mapped.as_array().expect("mapped must be an array");

    assert_eq!(array.len(), 2500);
    assert_eq!(array.first().and_then(Value16::as_number), Some(0.0));
    assert_eq!(array.last().and_then(Value16::as_number), Some(4998.0));
}
