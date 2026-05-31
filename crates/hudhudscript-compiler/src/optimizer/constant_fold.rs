use super::*;

/// Constant folding — disabled.
/// Stack-based arithmetic ops (Add, Sub, etc.) have been removed.
/// Register-based constant folding will be added in a future iteration.
pub fn constant_fold(
    _instructions: &mut Vec<Instruction>,
    _constants: &mut Vec<Value16>,
    _numeric_constants: &mut Vec<u64>,
) {
    // No-op: register-based constant folding not yet implemented.
}

pub fn abs_target(ip: usize, offset: i32) -> usize {
    (ip as i64 + offset as i64) as usize
}

pub fn constant_fold_with_positions(
    instructions: &mut Vec<Instruction>,
    constants: &mut Vec<Value16>,
    numeric_constants: &mut Vec<u64>,
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    constant_fold(instructions, constants, numeric_constants);
    source_positions.truncate(instructions.len());
}
