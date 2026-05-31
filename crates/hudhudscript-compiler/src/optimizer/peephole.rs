use super::*;

/// Peephole optimizer — disabled.
/// Stack-based patterns removed. Register-based peephole TBD.
pub fn peephole_optimize_with_positions(
    _instructions: &mut Vec<Instruction>,
    _source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    // No-op
}
