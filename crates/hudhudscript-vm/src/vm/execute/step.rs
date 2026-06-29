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
                PackedResult::Advance => {
                    *ip_ref = ip + 1;
                    return Ok(StepAction::Jumped);
                }
                PackedResult::Jump(target) => {
                    *ip_ref = target;
                    return Ok(StepAction::Jumped);
                }
                PackedResult::Return { src } => return Ok(StepAction::Return { src }),
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
                return Ok(StepAction::Return { src: *src });
            }
            Instruction::ReturnConst { const_idx } => {
                // Use bridge register 255 + function-chunk constants slice
                self.registers[255] = constants[*const_idx as usize];
                return Ok(StepAction::Return { src: 255 });
            }
            Instruction::Move { dst, src } => {
                self.registers[*dst as usize] = self.registers[*src as usize];
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            Instruction::LoadIntConst { dst, const_idx } => {
                self.registers[*dst as usize] = Value16::int(bytecode.int_constants[*const_idx as usize]);
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }
            Instruction::LoadConst { dst, const_idx } => {
                self.registers[*dst as usize] = constants[*const_idx as usize];
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            Instruction::JumpIfFalse { src, offset } => {
                if !self.registers[*src as usize].is_truthy() {
                    let new_ip = (ip as i64).wrapping_add(*offset as i64);
                    if new_ip < 0 || new_ip > instructions.len() as i64 {
                        return Err(Self::runtime_error_with_pos(
                            format!("JumpIfFalse out of bounds: ip={} offset={}", ip, offset),
                            bytecode,
                            ip,
                        ));
                    }
                    *ip_ref = new_ip as usize;
                    return Ok(StepAction::Jumped);
                }
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            Instruction::JumpIfTrue { src, offset } => {
                if self.registers[*src as usize].as_bool().unwrap_or(false) {
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }
            Instruction::Jump(offset) => {
                *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                return Ok(StepAction::Jumped);
            }

            // ── Integer arithmetic hot path (P0) — checked_add/checked_sub ──
            // Overflow → fall through to slow path (BigInt promotion).
            // Only handles pure Int+Int (no BigInt, no Float).
            Instruction::IntAdd { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if let Some(r) = x.checked_add(y) {
                        self.registers[*dst as usize] = Value16::int(r);
                        *ip_ref = ip + 1;
                        return Ok(StepAction::Jumped);
                    }
                }
            }
            Instruction::IntSub { dst, src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if let Some(r) = x.checked_sub(y) {
                        self.registers[*dst as usize] = Value16::int(r);
                        *ip_ref = ip + 1;
                        return Ok(StepAction::Jumped);
                    }
                }
            }
            Instruction::IntAddI { dst, src, imm } => {
                let a = self.registers[*src as usize];
                if let Some(x) = a.as_int() {
                    if let Some(r) = x.checked_add(*imm as i64) {
                        self.registers[*dst as usize] = Value16::int(r);
                        *ip_ref = ip + 1;
                        return Ok(StepAction::Jumped);
                    }
                }
            }
            Instruction::IntSubI { dst, src, imm } => {
                let a = self.registers[*src as usize];
                if let Some(x) = a.as_int() {
                    if let Some(r) = x.checked_sub(*imm as i64) {
                        self.registers[*dst as usize] = Value16::int(r);
                        *ip_ref = ip + 1;
                        return Ok(StepAction::Jumped);
                    }
                }
            }
            Instruction::IntCmpIJumpIfFalse {
                src,
                imm,
                op,
                offset,
            } => {
                let a = self.registers[*src as usize];
                if let Some(x) = a.as_int() {
                    let cond = match *op {
                        0 => x < *imm as i64,
                        1 => x <= *imm as i64,
                        2 => x > *imm as i64,
                        3 => x >= *imm as i64,
                        4 => x == *imm as i64,
                        5 => x != *imm as i64,
                        _ => false,
                    };
                    if !cond {
                        let new_ip = (ip as i64).wrapping_add(*offset as i64);
                        if new_ip < 0 || new_ip > instructions.len() as i64 {
                            return Err(Self::runtime_error_with_pos(
                                format!("IntCmpIJumpIfFalse out of bounds: ip={} offset={}", ip, offset),
                                bytecode, ip,
                            ));
                        }
                        *ip_ref = new_ip as usize;
                        return Ok(StepAction::Jumped);
                    }
                    *ip_ref = ip + 1;
                    return Ok(StepAction::Jumped);
                }
            }

            _ => {}
        }

        let mut ctx = StepContext {
            instructions,
            constants,
            bytecode,
            ip,
            ip_ref,
            chunk_ptr: self.current_chunk_ptr,
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
            | Instruction::PropertySubAssign { .. }
            | Instruction::LoadModule(_)
            | Instruction::DefineFunction(_) => self.step_classes_modules(instr, ctx),

            Instruction::MethodCall { .. }
            | Instruction::WriteBackReceiver(_)
            | Instruction::Await { .. }
            | Instruction::SuperCall { .. }
            | Instruction::GetStatic(_)
            | Instruction::ClassStaticDecl(_) => self.step_methods_async_generator(instr, ctx),

            Instruction::Yield { .. } | Instruction::MakeGenerator { .. } => {
                self.step_methods_generator(instr, ctx)
            }

            Instruction::DestructArray(_, _)
            | Instruction::DestructObject(_)
            | Instruction::SpreadIntoArray { .. }
            | Instruction::SpreadIntoObject { .. }



            | Instruction::IntSubCall1(_)
            | Instruction::IntAddCall1(_)
            | Instruction::IntLeJumpIfFalse(_)
            | Instruction::IntLtJumpIfFalse(_) => self.step_super_instructions(instr, ctx),

            // Register-based VM instructions
            | Instruction::IntAdd { .. }
            | Instruction::IntSub { .. }
            | Instruction::IntDivI { .. }
            | Instruction::IntModI { .. }
            | Instruction::IntModCmpI { .. }
            | Instruction::IntMul { .. }
            | Instruction::IntMulI { .. }
            | Instruction::IntAddI { .. }
            | Instruction::IntSubI { .. }
            | Instruction::LoadIntConst { .. }
            | Instruction::LoadConst { .. }
            | Instruction::LoadNumConst { .. }

            | Instruction::IntLeRRJumpIfFalse { .. }
            | Instruction::IntLtRRJumpIfFalse { .. }
            | Instruction::IntLtRRJumpPacked(_)
            | Instruction::IntLeRRJumpPacked(_)
            | Instruction::IntCmpIJumpIfFalse { .. }
            | Instruction::IntCmpRRJumpIfFalse { .. }
            | Instruction::IntAddIJump { .. }
            | Instruction::LoopEndIntAddIJump { .. }
            | Instruction::IntSubIJump { .. }
            | Instruction::IntCmpIJumpIfTrue { .. }
            | Instruction::ReturnConst { .. }
            | Instruction::IntAddReturn { .. }
            | Instruction::IntSubReturn { .. }
            | Instruction::IntMulReturn { .. }
            | Instruction::IntDivReturn { .. }
            | Instruction::IntCmpIReturn { .. }
            | Instruction::IntCmp { .. }
            | Instruction::IntCmpI { .. }
            | Instruction::NumAdd { .. }
            | Instruction::NumAddI { .. }
            | Instruction::NumSubI { .. }
            | Instruction::NumMulI { .. }
            | Instruction::NumDivI { .. }
            | Instruction::NumSub { .. }
            | Instruction::NumMul { .. }
            | Instruction::NumMulAddAssign { .. }
            | Instruction::NumMulAddIndexed { .. }
            | Instruction::FloatMulAdd { .. }
            | Instruction::FloatAdd { .. }
            | Instruction::FloatMul { .. }
            | Instruction::IntMulMod { .. }
            | Instruction::IntMulModI { .. }
            // P5: NumSqrt intrinsic
            | Instruction::NumSqrt { .. }
            // P8: MakeArray2
            | Instruction::MakeArray2 { .. }
            | Instruction::StrCharEqRR { .. }
            | Instruction::NumDiv { .. }
            | Instruction::IntDiv { .. }
            | Instruction::IntMod { .. }
            | Instruction::NumMod { .. }
            | Instruction::StrCat { .. }
            | Instruction::StrCat3 { .. }
            | Instruction::StrCatMut { .. }
            | Instruction::StringConcat { .. }
            | Instruction::StringIndexOf { .. }
            | Instruction::ArrayPushIntConst { .. }
            | Instruction::ArrayPushConst { .. }
            | Instruction::ArrayPush { .. }
            // P2: fast path length/pop ops
            | Instruction::ArrayLen { .. }
            | Instruction::StringLen { .. }
            | Instruction::ArrayPop { .. }
            | Instruction::Index { .. }
            | Instruction::IndexArray { .. }
            | Instruction::IndexStringAscii { .. }
            | Instruction::Index2D { .. }
            | Instruction::IndexAssign2D { .. }
            | Instruction::IndexAssign { .. }
            | Instruction::IndexAssignArray { .. }
            | Instruction::IntMulAddAssign { .. }
            | Instruction::Neg { .. }
            | Instruction::Not { .. }
            | Instruction::Move { .. }
            | Instruction::Return { .. }
            | Instruction::MakeArray { .. }
            | Instruction::MakeObject { .. }
            | Instruction::Call { .. }
            | Instruction::StoreConst { .. }
            | Instruction::StringIndexOf { .. }
            | Instruction::StringContains { .. }
            | Instruction::IntCmp { .. }
            | Instruction::IntCmpI { .. }
            // register-to-register moves and integer/slot super-instructions
            // are handled by the fast int/slot dispatcher.
            => self.step_int_slot_super(instr, ctx),
            Instruction::LoadGlobal { .. }
            | Instruction::StoreGlobal { .. }
            | Instruction::StoreGlobalConst { .. }
            | Instruction::DeclGlobal { .. } => self.step_variables(instr, ctx),
        }
    }
}
