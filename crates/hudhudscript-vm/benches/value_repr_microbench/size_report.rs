//! Size report printed before benchmarks.

use hudhudscript_bytecode::Value16;
use std::mem::size_of;

use super::variant_b::ReprArc;
use super::variant_c::ReprManual;

pub fn print_size_report() {
    eprintln!("─── size_of report ───");
    eprintln!(
        "A Value16          : size={:>3}  size<Option>={:>3}",
        size_of::<Value16>(),
        size_of::<Option<Value16>>()
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
