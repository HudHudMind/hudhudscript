//! Value Repr microbench — Struct-3 (Value 16B refactor) prerequisite
//!
//! Evaluates three candidate layouts under a fib-shape hot loop
//! (recursive tree: clone + drop heavy) to decide whether the
//! Rune-style single-indirection Repr is worth pursuing after the
//! failed C1 per-variant Box flip (perf_history.json entry 32 — +11%).
//!
//! Scope: SADECE benches/ — production code is untouched.

use criterion::{criterion_group, criterion_main, Criterion};

mod bench_groups;
mod raw_table;
mod size_report;
mod variant_a;
mod variant_b;
mod variant_c;

fn setup(c: &mut Criterion) {
    size_report::print_size_report();
    raw_table::raw_table();
    bench_groups::bench_inline_clone(c);
    bench_groups::bench_string_clone(c);
    bench_groups::bench_array_clone(c);
    bench_groups::bench_fib_mix(c);
}

criterion_group!(value_repr, setup);
criterion_main!(value_repr);
