use super::*;

/// Loop-Invariant Code Motion (LICM) — inactive skeleton (PERF-48 / Issue #951).
///
/// The original implementation hoisted `LoadVar(x)` reads into a prelude of
/// `LoadVar(x); DeclLocal(new_slot)` and rewrote in-body reads to
/// `LoadLocal(new_slot)`.  Those opcodes were removed when the VM moved to a
/// register-only local model under H.5, so the pass is currently a no-op.
/// The skeleton is preserved so a future register-based LICM pass can reuse
/// the loop-range analysis and source-position bookkeeping.
///
/// # Safety
/// * Only runs on the top-level `Bytecode::instructions`.  Function
///   chunks are never passed through this pass (see
///   `optimize_with_positions`).
/// * Loops containing `Call`, `MethodCall`, `DefineFunction`,
///   `TryBegin`, `Break`, `Yield`, `ForIn`, or any other opaque
///   instruction (returning `None` from `stack_effect`) are skipped
///   entirely — a wrong invariance assumption would corrupt semantics.
///
/// Kural 7c: no fallback path.  If an invariant is violated the pass
/// bails the offending loop (produces no change) rather than emitting
/// half-correct code.
pub fn loop_invariant_motion(
    instructions: &mut Vec<Instruction>,
    _constants: &[Value16],
    loop_payloads: &mut [hudhudscript_bytecode::LoopPayload],
    source_positions: &mut Vec<Option<(usize, usize)>>,
) {
    // Ensure source_positions matches instructions length up front.
    while source_positions.len() < instructions.len() {
        source_positions.push(None);
    }
    source_positions.truncate(instructions.len());

    // Iterate from the END backward so nested-loop insertions don't
    // invalidate indices of outer loops not yet processed.  Actually
    // we scan forward but jump past each loop's shifted `end` after
    // processing, and re-scan children in a second inner pass.  The
    // simplest safe strategy: keep re-scanning from position 0 until
    // no change is made in a full sweep — but that is O(N²) on deeply
    // nested loops.  Instead, process innermost first by finding all
    // LoopBegin positions, sorting by innermost (largest start,
    // smallest range), and hoisting one at a time while re-indexing
    // the surviving positions after each edit.
    //
    // MVP: single forward pass, skip nested loops by advancing `i`
    // past the outer loop's end (conservative — misses some hoists
    // in nested cases, but never wrong).
    let mut i = 0;
    while i < instructions.len() {
        // CROSS-2b: LoopBegin now carries a payload-pool index; resolve
        // start/end from the side table.
        let (loop_start, loop_end_raw) = match &instructions[i] {
            Instruction::LoopBegin(idx) => {
                let p = loop_payloads[*idx as usize];
                (p.start as usize, p.end as usize)
            }
            _ => {
                i += 1;
                continue;
            }
        };

        let loop_end: usize = loop_end_raw.min(instructions.len());
        let body_start = i + 1;
        if body_start >= loop_end {
            i = loop_end.max(i + 1);
            continue;
        }

        // Sanity: loop_start must precede the LoopBegin instruction
        // (compiler emits it that way — condition comes first).
        if loop_start > i {
            i += 1;
            continue;
        }

        // Validate the body: every instruction must have a known stack
        // effect.  Any opaque instruction = skip this loop entirely.
        let body_range = body_start..loop_end;
        let mut body_is_analyzable = true;
        for inst in &instructions[body_range.clone()] {
            if stack_effect(inst).is_none() {
                body_is_analyzable = false;
                break;
            }
        }
        if !body_is_analyzable {
            i = loop_end.max(i + 1);
            continue;
        }

        // Collect all variables written in the body — these are NOT
        // loop-invariant and cannot be hoisted.
        let mut written_vars: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for inst in &instructions[body_range.clone()] {
            if let Some(sym) = written_symbol(inst) {
                written_vars.insert(sym);
            }
        }

        // Collect candidate invariant LoadVars: symbols that appear
        // at least twice in the loop body (hoisting a single-use
        // LoadVar has no benefit), are never written, and are not
        // LICM disabled: LoadVar/DeclLocal removed from enum.
        for _ in body_range.clone() {}
        i = loop_end.max(i + 1);
        continue;

        // Step 3 — insert the prelude just before `loop_start`.  This
        // shifts every subsequent IP by `prelude_len`; relative jump
        // LICM disabled — LoadVar/DeclLocal removed. Body prelude
        // generation skipped. Jump offsets and payloads already adjusted
        // above.
        i = loop_end.max(i + 1);
        continue;
    }

    // Final invariant: source_positions must still be parallel.
    while source_positions.len() < instructions.len() {
        source_positions.push(None);
    }
    source_positions.truncate(instructions.len());
}
