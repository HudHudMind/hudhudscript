use crate::error::compile_codes;
use crate::{Bytecode, BytecodeResult, Instruction, Value16, BYTECODE_VERSION};

impl Bytecode {
    /// Serialize the canonical bytecode wire format.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bc = self.clone();
        bc.symbols = crate::interner::snapshot();
        bc.serialized_function_names = self
            .function_entries_by_index()
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        postcard::to_stdvec(&bc).map_err(|error| format!("postcard serialize: {}", error))
    }

    /// Deserialize the sole supported postcard wire format.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut bc: Self = postcard::from_bytes(bytes)
            .map_err(|error| format!("postcard deserialize: {}", error))?;
        if bc.version != BYTECODE_VERSION {
            return Err(format!(
                "Bytecode version mismatch: expected {}, got {}",
                BYTECODE_VERSION, bc.version
            ));
        }
        if !bc.symbols.is_empty() {
            crate::interner::restore(bc.symbols.clone())
                .map_err(|error| format!("interner restore: {}", error))?;
        }
        bc.rebuild_function_names()
            .map_err(|error| error.to_string())?;
        bc.resolve_call_payload_function_indices();
        Ok(bc)
    }

    /// Serialize to JSON for debugging and inspection.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize JSON produced by `to_json`.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate bytecode before execution.
    pub fn validate(&self) -> BytecodeResult<()> {
        Self::validate_instructions(&self.instructions, &self.constants, self, "main")?;
        for (index, chunk) in self.functions.borrow().iter().enumerate() {
            Self::validate_instructions(
                &chunk.instructions,
                &chunk.constants,
                self,
                &format!("fn:{}", index),
            )?;
        }
        Ok(())
    }

    fn validate_instructions(
        instructions: &[Instruction],
        constants: &[Value16],
        bytecode: &Bytecode,
        label: &str,
    ) -> BytecodeResult<()> {
        let len = instructions.len();
        for (ip, instruction) in instructions.iter().enumerate() {
            match instruction {
                Instruction::Jump(offset)
                | Instruction::TryBegin(offset)
                | Instruction::FinallyBegin(offset)
                | Instruction::FinallyExit(offset) => {
                    validate_relative_target(ip, *offset, len, label)?;
                }
                Instruction::JumpIfFalse { src: 255, offset }
                | Instruction::JumpIfTrue { src: 255, offset } => {
                    validate_relative_target(ip, i32::from(*offset), len, label)?;
                }
                Instruction::IterNext { end_offset, .. } => {
                    validate_relative_target(ip, i32::from(*end_offset), len, label)?;
                }
                Instruction::LoadConst { const_idx, .. } => {
                    validate_index(*const_idx as usize, constants.len(), "constant", ip, label)?;
                }
                Instruction::LoadIntConst { const_idx, .. } => {
                    validate_index(
                        *const_idx as usize,
                        bytecode.int_constants.len(),
                        "int constant",
                        ip,
                        label,
                    )?;
                }
                Instruction::LoopBegin(index) => validate_index(
                    *index as usize,
                    bytecode.loop_payloads.len(),
                    "loop payload",
                    ip,
                    label,
                )?,
                Instruction::EnumDecl(index) => validate_index(
                    *index as usize,
                    bytecode.enum_decl_payloads.len(),
                    "enum payload",
                    ip,
                    label,
                )?,
                Instruction::ClassDecl(index) => validate_index(
                    *index as usize,
                    bytecode.class_decl_payloads.len(),
                    "class payload",
                    ip,
                    label,
                )?,
                Instruction::TraitCheck(index) => validate_index(
                    *index as usize,
                    bytecode.trait_check_payloads.len(),
                    "trait payload",
                    ip,
                    label,
                )?,
                Instruction::LoadModule(index) => validate_index(
                    *index as usize,
                    bytecode.load_module_payloads.len(),
                    "load-module payload",
                    ip,
                    label,
                )?,
                Instruction::DefineFunction(index) => validate_index(
                    *index as usize,
                    bytecode.define_function_payloads.len(),
                    "define-function payload",
                    ip,
                    label,
                )?,
                Instruction::ClassStaticDecl(index) => validate_index(
                    *index as usize,
                    bytecode.class_static_decl_payloads.len(),
                    "class-static payload",
                    ip,
                    label,
                )?,
                Instruction::DestructObject(index) => validate_index(
                    *index as usize,
                    bytecode.destruct_object_payloads.len(),
                    "destruct-object payload",
                    ip,
                    label,
                )?,
                Instruction::Call { payload_idx, .. }
                | Instruction::MethodCall { payload_idx, .. }
                | Instruction::SuperCall { payload_idx, .. }
                | Instruction::NewInstance { payload_idx, .. }
                | Instruction::MakeGenerator { payload_idx, .. } => validate_index(
                    *payload_idx as usize,
                    bytecode.call_payloads.len(),
                    "call payload",
                    ip,
                    label,
                )?,
                Instruction::MatchVariant(index) | Instruction::GetStatic(index) => {
                    validate_index(
                        *index as usize,
                        bytecode.two_sym_payloads.len(),
                        "two-symbol payload",
                        ip,
                        label,
                    )?;
                }
                Instruction::DeclStore { payload_idx, .. } => validate_index(
                    *payload_idx as usize,
                    bytecode.two_sym_payloads.len(),
                    "two-symbol payload",
                    ip,
                    label,
                )?,
                Instruction::Remember { store_idx, .. }
                | Instruction::Recall { store_idx, .. }
                | Instruction::Forget { store_idx, .. } => validate_index(
                    *store_idx as usize,
                    bytecode.opt_sym_payloads.len(),
                    "optional-symbol payload",
                    ip,
                    label,
                )?,
                Instruction::Spawn { .. } | Instruction::TailCall { .. } => {}
                _ => {}
            }
        }
        Ok(())
    }
}

fn validate_relative_target(ip: usize, offset: i32, len: usize, label: &str) -> BytecodeResult<()> {
    let target = (ip as i64).wrapping_add(i64::from(offset));
    if target < 0 || target > len as i64 {
        return Err(compile_codes::runtime_error(format!(
            "Invalid relative jump at ip={} offset={} -> target={} out of range [0,{}] in {}",
            ip, offset, target, len, label
        )));
    }
    Ok(())
}

fn validate_index(
    index: usize,
    pool_len: usize,
    pool_name: &str,
    ip: usize,
    label: &str,
) -> BytecodeResult<()> {
    if index >= pool_len {
        return Err(compile_codes::runtime_error(format!(
            "Invalid {} index {} at instruction {} in {}; pool size is {}",
            pool_name, index, ip, label, pool_len
        )));
    }
    Ok(())
}
