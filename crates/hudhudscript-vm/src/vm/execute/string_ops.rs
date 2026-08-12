#![allow(unused_imports)]
use super::*;

impl VM {
    #[inline(always)]
    pub(crate) fn step_string_ops(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::StrCat { dst, src1, src2 } => {
                let l = self.registers[*src1 as usize];
                let r = self.registers[*src2 as usize];
                self.registers[*dst as usize] =
                    hudhudscript_bytecode::shared_value::shared_add(&l, &r)?;
            }
            Instruction::StrCat3 { dst, a, b, c } => {
                let va = self.registers[*a as usize];
                let vb = self.registers[*b as usize];
                let vc = self.registers[*c as usize];
                let ab = hudhudscript_bytecode::shared_value::shared_add(&va, &vb)?;
                self.registers[*dst as usize] =
                    hudhudscript_bytecode::shared_value::shared_add(&ab, &vc)?;
            }
            Instruction::StringConcat {
                regs_start,
                count,
                dst,
            } => {
                let count = *count as usize;
                let start = *regs_start as usize;
                let mut total_len = 0;
                let mut has_non_string = false;

                for i in 0..count {
                    let v = &self.registers[start + i];
                    if let Some(s) = v.as_str() {
                        total_len += s.len();
                    } else {
                        has_non_string = true;
                        break;
                    }
                }

                if !has_non_string {
                    let mut result = String::with_capacity(total_len);
                    for i in 0..count {
                        result.push_str(self.registers[start + i].as_str_unchecked());
                    }
                    self.registers[*dst as usize] = Value16::string(result);
                } else {
                    let mut result = String::new();
                    for i in 0..count {
                        result.push_str(&self.registers[start + i].display_string());
                    }
                    self.registers[*dst as usize] = Value16::string(result);
                }
            }
            Instruction::StrCatMut { dst, src2 } => {
                let r = self.registers[*src2 as usize];
                let dst_ref = &mut self.registers[*dst as usize];
                if let (Some(s), Some(r_str)) = (dst_ref.as_string_mut(), r.as_str()) {
                    let was_ascii = r_str.is_ascii();
                    s.push_str(r_str);
                    // P4: downgrade StringAscii→String if appended non-ASCII.
                    if !was_ascii {
                        dst_ref.downgrade_string_ascii();
                    }
                } else {
                    let l = *dst_ref;
                    *dst_ref = hudhudscript_bytecode::shared_value::shared_add(&l, &r)?;
                }
            }
            Instruction::StringIndexOf {
                dst,
                haystack,
                needle,
            } => {
                let s = self.registers[*haystack as usize].as_str_unchecked();
                let pat = self.registers[*needle as usize].as_str_unchecked();
                let idx = s.find(pat).map(|i| i as i64).unwrap_or(-1);
                self.registers[*dst as usize] = Value16::int(idx);
            }
            Instruction::StringContains {
                dst,
                haystack,
                needle,
            } => {
                let s = self.registers[*haystack as usize].as_str_unchecked();
                let pat = self.registers[*needle as usize].as_str_unchecked();
                self.registers[*dst as usize] = Value16::boolean(s.contains(pat));
            }
            Instruction::StrCharEqRR {
                dst,
                src_s,
                src_i,
                src_j,
            } => {
                let s_val = self.registers[*src_s as usize];
                let i_val = self.registers[*src_i as usize];
                let j_val = self.registers[*src_j as usize];
                let ni = crate::vm::index_helpers::numeric_index_i64(i_val)
                    .and_then(crate::vm::index_helpers::index_i64_to_usize);
                let nj = crate::vm::index_helpers::numeric_index_i64(j_val)
                    .and_then(crate::vm::index_helpers::index_i64_to_usize);
                let result = if let (Some(s), Some(i), Some(j)) = (s_val.as_str(), ni, nj) {
                    let bytes = s.as_bytes();
                    // ASCII byte fast path
                    if i < bytes.len()
                        && j < bytes.len()
                        && bytes[i].is_ascii()
                        && bytes[j].is_ascii()
                    {
                        bytes[i] == bytes[j]
                    } else {
                        let ci = s.chars().nth(i);
                        let cj = s.chars().nth(j);
                        ci == cj
                    }
                } else {
                    false
                };
                self.registers[*dst as usize] = Value16::bool_(result);
            }
            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
