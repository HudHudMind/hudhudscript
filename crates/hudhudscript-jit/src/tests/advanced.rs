//! Advanced data structures, objects, arrays, globals, and strings tests for JitRuntime.

use crate::JitRuntime;

#[test]
fn jit_object_literal_and_property() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let obj = { a: 10, b: 20 }
            return obj.a + obj.b
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 30);
}

#[test]
fn jit_object_property_set() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let obj = { x: 1 }
            obj.x = 41
            obj.y = 1
            return obj.x + obj.y
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 42);
}

#[test]
fn jit_object_in_loop_counter() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let stats = { toplam: 0 }
            for (let i = 1; i <= 10; i = i + 1) {
                stats.toplam = stats.toplam + i
            }
            return stats.toplam
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 55);
}

#[test]
fn jit_array_store_assignment() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let arr = [1, 2, 3]
            arr[0] = 10
            arr[2] = 30
            return arr[0] + arr[1] + arr[2]
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 42);
}

#[test]
fn jit_array_length_member() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let d = [1, 2, 3]
            d.push(4)
            return d.length * 10
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 40);
}

#[test]
fn jit_array_pop() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let d = [1, 2, 3]
            let a = d.pop()
            let b = d.pop()
            return a * 10 + b + d.length
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 33); // 3*10+2+1
}

#[test]
fn jit_module_level_globals() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let seed = 7
        fn bump(n) { seed = seed + n; return seed }
        let a = bump(5)
        print(a)
        print(bump(0))
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.exit_status, 0);
}

#[test]
fn jit_generic_param_conflict() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        fn use2(x, y) { return x + y }
        function main() { return use2(20, 22) }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 42);
}

#[test]
fn jit_string_index() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let a = "ax"
            let b = "xa"
            if (a[0] == b[1]) { return 1 }
            return 0
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 1);
}

#[test]
fn jit_string_length_property() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let s = "hello world"
            return s.length
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 11);
}

#[test]
fn jit_array_auto_resize_on_index_assign() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let board = []
            board[2] = 42
            return board[2] + board.length
        }
    "#;
    // index 2 causes resize to length 3; board[2] = 42 -> 42 + 3 = 45
    assert_eq!(rt.run(src).expect("run").return_value, 45);
}

#[test]
fn jit_global_assign_in_function_flow() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let seed = 100
        fn mutate_and_return(x) {
            seed = seed + x
            return seed * 2
        }
        function main() {
            return mutate_and_return(50)
        }
    "#;
    // seed becomes 150, returns 300
    assert_eq!(rt.run(src).expect("run").return_value, 300);
}

#[test]
fn jit_array_of_strings_concatenation() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let parts = []
            parts.push("abc")
            parts.push("def")
            let combined = parts[0] + parts[1]
            return combined.length
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 6);
}

#[test]
fn jit_string_equality_and_inequality() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let a = "hello"
            let b = "world"
            let c = "hello"
            let score = 0
            if (a == c) { score = score + 10 }
            if (a != b) { score = score + 20 }
            if (a != c) { score = score + 100 }
            return score
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 30);
}

#[test]
fn jit_array_join_and_substring() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function main() {
            let chars = []
            chars.push("H")
            chars.push("e")
            chars.push("y")
            let s = chars.join("")
            let sub = s.substring(1, 3)
            return sub.length
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 2);
}

#[test]
fn jit_typeof_operator() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function check(v) {
            if (typeof(v) == "number") { return 1; }
            if (typeof(v) == "string") { return 2; }
            if (typeof(v) == "object") { return 3; }
            return 0;
        }
        function main() {
            let obj = {};
            obj["x"] = 10;
            let a = check(42);
            let b = check("hud");
            let c = check(obj);
            return a * 100 + b * 10 + c;
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 123);
}

#[test]
fn jit_object_return_and_typeof() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function gen(d) {
            if (d == 1) { return 42; }
            let node = {};
            node["val"] = 99;
            return node;
        }
        function main() {
            let a = gen(1);
            let b = gen(0);
            let ta = typeof(a);
            let tb = typeof(b);
            let sa = (ta == "number") ? 10 : 0;
            let sb = (tb == "object") ? 1 : 0;
            return sa + sb;
        }
    "#;
    assert_eq!(rt.run(src).expect("run").return_value, 11);
}

#[test]
fn jit_array_map_filter_reduce() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        let base = [1, 2, 3, 4, 5];
        let d = base.map((x) => { return x * 2 + 1; });
        let f = d.filter((x) => { return x % 3 != 0; });
        let s = f.reduce((acc, x) => { return acc + x; }, 0);
        return s;
    "#;
    let result = rt.run(src).expect("run");
    // base = [1, 2, 3, 4, 5]
    // d = [3, 5, 7, 9, 11]
    // f = [5, 7, 11] (3 and 9 filtered out because x % 3 == 0)
    // s = 0 + 5 + 7 + 11 = 23
    assert_eq!(result.return_value, 23);
}

#[test]
fn jit_class_method_dispatch() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        class Shape { public fn score() { return 0; } }
        class A extends Shape {
            public fn score() { return self.v * 2; }
        }
        class B extends Shape {
            public fn score() { return self.v * 3 + 1; }
        }

        let a = new A();
        a.v = 10;
        let b = new B();
        b.v = 20;
        return a.score() + b.score();
    "#;
    let result = rt.run(src).expect("run");
    // a.score() = 10 * 2 = 20
    // b.score() = 20 * 3 + 1 = 61
    // total = 81
    assert_eq!(result.return_value, 81);
}

#[test]
fn jit_closure_counter() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function make_counter(start) {
            let c = start;
            return () => {
                c = c + 1;
                return c;
            };
        }
        function main() {
            let ctr = make_counter(10);
            return ctr() + ctr() + ctr();
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 36);
}

#[test]
fn jit_try_catch_throw() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        function fail_if_zero(x) {
            if (x == 0) {
                throw "zero";
            }
            return 100 / x;
        }
        function main() {
            let caught = 0;
            let sum = 0;
            let i = 0;
            while (i < 5) {
                try {
                    sum = sum + fail_if_zero(i);
                } catch (e) {
                    caught = caught + 1;
                }
                i = i + 1;
            }
            return caught * 1000 + sum;
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 1208);
}

#[test]
fn jit_subject_sop_abilities() {
    let mut rt = JitRuntime::new().expect("runtime");
    let src = r#"
        subject Worker {
            can compute,
            on compute(n) {
                return n * 3 + 1;
            }
        }
        function main() {
            let w = spawn Worker;
            return w.compute(7);
        }
    "#;
    let result = rt.run(src).expect("run");
    assert_eq!(result.return_value, 22);
}

#[cfg(feature = "llvm")]
#[test]
fn jit_llvm_backend_runs() {
    let mut rt = JitRuntime::with_backend("llvm").expect("backend");
    let result = rt.run("function main() { return 6 * 7 }").expect("run");
    assert_eq!(result.return_value, 42);
}
