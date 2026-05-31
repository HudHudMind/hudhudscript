#![allow(unused_imports)]

use super::*;

impl VM {
    /// Per-instruction step body extracted from `execute_instructions`'s
    /// former IIFE closure. Inlined into the hot dispatch loop; the
    /// closure allocation and its call-site frame go away.
    #[inline(always)]
    pub(crate) fn step_one(
        &mut self,
        instructions: &[Instruction],
        constants: &[Value16],
        bytecode: &Bytecode,
        packed: &[u32],
        ip_ref: &mut usize,
    ) -> CompileResult<StepAction> {
        let ip: usize = *ip_ref;
        // ── P7.3 Packed fast-dispatch ─────────────────────────
        // For hot-path instructions that were successfully packed
        // into a compact u32, dispatch via integer match on the
        // opcode byte. Avoids matching the large Instruction enum
        // for simple ops. `dispatch_packed` is `#[inline(always)]`
        // — inlined here so the hot dispatch path avoids a call frame.
        debug_assert!(
            ip < packed.len(),
            "packed index out of bounds: {} >= {}",
            ip,
            packed.len()
        );
        let p = unsafe { *packed.get_unchecked(ip) };
        if p != PACK_SENTINEL {
            match self.dispatch_packed(p, instructions, constants, bytecode, ip)? {
                PackedResult::Advance => return Ok(StepAction::Advance),
                PackedResult::Jump(target) => {
                    *ip_ref = target;
                    return Ok(StepAction::Jumped);
                }
                PackedResult::Return => return Ok(StepAction::Return),
                PackedResult::Fallthrough => { /* fall through to full match */ }
            }
        }

        debug_assert!(
            ip < instructions.len(),
            "instructions index out of bounds: {} >= {}",
            ip,
            instructions.len()
        );
        let instr = unsafe { instructions.get_unchecked(ip) };

        // ── Inline hot path — bypass dispatch_unpacked's 150+ arm match ──
        // These four instructions account for ~60% of all unpacked
        // dispatches in loop-heavy code (fib, prime, collatz).
        match instr {

            Instruction::Return { src } => {
                self.registers[255] = self.registers[*src as usize];
                return Ok(StepAction::Return);
            }
            Instruction::Move { dst, src } => {
                self.registers[*dst as usize] = self.registers[*src as usize];
                return Ok(StepAction::Advance);
            }
            Instruction::IntAdd { dst, src1, src2 } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                self.registers[*dst as usize] = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => Value16::int((p1 as i64).wrapping_add(p2 as i64)),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) | (ReprTag::Int, ReprTag::Number) => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        Value16::number(a + b)
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntAdd: operands not numeric", bytecode, ip)),
                };
                return Ok(StepAction::Advance);
            }
            Instruction::JumpIfFalse { src, offset } => {
                if !self.registers[*src as usize].is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(Self::runtime_error_with_pos(
                            format!("JumpIfFalse out of bounds: ip={} offset={}", ip, offset),
                            bytecode, ip));
                    }
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
                return Ok(StepAction::Advance);
            }
            Instruction::IntLtRIJumpIfFalse { src, imm, offset } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) < (*imm as i64),
                    ReprTag::Number => f64::from_bits(p) < (*imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntLtRIJumpIfFalse: src not numeric", bytecode, ip)),
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
                return Ok(StepAction::Advance);
            }
            Instruction::IntLeRIJumpIfFalse { src, imm, offset } => {
                let (tag, p) = self.registers[*src as usize].split_tag();
                let cond = match tag {
                    ReprTag::Int => (p as i64) <= (*imm as i64),
                    ReprTag::Number => f64::from_bits(p) <= (*imm as f64),
                    _ => return Err(Self::runtime_error_with_pos("IntLeRIJumpIfFalse: src not numeric", bytecode, ip)),
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
                return Ok(StepAction::Advance);
            }
            Instruction::IntLtRRJumpIfFalse { src1, src2, offset } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                let cond = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => (p1 as i64) < (p2 as i64),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        a < b
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntLtRRJumpIfFalse: operands not numeric", bytecode, ip)),
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
                return Ok(StepAction::Advance);
            }
            Instruction::IntLeRRJumpIfFalse { src1, src2, offset } => {
                let (t1, p1) = self.registers[*src1 as usize].split_tag();
                let (t2, p2) = self.registers[*src2 as usize].split_tag();
                let cond = match (t1, t2) {
                    (ReprTag::Int, ReprTag::Int) => (p1 as i64) <= (p2 as i64),
                    (ReprTag::Number, ReprTag::Number) | (ReprTag::Int, ReprTag::Number) | (ReprTag::Number, ReprTag::Int) => {
                        let a = if t1 == ReprTag::Int { p1 as i64 as f64 } else { f64::from_bits(p1) };
                        let b = if t2 == ReprTag::Int { p2 as i64 as f64 } else { f64::from_bits(p2) };
                        a <= b
                    }
                    _ => return Err(Self::runtime_error_with_pos("IntLeRRJumpIfFalse: operands not numeric", bytecode, ip)),
                };
                if !cond {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
                return Ok(StepAction::Advance);
            }
            _ => {}
        }

        let mut ctx = StepContext {
            instructions,
            constants,
            bytecode,
            ip,
            ip_ref,
        };

        self.dispatch_unpacked(instr, &mut ctx)
    }

    #[inline(never)]
    pub(crate) fn dispatch_unpacked(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        match instr {
            // LoadNumConst/LoadIntConst → LoadNumConst/LoadIntConst handled elsewhere
            // IndexFast/IndexAssignFast removed → Index/IndexAssign handled elsewhere
            Instruction::Jump(_)
            | Instruction::JumpIfFalse { .. }
            | Instruction::JumpIfTrue { .. }
            | Instruction::TailCall { .. }
            | Instruction::CallSpread(_)
            | Instruction::MethodCallSpread(_) => self.step_collections_calls(instr, ctx),

            Instruction::EnumDecl(_)
            | Instruction::MatchVariant(_)
            | Instruction::BindVar(_)
            | Instruction::Break
            | Instruction::Continue
            | Instruction::LoopBegin(_)
            | Instruction::LoopEnd
            | Instruction::ForIn { .. }
            | Instruction::IterNext { .. }
            | Instruction::TryBegin(_)
            | Instruction::TryEnd
            | Instruction::FinallyBegin(_)
            | Instruction::FinallyEnd
            | Instruction::FinallyExit(_)
            | Instruction::Throw { .. } => self.step_control_flow(instr, ctx),

            Instruction::DeclStore { .. }
            | Instruction::Spawn { .. }
            | Instruction::Despawn { .. }
            | Instruction::ViewAs { .. }
            | Instruction::Send { .. }
            | Instruction::Receive { .. }
            | Instruction::Require { .. }
            | Instruction::Perform { .. } => self.step_actors_decl(instr, ctx),

            Instruction::Remember { .. } | Instruction::Recall { .. } | Instruction::Forget { .. } => {
                self.step_rag(instr, ctx)
            }

            Instruction::ClassDecl(_)
            | Instruction::TraitCheck(_)
            | Instruction::NewInstance { .. }
            | Instruction::GetProperty { .. }
            | Instruction::SetProperty { .. }
            | Instruction::LoadModule(_)
            | Instruction::DefineFunction(_) => self.step_classes_modules(instr, ctx),

            Instruction::MethodCall { .. }
            | Instruction::WriteBackReceiver(_)
            | Instruction::Await { .. }
            | Instruction::SuperCall { .. }
            | Instruction::GetStatic(_)
            | Instruction::ClassStaticDecl(_)
            | Instruction::Yield { .. }
            | Instruction::MakeGenerator { .. } => self.step_methods_async_generator(instr, ctx),

            Instruction::DestructArray(_, _)
            | Instruction::DestructObject(_)
            | Instruction::SpreadIntoArray { .. }
            | Instruction::SpreadIntoObject { .. }



            | Instruction::IntSubCall1(_)
            | Instruction::IntAddCall1(_)
            | Instruction::IntLeJumpIfFalse(_)
            | Instruction::IntLtJumpIfFalse(_)
            | Instruction::IntIncrSlot { .. }
            | Instruction::IntSubLocalI { .. }
            | Instruction::IntAddLocalI { .. } => self.step_super_instructions(instr, ctx),

            // Register-based VM instructions
            | Instruction::IntAdd { .. }
            | Instruction::IntSub { .. }
            | Instruction::IntMul { .. }
            | Instruction::IntAddI { .. }
            | Instruction::IntSubI { .. }
            | Instruction::IntMulI { .. }
            | Instruction::IntDivI { .. }
            | Instruction::IntModI { .. }
            | Instruction::LoadIntConst { .. }
            | Instruction::LoadConst { .. }
            | Instruction::LoadNumConst { .. }

            | Instruction::IntLeRRJumpIfFalse { .. }
            | Instruction::IntLtRRJumpIfFalse { .. }
            | Instruction::IntLeRIJumpIfFalse { .. }
            | Instruction::IntLtRIJumpIfFalse { .. }
            | Instruction::IntAddReturn { .. }
            | Instruction::IntSubReturn { .. }
            | Instruction::IntCmp { .. }
            | Instruction::IntCmpI { .. }
            | Instruction::NumAdd { .. }
            | Instruction::NumAddI { .. }
            | Instruction::NumSubI { .. }
            | Instruction::NumMulI { .. }
            | Instruction::NumDivI { .. }
            | Instruction::NumSub { .. }
            | Instruction::NumMul { .. }
            | Instruction::NumDiv { .. }
            | Instruction::IntDiv { .. }
            | Instruction::IntMod { .. }
            | Instruction::NumMod { .. }
            | Instruction::StrCat { .. }
            | Instruction::StrCatMut { .. }
            | Instruction::ArrayPush { .. }
            | Instruction::IndexAssign { .. }
            | Instruction::Neg { .. }
            | Instruction::Not { .. }
            | Instruction::Move { .. }
            | Instruction::Return { .. }
            | Instruction::Index { .. }
            | Instruction::MakeArray { .. }
            | Instruction::MakeObject { .. }
            | Instruction::Call { .. }
            | Instruction::LoadGlobal { .. }
            | Instruction::StoreGlobal { .. }
            | Instruction::DeclGlobal { .. }
            | Instruction::StoreConst { .. }
            | Instruction::StringIndexOf { .. }
            | Instruction::StringContains { .. }
            // PushReg/PopReg replaced by Move to/from register 255
            => self.step_int_slot_super(instr, ctx),
        }
    }
}
