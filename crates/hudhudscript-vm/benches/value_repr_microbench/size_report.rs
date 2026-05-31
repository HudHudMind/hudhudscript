//! Size report printed before benchmarks.

use hudhudscript_bytecode::Value;
use std::mem::size_of;

use super::variant_b::ReprArc;
use super::variant_c::ReprManual;

pub fn print_size_report() {
    eprintln!("─── size_of report ───");
    eprintln!(
        "A Value            : size={:>3}  size<Option>={:>3}",
        size_of::<Value>(),
        size_of::<Option<Value>>()
    );
    eprintln!(
        "B ReprArc          : size={:>3}  size<Option>={:>3}",
        size_of::<ReprArc>(),
        size_of::<Option<ReprArc>>()
    );
    eprintln!(
        "C ReprManual       : size={:>3}  size<Option>={:>3}",
        size_of::<ReprManual>(),
        size_of::<Option<ReprManual>>()
    );
    eprintln!("──────────────────────");
}
