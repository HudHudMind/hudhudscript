#![allow(unused_imports)]
use super::*;
use crate::vm::index_helpers::{index_i64_to_usize, numeric_index_i64};

/// Safety limit for Vec::resize — prevents OOM from user-controlled index.
const MAX_ARRAY_SIZE: usize = 268_435_456; // 2^28

impl VM {
    #[inline(always)]
    pub(crate) fn step_indexing(
        &mut self,
        instr: &Instruction,
        ctx: &mut StepContext<'_>,
    ) -> CompileResult<StepAction> {
        let instructions = ctx.instructions;
        let bytecode = ctx.bytecode;
        let ip = ctx.ip;
        let _ip_ref = &mut *ctx.ip_ref;
        match instr {
            Instruction::Index { dst, obj, idx } => {
                #[cfg(feature = "telemetry")] { self.telemetry.site_index_count += 1; }
                let obj_val = &self.registers[*obj as usize];
                let idx_val = &self.registers[*idx as usize];

                if let Some(arr) = obj_val.as_array() {
                    let i = numeric_index_i64(*idx_val)
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            Self::runtime_error_with_pos(
                                "Index must be a non-negative number",
                                bytecode,
                                ip,
                            )
                        })?;
                    if i < arr.len() {
                        self.registers[*dst as usize] = arr[i].clone();
                    } else {
                        return Err(Self::runtime_error_with_pos(
                            format!("Array index out of bounds: {}", i),
                            bytecode,
                            ip,
                        ));
                    }
                } else if let Some(s) = obj_val.as_str() {
                    #[cfg(feature = "telemetry")]
                    { self.telemetry.string_index_count += 1; }
                    let i = numeric_index_i64(*idx_val)
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            Self::runtime_error_with_pos(
                                "Index must be a non-negative number",
                                bytecode,
                                ip,
                            )
                        })?;
                    // ASCII fast path: byte index avoids O(n) char iteration
                    let bytes = s.as_bytes();
                    let result = if i < bytes.len() && bytes[i].is_ascii() {
                        Value16::string_ascii(bytes[i])
                    } else {
                        s.chars()
                            .nth(i)
                            .map(|c| {
                                let cs = c.to_string();
                                #[cfg(feature = "telemetry")]
                                {
                                    self.telemetry.string_index_clone_count += 1;
                                    self.telemetry.string_index_clone_bytes += cs.len() as u64;
                                }
                                Value16::string(cs)
                            })
                            .ok_or_else(|| {
                                Self::runtime_error_with_pos(
                                    format!("String index out of bounds: {}", i),
                                    bytecode,
                                    ip,
                                )
                            })?
                    };
                    self.registers[*dst as usize] = result;
                } else if let Some(map) = obj_val.as_object() {
                    let key = idx_val.as_string().unwrap_or_default();
                    // SOP: subject instance state read via index — read from live instance
                    let result = if map.get("__type").and_then(|v| v.as_string()).as_deref()
                        == Some("subject_instance")
                    {
                        if let Some(id) = map.get("__instance_id").and_then(|v| v.as_string()) {
                            if let Some(inst) = self.subject_instances.get(&id) {
                                if let Some(val) = inst.state.get(&key) {
                                    *val
                                } else {
                                    map.get(&key).copied().unwrap_or(Value16::null())
                                }
                            } else {
                                // SOP0005: despawned subject accessed
                                return Err(compile_codes::runtime_error(format!(
                                    "Cannot access index '{}' on despawned subject '{}'",
                                    key, id
                                )));
                            }
                        } else {
                            map.get(&key).copied().unwrap_or(Value16::null())
                        }
                    } else {
                        map.get(&key).copied().unwrap_or(Value16::null())
                    };
                    self.registers[*dst as usize] = result;
                } else if let Some(inst) = obj_val.as_instance_data() {
                    let key = idx_val.as_string().unwrap_or_default();
                    let result = inst.fields.get(&key).copied().unwrap_or(Value16::null());
                    self.registers[*dst as usize] = result;
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "Index: expected array or string",
                        bytecode,
                        ip,
                    ));
                }
            }
            Instruction::IndexArray { dst, obj, idx } => {
                let arr = self.registers[*obj as usize].as_array().ok_or_else(|| {
                    Self::runtime_error_with_pos("IndexArray: expected array", bytecode, ip)
                })?;
                let i = numeric_index_i64(self.registers[*idx as usize])
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            "IndexArray: index must be a non-negative number",
                            bytecode,
                            ip,
                        )
                    })?;
                if i >= arr.len() {
                    return Err(Self::runtime_error_with_pos(
                        format!("Array index out of bounds: {}", i),
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*dst as usize] = arr[i].clone();
            }
            Instruction::IndexStringAscii { dst, obj, idx } => {
                let s = self.registers[*obj as usize].as_str().ok_or_else(|| {
                    Self::runtime_error_with_pos("IndexStringAscii: expected string", bytecode, ip)
                })?;
                #[cfg(feature = "telemetry")]
                { self.telemetry.string_index_count += 1; }
                let i = numeric_index_i64(self.registers[*idx as usize])
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            "IndexStringAscii: index must be a non-negative number",
                            bytecode,
                            ip,
                        )
                    })?;
                let bytes = s.as_bytes();
                self.registers[*dst as usize] = if i < bytes.len() && bytes[i].is_ascii() {
                    Value16::string_ascii(bytes[i])
                } else {
                    s.chars()
                        .nth(i)
                        .map(|c| {
                            let cs = c.to_string();
                            #[cfg(feature = "telemetry")]
                            {
                                self.telemetry.string_index_clone_count += 1;
                                self.telemetry.string_index_clone_bytes += cs.len() as u64;
                            }
                            Value16::string(cs)
                        })
                        .ok_or_else(|| {
                            Self::runtime_error_with_pos(
                                format!("String index out of bounds: {}", i),
                                bytecode,
                                ip,
                            )
                        })?
                };
            }
            Instruction::IndexAssign { obj, idx, val } => {
                let idx_val = self.registers[*idx as usize];
                let new_val = self.registers[*val as usize];
                let obj_ref = &mut self.registers[*obj as usize];
                if let Some(arr) = obj_ref.as_array_mut() {
                    let i = numeric_index_i64(idx_val)
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            Self::runtime_error_with_pos(
                                "Array index must be a number",
                                bytecode,
                                ip,
                            )
                        })?;
                    if i >= arr.len() {
                        if i >= MAX_ARRAY_SIZE {
                            return Err(Self::runtime_error_with_pos(
                                format!("Array index too large: {}", i), bytecode, ip));
                        }
                        arr.resize(i + 1, Value16::null());
                    }
                    arr[i] = new_val;
                } else if let Some(map) = obj_ref.as_object_mut() {
                    let key = idx_val.as_string().unwrap_or_default();
                    map.insert(key, new_val);
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "Cannot index-assign into non-array/object",
                        bytecode,
                        ip,
                    ));
                }
            }
            Instruction::IndexAssignArray { obj, idx, val } => {
                let idx_val = self.registers[*idx as usize];
                let new_val = self.registers[*val as usize];
                let i = numeric_index_i64(idx_val)
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            "IndexAssignArray: index must be a number",
                            bytecode,
                            ip,
                        )
                    })?;
                let arr = self.registers[*obj as usize]
                    .as_array_mut()
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            "IndexAssignArray: expected array",
                            bytecode,
                            ip,
                        )
                    })?;
                if i >= arr.len() {
                    if i >= MAX_ARRAY_SIZE {
                        return Err(Self::runtime_error_with_pos(
                            format!("Array index too large: {}", i), bytecode, ip));
                    }
                    arr.resize(i + 1, Value16::null());
                }
                arr[i] = new_val;
            }
            Instruction::Index2D {
                dst,
                obj,
                idx1,
                idx2,
            } => {
                let obj_val = self.registers[*obj as usize];
                let row_arr = if let Some(arr) = obj_val.as_array() {
                    let i = numeric_index_i64(self.registers[*idx1 as usize])
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            Self::runtime_error_with_pos("Index2D: idx1 not numeric", bytecode, ip)
                        })?;
                    arr.get(i).cloned().ok_or_else(|| {
                        Self::runtime_error_with_pos("Index2D: row1 OOB", bytecode, ip)
                    })?
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "Index2D: obj not array",
                        bytecode,
                        ip,
                    ));
                };
                let i2 = numeric_index_i64(self.registers[*idx2 as usize])
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos("Index2D: idx2 not numeric", bytecode, ip)
                    })?;
                let result = if let Some(inner) = row_arr.as_array() {
                    inner.get(i2).cloned().ok_or_else(|| {
                        Self::runtime_error_with_pos("Index2D: row2 OOB", bytecode, ip)
                    })?
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "Index2D: row not array",
                        bytecode,
                        ip,
                    ));
                };
                self.registers[*dst as usize] = result;
            }
            Instruction::IndexAssign2D {
                obj,
                idx1,
                idx2,
                val,
            } => {
                let mut obj_val = self.registers[*obj as usize];
                let row_arr = if let Some(arr) = obj_val.as_array_mut() {
                    let i = numeric_index_i64(self.registers[*idx1 as usize])
                        .and_then(index_i64_to_usize)
                        .ok_or_else(|| {
                            Self::runtime_error_with_pos(
                                "IndexAssign2D: idx1 not numeric",
                                bytecode,
                                ip,
                            )
                        })?;
                    arr.get_mut(i).ok_or_else(|| {
                        Self::runtime_error_with_pos("IndexAssign2D: row1 OOB", bytecode, ip)
                    })?
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "IndexAssign2D: obj not array",
                        bytecode,
                        ip,
                    ));
                };
                let i2 = numeric_index_i64(self.registers[*idx2 as usize])
                    .and_then(index_i64_to_usize)
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            "IndexAssign2D: idx2 not numeric",
                            bytecode,
                            ip,
                        )
                    })?;
                if let Some(inner) = row_arr.as_array_mut() {
                    if i2 < inner.len() {
                        inner[i2] = self.registers[*val as usize];
                    } else {
                        return Err(Self::runtime_error_with_pos(
                            "IndexAssign2D: row2 OOB",
                            bytecode,
                            ip,
                        ));
                    }
                } else {
                    return Err(Self::runtime_error_with_pos(
                        "IndexAssign2D: row not array",
                        bytecode,
                        ip,
                    ));
                }
                self.registers[*obj as usize] = obj_val;
            }

            _ => unreachable!("instruction routed to wrong execute helper"),
        }
        Ok(StepAction::Advance)
    }
}
