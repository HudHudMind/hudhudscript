#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_collections_fast(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::MakeArray { dst, count } => {
                // ISSUE-9b: compiler now passes the known literal element count so
                // we can pre-allocate capacity and avoid repeated ArrayPush reallocations.
                let cap = *count as usize;
                let arr = if cap > 0 {
                    Vec::with_capacity(cap)
                } else {
                    Vec::new()
                };
                self.registers[*dst as usize] = Value16::array(arr);
            }
            Instruction::MakeObject { dst, count } => {
                let n = *count as usize;
                let properties = hudhudscript_bytecode::ObjMap::with_capacity(n);
                self.registers[*dst as usize] = Value16::object(properties);
            }
            Instruction::ObjLitSet { obj, val, prop_sym } => {
                #[cfg(feature = "telemetry")] { self.telemetry.site_property_count += 1; }
                let v = self.registers[*val as usize];
                let mut obj_val = self.registers[*obj as usize];
                obj_val.as_object_mut_unchecked()
                    .insert(hudhudscript_bytecode::SymId(*prop_sym as u32), v);
            }
            Instruction::ArrayPush { dst, arr, val } => {
                let v = self.registers[*val as usize];
                let mut arr_val = self.registers[*arr as usize];
                debug_assert!(arr_val.0.tag() == ReprTag::Dynamic);
                arr_val.as_array_mut_unchecked().push(v);
                self.registers[*dst as usize] = arr_val;
            }
            Instruction::ArrayPushIntConst { arr, const_idx } => {
                let val = Value16::int(ctx.bytecode.int_constants[*const_idx as usize]);
                let mut arr_val = self.registers[*arr as usize];
                arr_val.as_array_mut_unchecked().push(val);
                self.registers[*arr as usize] = arr_val;
            }
            // P8: 2-element array literal — single instruction, no push sequence
            Instruction::MakeArray2 { dst, a, b } => {
                let va = self.registers[*a as usize];
                let vb = self.registers[*b as usize];
                let mut arr = Vec::with_capacity(2);
                arr.push(va);
                arr.push(vb);
                self.registers[*dst as usize] = Value16::array(arr);
            }
            Instruction::ArrayPushConst { arr, const_idx } => {
                let val = ctx.constants[*const_idx as usize];
                let mut arr_val = self.registers[*arr as usize];
                arr_val.as_array_mut_unchecked().push(val);
                self.registers[*arr as usize] = arr_val;
            }
            // P2: fast path for array/string length and array pop
            Instruction::ArrayLen { dst, obj } => {
                let arr = self.registers[*obj as usize].as_array().ok_or_else(|| {
                    Self::runtime_error_with_pos("ArrayLen: expected array", ctx.bytecode, ctx.ip)
                })?;
                self.registers[*dst as usize] = Value16::int(arr.len() as i64);
            }
            Instruction::StringLen { dst, obj } => {
                let s = self.registers[*obj as usize].as_string().ok_or_else(|| {
                    Self::runtime_error_with_pos("StringLen: expected string", ctx.bytecode, ctx.ip)
                })?;
                // P2a: use byte len to match existing GetProperty .length semantics
                self.registers[*dst as usize] = Value16::int(s.len() as i64);
            }
            Instruction::ArrayPop { dst, obj } => {
                let mut arr_val = self.registers[*obj as usize];
                let vec = arr_val.as_array_mut().ok_or_else(|| {
                    Self::runtime_error_with_pos("ArrayPop: expected array", ctx.bytecode, ctx.ip)
                })?;
                let val = vec.pop().ok_or_else(|| {
                    Self::runtime_error_with_pos(
                        "Cannot pop from empty array",
                        ctx.bytecode,
                        ctx.ip,
                    )
                })?;
                self.registers[*dst as usize] = val;
                self.registers[*obj as usize] = arr_val;
            }
            Instruction::IntMulAddAssign { acc, src1, src2 } => {
                let a = self.registers[*src1 as usize];
                let b = self.registers[*src2 as usize];
                let dst = *acc;
                let product = crate::vm::bigint_arith::int_mul(a, b).map_err(|code| {
                    Self::runtime_error_with_pos(&code.to_string(), ctx.bytecode, ctx.ip)
                })?;
                #[cfg(feature = "telemetry")]
                self.record_bigint_promotion(a, b, product);
                let acc_val = self.registers[*acc as usize];
                let sum = crate::vm::bigint_arith::int_add(acc_val, product)
                    .map_err(|code| {
                        Self::runtime_error_with_pos(&code.to_string(), ctx.bytecode, ctx.ip)
                    })?;
                #[cfg(feature = "telemetry")]
                self.record_bigint_promotion(acc_val, product, sum);
                self.registers[dst as usize] = sum;
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
