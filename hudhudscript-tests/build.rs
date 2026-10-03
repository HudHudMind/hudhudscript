// Link flags for the suite's own test binaries.
//
// The in-process MCJIT test (tests/unit/test_llvm_jit.rs) resolves host
// helper symbols (hudhud_select_i64, num_mul, …) from the process dynsym,
// so its binary must be linked with --export-dynamic. A dependency's
// build.rs link args never reach this package's binaries, so the flag is
// emitted here — but ONLY when the opt-in `llvm` feature is enabled:
// blanket -rdynamic on every suite binary recreates the pyo3 GC-root
// link failure (see v0.9.22/v0.9.23 history in the main repo).
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let llvm_backend = std::env::var_os("CARGO_FEATURE_LLVM").is_some();
    if llvm_backend && cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,--export-dynamic");
    }
}
