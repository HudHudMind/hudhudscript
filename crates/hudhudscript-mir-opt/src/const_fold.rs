//! Constant folding over MIR (JIT_AOT_ARCHITECTURE.md §6).
//!
//! In-place instruction replacement with STABLE value ids: `v2 = i64.add
//! v0, v1` where both operands are int constants becomes `v2 = const.i64
//! 5`. Later uses of `v2` remain valid; dead constants are removed by a
//! future DCE pass. The output always re-verifies.
//!
//! §18 semantics contract:
//! - signed integer overflow is NEVER folded (runtime overflow lane),
//! - integer division/rem by zero is NEVER folded (runtime error lane),
//! - IEEE-754 float ops fold only when bit-exact deterministic.

use hudhudscript_mir::{MirFunction, MirInst, MirType, ValueId};

/// Fold what is provably identical to execution; leave everything else
/// untouched. Returns the number of folded instructions.
pub fn const_fold(f: &MirFunction) -> (MirFunction, usize) {
    let mut out = f.clone();
    let mut folded = 0usize;
    for block in &mut out.blocks {
        // Constant environment: value id → literal, valid within this
        // block (linear SSA — AŞAMA-0 scope).
        let mut ints: Vec<Option<i64>> = Vec::new();
        let mut floats: Vec<Option<f64>> = Vec::new();

        let intern_int = |ints: &Vec<Option<i64>>, v: ValueId| -> Option<i64> {
            ints.get(v.0 as usize).copied().flatten()
        };
        let _ = intern_int;

        for inst in &mut block.insts {
            // Grow the environments to hold any new definition.
            if let Some(dst) = inst.result_value() {
                let idx = dst.0 as usize;
                while ints.len() <= idx {
                    ints.push(None);
                }
                while floats.len() <= idx {
                    floats.push(None);
                }
            }

            match inst.clone() {
                MirInst::ConstInt { dst, value, .. } => {
                    ints[dst.0 as usize] = Some(value);
                }
                MirInst::ConstFloat { dst, bits, .. } => {
                    floats[dst.0 as usize] = Some(f64::from_bits(bits));
                }
                MirInst::Add { dst, ty, lhs, rhs } => {
                    if ty == MirType::I64 {
                        if let (Some(a), Some(b)) = (get_int(&ints, lhs), get_int(&ints, rhs)) {
                            if let Some(sum) = a.checked_add(b) {
                                *inst = MirInst::ConstInt { dst, ty, value: sum };
                                ints[dst.0 as usize] = Some(sum);
                                folded += 1;
                            }
                        }
                    } else if ty == MirType::F64 {
                        if let (Some(a), Some(b)) = (get_float(&floats, lhs), get_float(&floats, rhs)) {
                            let s = a + b;
                            *inst = MirInst::ConstFloat { dst, ty, bits: s.to_bits() };
                            floats[dst.0 as usize] = Some(s);
                            folded += 1;
                        }
                    }
                }
                MirInst::Sub { dst, ty, lhs, rhs } => {
                    if ty == MirType::I64 {
                        if let (Some(a), Some(b)) = (get_int(&ints, lhs), get_int(&ints, rhs)) {
                            if let Some(diff) = a.checked_sub(b) {
                                *inst = MirInst::ConstInt { dst, ty, value: diff };
                                ints[dst.0 as usize] = Some(diff);
                                folded += 1;
                            }
                        }
                    }
                }
                MirInst::Mul { dst, ty, lhs, rhs } => {
                    if ty == MirType::I64 {
                        if let (Some(a), Some(b)) = (get_int(&ints, lhs), get_int(&ints, rhs)) {
                            if let Some(prod) = a.checked_mul(b) {
                                *inst = MirInst::ConstInt { dst, ty, value: prod };
                                ints[dst.0 as usize] = Some(prod);
                                folded += 1;
                            }
                        }
                    }
                }
                // Div/Rem: sıfırla bölme ve tamsayı taşması yürütme-zamanı
                // hata şeridine aittir — asla katlanmaz (§18).
                MirInst::Div { .. } | MirInst::Rem { .. } => {}
                _ => {}
            }
        }
    }
    (out, folded)
}

fn get_int(env: &[Option<i64>], v: ValueId) -> Option<i64> {
    env.get(v.0 as usize).copied().flatten()
}

fn get_float(env: &[Option<f64>], v: ValueId) -> Option<f64> {
    env.get(v.0 as usize).copied().flatten()
}
