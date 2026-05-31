use hudhudscript_bytecode::{packed_instruction, Instruction};

pub(crate) const PACK_SENTINEL: u32 = u32::MAX;

/// Pre-pack all instructions into compact 32-bit form for fast dispatch.
/// Instructions that cannot be packed (complex payloads) get
/// `PACK_SENTINEL` (= `u32::MAX`), which `execute_instructions`
/// checks against to fall through to the full `match`.
pub(crate) fn prepack_instructions(instructions: &[Instruction]) -> Vec<u32> {
    instructions
        .iter()
        .map(|i| packed_instruction::pack(i).unwrap_or(PACK_SENTINEL))
        .collect()
}
