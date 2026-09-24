//! Backend-independent MIR optimization passes (JIT_AOT_ARCHITECTURE §6).
//!
//! AŞAMA-0 pass: `const_fold`. Semantics rule (§18): folding must be
//! OBSERVATIONALLY IDENTICAL to execution — integer overflow and
//! division-by-zero are NEVER folded (they belong to the runtime error
//! lanes); IEEE-deterministic float ops fold.

use hudhudscript_mir::MirFunction;

pub mod const_fold;
pub mod const_prop;
pub mod dce;

pub use const_fold::const_fold;
pub use const_prop::const_prop;
pub use dce::dce;

/// Tam optimizasyon pipeline'ı: fold → prop → dce döngüsü sabit noktaya
/// dek (maks 4 tur). §18: taşma/bölme asla katlanmaz (const_fold sözleşmesi).
pub fn optimize(f: &MirFunction) -> (MirFunction, usize) {
    let mut cur = f.clone();
    let mut total = 0usize;
    for _ in 0..4 {
        let (a, n1) = const_fold(&cur);
        let (b, n2) = const_prop(&a);
        let (c, n3) = dce(&b);
        let round = n1 + n2 + n3;
        cur = c;
        total += round;
        if round == 0 {
            break;
        }
    }
    (cur, total)
}
