//! Raw Instant-based measurements (printed to stderr for the summary table).

use std::hint::black_box;
use std::time::Instant;

use super::variant_a::{a_dispatch, a_make_array, a_make_number, a_make_string};
use super::variant_b::{b_dispatch, b_make_array, b_make_number, b_make_string};
use super::variant_c::{c_dispatch, c_make_array, c_make_number, c_make_string};

pub fn raw_table() {
    let iters_inline: usize = 10_000_000;
    let iters_heap: usize = 1_000_000;
    let iters_array: usize = 100_000;

    let a_s = a_make_string("hello world HudHudScript");
    let b_s = b_make_string("hello world HudHudScript");
    let c_s = c_make_string("hello world HudHudScript");

    let a_arr = a_make_array(10);
    let b_arr = b_make_array(10);
    let c_arr = c_make_array(10);

    let a_num = a_make_number(3.14);
    let b_num = b_make_number(3.14);
    let c_num = c_make_number(3.14);

    // Inline (Number) clone — 10M iters.
    let t = Instant::now();
    for _ in 0..iters_inline {
        let v = black_box(a_num.clone());
        drop(black_box(v));
    }
    let a_num_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    for _ in 0..iters_inline {
        let v = black_box(b_num.clone());
        drop(black_box(v));
    }
    let b_num_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    for _ in 0..iters_inline {
        let v = black_box(c_num.clone());
        drop(black_box(v));
    }
    let c_num_ns = t.elapsed().as_nanos();

    // Heap (String) clone — 1M iters.
    let t = Instant::now();
    for _ in 0..iters_heap {
        let v = black_box(a_s.clone());
        drop(black_box(v));
    }
    let a_str_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    for _ in 0..iters_heap {
        let v = black_box(b_s.clone());
        drop(black_box(v));
    }
    let b_str_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    for _ in 0..iters_heap {
        let v = black_box(c_s.clone());
        drop(black_box(v));
    }
    let c_str_ns = t.elapsed().as_nanos();

    // Array (Vec<10 Number>) clone — 100k iters.
    let t = Instant::now();
    for _ in 0..iters_array {
        let v = black_box(a_arr.clone());
        drop(black_box(v));
    }
    let a_arr_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    for _ in 0..iters_array {
        let v = black_box(b_arr.clone());
        drop(black_box(v));
    }
    let b_arr_ns = t.elapsed().as_nanos();
    let t = Instant::now();
    for _ in 0..iters_array {
        let v = black_box(c_arr.clone());
        drop(black_box(v));
    }
    let c_arr_ns = t.elapsed().as_nanos();

    // Fib-shape mixed pattern-match + drop.
    let fib_iters: usize = 1_000_000;

    let pool_a = build_pool_a();
    let pool_b = build_pool_b();
    let pool_c = build_pool_c();

    let t = Instant::now();
    let mut acc = 0u64;
    for i in 0..fib_iters {
        let v = black_box(pool_a[i & 15].clone());
        acc = acc.wrapping_add(a_dispatch(&v));
        drop(black_box(v));
    }
    black_box(acc);
    let a_fib_ns = t.elapsed().as_nanos();

    let t = Instant::now();
    let mut acc = 0u64;
    for i in 0..fib_iters {
        let v = black_box(pool_b[i & 15].clone());
        acc = acc.wrapping_add(b_dispatch(&v));
        drop(black_box(v));
    }
    black_box(acc);
    let b_fib_ns = t.elapsed().as_nanos();

    let t = Instant::now();
    let mut acc = 0u64;
    for i in 0..fib_iters {
        let v = black_box(pool_c[i & 15].clone());
        acc = acc.wrapping_add(c_dispatch(&v));
        drop(black_box(v));
    }
    black_box(acc);
    let c_fib_ns = t.elapsed().as_nanos();

    eprintln!("\n═══ RAW TIMINGS (Instant, single-threaded) ═══");
    eprintln!(
        "{:<10} {:>18} {:>18} {:>18} {:>18}",
        "variant", "inline 10M (ns)", "string 1M (ns)", "array 100k (ns)", "fib-mix 1M (ns)"
    );
    eprintln!(
        "{:<10} {:>18} {:>18} {:>18} {:>18}",
        "A base", a_num_ns, a_str_ns, a_arr_ns, a_fib_ns
    );
    eprintln!(
        "{:<10} {:>18} {:>18} {:>18} {:>18}",
        "B Arc", b_num_ns, b_str_ns, b_arr_ns, b_fib_ns
    );
    eprintln!(
        "{:<10} {:>18} {:>18} {:>18} {:>18}",
        "C Manual", c_num_ns, c_str_ns, c_arr_ns, c_fib_ns
    );

    let pct = |new: u128, base: u128| -> f64 { (new as f64 - base as f64) / base as f64 * 100.0 };
    eprintln!("\n── deltas vs A baseline ──");
    eprintln!(
        "B Arc    Δ: inline {:+.1}% | string {:+.1}% | array {:+.1}% | fib-mix {:+.1}%",
        pct(b_num_ns, a_num_ns),
        pct(b_str_ns, a_str_ns),
        pct(b_arr_ns, a_arr_ns),
        pct(b_fib_ns, a_fib_ns),
    );
    eprintln!(
        "C Manual Δ: inline {:+.1}% | string {:+.1}% | array {:+.1}% | fib-mix {:+.1}%",
        pct(c_num_ns, a_num_ns),
        pct(c_str_ns, a_str_ns),
        pct(c_arr_ns, a_arr_ns),
        pct(c_fib_ns, a_fib_ns),
    );
    eprintln!("═════════════════════════════════════════════════\n");
}

fn build_pool_a() -> Vec<hudhudscript_bytecode::Value16> {
    (0..16)
        .map(|i| {
            if i % 3 == 0 {
                a_make_number(i as f64)
            } else if i % 3 == 1 {
                a_make_string("x")
            } else {
                a_make_array(4)
            }
        })
        .collect()
}

fn build_pool_b() -> Vec<super::variant_b::ReprArc> {
    (0..16)
        .map(|i| {
            if i % 3 == 0 {
                b_make_number(i as f64)
            } else if i % 3 == 1 {
                b_make_string("x")
            } else {
                b_make_array(4)
            }
        })
        .collect()
}

fn build_pool_c() -> Vec<super::variant_c::ReprManual> {
    (0..16)
        .map(|i| {
            if i % 3 == 0 {
                c_make_number(i as f64)
            } else if i % 3 == 1 {
                c_make_string("x")
            } else {
                c_make_array(4)
            }
        })
        .collect()
}
