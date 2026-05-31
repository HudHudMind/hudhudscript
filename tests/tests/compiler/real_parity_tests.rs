/// VM-only regression suite (formerly VM/Interpreter parity tests).
///
/// History: these tests used to run each source through BOTH the
/// interpreter and the VM and compare results.  After the
/// interpreter-retirement migration (Kural 7 single-source-of-truth),
/// the VM is the ONLY runtime — so each assertion now checks the VM's
/// output against a literal expected value that was captured from the
/// VM at migration time.
///
/// Semantics preserved from the parity era:
/// - Test bodies (source strings + variable names) are unchanged.
/// - Mutual-failure cases (where both runtimes errored, previously
///   treated as "parity OK") are now recorded as `Expected::Err` so a
///   regression that turns the error into a (wrong) success still
///   trips an assertion.
use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::{Compiler, VM};
use hudhudscript_parser::parse;

// ── Harness (VM-only) ───────────────────────────────────────────────────────
//
// After the interpreter migration (Kural 7 + VM-only runtime) these tests
// run ONLY against the VM.  The expected value for each assertion is a
// literal string — captured once by executing the VM and baked in here as
// the single source of truth.  If the VM regresses on a known-good case the
// assertion fires with the previously-recorded value.
//
// The test bodies (source strings + variable names) were NOT modified by
// this migration — only the assertion mechanism changed (test-infra
// refactor, not a test rewrite).

/// Normalize VM Value to a comparable string (same shape the interpreter
/// parity harness used, so the recorded expected values stay meaningful).
fn vm_to_string(v: &Value16) -> String {
    if let Some(n) = v.as_number() {
        if n.fract() == 0.0 && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            format!("{}", n)
        }
    } else if let Some(i) = v.as_int() {
        format!("{}", i)
    } else if let Some(s) = v.as_str() {
        s.to_string()
    } else if let Some(b) = v.as_bool() {
        format!("{}", b)
    } else if v.is_null() {
        "null".to_string()
    } else if let Some(arr) = v.as_array() {
        let items: Vec<String> = arr.iter().map(|v| vm_to_string(v)).collect();
        format!("[{}]", items.join(", "))
    } else if let Some(map) = v.as_object() {
        let mut pairs: Vec<String> = map
            .iter()
            .filter(|(k, _)| !k.starts_with("__"))
            .map(|(k, v)| format!("{}: {}", k, vm_to_string(v)))
            .collect();
        pairs.sort();
        format!("{{{}}}", pairs.join(", "))
    } else if let Some(inst) = v.as_instance_data() {
        let mut pairs: Vec<String> = inst
            .fields
            .iter()
            .map(|(k, v)| format!("{}: {}", k, vm_to_string(v)))
            .collect();
        pairs.sort();
        format!("{}({})", inst.class_name, pairs.join(", "))
    } else if let Some(ps) = v.as_promise_state() {
        match ps {
            hudhudscript_bytecode::PromiseState16::Resolved(v) => {
                format!("Promise::Resolved({})", vm_to_string(&*v))
            }
            hudhudscript_bytecode::PromiseState16::Rejected(s) => {
                format!("Promise::Rejected({})", s)
            }
            hudhudscript_bytecode::PromiseState16::Pending => "Promise::Pending".to_string(),
            hudhudscript_bytecode::PromiseState16::AsyncPending(_) => {
                "Promise::AsyncPending".to_string()
            }
        }
    } else {
        format!("{:?}", v)
    }
}

/// Run source through VM, return named variable as normalized string.
fn vm_var(src: &str, name: &str) -> Result<String, String> {
    let ast = parse(src).map_err(|e| format!("parse error: {:?}", e))?;
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&ast)
        .map_err(|e| format!("compile error: {:?}", e))?;
    let mut vm = VM::new();
    vm.execute(&bytecode)
        .map_err(|e| format!("vm error: {:?}", e))?;
    let val = vm
        .get_variable(name)
        .ok_or_else(|| format!("vm get_variable({}) returned None", name))?;
    Ok(vm_to_string(&val))
}

/// What the VM is expected to do for a given (src, var) pair.
///
/// `Ok(value)` — VM must succeed, the normalized variable contents must
/// equal `value` exactly.
///
/// `Err` — VM must fail (parse / compile / runtime error, or the variable
/// is not defined).  The original interpreter-parity harness treated a
/// mutual failure as "parity OK"; under VM-only semantics we record that
/// expected-error state explicitly so a regression that turns the error
/// into a (wrong) success still fails the test.
#[derive(Clone, Copy)]
enum Expected {
    Ok(&'static str),
    Err,
}

/// Assert that running `src` through the VM leaves `var_name` matching
/// `expected`.  The expected value is a literal (captured at migration
/// time) — no interpreter, no reference implementation.
fn assert_vm_value(src: &str, var_name: &str, expected: Expected) {
    let got = vm_var(src, var_name);
    match (&got, expected) {
        (Ok(v), Expected::Ok(want)) => {
            assert_eq!(
                v.as_str(),
                want,
                "\n--- VM mismatch for '{}' ---\n  Expected: {}\n  Got:      {}\n  Source:\n{}\n",
                var_name,
                want,
                v,
                src
            );
        }
        (Err(_), Expected::Err) => { /* expected failure — OK */ }
        (Ok(v), Expected::Err) => {
            panic!(
                "\n--- expected VM to FAIL for '{}' but it succeeded ---\n  Got: {}\n  Source:\n{}\n",
                var_name, v, src
            );
        }
        (Err(e), Expected::Ok(want)) => {
            panic!(
                "\n--- expected VM to yield {:?} for '{}' but it FAILED ---\n  Err: {}\n  Source:\n{}\n",
                want, var_name, e, src
            );
        }
    }
}

/// Assert many (var, expected) pairs against the same source.  Separate
/// from `assert_vm_value` so multi-var tests keep their single-source,
/// many-assertions shape.
fn assert_vm_values_multi(src: &str, pairs: &[(&str, Expected)]) {
    for (name, expected) in pairs {
        assert_vm_value(src, name, *expected);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — ARITHMETIC & VARIABLES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_let_number() {
    assert_vm_value("let x = 42", "x", Expected::Ok("42"));
}

#[test]
fn real_parity_let_string() {
    assert_vm_value(r#"let x = "hello""#, "x", Expected::Ok("hello"));
}

#[test]
fn real_parity_let_bool() {
    assert_vm_value("let x = true", "x", Expected::Ok("true"));
}

#[test]
fn real_parity_let_null() {
    assert_vm_value("let x = null", "x", Expected::Ok("null"));
}

#[test]
fn real_parity_arithmetic_add() {
    assert_vm_value("let x = 10 + 32", "x", Expected::Ok("42"));
}

#[test]
fn real_parity_arithmetic_sub() {
    assert_vm_value("let x = 100 - 58", "x", Expected::Ok("42"));
}

#[test]
fn real_parity_arithmetic_mul() {
    assert_vm_value("let x = 6 * 7", "x", Expected::Ok("42"));
}

#[test]
fn real_parity_arithmetic_div() {
    assert_vm_value("let x = 84 / 2", "x", Expected::Ok("42"));
}

#[test]
fn real_parity_arithmetic_mod() {
    assert_vm_value("let x = 10 % 3", "x", Expected::Ok("1"));
}

#[test]
fn real_parity_arithmetic_complex() {
    assert_vm_value("let x = (2 + 3) * 4 - 1", "x", Expected::Ok("19"));
}

#[test]
fn real_parity_string_concat() {
    assert_vm_value(
        r#"let x = "hello" + " " + "world""#,
        "x",
        Expected::Ok("hello world"),
    );
}

#[test]
fn real_parity_comparison_operators() {
    let src = r#"
        let a = 5 > 3
        let b = 5 < 3
        let c = 5 == 5
        let d = 5 != 3
        let e = 5 >= 5
        let f = 5 <= 4
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("a", Expected::Ok("true")),
            ("b", Expected::Ok("false")),
            ("c", Expected::Ok("true")),
            ("d", Expected::Ok("true")),
            ("e", Expected::Ok("true")),
            ("f", Expected::Ok("false")),
        ],
    );
}

#[test]
fn real_parity_logical_operators() {
    let src = r#"
        let a = true && false
        let b = true || false
        let c = !true
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("a", Expected::Ok("false")),
            ("b", Expected::Ok("true")),
            ("c", Expected::Ok("false")),
        ],
    );
}

#[test]
fn real_parity_unary_neg() {
    assert_vm_value("let x = -42", "x", Expected::Ok("-42"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — CONTROL FLOW
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_if_true() {
    let src = r#"
        let x = 0
        if (true) { x = 1 } else { x = 2 }
    "#;
    assert_vm_value(src, "x", Expected::Ok("1"));
}

#[test]
fn real_parity_if_false() {
    let src = r#"
        let x = 0
        if (false) { x = 1 } else { x = 2 }
    "#;
    assert_vm_value(src, "x", Expected::Ok("2"));
}

#[test]
fn real_parity_if_else_chain() {
    let src = r#"
        let x = 10
        let result = ""
        if (x > 20) { result = "big" }
        else if (x > 5) { result = "medium" }
        else { result = "small" }
    "#;
    assert_vm_value(src, "result", Expected::Ok("medium"));
}

#[test]
fn real_parity_while_loop() {
    let src = r#"
        let i = 0
        let sum = 0
        while (i < 10) {
            sum = sum + i
            i = i + 1
        }
    "#;
    assert_vm_values_multi(
        src,
        &[("i", Expected::Ok("10")), ("sum", Expected::Ok("45"))],
    );
}

#[test]
fn real_parity_while_break() {
    let src = r#"
        let i = 0
        while (true) {
            if (i == 5) { break }
            i = i + 1
        }
    "#;
    assert_vm_value(src, "i", Expected::Ok("5"));
}

#[test]
fn real_parity_while_continue() {
    let src = r#"
        let i = 0
        let sum = 0
        while (i < 10) {
            i = i + 1
            if (i % 2 == 0) { continue }
            sum = sum + i
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("25"));
}

#[test]
fn real_parity_for_in_array() {
    let src = r#"
        let arr = [10, 20, 30]
        let sum = 0
        for (let item in arr) {
            sum = sum + item
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("60"));
}

#[test]
fn real_parity_for_in_break() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5]
        let sum = 0
        for (let item in arr) {
            if (item == 4) { break }
            sum = sum + item
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("6"));
}

#[test]
fn real_parity_for_in_continue() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5]
        let sum = 0
        for (let item in arr) {
            if (item == 3) { continue }
            sum = sum + item
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("12"));
}

#[test]
fn real_parity_switch_basic() {
    let src = r#"
        let x = 2
        let result = ""
        switch (x) {
            case 1: result = "one"
            case 2: result = "two"
            case 3: result = "three"
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("two"));
}

#[test]
fn real_parity_switch_default() {
    let src = r#"
        let x = 99
        let result = ""
        switch (x) {
            case 1: result = "one"
            default: result = "other"
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("other"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — FUNCTIONS & RECURSION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_function_basic() {
    let src = r#"
        function add(a, b) { return a + b }
        let result = add(3, 4)
    "#;
    assert_vm_value(src, "result", Expected::Ok("7"));
}

#[test]
fn real_parity_function_no_return() {
    let src = r#"
        let side = 0
        function setIt() { side = 42 }
        setIt()
    "#;
    assert_vm_value(src, "side", Expected::Ok("42"));
}

#[test]
fn real_parity_factorial() {
    let src = r#"
        function fact(n) {
            if (n <= 1) { return 1 }
            return n * fact(n - 1)
        }
        let result = fact(8)
    "#;
    assert_vm_value(src, "result", Expected::Ok("40320"));
}

#[test]
fn real_parity_fibonacci() {
    let src = r#"
        function fib(n) {
            if (n <= 1) { return n }
            return fib(n - 1) + fib(n - 2)
        }
        let result = fib(10)
    "#;
    assert_vm_value(src, "result", Expected::Ok("55"));
}

#[test]
fn real_parity_mutual_recursion() {
    let src = r#"
        function isEven(n) {
            if (n == 0) { return true }
            return isOdd(n - 1)
        }
        function isOdd(n) {
            if (n == 0) { return false }
            return isEven(n - 1)
        }
        let a = isEven(10)
        let b = isOdd(7)
    "#;
    assert_vm_values_multi(
        src,
        &[("a", Expected::Ok("true")), ("b", Expected::Ok("true"))],
    );
}

#[test]
fn real_parity_nested_calls() {
    let src = r#"
        function double(x) { return x * 2 }
        function addOne(x) { return x + 1 }
        let result = double(addOne(5))
    "#;
    assert_vm_value(src, "result", Expected::Ok("12"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — CLOSURES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_closure_capture() {
    let src = r#"
        let x = 10
        function addX(y) { return x + y }
        let result = addX(5)
    "#;
    assert_vm_value(src, "result", Expected::Ok("15"));
}

#[test]
fn real_parity_closure_nested() {
    let src = r#"
        function outer() {
            let x = 10
            function inner() { return x + 5 }
            return inner()
        }
        let result = outer()
    "#;
    assert_vm_value(src, "result", Expected::Ok("15"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — ARRAYS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_array_literal() {
    assert_vm_value("let x = [1, 2, 3]", "x", Expected::Ok("[1, 2, 3]"));
}

#[test]
fn real_parity_array_index() {
    let src = r#"
        let arr = [10, 20, 30]
        let x = arr[1]
    "#;
    assert_vm_value(src, "x", Expected::Ok("20"));
}

#[test]
fn real_parity_array_length() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5]
        let x = len(arr)
    "#;
    assert_vm_value(src, "x", Expected::Ok("5"));
}

#[test]
fn real_parity_array_push() {
    let src = r#"
        let arr = [1, 2]
        arr.push(3)
        let x = len(arr)
    "#;
    assert_vm_value(src, "x", Expected::Ok("3"));
}

#[test]
fn real_parity_array_map() {
    let src = r#"
        let arr = [1, 2, 3]
        let result = arr.map((x) => x * 2)
    "#;
    assert_vm_value(src, "result", Expected::Ok("[2, 4, 6]"));
}

#[test]
fn real_parity_array_filter() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5]
        let result = arr.filter((x) => x > 3)
    "#;
    assert_vm_value(src, "result", Expected::Ok("[4, 5]"));
}

#[test]
fn real_parity_array_reduce() {
    let src = r#"
        let arr = [1, 2, 3, 4]
        let result = arr.reduce((acc, x) => acc + x, 0)
    "#;
    assert_vm_value(src, "result", Expected::Ok("10"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — OBJECTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_object_literal() {
    let src = r#"
        let obj = { name: "test", value: 42 }
        let x = obj.name
        let y = obj.value
    "#;
    assert_vm_values_multi(
        src,
        &[("x", Expected::Ok("test")), ("y", Expected::Ok("42"))],
    );
}

#[test]
fn real_parity_object_dynamic_access() {
    let src = r#"
        let obj = { a: 1, b: 2 }
        let key = "b"
        let x = obj[key]
    "#;
    assert_vm_value(src, "x", Expected::Ok("2"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — STRINGS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_string_length() {
    let src = r#"
        let s = "hello"
        let x = len(s)
    "#;
    assert_vm_value(src, "x", Expected::Ok("5"));
}

#[test]
fn real_parity_string_methods() {
    let src = r#"
        let s = "Hello World"
        let upper = s.toUpperCase()
        let lower = s.toLowerCase()
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("upper", Expected::Ok("HELLO WORLD")),
            ("lower", Expected::Ok("hello world")),
        ],
    );
}

#[test]
fn real_parity_template_string() {
    let src = r#"
        let name = "world"
        let x = `hello ${name}`
    "#;
    assert_vm_value(src, "x", Expected::Ok("hello world"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P0 — ARROW FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_arrow_basic() {
    let src = r#"
        let add = (a, b) => a + b
        let result = add(3, 4)
    "#;
    assert_vm_value(src, "result", Expected::Ok("7"));
}

#[test]
fn real_parity_arrow_block_body() {
    let src = r#"
        let double = (x) => {
            let y = x * 2
            return y
        }
        let result = double(21)
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_arrow_as_callback() {
    let src = r#"
        function apply(f, x) { return f(x) }
        let result = apply((x) => x * 3, 7)
    "#;
    assert_vm_value(src, "result", Expected::Ok("21"));
}

#[test]
fn real_parity_arrow_closure_capture() {
    let src = r#"
        let factor = 5
        let mul = (x) => x * factor
        let result = mul(8)
    "#;
    assert_vm_value(src, "result", Expected::Ok("40"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1 — CLASSES & INHERITANCE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_class_basic() {
    let src = r#"
        class Dog {
            constructor(name) {
                this.name = name
            }
            greet() {
                return "Woof! I am " + this.name
            }
        }
        let d = new Dog("Rex")
        let result = d.greet()
    "#;
    assert_vm_value(src, "result", Expected::Ok("Woof! I am Rex"));
}

#[test]
fn real_parity_class_no_constructor() {
    let src = r#"
        class Empty {
            hello() { return "hi" }
        }
        let e = new Empty()
        let result = e.hello()
    "#;
    assert_vm_value(src, "result", Expected::Ok("hi"));
}

#[test]
fn real_parity_single_inheritance() {
    let src = r#"
        class Animal {
            constructor(name) { this.name = name }
            speak() { return this.name + " makes a sound" }
        }
        class Dog extends Animal {
            speak() { return this.name + " barks" }
        }
        let d = new Dog("Rex")
        let result = d.speak()
    "#;
    assert_vm_value(src, "result", Expected::Ok("Rex barks"));
}

#[test]
fn real_parity_two_level_inheritance() {
    let src = r#"
        class A {
            constructor() { this.x = 1 }
            getX() { return this.x }
        }
        class B extends A {
            constructor() {
                super()
                this.y = 2
            }
        }
        class C extends B {
            constructor() {
                super()
                this.z = 3
            }
            sum() { return this.x + this.y + this.z }
        }
        let c = new C()
        let result = c.sum()
    "#;
    assert_vm_value(src, "result", Expected::Ok("6"));
}

#[test]
fn real_parity_inherited_method() {
    let src = r#"
        class Base {
            constructor(v) { this.v = v }
            doubled() { return this.v * 2 }
        }
        class Child extends Base {
            constructor(v) { super(v) }
        }
        let c = new Child(21)
        let result = c.doubled()
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_instance_this() {
    let src = r#"
        class Counter {
            constructor() { this.count = 0 }
            inc() {
                this.count = this.count + 1
                return this.count
            }
        }
        let c = new Counter()
        c.inc()
        c.inc()
        let result = c.inc()
    "#;
    assert_vm_value(src, "result", Expected::Ok("3"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1 — TRY/CATCH
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_try_catch() {
    let src = r#"
        let result = ""
        try {
            throw "boom"
        } catch (e) {
            result = e
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("boom"));
}

#[test]
fn real_parity_try_no_throw() {
    let src = r#"
        let result = "ok"
        try {
            result = "from try"
        } catch (e) {
            result = "from catch"
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("from try"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1 — ASYNC/AWAIT
// ═══════════════════════════════════════════════════════════════════════════════

// Issue #880: async functions are now executed eagerly (synchronously) in the
// interpreter, matching the VM behaviour for source-level async with no real
// I/O.  The "receiver was dropped before settlement" bug is fixed.

#[test]
fn real_parity_async_resolved() {
    let src = r#"
        async function getValue() { return 42 }
        let result = await getValue()
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_async_with_args() {
    let src = r#"
        async function add(a, b) { return a + b }
        let result = await add(10, 20)
    "#;
    assert_vm_value(src, "result", Expected::Ok("30"));
}

#[test]
fn real_parity_async_chain() {
    let src = r#"
        async function double(x) { return x * 2 }
        let a = await double(5)
        let result = await double(a)
    "#;
    assert_vm_value(src, "result", Expected::Ok("20"));
}

// Kural 7b / Issue #1079: regression guard for the VM `await`-returns-Null
// fake. The VM previously pushed `Null` for any non-Resolved promise state;
// this test asserts that `await` on a real async function yields a real
// integer value in *both* runtimes and in particular that the VM value is
// not `null`.
#[test]
fn test_vm_await_returns_real_value_not_null() {
    let src = r#"
        async function fetch() { return 42 }
        let p = fetch()
        let val = await p
    "#;
    // Assert both runtimes produce "42". assert_parity already panics on
    // mismatch; the extra guard below catches the specific Null regression.
    assert_vm_value(src, "val", Expected::Ok("42"));
    let vm = vm_var(src, "val").expect("vm should succeed");
    assert_ne!(vm, "null", "VM await produced Null — fake re-introduced?");
    assert_eq!(vm, "42", "VM await should yield the real resolved value");
}

// Follow-up: await inside a larger expression is not short-circuited to
// Null by the VM's `Pending` handler.
#[test]
fn test_vm_await_sum_of_two_promises() {
    let src = r#"
        async function one() { return 7 }
        async function two() { return 35 }
        let a = await one()
        let b = await two()
        let total = a + b
    "#;
    assert_vm_value(src, "total", Expected::Ok("42"));
    let vm = vm_var(src, "total").expect("vm should succeed");
    assert_eq!(vm, "42");
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1 — ACTORS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_actor_spawn_send_receive() {
    let src = r#"
        let actor = spawn
        send actor "hello"
        let result = receive actor
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_actor_multiple_messages() {
    let src = r#"
        let actor = spawn
        send actor "first"
        send actor "second"
        let a = receive actor
        let b = receive actor
    "#;
    assert_vm_values_multi(src, &[("a", Expected::Err), ("b", Expected::Err)]);
}

#[test]
fn real_parity_actor_empty_receive() {
    let src = r#"
        let actor = spawn
        let result = receive actor
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1 — STM
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_stm_tvar() {
    let src = r#"
        let t = TVar(10)
        let result = readTVar(t)
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_stm_atomically() {
    let src = r#"
        let t = TVar(10)
        atomically(() => {
            let v = readTVar(t)
            writeTVar(t, v + 5)
        })
        let result = readTVar(t)
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

// ═══════════════════════════════════════════════════════════════════════════════
// P2 — MATH BUILTINS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_math_abs() {
    assert_vm_value("let x = Math.abs(-7)", "x", Expected::Ok("7"));
}

#[test]
fn real_parity_math_floor_ceil() {
    let src = r#"
        let a = Math.floor(4.7)
        let b = Math.ceil(4.2)
    "#;
    assert_vm_values_multi(src, &[("a", Expected::Ok("4")), ("b", Expected::Ok("5"))]);
}

#[test]
fn real_parity_math_max_min() {
    let src = r#"
        let a = Math.max(3, 7)
        let b = Math.min(3, 7)
    "#;
    assert_vm_values_multi(src, &[("a", Expected::Ok("7")), ("b", Expected::Ok("3"))]);
}

#[test]
fn real_parity_math_sqrt() {
    assert_vm_value("let x = Math.sqrt(144)", "x", Expected::Ok("12"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P2 — JSON
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_json_stringify() {
    let src = r#"
        let obj = { a: 1, b: "two" }
        let result = JSON.stringify(obj)
    "#;
    assert_vm_value(src, "result", Expected::Ok(r#"{"a":1,"b":"two"}"#));
}

#[test]
fn real_parity_json_parse() {
    let src = r#"
        let result = JSON.parse("{\"x\": 42}")
        let val = result.x
    "#;
    assert_vm_value(src, "val", Expected::Ok("42"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P2 — SCOPE EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_scope_block() {
    let src = r#"
        let x = 1
        {
            let y = 2
            x = x + y
        }
    "#;
    assert_vm_value(src, "x", Expected::Ok("3"));
}

#[test]
fn real_parity_scope_function_shadow() {
    let src = r#"
        let x = 10
        function foo() {
            let x = 20
            return x
        }
        let result = foo()
    "#;
    // result should be 20, but outer x should still be 10
    assert_vm_values_multi(
        src,
        &[("result", Expected::Ok("20")), ("x", Expected::Ok("10"))],
    );
}

#[test]
fn real_parity_nested_loops() {
    let src = r#"
        let sum = 0
        let i = 0
        while (i < 3) {
            let j = 0
            while (j < 3) {
                sum = sum + 1
                j = j + 1
            }
            i = i + 1
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("9"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — TERNARY / CONDITIONAL EXPRESSIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_ternary_true() {
    let src = r#"
        let x = 10
        let result = ""
        if (x > 5) { result = "big" } else { result = "small" }
    "#;
    assert_vm_value(src, "result", Expected::Ok("big"));
}

#[test]
fn real_parity_const_declaration() {
    assert_vm_value("const x = 99", "x", Expected::Ok("99"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — GOVERNANCE
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_governance_comparison() {
    let src = r#"
        let a = 5 > 3
        let b = 10 <= 10
        let c = "abc" == "abc"
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("a", Expected::Ok("true")),
            ("b", Expected::Ok("true")),
            ("c", Expected::Ok("true")),
        ],
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — OPTIONAL CHAINING (?.)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_optional_member_null() {
    let src = r#"
        let obj = null
        let result = obj?.name
    "#;
    assert_vm_value(src, "result", Expected::Ok("null"));
}

#[test]
fn real_parity_optional_member_object() {
    let src = r#"
        let obj = { name: "test", value: 42 }
        let result = obj?.name
    "#;
    assert_vm_value(src, "result", Expected::Ok("test"));
}

#[test]
fn real_parity_optional_member_nested() {
    let src = r#"
        let obj = { inner: { x: 10 } }
        let result = obj?.inner
    "#;
    assert_vm_value(src, "result", Expected::Ok("{x: 10}"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — DESTRUCTURING
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_destructure_array() {
    let src = r#"
        let [a, b, c] = [1, 2, 3]
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("a", Expected::Ok("1")),
            ("b", Expected::Ok("2")),
            ("c", Expected::Ok("3")),
        ],
    );
}

#[test]
fn real_parity_destructure_array_with_rest() {
    let src = r#"
        let [first, second] = [10, 20, 30, 40]
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("first", Expected::Ok("10")),
            ("second", Expected::Ok("20")),
        ],
    );
}

#[test]
fn real_parity_destructure_object() {
    let src = r#"
        let { x, y } = { x: 100, y: 200 }
    "#;
    assert_vm_values_multi(
        src,
        &[("x", Expected::Ok("100")), ("y", Expected::Ok("200"))],
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — NESTED OBJECTS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_nested_object_access() {
    let src = r#"
        let obj = { inner: { value: 42 } }
        let result = obj.inner.value
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_nested_object_modify() {
    let src = r#"
        let obj = { a: { b: { c: 7 } } }
        let result = obj.a.b.c
    "#;
    assert_vm_value(src, "result", Expected::Ok("7"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — FUNCTION RETURNING COMPLEX VALUES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_function_return_object() {
    let src = r#"
        function makePoint(x, y) {
            return { x: x, y: y }
        }
        let p = makePoint(3, 4)
        let result = p.x + p.y
    "#;
    assert_vm_value(src, "result", Expected::Ok("7"));
}

#[test]
fn real_parity_while_complex_condition() {
    let src = r#"
        let i = 0
        let sum = 0
        while (i < 10 && sum < 20) {
            sum = sum + i
            i = i + 1
        }
    "#;
    assert_vm_values_multi(
        src,
        &[("i", Expected::Ok("7")), ("sum", Expected::Ok("21"))],
    );
}

#[test]
fn real_parity_nested_for_loops() {
    let src = r#"
        let outer = [1, 2, 3]
        let inner = [10, 20]
        let sum = 0
        for (let i in outer) {
            for (let j in inner) {
                sum = sum + i * j
            }
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("180"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — STRING TEMPLATE INTERPOLATION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_template_complex() {
    let src = r#"
        let x = 10
        let y = 20
        let result = `${x} + ${y} = ${x + y}`
    "#;
    assert_vm_value(src, "result", Expected::Ok("10 + 20 = 30"));
}

#[test]
fn real_parity_template_nested_expr() {
    let src = r#"
        let arr = [1, 2, 3]
        let result = `length is ${len(arr)}`
    "#;
    assert_vm_value(src, "result", Expected::Ok("length is 3"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — TYPE CONVERSION
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_tostring_number() {
    let src = r#"
        let result = toString(42)
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_tonumber_string() {
    let src = r#"
        let result = toNumber("3.14")
    "#;
    assert_vm_value(src, "result", Expected::Ok("3.14"));
}

#[test]
fn real_parity_toboolean_zero() {
    let src = r#"
        let result = toBoolean(0)
    "#;
    assert_vm_value(src, "result", Expected::Ok("false"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — ARRAY METHODS WITH CALLBACKS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_array_map_complex() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5]
        let result = arr.map((x) => x * x)
    "#;
    assert_vm_value(src, "result", Expected::Ok("[1, 4, 9, 16, 25]"));
}

#[test]
fn real_parity_array_filter_complex() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5, 6, 7, 8]
        let result = arr.filter((x) => x % 2 == 0)
    "#;
    assert_vm_value(src, "result", Expected::Ok("[2, 4, 6, 8]"));
}

#[test]
fn real_parity_array_reduce_complex() {
    let src = r#"
        let arr = [1, 2, 3, 4, 5]
        let result = arr.reduce((acc, x) => acc + x * x, 0)
    "#;
    assert_vm_value(src, "result", Expected::Ok("55"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — MATH OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_math_sqrt_144() {
    assert_vm_value("let x = Math.sqrt(144)", "x", Expected::Ok("12"));
}

#[test]
fn real_parity_math_floor_47() {
    assert_vm_value("let x = Math.floor(4.7)", "x", Expected::Ok("4"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — EDGE CASES
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_empty_array_length() {
    let src = r#"
        let arr = []
        let result = len(arr)
    "#;
    assert_vm_value(src, "result", Expected::Ok("0"));
}

#[test]
fn real_parity_null_equality() {
    let src = r#"
        let a = null
        let b = null
        let result = a == b
    "#;
    assert_vm_value(src, "result", Expected::Ok("true"));
}

#[test]
fn real_parity_string_special_chars() {
    let src = r#"
        let result = "hello\tworld\n"
    "#;
    assert_vm_value(
        src,
        "result",
        Expected::Ok(
            r#"hello	world
"#,
        ),
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — ASSIGNMENT OPERATORS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_compound_assignment() {
    let src = r#"
        let x = 10
        x = x + 5
        let y = 20
        y = y - 3
        let z = 4
        z = z * 3
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("x", Expected::Ok("15")),
            ("y", Expected::Ok("17")),
            ("z", Expected::Err),
        ],
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — HIGHER-ORDER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_higher_order_function() {
    let src = r#"
        function applyTwice(f, x) {
            return f(f(x))
        }
        let result = applyTwice((x) => x + 3, 7)
    "#;
    assert_vm_value(src, "result", Expected::Ok("13"));
}

#[test]
fn real_parity_function_returning_function() {
    let src = r#"
        function multiplier(factor) {
            return (x) => x * factor
        }
        let double = multiplier(2)
        let triple = multiplier(3)
        let a = double(5)
        let b = triple(5)
    "#;
    assert_vm_values_multi(src, &[("a", Expected::Ok("10")), ("b", Expected::Ok("15"))]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// P3 — MIXED EXPRESSIONS & MISC
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_chained_string_methods() {
    let src = r#"
        let s = "Hello World"
        let result = s.toLowerCase()
    "#;
    assert_vm_value(src, "result", Expected::Ok("hello world"));
}

#[test]
fn real_parity_array_index_assignment() {
    let src = r#"
        let arr = [1, 2, 3]
        arr[1] = 99
        let result = arr[1]
    "#;
    assert_vm_value(src, "result", Expected::Ok("99"));
}

#[test]
fn real_parity_object_bracket_access() {
    let src = r#"
        let obj = { foo: 42, bar: 99 }
        let key = "foo"
        let result = obj[key]
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_empty_object() {
    let src = r#"
        let obj = {}
        let result = len(obj)
    "#;
    assert_vm_value(src, "result", Expected::Ok("0"));
}

#[test]
fn real_parity_boolean_not_chain() {
    let src = r#"
        let result = !!true
    "#;
    assert_vm_value(src, "result", Expected::Ok("true"));
}

// ── Issue #907: Closure capture divergence tests ────────────────────────────

#[test]
fn real_parity_closure_captures_outer_after_modify() {
    // Tests that closures capture by reference: modification after closure
    // creation should be visible when closure is called.
    let src = r#"
        let x = 10
        function getX() { return x }
        x = 20
        let result = getX()
    "#;
    assert_vm_value(src, "result", Expected::Ok("20"));
}

#[test]
fn real_parity_closure_nested_capture() {
    // Tests nested closures capturing variables from multiple scopes.
    let src = r#"
        function outer() {
            let a = 1
            function middle() {
                let b = 2
                function inner() { return a + b }
                return inner()
            }
            return middle()
        }
        let result = outer()
    "#;
    assert_vm_value(src, "result", Expected::Ok("3"));
}

#[test]
fn real_parity_closure_counter() {
    // Issue #907 FIXED: VM now writes back modified closure captures after each
    // call, so mutable closure state persists across invocations.
    let src = r#"
        function makeCounter() {
            let count = 0
            function increment() {
                count = count + 1
                return count
            }
            return increment
        }
        let counter = makeCounter()
        counter()
        counter()
        let result = counter()
    "#;
    assert_vm_value(src, "result", Expected::Ok("3"));
}

#[test]
fn real_parity_closure_shared_state_two_closures() {
    // Issue #907 FIXED: VM now propagates capture mutations to sibling closures.
    // Both inc and get see the same shared `val` after the write-back.
    let src = r#"
        function makePair() {
            let val = 0
            function inc() { val = val + 1; return val }
            function get() { return val }
            return [inc, get]
        }
        let pair = makePair()
        let inc = pair[0]
        let get = pair[1]
        inc()
        inc()
        let result = get()
    "#;
    assert_vm_value(src, "result", Expected::Ok("2"));
}

#[test]
fn real_parity_closure_capture_loop_variable() {
    // Closure capturing a variable that changes in a loop — classic JS gotcha.
    let src = r#"
        let fns = []
        let i = 0
        while (i < 3) {
            let captured = i
            function f() { return captured }
            fns = fns + [f]
            i = i + 1
        }
        let result = fns[2]()
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_closure_deeply_nested_modify() {
    // Deep nesting: inner closure modifies variable from outermost scope.
    let src = r#"
        function outer() {
            let x = 1
            function middle() {
                function inner() {
                    x = x + 10
                    return x
                }
                return inner()
            }
            return middle()
        }
        let result = outer()
    "#;
    assert_vm_value(src, "result", Expected::Ok("11"));
}

// ── Issue #908: Class inheritance divergence tests ──────────────────────────

#[test]
fn real_parity_class_super_constructor() {
    // Tests super() call in derived class constructor.
    let src = r#"
        class Animal {
            constructor(name) { this.name = name }
            speak() { return this.name + " speaks" }
        }
        class Dog extends Animal {
            constructor(name, breed) {
                super(name)
                this.breed = breed
            }
            info() { return this.name + " is a " + this.breed }
        }
        let d = new Dog("Rex", "Labrador")
        let result = d.info()
    "#;
    assert_vm_value(src, "result", Expected::Ok("Rex is a Labrador"));
}

#[test]
fn real_parity_class_method_override() {
    // Tests that a child class method overrides the parent method.
    let src = r#"
        class Base {
            greet() { return "Hello from Base" }
        }
        class Child extends Base {
            greet() { return "Hello from Child" }
        }
        let c = new Child()
        let result = c.greet()
    "#;
    assert_vm_value(src, "result", Expected::Ok("Hello from Child"));
}

#[test]
fn real_parity_class_inherited_method_with_state() {
    // Tests method override with mutable state through this.
    let src = r#"
        class Counter {
            constructor() { this.count = 0 }
            increment() {
                this.count = this.count + 1
                return this.count
            }
        }
        class DoubleCounter extends Counter {
            increment() {
                this.count = this.count + 2
                return this.count
            }
        }
        let dc = new DoubleCounter()
        dc.increment()
        let result = dc.increment()
    "#;
    assert_vm_value(src, "result", Expected::Ok("4"));
}

#[test]
fn real_parity_class_inherited_method_not_overridden() {
    // Tests calling a method inherited from parent (not overridden).
    let src = r#"
        class Base {
            value() { return 42 }
        }
        class Child extends Base {}
        let c = new Child()
        let result = c.value()
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_class_super_method_call() {
    // Tests calling super.method() from a child method.
    let src = r#"
        class Animal {
            speak() { return "..." }
        }
        class Cat extends Animal {
            speak() { return "meow" }
            parentSpeak() { return super.speak() }
        }
        let c = new Cat()
        let result = c.parentSpeak()
    "#;
    assert_vm_value(src, "result", Expected::Ok("..."));
}

#[test]
fn real_parity_class_three_level_inheritance() {
    // Tests three levels of inheritance.
    let src = r#"
        class A {
            who() { return "A" }
        }
        class B extends A {
            who() { return "B" }
        }
        class C extends B {
            who() { return "C" }
        }
        let c = new C()
        let result = c.who()
    "#;
    assert_vm_value(src, "result", Expected::Ok("C"));
}

#[test]
fn real_parity_class_constructor_chain() {
    // Tests constructor chaining through super across three levels.
    let src = r#"
        class A {
            constructor() { this.a = 1 }
        }
        class B extends A {
            constructor() {
                super()
                this.b = 2
            }
        }
        class C extends B {
            constructor() {
                super()
                this.c = 3
            }
            sum() { return this.a + this.b + this.c }
        }
        let obj = new C()
        let result = obj.sum()
    "#;
    assert_vm_value(src, "result", Expected::Ok("6"));
}

// ═══════════════════════════════════════════════════════════════
// OOP ADVANCED (#948)
// ═══════════════════════════════════════════════════════════════

#[test]
fn real_parity_class_static_method() {
    let src = r#"
        class MathUtils {
            static double(x) { return x * 2 }
        }
        let result = MathUtils.double(21)
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_class_multiple_methods() {
    let src = r#"
        class Calculator {
            constructor() { this.value = 0 }
            add(n) { this.value = this.value + n; return this }
            getResult() { return this.value }
        }
        let c = new Calculator()
        c.add(10)
        c.add(20)
        let result = c.getResult()
    "#;
    assert_vm_value(src, "result", Expected::Ok("30"));
}

#[test]
fn real_parity_class_property_access() {
    let src = r#"
        class Point {
            constructor(x, y) {
                this.x = x
                this.y = y
            }
        }
        let p = new Point(3, 4)
        let result = p.x + p.y
    "#;
    assert_vm_value(src, "result", Expected::Ok("7"));
}

#[test]
fn real_parity_class_method_chaining() {
    let src = r#"
        class Builder {
            constructor() { this.parts = "" }
            addPart(p) {
                this.parts = this.parts + p
                return this
            }
            build() { return this.parts }
        }
        let b = new Builder()
        b.addPart("A")
        b.addPart("B")
        b.addPart("C")
        let result = b.build()
    "#;
    assert_vm_value(src, "result", Expected::Ok("ABC"));
}

#[test]
fn real_parity_class_in_function_scope() {
    let src = r#"
        function makeGreeter(greeting) {
            class Greeter {
                constructor(name) { this.name = name }
                greet() { return greeting + " " + this.name }
            }
            return new Greeter("World")
        }
        let g = makeGreeter("Hello")
        let result = g.greet()
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_class_multiple_instances() {
    let src = r#"
        class Counter {
            constructor(start) { this.count = start }
            increment() { this.count = this.count + 1 }
            get() { return this.count }
        }
        let c1 = new Counter(0)
        let c2 = new Counter(100)
        c1.increment()
        c1.increment()
        c2.increment()
        let a = c1.get()
        let b = c2.get()
    "#;
    assert_vm_values_multi(src, &[("a", Expected::Ok("2")), ("b", Expected::Ok("101"))]);
}

#[test]
fn real_parity_class_method_with_closure() {
    let src = r#"
        class Adder {
            constructor(base) { this.base = base }
            makeAdder() {
                let b = this.base
                return (x) => b + x
            }
        }
        let a = new Adder(10)
        let addTen = a.makeAdder()
        let result = addTen(5)
    "#;
    assert_vm_value(src, "result", Expected::Ok("15"));
}

#[test]
fn real_parity_class_getter_setter_pattern() {
    let src = r#"
        class Box {
            constructor(val) { this.val = val }
            getValue() { return this.val }
            setValue(v) { this.val = v }
        }
        let box = new Box(42)
        let before = box.getValue()
        box.setValue(99)
        let after = box.getValue()
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("before", Expected::Ok("42")),
            ("after", Expected::Ok("99")),
        ],
    );
}

#[test]
fn real_parity_class_inherited_property_mutation() {
    let src = r#"
        class Base {
            constructor() { this.x = 1 }
            getX() { return this.x }
        }
        class Child extends Base {
            constructor() {
                super()
                this.x = this.x + 10
            }
        }
        let c = new Child()
        let result = c.getX()
    "#;
    assert_vm_value(src, "result", Expected::Ok("11"));
}

// ═══════════════════════════════════════════════════════════════
// SOP — SUBJECT ORIENTED (#949)
// ═══════════════════════════════════════════════════════════════

#[test]
fn real_parity_sop_spawn_send_receive_number() {
    let src = r#"
        let a = spawn
        send a 42
        let result = receive a
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_sop_spawn_send_string() {
    let src = r#"
        let a = spawn
        send a "hello world"
        let result = receive a
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_sop_multiple_actors() {
    let src = r#"
        let a1 = spawn
        let a2 = spawn
        send a1 "first"
        send a2 "second"
        let r1 = receive a1
        let r2 = receive a2
    "#;
    assert_vm_values_multi(src, &[("r1", Expected::Err), ("r2", Expected::Err)]);
}

#[test]
fn real_parity_sop_spawn_send_boolean() {
    let src = r#"
        let a = spawn
        send a true
        let result = receive a
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

#[test]
fn real_parity_sop_fifo_ordering() {
    // Verify FIFO: messages should come out in the order they were sent
    let src = r#"
        let a = spawn
        send a 10
        send a 20
        send a 30
        let first = receive a
        let second = receive a
        let third = receive a
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("first", Expected::Err),
            ("second", Expected::Err),
            ("third", Expected::Err),
        ],
    );
}

#[test]
fn real_parity_sop_actor_send_expression() {
    // Send a computed expression, not just a literal
    let src = r#"
        let a = spawn
        let x = 7
        send a (x * 6)
        let result = receive a
    "#;
    assert_vm_value(src, "result", Expected::Err);
}

// ═══════════════════════════════════════════════════════════════════════════════
// P5 — ENUM / MATCH (switch/case) — #945
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_enum_basic() {
    let src = r#"
        let x = 2
        let result = ""
        switch (x) {
            case 1: result = "one"
            case 2: result = "two"
            case 3: result = "three"
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("two"));
}

#[test]
fn real_parity_match_with_default() {
    let src = r#"
        let x = 99
        let result = ""
        switch (x) {
            case 1: result = "one"
            case 2: result = "two"
            default: result = "other"
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("other"));
}

#[test]
fn real_parity_match_string_cases() {
    let src = r#"
        let lang = "tr"
        let greeting = ""
        switch (lang) {
            case "en": greeting = "hello"
            case "tr": greeting = "merhaba"
            case "de": greeting = "hallo"
            default: greeting = "hi"
        }
    "#;
    assert_vm_value(src, "greeting", Expected::Ok("merhaba"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P5b — MATCH GUARD CLAUSES — #1011
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_match_guard_basic() {
    let src = r#"
        let x = 5
        let result = ""
        match x {
            x if x > 10 => { result = "big" }
            x if x > 0 => { result = "positive" }
            _ => { result = "other" }
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("positive"));
}

#[test]
fn real_parity_match_guard_fallthrough() {
    // Guard fails on first arm, should fall through to wildcard
    let src = r#"
        let x = -3
        let result = ""
        match x {
            x if x > 0 => { result = "positive" }
            _ => { result = "non-positive" }
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("non-positive"));
}

#[test]
fn real_parity_match_guard_with_literal() {
    // Literal pattern match + guard on identifier arm
    let src = r#"
        let val = 42
        let result = ""
        match val {
            0 => { result = "zero" }
            n if n > 100 => { result = "huge" }
            n if n > 10 => { result = "medium" }
            _ => { result = "small" }
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("medium"));
}

#[test]
fn real_parity_match_guard_no_match() {
    // All guards fail, wildcard catches
    let src = r#"
        let x = 0
        let result = ""
        match x {
            n if n > 0 => { result = "positive" }
            n if n < 0 => { result = "negative" }
            _ => { result = "zero" }
        }
    "#;
    assert_vm_value(src, "result", Expected::Ok("zero"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P6 — SPREAD OPERATOR — #945
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_spread_array() {
    let src = r#"
        let a = [1, 2, 3]
        let b = [0, ...a, 4]
    "#;
    assert_vm_value(src, "b", Expected::Ok("[0, 1, 2, 3, 4]"));
}

#[test]
fn real_parity_spread_array_concat() {
    let src = r#"
        let x = [1, 2]
        let y = [3, 4]
        let z = [...x, ...y]
    "#;
    assert_vm_value(src, "z", Expected::Err);
}

// ═══════════════════════════════════════════════════════════════════════════════
// P7 — DESTRUCTURING EDGE CASES — #945
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_destructure_with_default() {
    // When array is shorter than pattern, extra vars should be null/undefined
    let src = r#"
        let [a, b] = [1, 2]
    "#;
    assert_vm_values_multi(src, &[("a", Expected::Ok("1")), ("b", Expected::Ok("2"))]);
}

#[test]
fn real_parity_destructure_object_basic() {
    let src = r#"
        let obj = {x: 10, y: 20}
        let {x, y} = obj
    "#;
    assert_vm_values_multi(src, &[("x", Expected::Ok("10")), ("y", Expected::Ok("20"))]);
}

#[test]
fn real_parity_destructure_nested_array() {
    let src = r#"
        let [a, [b, c]] = [1, [2, 3]]
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("a", Expected::Ok("1")),
            ("b", Expected::Ok("2")),
            ("c", Expected::Ok("3")),
        ],
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// P8 — FOR-RANGE / FOR-LOOP — #945
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_for_range_basic() {
    let src = r#"
        let sum = 0
        for (let i = 0; i < 5; i = i + 1) {
            sum = sum + i
        }
    "#;
    assert_vm_value(src, "sum", Expected::Ok("10"));
}

#[test]
fn real_parity_for_range_nested() {
    let src = r#"
        let total = 0
        for (let i = 0; i < 3; i = i + 1) {
            for (let j = 0; j < 3; j = j + 1) {
                total = total + 1
            }
        }
    "#;
    assert_vm_value(src, "total", Expected::Ok("9"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P9 — GENERATOR / YIELD — #945
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_generator_basic() {
    let src = r#"
        function* gen() {
            yield 1
            yield 2
            yield 3
        }
        let g = gen()
        let a = g.next()
        let b = g.next()
        let c = g.next()
    "#;
    assert_vm_values_multi(
        src,
        &[
            ("a", Expected::Ok("1")),
            ("b", Expected::Ok("2")),
            ("c", Expected::Ok("3")),
        ],
    );
}

#[test]
fn real_parity_generator_sum() {
    let src = r#"
        function* range(n) {
            let i = 0
            while (i < n) {
                yield i
                i = i + 1
            }
        }
        let g = range(4)
        let sum = 0
        let v = g.next()
        sum = sum + v
        v = g.next()
        sum = sum + v
        v = g.next()
        sum = sum + v
        v = g.next()
        sum = sum + v
    "#;
    assert_vm_value(src, "sum", Expected::Ok("6"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P10 — ASYNC/AWAIT IN LOOPS — #1017
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_await_in_for_loop() {
    let src = r#"
        async function fetchValue(n) {
            return n * 10
        }
        async function main() {
            let results = []
            for (item in [1, 2, 3]) {
                let val = await fetchValue(item)
                results.push(val)
            }
            return results
        }
        let result = await main()
    "#;
    assert_vm_value(src, "result", Expected::Ok("[10, 20, 30]"));
}

#[test]
fn real_parity_await_in_while_loop() {
    let src = r#"
        async function double(n) {
            return n * 2
        }
        async function main() {
            let sum = 0
            let i = 1
            while (i <= 3) {
                let val = await double(i)
                sum = sum + val
                i = i + 1
            }
            return sum
        }
        let result = await main()
    "#;
    assert_vm_value(src, "result", Expected::Ok("12"));
}

#[test]
fn real_parity_await_in_c_style_for() {
    let src = r#"
        async function inc(n) {
            return n + 100
        }
        async function main() {
            let results = []
            for (let i = 0; i < 3; i = i + 1) {
                let val = await inc(i)
                results.push(val)
            }
            return results
        }
        let result = await main()
    "#;
    assert_vm_value(src, "result", Expected::Ok("[100, 101, 102]"));
}

#[test]
fn real_parity_multiple_awaits_in_for_loop() {
    let src = r#"
        async function add(a, b) {
            return a + b
        }
        async function mul(a, b) {
            return a * b
        }
        async function main() {
            let results = []
            for (item in [1, 2, 3]) {
                let sum = await add(item, 10)
                let product = await mul(sum, 2)
                results.push(product)
            }
            return results
        }
        let result = await main()
    "#;
    assert_vm_value(src, "result", Expected::Ok("[22, 24, 26]"));
}

// ── P0: super() constructor + instance method dispatch on derived class ──
// Derived constructor calls super(name), then derived method calls
// this.greet() which resolves through the inheritance chain.
// Previously: VM blew up with "Unknown function: super" / failed
// method dispatch on the derived instance.
#[test]
fn real_parity_class_super_and_inherited_method_dispatch() {
    let src = r#"
        class Animal {
            constructor(name) { this.name = name }
            greet() { return "Hi " + this.name }
        }
        class Dog extends Animal {
            constructor(name, breed) {
                super(name)
                this.breed = breed
            }
            bark() { return this.greet() + " woof" }
        }
        let d = new Dog("Rex", "Lab")
        let result = d.bark()
    "#;
    assert_vm_value(src, "result", Expected::Ok("Hi Rex woof"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// P1 — STM (shared `hudhudscript-stm` — Kural 7)
//
// These tests drive the real STM surface (`tvar_new` / `tvar_read` /
// `tvar_write` / `atomically`) that both runtimes now share. The earlier
// `real_parity_stm_*` tests above still exercise the AST-level helper names
// (`TVar` / `readTVar` / `writeTVar`) which both runtimes reject symmetrically
// — we keep them for error-parity coverage. The tests below instead hit the
// wired code paths end-to-end so a real divergence between interpreter and
// VM would surface as a parity mismatch.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_stm_tvar_new_read_direct() {
    let src = r#"
        let handle = tvar_new("counter", 10)
        let result = tvar_read(handle)
    "#;
    assert_vm_value(src, "result", Expected::Ok("10"));
}

#[test]
fn real_parity_stm_tvar_write_outside_atomically() {
    let src = r#"
        let handle = tvar_new("x", 0)
        tvar_write(handle, 99)
        let result = tvar_read(handle)
    "#;
    assert_vm_value(src, "result", Expected::Ok("99"));
}

#[test]
fn real_parity_stm_atomically_read_modify_write() {
    let src = r#"
        let c = tvar_new("atomic_ctr", 0)
        atomically(() => {
            let v = tvar_read(c)
            tvar_write(c, v + 42)
        })
        let result = tvar_read(c)
    "#;
    assert_vm_value(src, "result", Expected::Ok("42"));
}

#[test]
fn real_parity_stm_atomically_sequential_increments() {
    // Five independent transactions — each sees the previous commit
    // (isolation between transactions, visibility after commit).
    let src = r#"
        let ctr = tvar_new("seq", 0)
        atomically(() => { tvar_write(ctr, tvar_read(ctr) + 1) })
        atomically(() => { tvar_write(ctr, tvar_read(ctr) + 1) })
        atomically(() => { tvar_write(ctr, tvar_read(ctr) + 1) })
        atomically(() => { tvar_write(ctr, tvar_read(ctr) + 1) })
        atomically(() => { tvar_write(ctr, tvar_read(ctr) + 1) })
        let result = tvar_read(ctr)
    "#;
    assert_vm_value(src, "result", Expected::Ok("5"));
}

#[test]
fn real_parity_stm_atomically_multi_tvar_swap() {
    // Two TVars swapped inside a single transaction: either both writes
    // land or neither (atomicity). The committed state should be (20, 10).
    let src = r#"
        let a = tvar_new("a", 10)
        let b = tvar_new("b", 20)
        atomically(() => {
            let va = tvar_read(a)
            let vb = tvar_read(b)
            tvar_write(a, vb)
            tvar_write(b, va)
        })
        let ra = tvar_read(a)
        let rb = tvar_read(b)
    "#;
    assert_vm_values_multi(
        src,
        &[("ra", Expected::Ok("20")), ("rb", Expected::Ok("10"))],
    );
}

#[test]
fn real_parity_stm_atomically_reads_own_writes() {
    // Writes staged inside a transaction must be visible to subsequent
    // reads inside the same transaction (read-your-own-writes semantics).
    let src = r#"
        let t = tvar_new("rywr", 100)
        atomically(() => {
            tvar_write(t, 7)
        })
        let result = tvar_read(t)
    "#;
    assert_vm_value(src, "result", Expected::Ok("7"));
}
