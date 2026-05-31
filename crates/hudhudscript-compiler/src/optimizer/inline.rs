use super::*;

/// Function inlining — detection pass (Issue #952).
///
/// Scans `Call` instructions and checks whether the called function is a small,
/// side-effect-free candidate for inlining (< 10 instructions, no loops, no
/// recursion, no try/catch).
///
/// This is a **detection-only** pass — actual inlining requires careful stack
/// management and instruction rewriting that is deferred until an SSA-based IR
/// is available.
///
/// CROSS-2c: `Call` now carries a `u32` payload index — `call_payloads`
/// is threaded through so we can still resolve callee names for the
/// self-recursion check.
pub fn inline_small_functions(
    instructions: &mut [Instruction],
    bytecode_functions: &HashMap<String, Arc<FunctionChunk>>,
    call_payloads: &[CallPayload],
) {
    let mut _inlinable_count: usize = 0;

    for instr in instructions.iter() {
        if let Instruction::Call { payload_idx, .. } = instr {
            let Some(payload) = call_payloads.get(*payload_idx as usize) else {
                continue;
            };
            let name = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(payload.sym.0));
            if let Some(chunk) = bytecode_functions.get(&name) {
                let body_len = chunk.instructions.len();
                if body_len == 0 || body_len >= 10 {
                    continue;
                }

                // Reject functions that contain loops, recursion, or try/catch.
                let name_sym = payload.sym;
                let has_disqualifier = chunk.instructions.iter().any(|ci| match ci {
                    Instruction::LoopBegin(_) | Instruction::TryBegin(_) => true,
                    Instruction::Call { payload_idx: idx, .. } => call_payloads
                        .get(*idx as usize)
                        .map(|p| p.sym == name_sym)
                        .unwrap_or(false),
                    _ => false,
                });

                if !has_disqualifier {
                    _inlinable_count += 1;
                }
            }
        }
    }
}
