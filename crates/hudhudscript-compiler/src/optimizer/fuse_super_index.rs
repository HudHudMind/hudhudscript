//! Index ailesi süper-eritmeleri (Index2D / IndexAssign2D).
//!
//! `fuse_super_extra.rs` 400 satır sınırını aştığı için index füzyonları
//! buraya ayrıldı (dosya boyutu anayasası).

use super::fuse_super_extra::remove_at;
use hudhudscript_bytecode::{Instruction, LoopPayload};
type SourcePositions = Vec<Option<(usize, usize)>>;

pub(super) fn try_index2d(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    // P3: also match IndexArray (P1) for nested fusion.
    // IndexStringAscii is intentionally NOT fused — string-of-strings is not a 2D pattern.
    //
    // B1 (filo raporu): sadece TABANI derleyici tarafından kanıtlanmış dizi
    // olan çiftler eritilir. `IndexArray` yalnızca `ct_local_type == Array`
    // kanıtıyla yayılır (compile_reg.rs / compile_complex.rs); genel `Index`
    // tabanı harita olabilir (`m["k"][i]`). Index2D VM'de yalnızca
    // dizi-içinden-dizi okur (`obj not array` / `row not array` hatası
    // fırlatır) — kanıt yoksa eritme, genel Index dizisi kalsın: hem harita
    // hem dizi doğru çalışır. Dizi-kanıtlı hız yolu (matris `a[i][k]`) korunur.
    let (mid, outer, idx1, inner2, dst, idx2) = match (&instructions[i], &instructions[i + 1]) {
        (
            Instruction::IndexArray {
                dst: m,
                obj: o,
                idx: i1,
            },
            Instruction::IndexArray {
                dst,
                obj: inn,
                idx: i2,
            },
        ) => (*m, *o, *i1, *inn, *dst, *i2),
        (
            Instruction::IndexArray {
                dst: m,
                obj: o,
                idx: i1,
            },
            Instruction::Index {
                dst,
                obj: inn,
                idx: i2,
            },
        ) => (*m, *o, *i1, *inn, *dst, *i2),
        _ => return false,
    };
    if mid == inner2 {
        instructions[i] = Instruction::Index2D {
            dst,
            obj: outer,
            idx1,
            idx2,
        };
        remove_at(instructions, loop_payloads, source_positions, i + 1);
        return true;
    }
    false
}

pub(super) fn try_index_assign2d(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    // P3: also match IndexArray (P1) for nested fusion
    //
    // B1 (filo raporu): yalnız TABANI dizi-kanıtlı (`IndexArray` başlangıçlı)
    // çiftler eritilir. Genel `Index` başlangıçlı çiftlerde taban harita
    // olabilir (`m["k"][i] = v`) — IndexAssign2D VM'de tabanın dizi olmasını
    // şart koşar; kanıt yoksa eritme, genel dizi kalsın.
    match (&instructions[i], &instructions[i + 1]) {
        (
            Instruction::IndexArray {
                dst: row,
                obj: outer,
                idx: idx1,
            },
            Instruction::IndexAssignArray {
                obj: inner,
                idx: idx2,
                val,
            },
        )
        | (
            Instruction::IndexArray {
                dst: row,
                obj: outer,
                idx: idx1,
            },
            Instruction::IndexAssign {
                obj: inner,
                idx: idx2,
                val,
            },
        ) => {
            if *row == *inner {
                instructions[i] = Instruction::IndexAssign2D {
                    obj: *outer,
                    idx1: *idx1,
                    idx2: *idx2,
                    val: *val,
                };
                remove_at(instructions, loop_payloads, source_positions, i + 1);
                return true;
            }
        }
        _ => {}
    }

    if i + 2 >= instructions.len() {
        return false;
    }
    let middle = instructions[i + 1].clone();
    if !matches!(
        &middle,
        Instruction::LoadIntConst { .. } | Instruction::LoadConst { .. }
    ) {
        return false;
    }
    // 3-komutluk pencere: IndexArray + (sabit yükleme) + IndexAssign.
    // Yalnız dizi-kanıtlı taban (ilk komut IndexArray) eritilir (B1).
    let fused = if let (
        Instruction::IndexArray {
            dst: row,
            obj: outer,
            idx: idx1,
        },
        Instruction::IndexAssign {
            obj: inner,
            idx: idx2,
            val,
        },
    ) = (&instructions[i], &instructions[i + 2])
    {
        let middle_dst = match &middle {
            Instruction::LoadIntConst { dst, .. } | Instruction::LoadConst { dst, .. } => *dst,
            _ => unreachable!(),
        };
        if *row == *inner && middle_dst == *val {
            Some(Instruction::IndexAssign2D {
                obj: *outer,
                idx1: *idx1,
                idx2: *idx2,
                val: *val,
            })
        } else {
            None
        }
    } else {
        None
    };
    if let Some(instr) = fused {
        instructions[i] = middle;
        instructions[i + 1] = instr;
        remove_at(instructions, loop_payloads, source_positions, i + 2);
        return true;
    }
    false
}
