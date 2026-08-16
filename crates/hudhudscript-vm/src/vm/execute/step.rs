#![allow(unused_imports)]

use super::module_ops::*;
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
        // FAZ0-A/E2: fused-opcode executed sayacı — TEK nokta, TEK envanter
        // (telemetry::fused_name, Instruction-uzayı). Her komut dispatch'i
        // step_one'a tam bir kez girer (Fallthrough aynı çağrı içinde
        // unpacked'a devam eder, yeniden girmez) → çifte sayım yok.
        #[cfg(feature = "telemetry")]
        if let Some(instr) = instructions.get(ip) {
            if let Some(name) = crate::vm::telemetry::fused_name(instr) {
                *self
                    .telemetry
                    .fusion_executed_by_opcode
                    .entry(name)
                    .or_insert(0) += 1;
            }
        }
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
            #[cfg(feature = "telemetry")]
            {
                let dense = (p & 0xFF) as usize;
                self.telemetry.packed_dispatch_count += 1;
                if dense < self.telemetry.opcode_counts.len() {
                    self.telemetry.opcode_counts[dense] += 1;
                }
                let prev = self.telemetry.last_dense;
                if prev != 0xFFFF {
                    *self
                        .telemetry
                        .opcode_bigrams
                        .entry((prev, dense as u16))
                        .or_insert(0) += 1;
                }
                self.telemetry.last_dense = dense as u16;
            }
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
                PackedResult::Fallthrough => {
                    #[cfg(feature = "telemetry")]
                    {
                        let dense = (p & 0xFF) as usize;
                        self.telemetry.packed_dispatch_count -= 1;
                        self.telemetry.packed_fallthrough_count += 1;
                        if dense < self.telemetry.fallthrough_by_opcode.len() {
                            self.telemetry.fallthrough_by_opcode[dense] += 1;
                        }
                        self.telemetry.last_dense = 0xFFFF;
                    }
                }
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
                self.registers[*dst as usize] =
                    Value16::int(bytecode.int_constants[*const_idx as usize]);
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
                if self.registers[*src as usize].is_truthy() {
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
                // Fast-path failed — dispatch directly to int_arith instead of
                // falling through to the 150+ arm dispatch_unpacked match.
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_arith(instr, &mut ctx);
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
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_arith(instr, &mut ctx);
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
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_arith(instr, &mut ctx);
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
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_arith(instr, &mut ctx);
            }
            Instruction::IntAddReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if let Some(r) = x.checked_add(y) {
                        self.registers[255] = Value16::int(r);
                        return Ok(StepAction::Return { src: 255 });
                    }
                }
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_slot_super(instr, &mut ctx);
            }
            Instruction::IntSubReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if let Some(r) = x.checked_sub(y) {
                        self.registers[255] = Value16::int(r);
                        return Ok(StepAction::Return { src: 255 });
                    }
                }
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_slot_super(instr, &mut ctx);
            }
            Instruction::IntMulReturn { src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if let Some(r) = x.checked_mul(y) {
                        self.registers[255] = Value16::int(r);
                        return Ok(StepAction::Return { src: 255 });
                    }
                }
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: std::ptr::null(),
                };
                return self.step_int_slot_super(instr, &mut ctx);
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
                                format!(
                                    "IntCmpIJumpIfFalse out of bounds: ip={} offset={}",
                                    ip, offset
                                ),
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
                // Non-Int → fall through to general handler
                let mut ctx = StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_branch(instr, &mut ctx);
            }

            // ── C4: Inline register-register loop compare ─────
            // while (i < n) and for (...; i < n; ...) patterns where
            // the bound n is a register. Fast-path for Int+Int.
            Instruction::IntLtRRJumpIfFalse { src1, src2, offset } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if !(x < y) {
                        let new_ip = (ip as i64).wrapping_add(*offset as i64);
                        *ip_ref = new_ip as usize;
                        return Ok(StepAction::Jumped);
                    }
                    *ip_ref = ip + 1;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntLeRRJumpIfFalse { src1, src2, offset } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    if !(x <= y) {
                        let new_ip = (ip as i64).wrapping_add(*offset as i64);
                        *ip_ref = new_ip as usize;
                        return Ok(StepAction::Jumped);
                    }
                    *ip_ref = ip + 1;
                    return Ok(StepAction::Jumped);
                }
            }

            // ── P1: Loop back-edge (IntAddIJump / LoopEndIntAddIJump / IntSubIJump) ──
            // Every while/for loop's back-edge hits one of these.
            Instruction::IntAddIJump { reg, imm, offset } => {
                let val = self.registers[*reg as usize];
                if let Some(x) = val.as_int() {
                    if let Some(r) = x.checked_add(*imm as i64) {
                        self.registers[*reg as usize] = Value16::int(r);
                    } else {
                        let big = Value16::bigint(
                            num_bigint::BigInt::from(x) + num_bigint::BigInt::from(*imm as i64),
                        );
                        self.registers[*reg as usize] = big;
                    }
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::LoopEndIntAddIJump { reg, imm, offset } => {
                self.loop_headers.pop();
                let val = self.registers[*reg as usize];
                if let Some(x) = val.as_int() {
                    if let Some(r) = x.checked_add(*imm as i64) {
                        self.registers[*reg as usize] = Value16::int(r);
                    } else {
                        let big = Value16::bigint(
                            num_bigint::BigInt::from(x) + num_bigint::BigInt::from(*imm as i64),
                        );
                        self.registers[*reg as usize] = big;
                    }
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }
            Instruction::IntSubIJump { reg, imm, offset } => {
                let val = self.registers[*reg as usize];
                if let Some(x) = val.as_int() {
                    if let Some(r) = x.checked_sub(*imm as i64) {
                        self.registers[*reg as usize] = Value16::int(r);
                    } else {
                        let big = Value16::bigint(
                            num_bigint::BigInt::from(x) - num_bigint::BigInt::from(*imm as i64),
                        );
                        self.registers[*reg as usize] = big;
                    }
                    *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                    return Ok(StepAction::Jumped);
                }
            }

            // ── P1: Loop condition (IntCmpRRJumpIfFalse / IntCmpIJumpIfTrue) ──
            Instruction::IntCmpRRJumpIfFalse {
                src1,
                src2,
                op,
                offset,
            } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                if let (Some(x), Some(y)) = (a.as_int(), b.as_int()) {
                    let cond = match *op {
                        0 => x < y,
                        1 => x <= y,
                        2 => x > y,
                        3 => x >= y,
                        4 => x == y,
                        5 => x != y,
                        _ => false,
                    };
                    if !cond {
                        *ip_ref = (ip as i64).wrapping_add(*offset as i64) as usize;
                        return Ok(StepAction::Jumped);
                    }
                    *ip_ref = ip + 1;
                    return Ok(StepAction::Jumped);
                }
                // Non-Int operands → fall through to general handler
                let mut ctx = StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_branch(instr, &mut ctx);
            }

            // ── P1: NumMulAddIndexed — polynomial_eval Horner inner loop ──
            Instruction::NumMulAddIndexed { acc, mul, arr, idx } => {
                let acc_val = self.registers[*acc as usize];
                let mul_val = self.registers[*mul as usize];
                let idx_val = self.registers[*idx as usize];
                let a = acc_val.as_number_fast().unwrap_or(0.0);
                let b = mul_val.as_number_fast().unwrap_or(0.0);
                let c = self.registers[*arr as usize]
                    .as_array()
                    .and_then(|av| {
                        let i = crate::vm::index_helpers::numeric_index_i64(idx_val)
                            .and_then(crate::vm::index_helpers::index_i64_to_usize)
                            .unwrap_or(0);
                        av.get(i).and_then(|v| v.as_number_fast())
                    })
                    .unwrap_or(0.0);
                self.registers[*acc as usize] = Value16::number(a * b + c);
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            // ── P1: IndexAssignArray — duffs_device unrolled array writes ──
            Instruction::IndexAssignArray { obj, idx, val } => {
                let idx_val = self.registers[*idx as usize];
                let new_val = self.registers[*val as usize];
                let i = crate::vm::index_helpers::numeric_index_i64(idx_val)
                    .and_then(crate::vm::index_helpers::index_i64_to_usize)
                    .unwrap_or(0);
                if let Some(arr) = self.registers[*obj as usize].as_array_mut() {
                    if i >= arr.len() {
                        arr.resize(i + 1, Value16::null());
                    }
                    arr[i] = new_val;
                }
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            // ── P1: StrCharEqRR — palindrome/revcomp char comparison ──
            Instruction::StrCharEqRR {
                dst,
                src_s,
                src_i,
                src_j,
            } => {
                let s_val = self.registers[*src_s as usize];
                let ni =
                    crate::vm::index_helpers::numeric_index_i64(self.registers[*src_i as usize])
                        .and_then(crate::vm::index_helpers::index_i64_to_usize);
                let nj =
                    crate::vm::index_helpers::numeric_index_i64(self.registers[*src_j as usize])
                        .and_then(crate::vm::index_helpers::index_i64_to_usize);
                let eq = if let (Some(s), Some(i), Some(j)) = (s_val.as_str(), ni, nj) {
                    let bytes = s.as_bytes();
                    i < bytes.len() && j < bytes.len() && bytes[i] == bytes[j]
                } else {
                    false
                };
                self.registers[*dst as usize] = Value16::bool_(eq);
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            // ── P1: LoadNumConst — float constant load ──
            Instruction::LoadNumConst { dst, const_idx } => {
                self.registers[*dst as usize] = Value16::number(f64::from_bits(
                    bytecode.numeric_constants[*const_idx as usize],
                ));
                *ip_ref = ip + 1;
                return Ok(StepAction::Jumped);
            }

            // ── P2.3: IntSubCall1/IntAddCall1 inline — fib's #1 unpacked source ──
            Instruction::IntSubCall1(_) | Instruction::IntAddCall1(_) => {
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_super_instructions(instr, &mut ctx);
            }

            // ── P2.3: LoadGlobal/StoreGlobal/ClosureSlot inline ──
            Instruction::LoadGlobal { .. }
            | Instruction::StoreGlobal { .. }
            | Instruction::DeclGlobal { .. }
            | Instruction::StoreGlobalConst { .. }
            | Instruction::LoadClosureSlot { .. }
            | Instruction::StoreClosureSlot { .. } => {
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_variables(instr, &mut ctx);
            }

            // ── P2.3: IntModCmpI inline — higher_order's #1 unpacked source (2M) ──
            Instruction::IntModCmpI { .. } => {
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_int_cmp(instr, &mut ctx);
            }

            // ── P2.3: MethodCall inline — method_dispatch's 33% unpacked source ──
            Instruction::MethodCall { .. } => {
                #[cfg(feature = "telemetry")]
                {
                    self.telemetry.site_call_count += 1;
                }
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_methods_async_generator(instr, &mut ctx);
            }

            // ── P2.3: MakeArray/MakeArray2 inline — binary_trees 34% unpacked ──
            Instruction::MakeArray { .. } | Instruction::MakeArray2 { .. } => {
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_int_slot_super(instr, &mut ctx);
            }

            // ── P2.3: GetProperty/SetProperty inline — method_dispatch/binary_trees 33% unpacked ──
            Instruction::GetProperty { .. } | Instruction::SetProperty { .. } => {
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_classes_modules(instr, &mut ctx);
            }

            // ── P3.3: Call inline — bypasses dispatch_unpacked entirely ──
            Instruction::Call { .. } => {
                #[cfg(feature = "telemetry")]
                {
                    self.telemetry.site_call_count += 1;
                }
                let mut ctx = crate::vm::execute::StepContext {
                    instructions,
                    constants,
                    bytecode,
                    ip,
                    ip_ref,
                    chunk_ptr: self.current_chunk_ptr,
                };
                return self.step_call_load(instr, &mut ctx);
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
        #[cfg(feature = "telemetry")]
        {
            self.telemetry.unpacked_dispatch_count += 1;
            self.telemetry.last_dense = 0xFFFF;
            let name = crate::vm::telemetry::instruction_name(instr);
            *self
                .telemetry
                .unpacked_opcode_counts
                .entry(name)
                .or_insert(0) += 1;
        }
        match instr {
            // LoadNumConst/LoadIntConst → LoadNumConst/LoadIntConst handled elsewhere
            // IndexFast/IndexAssignFast removed → Index/IndexAssign handled elsewhere
            Instruction::Jump(_)
            | Instruction::JumpIfFalse { .. }
            | Instruction::JumpIfTrue { .. }
            | Instruction::TailCall { .. }
            | Instruction::CallSpread(_)
            | Instruction::MethodCallSpread { .. } => self.step_collections_calls(instr, ctx),

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
            | Instruction::PropertySubAssign { .. } => self.step_classes_modules(instr, ctx),

            Instruction::LoadModule(_) | Instruction::DefineFunction(_) => {
                self.step_module_ops(instr, ctx)
            }

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


            | Instruction::IntLeJumpIfFalse(_)
            | Instruction::IntLtJumpIfFalse(_) => self.step_super_instructions(instr, ctx),

            // Register-based VM instructions
            | Instruction::IntAdd { .. }
            | Instruction::IntSub { .. }
            | Instruction::IntDivI { .. }
            | Instruction::IntModI { .. }
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
            | Instruction::IntCmpRRJumpPacked { .. }
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
            | Instruction::NumSin { .. }
            | Instruction::NumCos { .. }
            | Instruction::FLoadNum { .. }
            | Instruction::FStoreNum { .. }
            | Instruction::FAdd { .. }
            | Instruction::FSub { .. }
            | Instruction::FMul { .. }
            | Instruction::FDiv { .. }
            | Instruction::FSin { .. }
            | Instruction::FCos { .. }
            | Instruction::FSqrt { .. }
            | Instruction::FConst { .. }
            | Instruction::FMove { .. }
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
            | Instruction::ObjLitSet { .. }
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
            | Instruction::MakeObject { .. }
            | Instruction::StoreConst { .. }
            | Instruction::StringContains { .. }
            | Instruction::CharDispatch { .. }
            // register-to-register moves and integer/slot super-instructions
            // are handled by the fast int/slot dispatcher.
            => self.step_int_slot_super(instr, ctx),
            // Opcodes handled by inline-hot match above — must never reach here.
            Instruction::Call { .. }
            | Instruction::IntSubCall1(_)
            | Instruction::IntAddCall1(_)
            | Instruction::IntModCmpI { .. }
            | Instruction::MethodCall { .. }
            | Instruction::MakeArray { .. }
            | Instruction::MakeArray2 { .. }
            | Instruction::GetProperty { .. }
            | Instruction::SetProperty { .. }
            | Instruction::LoadGlobal { .. }
            | Instruction::StoreGlobal { .. }
            | Instruction::StoreGlobalConst { .. }
            | Instruction::DeclGlobal { .. }
            | Instruction::LoadClosureSlot { .. }
            | Instruction::StoreClosureSlot { .. } => Err(Self::runtime_error_with_pos(
                "dispatch_unpacked: opcode should have been handled by inline-hot match",
                ctx.bytecode, ctx.ip)),
        }
    }
}
