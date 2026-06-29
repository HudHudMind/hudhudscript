use crate::optimizer::utils::adjust_jumps_after_remove;
use hudhudscript_bytecode::{Instruction, LoopPayload};
type SourcePositions = Vec<Option<(usize, usize)>>;

pub(super) fn try_fuse_extra_pattern(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    try_array_push_const(instructions, loop_payloads, source_positions, i)
        || try_index2d(instructions, loop_payloads, source_positions, i)
        || try_index_assign2d(instructions, loop_payloads, source_positions, i)
        || try_int_mul_add_assign(instructions, loop_payloads, source_positions, i)
        || try_property_assign(instructions, loop_payloads, source_positions, i)
        || try_strcat(instructions, loop_payloads, source_positions, i)
}
fn remove_at(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    idx: usize,
) {
    adjust_jumps_after_remove(instructions, loop_payloads, idx);
    instructions.remove(idx);
    if idx < source_positions.len() {
        source_positions.remove(idx);
    }
}
fn try_array_push_const(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    if let Some((arr, instr)) = array_push_const_pair(instructions, i) {
        instructions[i] = instr.with_array(arr);
        remove_at(instructions, loop_payloads, source_positions, i + 1);
        return true;
    }

    if i + 2 >= instructions.len() {
        return false;
    }
    let fused = match (&instructions[i], &instructions[i + 1], &instructions[i + 2]) {
        (
            Instruction::LoadIntConst { dst, const_idx },
            Instruction::Move { dst: arg, src },
            Instruction::ArrayPush {
                dst: push_dst,
                arr,
                val,
            },
        ) if *dst == *src && *arg == *val && *push_dst == *arr => {
            Some(Instruction::ArrayPushIntConst {
                arr: *arr,
                const_idx: *const_idx,
            })
        }
        (
            Instruction::LoadConst { dst, const_idx },
            Instruction::Move { dst: arg, src },
            Instruction::ArrayPush {
                dst: push_dst,
                arr,
                val,
            },
        ) if *dst == *src && *arg == *val && *push_dst == *arr => {
            Some(Instruction::ArrayPushConst {
                arr: *arr,
                const_idx: *const_idx,
            })
        }
        _ => None,
    };
    if let Some(instr) = fused {
        instructions[i] = instr;
        remove_at(instructions, loop_payloads, source_positions, i + 2);
        remove_at(instructions, loop_payloads, source_positions, i + 1);
        return true;
    }
    false
}
enum ArrayPushConst {
    Int(u16),
    Any(u16),
}
impl ArrayPushConst {
    fn with_array(self, arr: u8) -> Instruction {
        match self {
            Self::Int(const_idx) => Instruction::ArrayPushIntConst { arr, const_idx },
            Self::Any(const_idx) => Instruction::ArrayPushConst { arr, const_idx },
        }
    }
}
fn array_push_const_pair(instructions: &[Instruction], i: usize) -> Option<(u8, ArrayPushConst)> {
    match (&instructions[i], &instructions[i + 1]) {
        (
            Instruction::LoadIntConst { dst, const_idx },
            Instruction::ArrayPush {
                dst: push_dst,
                arr,
                val,
            },
        ) if *dst == *val && *push_dst == *arr => Some((*arr, ArrayPushConst::Int(*const_idx))),
        (
            Instruction::LoadConst { dst, const_idx },
            Instruction::ArrayPush {
                dst: push_dst,
                arr,
                val,
            },
        ) if *dst == *val && *push_dst == *arr => Some((*arr, ArrayPushConst::Any(*const_idx))),
        _ => None,
    }
}
fn try_index2d(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    // P3: also match IndexArray (P1) for nested fusion.
    // IndexStringAscii is intentionally NOT fused — string-of-strings is not a 2D pattern.
    let (mid, outer, idx1, inner2, dst, idx2) = match (&instructions[i], &instructions[i + 1]) {
        (Instruction::Index { dst: m, obj: o, idx: i1 }, Instruction::Index { dst, obj: inn, idx: i2 }) => {
            (*m, *o, *i1, *inn, *dst, *i2)
        }
        (Instruction::IndexArray { dst: m, obj: o, idx: i1 }, Instruction::IndexArray { dst, obj: inn, idx: i2 }) => {
            (*m, *o, *i1, *inn, *dst, *i2)
        }
        (Instruction::Index { dst: m, obj: o, idx: i1 }, Instruction::IndexArray { dst, obj: inn, idx: i2 }) => {
            (*m, *o, *i1, *inn, *dst, *i2)
        }
        (Instruction::IndexArray { dst: m, obj: o, idx: i1 }, Instruction::Index { dst, obj: inn, idx: i2 }) => {
            (*m, *o, *i1, *inn, *dst, *i2)
        }
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
fn try_index_assign2d(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    // P3: also match IndexArray (P1) for nested fusion
    match (&instructions[i], &instructions[i + 1]) {
        (
            Instruction::Index { dst: row, obj: outer, idx: idx1 },
            Instruction::IndexAssign { obj: inner, idx: idx2, val },
        )
        | (
            Instruction::IndexArray { dst: row, obj: outer, idx: idx1 },
            Instruction::IndexAssignArray { obj: inner, idx: idx2, val },
        )
        | (
            Instruction::Index { dst: row, obj: outer, idx: idx1 },
            Instruction::IndexAssignArray { obj: inner, idx: idx2, val },
        )
        | (
            Instruction::IndexArray { dst: row, obj: outer, idx: idx1 },
            Instruction::IndexAssign { obj: inner, idx: idx2, val },
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
    let fused = if let (
        Instruction::Index {
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
    } else if let (
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
fn try_int_mul_add_assign(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    if let Some((acc, src1, src2, remove_count)) = int_mul_add_match(instructions, i) {
        instructions[i] = Instruction::IntMulAddAssign { acc, src1, src2 };
        if remove_count == 2 {
            remove_at(instructions, loop_payloads, source_positions, i + 2);
        }
        remove_at(instructions, loop_payloads, source_positions, i + 1);
        return true;
    }
    false
}
fn int_mul_add_match(instructions: &[Instruction], i: usize) -> Option<(u8, u8, u8, usize)> {
    let (mul_dst, mul_src1, mul_src2, add_dst, add_src1, add_src2) =
        match (&instructions[i], &instructions[i + 1]) {
            (
                Instruction::IntMul {
                    dst: mul_dst,
                    src1: mul_src1,
                    src2: mul_src2,
                },
                Instruction::IntAdd {
                    dst: add_dst,
                    src1: add_src1,
                    src2: add_src2,
                },
            ) => (
                *mul_dst, *mul_src1, *mul_src2, *add_dst, *add_src1, *add_src2,
            ),
            _ => return None,
        };

    if mul_dst == add_src2 && add_dst == add_src1 {
        return Some((add_dst, mul_src1, mul_src2, 1));
    }
    if mul_dst == add_src1 && add_dst == add_src2 {
        return Some((add_dst, mul_src1, mul_src2, 1));
    }

    if i + 2 >= instructions.len() {
        return None;
    }
    if let Instruction::Move { dst: acc, src } = instructions[i + 2] {
        if src == add_dst
            && ((mul_dst == add_src2 && acc == add_src1)
                || (mul_dst == add_src1 && acc == add_src2))
        {
            return Some((acc, mul_src1, mul_src2, 2));
        }
    }
    None
}
fn try_property_assign(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    if i + 2 >= instructions.len() {
        return false;
    }
    if let Some(instr) = property_assign_match(instructions, i) {
        instructions[i] = instr;
        remove_at(instructions, loop_payloads, source_positions, i + 2);
        remove_at(instructions, loop_payloads, source_positions, i + 1);
        return true;
    }
    false
}
fn property_assign_match(instructions: &[Instruction], i: usize) -> Option<Instruction> {
    match (&instructions[i], &instructions[i + 1], &instructions[i + 2]) {
        (
            Instruction::GetProperty {
                dst: got,
                obj,
                prop_sym,
            },
            Instruction::IntSub { dst, src1, src2 },
            Instruction::SetProperty {
                obj: set_obj,
                prop_sym: set_sym,
                val,
                ..
            },
        ) if *got == *src1 && *dst == *val && *obj == *set_obj && *prop_sym == *set_sym => {
            Some(Instruction::PropertySubAssign {
                obj: *obj,
                prop_sym: *prop_sym,
                src: *src2,
            })
        }
        _ => None,
    }
}
fn try_strcat(
    instructions: &mut Vec<Instruction>,
    loop_payloads: &mut [LoopPayload],
    source_positions: &mut SourcePositions,
    i: usize,
) -> bool {
    let fused = match (&instructions[i], &instructions[i + 1]) {
        (
            Instruction::StrCat {
                dst: mid,
                src1: a,
                src2: b,
            },
            Instruction::StrCat {
                dst,
                src1: mid2,
                src2: c,
            },
        ) if *mid == *mid2 => Some(Instruction::StrCat3 {
            dst: *dst,
            a: *a,
            b: *b,
            c: *c,
        }),
        _ => None,
    };
    if let Some(instr) = fused {
        instructions[i] = instr;
        remove_at(instructions, loop_payloads, source_positions, i + 1);
        return true;
    }
    false
}
