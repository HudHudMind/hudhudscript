//! Karşılaştırma çekirdekleri — G4 `cmp_rr_generic` (birleşik cmp+branch) ve
//! `IntCmpI*JumpIf*` (register vs immediate) ortak merdiveni.
//!
//! H.5/H.6: sıcak çekirdeklerde tek `split_tag`, `(Int, Int)` daima ilk sırada;
//! uyumsaz-tip çözümü `cmp_fallback.rs`'teki `#[cold]` fonksiyonda.
//!
//! PERF (v0.9.58): `cmp_rr_generic` imzası v0.9.53'e DÖNDÜ — 3 argüman.
//! v0.9.54'te eklenen `policy` 7. tamsayı argümanı SysV ABI'de STACK'ten
//! geçiyordu (sret + v1(2) + v2(2) + op = 6 register dolu); her packed
//! cmp+branch'ta stack store/load. Policy artık yalnızca Incompatible
//! sinyalinin soğuk çözücüsünde okunur (`cmp_fallback.rs`).

use crate::vm::VM;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::{Bytecode, ReprTag, Value16};

/// `cmp_rr_generic` merdiveninin uyumsuz-tip sinyali: çağıran bu hatayı
/// yakalayıp policy'i okuyan `#[cold]` çözücüye düşer (cmp_fallback.rs).
/// Sinyal kanalı bilinçli olarak `Err(&str)`: dönüş tipi `Result<bool, &str>`
/// kalır — v0.9.53 ABI'ı ile bayt-bayt aynı.
pub(crate) const CMP_RR_INCOMPATIBLE: &str = "cmp_rr: incompatible operand types";

/// G4 — TEK cmp çekirdeği (Kural 7): `IntCmpRRJumpIfFalse` (unpacked),
/// `IntCmpRRJumpPacked` (unpacked yolu) ve packed `D_INT_CMP_RR_JUMP_P`
/// AYNI karşılaştırma semantiğini buradan alır. op: 0 `<` 1 `<=` 2 `>`
/// 3 `>=` 4 `==` 5 `!=`. Uyumsuz tip çifti `Err(CMP_RR_INCOMPATIBLE)`
/// sinyali verir; çağıran F3-pariteli soğuk çözücüye düşer — Eq/Ne
/// `object_equality` policy'sine, sıralama op'ları false'a iner.
#[inline(always)]
pub(crate) fn cmp_rr_generic(
    v1: Value16,
    v2: Value16,
    op: u8,
) -> Result<bool, &'static str> {
    let (t1, p1) = v1.split_tag();
    let (t2, p2) = v2.split_tag();
    let cond = match (t1, t2) {
        (ReprTag::Int, ReprTag::Int) => {
            let a = p1 as i64;
            let b = p2 as i64;
            match op {
                0 => a < b,
                1 => a <= b,
                2 => a > b,
                3 => a >= b,
                4 => a == b,
                5 => a != b,
                _ => return Err("cmp_rr_generic: unknown op"),
            }
        }
        (ReprTag::Number, ReprTag::Number)
        | (ReprTag::Number, ReprTag::Int)
        | (ReprTag::Int, ReprTag::Number) => {
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
            match op {
                0 => a < b,
                1 => a <= b,
                2 => a > b,
                3 => a >= b,
                4 => a == b,
                5 => a != b,
                _ => return Err("cmp_rr_generic: unknown op"),
            }
        }
        // G9: iki taraf da INLINE string (≤15 bayt, payload'da) ise EQ/NE
        // bit karşılaştırmasıdır — inline temsil kanoniktir (aynı içerik =
        // aynı bitler), heap deref ve strcmp tamamen atlanır. Sıralama
        // op'ları (<, <= …) bayt sırası ≠ bit sırası olduğundan genel
        // merdivene düşer.
        (ReprTag::InlineString, ReprTag::InlineString) if op == 4 || op == 5 => {
            let eq = v1.0 == v2.0;
            if op == 4 {
                eq
            } else {
                !eq
            }
        }
        _ => {
            if let (Some(a), Some(b)) = (v1.to_bigint_value(), v2.to_bigint_value()) {
                match op {
                    0 => a < b,
                    1 => a <= b,
                    2 => a > b,
                    3 => a >= b,
                    4 => a == b,
                    5 => a != b,
                    _ => return Err("cmp_rr_generic: unknown op"),
                }
            } else if let (Some(a), Some(b)) = (v1.as_str(), v2.as_str()) {
                match op {
                    0 => a < b,
                    1 => a <= b,
                    2 => a > b,
                    3 => a >= b,
                    4 => a == b,
                    5 => a != b,
                    _ => return Err("cmp_rr_generic: unknown op"),
                }
            } else if let (Some(a), Some(b)) = (v1.as_bool(), v2.as_bool()) {
                match op {
                    4 => a == b,
                    5 => a != b,
                    0 => !a && b,
                    1 => !a || a == b,
                    2 => a && !b,
                    3 => a || a == b,
                    _ => return Err("cmp_rr_generic: unknown op"),
                }
            } else if v1.is_null() || v2.is_null() {
                let both_null = v1.is_null() && v2.is_null();
                match op {
                    4 => both_null,
                    5 => !both_null,
                    _ => false,
                }
            } else {
                // v0.9.54 3. regresyon: uyumsuz tipler hata DEĞİLDİR —
                // birleştirilmemiş IntCmp (F3) policy ile çözer. Sinyal
                // kanalıyla soğuk çözücüye devret (H.6.3).
                return Err(CMP_RR_INCOMPATIBLE);
            }
        }
    };
    Ok(cond)
}

impl VM {
    /// `IntCmpIJumpIfFalse` / `IntCmpIJumpIfTrue` ortak çekirdeği: src
    /// register'ı immediate ile karşılaştırır, koşulu döndürür (jump
    /// polaritesi çağıran arm'da). İki armın merdiveni birebir aynıydı —
    /// tek yerde (branch.rs ≤400 satır anayasası).
    #[inline(always)]
    pub(crate) fn int_cmp_i_cond(
        &self,
        src: u8,
        imm: i16,
        op: u8,
        instr: &'static str,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<bool> {
        let (tag, p) = self.registers[src as usize].split_tag();
        let cond = match tag {
            ReprTag::Int => {
                let a = p as i64;
                let b = imm as i64;
                match op {
                    0 => a < b,
                    1 => a <= b,
                    2 => a > b,
                    3 => a >= b,
                    4 => a == b,
                    5 => a != b,
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            &format!("{}: unknown op {}", instr, op),
                            bytecode,
                            ip,
                        ))
                    }
                }
            }
            ReprTag::Number => {
                let a = f64::from_bits(p);
                let b = imm as f64;
                match op {
                    0 => a < b,
                    1 => a <= b,
                    2 => a > b,
                    3 => a >= b,
                    4 => a == b,
                    5 => a != b,
                    _ => {
                        return Err(Self::runtime_error_with_pos(
                            &format!("{}: unknown op {}", instr, op),
                            bytecode,
                            ip,
                        ))
                    }
                }
            }
            ReprTag::Dynamic => {
                let v = self.registers[src as usize];
                if let Some(a) = v.to_bigint_value() {
                    let b = num_bigint::BigInt::from(imm as i64);
                    match op {
                        0 => a < b,
                        1 => a <= b,
                        2 => a > b,
                        3 => a >= b,
                        4 => a == b,
                        5 => a != b,
                        _ => {
                            return Err(Self::runtime_error_with_pos(
                                &format!("{}: unknown op {}", instr, op),
                                bytecode,
                                ip,
                            ))
                        }
                    }
                } else {
                    return Err(Self::runtime_error_with_pos(
                        &format!("{}: src not numeric", instr),
                        bytecode,
                        ip,
                    ));
                }
            }
            _ => {
                return Err(Self::runtime_error_with_pos(
                    &format!("{}: src not numeric", instr),
                    bytecode,
                    ip,
                ))
            }
        };
        Ok(cond)
    }
}
