use super::*;
use hudhudscript_bytecode::{Instruction, LoopPayload};

/// Stack effect for remaining instructions. Most arithmetic ops removed.
pub fn stack_effect(inst: &Instruction) -> Option<i32> {
    use Instruction::*;
    Some(match inst {
        LoadConst { .. } | LoadNumConst { .. } | LoadIntConst { .. } => 1,
        StrCat { .. } => -1,
        // DeclVar / local register write → use DeclGlobal/StoreGlobal for globals
        Index { .. } | IndexArray { .. } | IndexStringAscii { .. } => 1,
        IndexAssign { .. } | IndexAssignArray { .. } => -1,
        Return { .. } => return None,
        Jump(_) => 0,
        JumpIfFalse { .. } | JumpIfTrue { .. } => -1,
        LoopBegin(_) | LoopEnd => 0,
        GetProperty { .. } => 0,
        SetProperty { .. } => -1,
        Await { .. } => 0,
        Throw { .. } => -1,
        MethodCall { .. }
        | SuperCall { .. }
        | NewInstance { .. }
        | CallSpread(_)
        | MethodCallSpread(_)
        | MakeGenerator { .. }
        | Spawn { .. }
        | Despawn { .. }
        | Send { .. }
        | Receive { .. }
        | Perform { .. }
        | Require { .. }
        | Remember { .. }
        | Recall { .. }
        | Forget { .. } => return None,
        TryBegin(_)
        | TryEnd
        | FinallyBegin(_)
        | FinallyEnd
        | FinallyExit(_)
        | Break
        | Continue
        | Yield { .. } => return None,
        ArrayPush { .. }
        | SpreadIntoArray { .. }
        | SpreadIntoObject { .. }
        | ForIn { .. }
        | IterNext { .. }
        | EnumDecl(_)
        | MatchVariant(_)
        | BindVar(_)
        | DeclStore { .. }
        | ClassDecl(_)
        | TraitCheck(_)
        | ClassStaticDecl(_)
        | GetStatic(_)
        | LoadModule(_)
        | DefineFunction(_)
        | WriteBackReceiver(_)
        | DestructArray(_, _)
        | DestructObject(_) => return None,
        TailCall { .. } => return None,
        IntSubCall1(_) | IntAddCall1(_) => return None,
        IntLeJumpIfFalse(_) | IntLtJumpIfFalse(_) => return None,
        // Register-based VM instructions — opaque to stack-effect analysis
        IntAdd { .. }
        | IntSub { .. }
        | IntMul { .. }
        | IntAddI { .. }
        | IntSubI { .. }
        | IntCmp { .. }
        | NumAdd { .. }
        | NumSub { .. }
        | NumMul { .. }
        | NumDiv { .. }
        | IntDiv { .. }
        | IntMod { .. }
        | NumMod { .. }
        | IntLeRRJumpIfFalse { .. }
        | IntLtRRJumpIfFalse { .. }
        | MakeArray { .. }
        | MakeObject { .. }
        | Call { .. }
        | LoadGlobal { .. }
        | StoreGlobal { .. }
        | DeclGlobal { .. } => return None,
        _ => return None,
    })
}

/// Return the symbol that this instruction WRITES to, if any.
pub fn written_symbol(inst: &Instruction) -> Option<u32> {
    match inst {
        Instruction::DeclGlobal { sym, .. } => Some(*sym as u32),
        _ => None,
    }
}

pub fn abs_target_for_jump(ip: usize, offset: i32) -> usize {
    (ip as i64 + offset as i64) as usize
}

/// Adjust all jump offsets and loop payloads after removing the instruction
/// at `removed_at`.  Must be called BEFORE `instructions.remove(removed_at)`.
pub(crate) fn adjust_jumps_after_remove(
    instructions: &mut [Instruction],
    loop_payloads: &mut [LoopPayload],
    removed_at: usize,
) {
    // 1. Adjust instruction-embedded jump offsets
    let len = instructions.len();
    for ip in 0..len {
        let adjust = |off: &mut i32, ip: usize| {
            let target = (ip as i64 + *off as i64) as usize;
            let new_ip = if ip > removed_at { ip - 1 } else { ip };
            let new_target = if target > removed_at {
                target - 1
            } else {
                target
            };
            *off = (new_target as i64 - new_ip as i64) as i32;
        };
        match &mut instructions[ip] {
            Instruction::Jump(o)
            | Instruction::TryBegin(o)
            | Instruction::FinallyBegin(o)
            | Instruction::FinallyExit(o) => adjust(o, ip),
            Instruction::JumpIfFalse { offset, .. } | Instruction::JumpIfTrue { offset, .. } => {
                let mut o32 = *offset as i32;
                adjust(&mut o32, ip);
                *offset = o32 as i16;
            }
            Instruction::ForIn { end_offset, .. } | Instruction::IterNext { end_offset, .. } => {
                let mut o32 = *end_offset as i32;
                adjust(&mut o32, ip);
                *end_offset = o32 as i16;
            }
            Instruction::IntLeRRJumpIfFalse { offset, .. }
            | Instruction::IntLtRRJumpIfFalse { offset, .. }
            | Instruction::IntCmpIJumpIfFalse { offset, .. }
            | Instruction::IntCmpRRJumpIfFalse { offset, .. }
            | Instruction::IntAddIJump { offset, .. }
            | Instruction::LoopEndIntAddIJump { offset, .. }
            | Instruction::IntSubIJump { offset, .. }
            | Instruction::IntCmpIJumpIfTrue { offset, .. } => {
                let mut o32 = *offset as i32;
                adjust(&mut o32, ip);
                *offset = o32 as i16;
            }
            _ => {}
        }
    }
    // 2. Adjust loop payload absolute IPs (Break/Continue use these)
    for lp in loop_payloads.iter_mut() {
        if lp.start as usize > removed_at {
            lp.start -= 1;
        }
        if lp.end as usize > removed_at {
            lp.end -= 1;
        }
    }
}

/// Adjust a jump offset when instructions are inserted at `insert_at`.
pub fn adjust_jump_for_insert(ip: usize, offset: i32, insert_at: usize, count: usize) -> i32 {
    let target = (ip as i64 + offset as i64) as usize;
    if ip < insert_at && target >= insert_at {
        offset + count as i32
    } else if ip >= insert_at && target < insert_at {
        offset - count as i32
    } else {
        offset
    }
}
