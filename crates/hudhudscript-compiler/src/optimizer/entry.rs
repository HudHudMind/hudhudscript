use super::*;

/// Run optimization passes on the instruction stream at the given level.
pub fn optimize(
    instructions: &mut Vec<Instruction>,
    constants: &mut Vec<Value16>,
    level: OptimizationLevel,
    bytecode_functions: Option<&HashMap<String, Arc<FunctionChunk>>>,
) {
    optimize_with_numerics(
        instructions,
        constants,
        &mut Vec::new(),
        &[],
        &mut Vec::new(),
        &mut Vec::new(),
        level,
        bytecode_functions,
        &[],
    );
}

/// Run optimization passes with access to the packed numeric constant pool.
///
/// CROSS-2b: also threads the `loop_payloads` side table so the LICM pass
/// can rewrite `LoopBegin` operand fields (which now live in the pool, not
/// inline in the instruction).
/// CROSS-2c: `call_payloads` threaded so `inline_small_functions` can
/// still resolve callee names after the `Call(u32)` externalisation.
/// A2: `super_instr_payloads` threaded so super-instruction fusion
/// (`IntSubCall1`) can register its three-operand payloads.
pub fn optimize_with_numerics(
    instructions: &mut Vec<Instruction>,
    constants: &mut Vec<Value16>,
    numeric_constants: &mut Vec<u64>,
    int_constants: &[i64],
    loop_payloads: &mut Vec<hudhudscript_bytecode::LoopPayload>,
    super_instr_payloads: &mut Vec<hudhudscript_bytecode::SuperInstrPayload>,
    level: OptimizationLevel,
    bytecode_functions: Option<&HashMap<String, Arc<FunctionChunk>>>,
    call_payloads: &[CallPayload],
) {
    // Kept for external callers that don't care about source positions.
    // Internally we prefer `optimize_with_positions` so the debug hook's
    // ip → (line, col) lookup survives optimizer-induced shifts.
    let mut scratch = Vec::new();
    optimize_with_positions(
        instructions,
        constants,
        numeric_constants,
        int_constants,
        loop_payloads,
        super_instr_payloads,
        &mut scratch,
        level,
        bytecode_functions,
        call_payloads,
    );
}

/// Run optimization passes while keeping `source_positions` parallel to
/// `instructions` after every pass.
///
/// The DAP debugger's `on_statement` hook relies on
/// `bytecode.source_positions[ip]` matching `bytecode.instructions[ip]`
/// — if the optimizer drops or shifts instructions without touching
/// source_positions, the debugger pauses at the wrong file:line.
pub fn optimize_with_positions(
    instructions: &mut Vec<Instruction>,
    constants: &mut Vec<Value16>,
    numeric_constants: &mut Vec<u64>,
    int_constants: &[i64],
    loop_payloads: &mut Vec<hudhudscript_bytecode::LoopPayload>,
    super_instr_payloads: &mut Vec<hudhudscript_bytecode::SuperInstrPayload>,
    source_positions: &mut Vec<Option<(usize, usize)>>,
    level: OptimizationLevel,
    bytecode_functions: Option<&HashMap<String, Arc<FunctionChunk>>>,
    call_payloads: &[CallPayload],
) {
    // Ensure the invariant holds at entry — any raw push sites that
    // bypassed push_instr will leave source_positions shorter.
    while source_positions.len() < instructions.len() {
        source_positions.push(None);
    }
    source_positions.truncate(instructions.len());

    match level {
        OptimizationLevel::None => {}
        OptimizationLevel::Basic => {
            constant_fold_with_positions(
                instructions,
                constants,
                numeric_constants,
                source_positions,
            );
            dead_code_eliminate_with_positions(instructions, source_positions);
            // I6: slot+immediate fusion runs in Basic too so the default
            // compile path benefits (peephole / LICM stay gated to
            // Aggressive, but the fusion is always safe).
            fuse_slot_immediate_with_positions(
                instructions,
                numeric_constants,
                int_constants,
                loop_payloads,
                source_positions,
            );
            // A2: super-instruction fusion (bigram collapse) runs AFTER
            // `fuse_slot_immediate_with_positions` so it can see the
            // post-I6 shape (`IntSubISlot + Call` etc).  Safe in Basic
            // because each rewrite is pattern-matched and never crosses
            // a jump target (Kural 7c — no fallback path).
            fuse_super_instructions_with_positions(
                instructions,
                loop_payloads,
                call_payloads,
                super_instr_payloads,
                source_positions,
            );
        }
        OptimizationLevel::Aggressive => {
            constant_fold_with_positions(
                instructions,
                constants,
                numeric_constants,
                source_positions,
            );
            dead_code_eliminate_with_positions(instructions, source_positions);
            peephole_optimize_with_positions(instructions, source_positions);
            loop_invariant_motion(instructions, constants, loop_payloads, source_positions);
            // I6: fusion runs AFTER LICM (see brief): LICM relies on
            // unfused `LoadLocal` shapes to reason about invariance; once
            // LICM has finished, fusing the remaining hot triples is safe.
            fuse_slot_immediate_with_positions(
                instructions,
                numeric_constants,
                int_constants,
                loop_payloads,
                source_positions,
            );
            // A2: super-instruction fusion — must run AFTER I6 so it can
            // pick up the `IntSubISlot + Call` bigram that I6 produces.
            fuse_super_instructions_with_positions(
                instructions,
                loop_payloads,
                call_payloads,
                super_instr_payloads,
                source_positions,
            );
            if let Some(funcs) = bytecode_functions {
                inline_small_functions(instructions, funcs, call_payloads);
            }
        }
    }

    // Final safety net: if any pass drifted the parallel-vector
    // invariant, restore it.
    while source_positions.len() < instructions.len() {
        source_positions.push(None);
    }
    source_positions.truncate(instructions.len());
}
