use super::*;

/// Stack effect for remaining instructions. Most arithmetic ops removed.
pub fn stack_effect(inst: &Instruction) -> Option<i32> {
    use Instruction::*;
    Some(match inst {
        LoadConst { .. } | LoadNumConst { .. } | LoadIntConst { .. } => 1,
        StrCat { .. } => -1,
        // DeclVar/StoreLocal/DeclLocal → use DeclGlobal/StoreLocal
        Index { .. } => 1,
        IndexAssign { .. } => -1,
        Return { .. } => return None,
        Jump(_) => 0,
        JumpIfFalse { .. } | JumpIfTrue { .. } => -1,
        LoopBegin(_) | LoopEnd => 0,
        GetProperty { .. } => 0,
        SetProperty { .. } => -1,
        Await { .. } => 0,
        Throw { .. } => -1,
        MethodCall { .. } | SuperCall { .. } | NewInstance { .. }
        | CallSpread(_) | MethodCallSpread(_) | MakeGenerator { .. }
        | Spawn { .. } | Despawn { .. } | Send { .. } | Receive { .. } | Perform { .. } | Require { .. }
        | Remember { .. } | Recall { .. } | Forget { .. } => return None,
        TryBegin(_) | TryEnd | FinallyBegin(_) | FinallyEnd | FinallyExit(_)
        | Break | Continue | Yield { .. } => return None,
        ArrayPush { .. } | SpreadIntoArray { .. } | SpreadIntoObject { .. }
        | ForIn { .. } | IterNext { .. }
        | EnumDecl(_) | MatchVariant(_) | BindVar(_)
        | DeclStore { .. } | ClassDecl(_) | TraitCheck(_)
        | ClassStaticDecl(_) | GetStatic(_)
        | LoadModule(_) | DefineFunction(_)
        | WriteBackReceiver(_)
        | DestructArray(_, _) | DestructObject(_) => return None,
        TailCall { .. } => return None,
        IntSubCall1(_) | IntAddCall1(_) => return None,
        IntLeJumpIfFalse(_) | IntLtJumpIfFalse(_) => return None,
        // Register-based VM instructions — opaque to stack-effect analysis
        IntAdd { .. } | IntSub { .. } | IntMul { .. }
        | IntAddI { .. } | IntSubI { .. } | IntCmp { .. }
        | NumAdd { .. } | NumSub { .. } | NumMul { .. } | NumDiv { .. } | IntDiv { .. } | IntMod { .. } | NumMod { .. }

        | IntLeRRJumpIfFalse { .. } | IntLtRRJumpIfFalse { .. }
        | IntLeRIJumpIfFalse { .. } | IntLtRIJumpIfFalse { .. }
        | MakeArray { .. } | MakeObject { .. } | Call { .. } | LoadGlobal { .. } | StoreGlobal { .. } | DeclGlobal { .. } => return None,
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
