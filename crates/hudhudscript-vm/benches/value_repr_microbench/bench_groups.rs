//! Criterion benchmark groups.

use criterion::Criterion;
use std::hint::black_box;

use super::variant_a::{a_make_array, a_make_number, a_make_string};
use super::variant_b::{b_make_array, b_make_number, b_make_string};
use super::variant_c::{c_make_array, c_make_number, c_make_string};

pub fn bench_inline_clone(c: &mut Criterion) {
    let a = a_make_number(3.14);
    let b = b_make_number(3.14);
    let cc = c_make_number(3.14);

    let mut g = c.benchmark_group("inline_number_clone");
    g.bench_function("A_baseline", |bench| {
        bench.iter(|| {
            let v = black_box(a.clone());
            drop(black_box(v));
        })
    });
    g.bench_function("B_arc", |bench| {
        bench.iter(|| {
            let v = black_box(b.clone());
            drop(black_box(v));
        })
    });
    g.bench_function("C_manual", |bench| {
        bench.iter(|| {
            let v = black_box(cc.clone());
            drop(black_box(v));
        })
    });
    g.finish();
}

pub fn bench_string_clone(c: &mut Criterion) {
    let a = a_make_string("hello world HudHudScript");
    let b = b_make_string("hello world HudHudScript");
    let cc = c_make_string("hello world HudHudScript");

    let mut g = c.benchmark_group("string_heap_clone");
    g.bench_function("A_baseline", |bench| {
        bench.iter(|| {
            let v = black_box(a.clone());
            drop(black_box(v));
        })
    });
    g.bench_function("B_arc", |bench| {
        bench.iter(|| {
            let v = black_box(b.clone());
            drop(black_box(v));
        })
    });
    g.bench_function("C_manual", |bench| {
        bench.iter(|| {
            let v = black_box(cc.clone());
            drop(black_box(v));
        })
    });
    g.finish();
}

pub fn bench_array_clone(c: &mut Criterion) {
    let a = a_make_array(10);
    let b = b_make_array(10);
    let cc = c_make_array(10);

    let mut g = c.benchmark_group("array10_clone");
    g.bench_function("A_baseline", |bench| {
        bench.iter(|| {
            let v = black_box(a.clone());
            drop(black_box(v));
        })
    });
    g.bench_function("B_arc", |bench| {
        bench.iter(|| {
            let v = black_box(b.clone());
            drop(black_box(v));
        })
    });
    g.bench_function("C_manual", |bench| {
        bench.iter(|| {
            let v = black_box(cc.clone());
            drop(black_box(v));
        })
    });
    g.finish();
}

pub fn bench_fib_mix(c: &mut Criterion) {
    let pool_a: Vec<hudhudscript_bytecode::Value16> = (0..16)
        .map(|i| {
            if i % 3 == 0 {
                a_make_number(i as f64)
            } else if i % 3 == 1 {
                a_make_string("x")
            } else {
                a_make_array(4)
            }
        })
        .collect();
    let pool_b: Vec<super::variant_b::ReprArc> = (0..16)
        .map(|i| {
            if i % 3 == 0 {
                b_make_number(i as f64)
            } else if i % 3 == 1 {
                b_make_string("x")
            } else {
                b_make_array(4)
            }
        })
        .collect();
    let pool_c: Vec<super::variant_c::ReprManual> = (0..16)
        .map(|i| {
            if i % 3 == 0 {
                c_make_number(i as f64)
            } else if i % 3 == 1 {
                c_make_string("x")
            } else {
                c_make_array(4)
            }
        })
        .collect();

    let mut g = c.benchmark_group("fib_shape_mix");
    g.bench_function("A_baseline", |bench| {
        let mut i = 0usize;
        bench.iter(|| {
            let v = black_box(pool_a[i & 15].clone());
            i = i.wrapping_add(1);
            drop(black_box(v));
        })
    });
    g.bench_function("B_arc", |bench| {
        let mut i = 0usize;
        bench.iter(|| {
            let v = black_box(pool_b[i & 15].clone());
            i = i.wrapping_add(1);
            drop(black_box(v));
        })
    });
    g.bench_function("C_manual", |bench| {
        let mut i = 0usize;
        bench.iter(|| {
            let v = black_box(pool_c[i & 15].clone());
            i = i.wrapping_add(1);
            drop(black_box(v));
        })
    });
    g.finish();
}
