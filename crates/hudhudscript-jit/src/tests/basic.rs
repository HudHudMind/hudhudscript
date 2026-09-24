//! Basic computation, control flow, and loop tests for JitRuntime.

use hudhudscript_native_abi::JIT_EXIT_RETURNED;

use crate::JitRuntime;

#[test]
fn jit_run_print() {
    let mut rt = JitRuntime::new().expect("runtime");
    let result = rt.run("function main() { print(42) }").expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
    assert_eq!(result.functions_compiled, 1);
}

#[test]
fn jit_run_computation() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function double(x) { return x * 2 }
        function main() { return double(21) }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
    assert_eq!(result.return_value, 42);
    assert_eq!(result.functions_compiled, 2);
}

#[test]
fn jit_run_control_flow() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let total = 0
            let i = 1
            while (i <= 10) { total = total + i; i = i + 1 }
            return total
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 55); // 1+2+...+10
}

#[test]
fn jit_top_level_code_works() {
    let mut rt = JitRuntime::new().expect("runtime");
    let result = rt.run("print(42)").expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
}

#[test]
fn jit_top_level_with_functions() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function double(x) { return x * 2 }
        let result = double(21)
        print(result)
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
}

#[test]
fn jit_main_still_works() {
    let mut rt = JitRuntime::new().expect("runtime");
    let result = rt.run("function main() { return 7 }").expect("run");
    assert_eq!(result.return_value, 7);
}

#[test]
fn jit_for_in_loop() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let total = 0
            let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            for (x in arr) { total = total + x }
            return total
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 55);
}

#[test]
fn jit_for_c_style() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let sum = 0
            for (let i = 0; i < 10; i = i + 1) { sum = sum + i }
            return sum
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 45);
}

#[test]
fn jit_ternary() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let a = 10
            let b = 20
            return a > b ? a : b
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 20);
}

#[test]
fn jit_ternary_nested_in_loop() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let acc = 0
            for (let i = 0; i < 10; i = i + 1) {
                acc = acc + (i % 2 == 0 ? i : -i)
            }
            return acc
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, -5);
}

#[test]
fn jit_toplevel_for_in() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let total = 0
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        for (x in arr) { total = total + x }
        print(total)
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
    assert_eq!(result.functions_compiled, 1);
}

#[test]
fn jit_toplevel_c_style_for() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let sum = 0
        for (let i = 0; i < 10; i = i + 1) { sum = sum + i }
        print(sum)
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
}

#[test]
fn jit_toplevel_string_ternary_compiles() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let t = 55
        let label = t > 50 ? "buyuk" : "kucuk"
        print(label)
        let s = "a" + (t > 50 ? "b" : "c")
        print(s)
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.exit_status, JIT_EXIT_RETURNED);
    assert_eq!(result.functions_compiled, 1);
}

#[test]
fn jit_increment_statement() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let n = 5
            n++
            n++
            return n
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 7);
}

#[test]
fn jit_nested_calls() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function double(x) { return x * 2 }
        function addone(x) { return x + 1 }
        function main() { return double(addone(double(10))) }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 42);
}

#[test]
fn jit_break_in_while() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let t = 0
            let k = 0
            while (k < 100) {
                k = k + 1
                if (k > 10) { break }
                t = t + k
            }
            return t * 100 + k
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 5511);
}

#[test]
fn jit_break_phi_values() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let acc = 0
            let n = 0
            while (n < 50) {
                n = n + 1
                acc = acc + n
                if (acc > 20) { break }
            }
            return acc
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 21);
}

#[test]
fn jit_continue_in_loop() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let t = 0
            let k = 0
            while (k < 10) {
                k = k + 1
                if (k % 2 == 0) { continue }
                t = t + k
            }
            return t * 10 + k
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 260);
}

#[test]
fn jit_logical_not_in_if() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let t = 0
            let i = 0
            while (i < 10) {
                i = i + 1
                if (!(i == 7)) { t = t + 1 }
            }
            return t
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 9);
}

#[test]
fn jit_bool_return_functions() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        fn is_even(n) { if (n == 0) { return true; } return is_odd(n - 1); }
        fn is_odd(n) { if (n == 0) { return false; } return is_even(n - 1); }
        function main() { if (is_even(10)) { return 1 } return 0 }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 1);
}
