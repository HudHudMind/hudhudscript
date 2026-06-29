// helpers_loop.rs — loop/chain compiler utilities
//
// Previously held `compute_max_register` which scanned instructions for the
// highest register index.  That function is no longer needed: the canonical
// function compilation context (`compile_function_chunk_with`) tracks
// `current_max_register` via the emitter and writes it into FunctionChunk
// automatically.
//
// This file is kept as a module place holder.  Future loop helpers (e.g.
// loop-specific validation, CFG analysis) may live here.
