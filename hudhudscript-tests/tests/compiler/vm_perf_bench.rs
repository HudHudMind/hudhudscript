//! VM performance benchmark — not a correctness test.
//!
//! Runs fib(30) on the VM and prints the wall-clock duration. Use with
//! `cargo test --release -p hudhudscript-compiler --test vm_perf_bench -- --nocapture`.
//!
//! This file only adds new tests (Kural 1 compliant). It exists so we can
//! measure VM optimisations (scope pool, indexed locals, etc.) against a
//! fixed workload.

use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;
use std::time::Instant;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn compile_src(src: &str) -> hudhudscript_bytecode::Bytecode {
    let ast = parse(src).expect("parse failed");
    let mut compiler = Compiler::new();
    compiler.compile(&ast).expect("compile failed")
}

#[test]
fn bench_vm_fib30() {
    let src = r#"
let fib = (n) => {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
};

let result = fib(30);
"#;
    let bc = compile_src(src);

    // Warm-up to JIT caches etc.
    {
        let mut vm = VM::new();
        vm.execute(&bc).expect("vm execute failed");
    }

    let mut best = std::time::Duration::from_secs(3600);
    let mut total = std::time::Duration::ZERO;
    let iters = 3;
    for _ in 0..iters {
        let mut vm = VM::new();
        let t0 = Instant::now();
        vm.execute(&bc).expect("vm execute failed");
        let dt = t0.elapsed();
        if dt < best {
            best = dt;
        }
        total += dt;
        let v = vm
            .get_variable("result")
            .cloned()
            .unwrap_or(hudhudscript_bytecode::Value16::null());
        // A3c: integer-valued results now stay as `Value::Int(i64)` through
        // the Int fast-path (no f64 widening round-trip).  fib(30) = 832040
        // fits in `i64` end-to-end.  Accept either representation for
        // backward compat with A3a/A3b Number-only result shape.
        assert!(
            v.as_int() == Some(832040)
                || v.as_number().map_or(false, |n| (n - 832040.0).abs() < 0.5),
            "fib(30) expected 832040 (Int or Number), got {:?}",
            v
        );
    }
    eprintln!(
        "\n[BENCH] VM fib(30) — best of {iters}: {:?} | avg: {:?}",
        best,
        total / iters as u32
    );
}

#[test]
fn bench_vm_fib28() {
    let src = r#"
let fib = (n) => {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
};

let result = fib(28);
"#;
    let bc = compile_src(src);

    // Warm-up
    {
        let mut vm = VM::new();
        vm.execute(&bc).expect("vm execute failed");
    }

    let mut best = std::time::Duration::from_secs(3600);
    let mut total = std::time::Duration::ZERO;
    let iters = 5;
    for _ in 0..iters {
        let mut vm = VM::new();
        let t0 = Instant::now();
        vm.execute(&bc).expect("vm execute failed");
        let dt = t0.elapsed();
        if dt < best {
            best = dt;
        }
        total += dt;
    }
    eprintln!(
        "\n[BENCH] VM fib(28) — best of {iters}: {:?} | avg: {:?}",
        best,
        total / iters as u32
    );
}
