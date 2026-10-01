//! `dispatch_chunk5`'in register-karşılaştırma (D_INT_{EQ,LT,LE,NE}_RR) ve
//! local±immediate (D_INT_{SUB,ADD}_LOCAL_I) kolları — dispatch_general.rs
//! 400-satır anayasası için ayrı dosya. Yardımcılar `#[inline]`; aynı crate
//! içinde LLVM bunları `#[inline(always)]` dispatch_chunk5'e geri inline
//! eder, sıcak yol kodgen'i değişmez (H.6).

use crate::vm::{PackedResult, VM};
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

impl VM {
    /// Register-register karşılaştırma — Int/Int hızlı yolu daima ilk
    /// sırada (H.6.2); BigInt/String/Bool/Null merdiveni ve F3
    /// object_equality politikası merdivenin sonunda.
    #[inline(always)]
    pub(crate) fn int_rr_cmp_result(&self, dense: u8, src1: usize, src2: usize) -> bool {
        use crate::vm::dense_ops::*;
        let (t1, p1) = self.registers[src1].split_tag();
        let (t2, p2) = self.registers[src2].split_tag();
        match (t1, t2) {
            (ReprTag::Int, ReprTag::Int) => {
                let a = p1 as i64;
                let b = p2 as i64;
                match dense {
                    D_INT_EQ_RR => a == b,
                    D_INT_LT_RR => a < b,
                    D_INT_LE_RR => a <= b,
                    D_INT_NE_RR => a != b,
                    _ => unreachable!(),
                }
            }
            (ReprTag::Number, ReprTag::Number)
            | (ReprTag::Int, ReprTag::Number)
            | (ReprTag::Number, ReprTag::Int) => {
                let a = if t1 == ReprTag::Int {
                    p1 as i64 as f64
                } else {
                    f64::from_bits(p1)
                };
                let b = if t2 == ReprTag::Int {
                    p2 as i64 as f64
                } else {
                    f64::from_bits(p2)
                };
                match dense {
                    D_INT_EQ_RR => a == b,
                    D_INT_LT_RR => a < b,
                    D_INT_LE_RR => a <= b,
                    D_INT_NE_RR => a != b,
                    _ => unreachable!(),
                }
            }
            _ => {
                // BigInt comparison
                let v1 = self.registers[src1];
                let v2 = self.registers[src2];
                if let (Some(a), Some(b)) = (v1.to_bigint_value(), v2.to_bigint_value()) {
                    match dense {
                        D_INT_EQ_RR => a == b,
                        D_INT_LT_RR => a < b,
                        D_INT_LE_RR => a <= b,
                        D_INT_NE_RR => a != b,
                        _ => unreachable!(),
                    }
                } else if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                    match dense {
                        D_INT_EQ_RR => a == b,
                        D_INT_LT_RR => a < b,
                        D_INT_LE_RR => a <= b,
                        D_INT_NE_RR => a != b,
                        _ => unreachable!(),
                    }
                } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                    match dense {
                        D_INT_EQ_RR => a == b,
                        D_INT_NE_RR => a != b,
                        D_INT_LT_RR => !a && b,
                        D_INT_LE_RR => !a || a == b,
                        _ => unreachable!(),
                    }
                } else if v1.is_null() || v2.is_null() {
                    let both = v1.is_null() && v2.is_null();
                    match dense {
                        D_INT_EQ_RR => both,
                        D_INT_NE_RR => !both,
                        _ => false,
                    }
                } else {
                    // F3: Object/array equality — object_equality policy
                    let (t1, t2) = (v1.split_tag().0, v2.split_tag().0);
                    if t1 == ReprTag::Dynamic && t2 == ReprTag::Dynamic {
                        match self.object_equality {
                            crate::vm::config_types::ObjectEquality::Identity => {
                                let p1 = v1.split_tag().1;
                                let p2 = v2.split_tag().1;
                                match dense {
                                    D_INT_EQ_RR => p1 == p2,
                                    D_INT_NE_RR => p1 != p2,
                                    D_INT_LT_RR => p1 < p2,
                                    D_INT_LE_RR => p1 <= p2,
                                    _ => false,
                                }
                            }
                            crate::vm::config_types::ObjectEquality::Never => match dense {
                                D_INT_EQ_RR => false,
                                D_INT_NE_RR => true,
                                _ => false,
                            },
                            crate::vm::config_types::ObjectEquality::Deep => {
                                let eq = v1.values_equal(&v2);
                                match dense {
                                    D_INT_EQ_RR => eq,
                                    D_INT_NE_RR => !eq,
                                    _ => false,
                                }
                            }
                        }
                    } else {
                        // B8: non-Dynamic types — all comparison ops
                        match dense {
                            D_INT_EQ_RR => false,
                            D_INT_NE_RR => true,
                            _ => false,
                        }
                    }
                }
            }
        }
    }

    /// Local ± immediate süper komutu (D_INT_SUB_LOCAL_I | D_INT_ADD_LOCAL_I):
    /// slot register'ını immediate ile topla/çıkar, hedefe yaz. Overflow
    /// BigInt'e yükselir (G3.1), float yolu Number hattında.
    #[inline(always)]
    pub(crate) fn int_local_i_op(
        &mut self,
        dense: u8,
        dst: usize,
        payload_idx: u16,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<PackedResult> {
        use crate::vm::dense_ops::*;
        let payload = bytecode.get_super_instr_payload(payload_idx as u32);
        let slot_idx = payload.slot as usize;
        let (tag, p) = self.registers[slot_idx].split_tag();
        self.registers[dst] = match tag {
            ReprTag::Int => {
                let a = p as i64;
                let imm = payload.imm as i64;
                let r = if dense == D_INT_SUB_LOCAL_I {
                    a.checked_sub(imm)
                } else {
                    a.checked_add(imm)
                };
                match r {
                    Some(v) => Value16::int(v),
                    None => {
                        // G3.1: overflow → BigInt via bigint_arith
                        if dense == D_INT_SUB_LOCAL_I {
                            crate::vm::bigint_arith::int_sub(Value16::int(a), Value16::int(imm))
                                .unwrap_or_else(|_| Value16::null())
                        } else {
                            crate::vm::bigint_arith::int_add(Value16::int(a), Value16::int(imm))
                                .unwrap_or_else(|_| Value16::null())
                        }
                    }
                }
            }
            ReprTag::Number => {
                let a = f64::from_bits(p);
                let r = if dense == D_INT_SUB_LOCAL_I {
                    a - payload.imm as f64
                } else {
                    a + payload.imm as f64
                };
                Value16::number(r)
            }
            _ => {
                return Err(Self::runtime_error_with_pos(
                    "IntSubLocalI/IntAddLocalI: expected numeric local",
                    bytecode,
                    ip,
                ))
            }
        };
        Ok(PackedResult::Advance)
    }
}
