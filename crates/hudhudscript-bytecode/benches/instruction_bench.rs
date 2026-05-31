//! Bytecode instruction benchmarks (Issue #1061, P7.4).
//!
//! Measures hot-path throughput for instruction packing, symbol
//! interning, constant pool dedup, and serialization.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hudhudscript_bytecode::packed_instruction::{pack, unpack};
use hudhudscript_bytecode::{Bytecode, Instruction, SymId, Value};

// ── Helpers ────────────────────────────────────────────────────────

/// Build a representative mix of 1000 packable instructions.
fn make_instructions() -> Vec<Instruction> {
    let mut instrs = Vec::with_capacity(1000);
    for i in 0..1000u16 {
        let instr = match i % 10 {
            0 => Instruction::LoadConst { dst: 0, const_idx: i as u16 },
            1 => Instruction::IntAdd { dst: 0, src1: 1, src2: 2 },
            2 => Instruction::LoadGlobal { dst: 0, sym: i as u32 },
            3 => Instruction::StoreGlobal { src: 0, sym: i as u32 },
            4 => Instruction::JumpIfFalse { src: 255, offset: i as i16 },
            // CROSS-2c: Call now carries a u32 call_payloads index.
            5 => Instruction::Call { dst: 0, payload_idx: i as u32, first_arg: 0, arg_count: 0 },
            6 => Instruction::GetProperty { obj: 255, field: SymId(i as u32) },
            7 => Instruction::NumAdd { dst: 0, src1: 1, src2: 2 },
            8 => Instruction::MakeArray { dst: 0, count: *i as u16 },
            9 => Instruction::Return { src: 255 },
            _ => unreachable!(),
        };
        instrs.push(instr);
    }
    instrs
}

/// Build a Bytecode with populated symbol_lists for lookup benchmarks.
fn make_bytecode_with_symbol_lists() -> Bytecode {
    let mut bc = Bytecode::new();
    for i in 0..100u32 {
        let list: Vec<SymId> = (0..10).map(|j| SymId(i * 10 + j)).collect();
        bc.add_symbol_list(list);
    }
    bc
}

/// Build a Bytecode with constants and instructions for serialization.
fn make_bytecode_for_serialization() -> Bytecode {
    let mut bc = Bytecode::new();
    for i in 0..200 {
        bc.add_constant(Value::Number(i as f64));
        bc.add_constant(Value::string(format!("str_{}", i)));
    }
    for i in 0..500u16 {
        bc.instructions.push(match i % 4 {
            0 => Instruction::LoadConst((i as usize) % 200),
            1 => Instruction::Add,
            2 => Instruction::LoadVar(i as u32),
            3 => Instruction::Return,
            _ => unreachable!(),
        });
    }
    // Add some symbols
    for i in 0..50 {
        bc.intern_symbol(&format!("var_{}", i));
    }
    bc
}

// ── Benchmarks ─────────────────────────────────────────────────────

/// 1. Pack/unpack round-trip throughput (1000 instructions).
fn bench_pack_unpack_round_trip(c: &mut Criterion) {
    let instrs = make_instructions();

    c.bench_function("pack_1000_instructions", |b| {
        b.iter(|| {
            for instr in &instrs {
                black_box(pack(instr));
            }
        })
    });

    // Pre-pack for unpack benchmark
    let packed: Vec<u32> = instrs.iter().filter_map(|i| pack(i)).collect();

    c.bench_function("unpack_1000_instructions", |b| {
        b.iter(|| {
            for &p in &packed {
                black_box(unpack(p));
            }
        })
    });

    c.bench_function("pack_unpack_round_trip_1000", |b| {
        b.iter(|| {
            for instr in &instrs {
                if let Some(p) = pack(instr) {
                    black_box(unpack(p));
                }
            }
        })
    });
}

/// 2. Symbol list lookup throughput.
fn bench_symbol_list_lookup(c: &mut Criterion) {
    let bc = make_bytecode_with_symbol_lists();

    c.bench_function("symbol_list_lookup_100", |b| {
        b.iter(|| {
            for i in 0..100u32 {
                black_box(bc.get_symbol_list(i));
            }
        })
    });

    c.bench_function("symbol_list_add_dedup", |b| {
        b.iter(|| {
            let mut bc = Bytecode::new();
            // Add 50 unique lists then re-add them (dedup path)
            for i in 0..50u32 {
                let list: Vec<SymId> = (0..5).map(|j| SymId(i * 5 + j)).collect();
                bc.add_symbol_list(list);
            }
            for i in 0..50u32 {
                let list: Vec<SymId> = (0..5).map(|j| SymId(i * 5 + j)).collect();
                black_box(bc.add_symbol_list(list));
            }
        })
    });
}

/// 3. SymId intern/resolve throughput.
fn bench_symid_intern_resolve(c: &mut Criterion) {
    // Pre-intern some symbols
    let names: Vec<String> = (0..200).map(|i| format!("symbol_{}", i)).collect();
    let ids: Vec<SymId> = names.iter().map(|n| SymId::from(n.as_str())).collect();

    c.bench_function("symid_intern_200", |b| {
        b.iter(|| {
            for name in &names {
                black_box(SymId::from(name.as_str()));
            }
        })
    });

    c.bench_function("symid_resolve_200", |b| {
        b.iter(|| {
            for id in &ids {
                black_box(id.resolve());
            }
        })
    });
}

/// 4. Bytecode serialization throughput (to_bytes / from_bytes).
fn bench_bytecode_serialization(c: &mut Criterion) {
    let bc = make_bytecode_for_serialization();
    let bytes = bc.to_bytes().expect("serialization should succeed");

    c.bench_function("bytecode_to_bytes", |b| {
        b.iter(|| {
            black_box(bc.to_bytes().unwrap());
        })
    });

    c.bench_function("bytecode_from_bytes", |b| {
        b.iter(|| {
            black_box(Bytecode::from_bytes(&bytes).unwrap());
        })
    });
}

/// 5. Constant pool dedup throughput.
fn bench_constant_pool_dedup(c: &mut Criterion) {
    c.bench_function("constant_pool_dedup_200", |b| {
        b.iter(|| {
            let mut bc = Bytecode::new();
            // First pass: add 200 unique constants
            for i in 0..200 {
                bc.add_constant(Value::Number(i as f64));
            }
            // Second pass: re-add same 200 (all dedup hits)
            for i in 0..200 {
                black_box(bc.add_constant(Value::Number(i as f64)));
            }
        })
    });

    c.bench_function("constant_pool_dedup_strings_100", |b| {
        b.iter(|| {
            let mut bc = Bytecode::new();
            for i in 0..100 {
                bc.add_constant(Value::string(format!("const_{}", i)));
            }
            // Re-add same strings (dedup path)
            for i in 0..100 {
                black_box(bc.add_constant(Value::string(format!("const_{}", i))));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_pack_unpack_round_trip,
    bench_symbol_list_lookup,
    bench_symid_intern_resolve,
    bench_bytecode_serialization,
    bench_constant_pool_dedup,
);
criterion_main!(benches);
