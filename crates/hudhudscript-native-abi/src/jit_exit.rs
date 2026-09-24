//! Uniform native-entry exit block (JIT_AOT_ARCHITECTURE §6.2/§12.4).
//!
//! Native entry ABI (stage one):
//! `extern "C" fn(argc: u32, args: *const i64, out: *mut JitExit)`
//! — parameters are read from `args`, results are WRITTEN through `out`
//! (sret-by-pointer: portable across SysV/Win64/AAPCS64, no struct-return
//! ambiguity), the function itself returns nothing.

/// Exit status codes. Stage one: Returned + the §18 integer lanes.
pub const JIT_EXIT_RETURNED: i32 = 0;
pub const JIT_EXIT_OVERFLOW: i32 = 1;
pub const JIT_EXIT_DIV_ZERO: i32 = 2;
pub const JIT_EXIT_UNCAUGHT_EXCEPTION: i32 = 3;

/// Result block written by every native function before returning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct JitExit {
    pub status: i32,
    pub value: i64,
}

impl JitExit {
    pub const fn returned(value: i64) -> JitExit {
        JitExit { status: JIT_EXIT_RETURNED, value }
    }

    pub const fn overflow() -> JitExit {
        JitExit { status: JIT_EXIT_OVERFLOW, value: 0 }
    }

    pub fn status_name(&self) -> &'static str {
        match self.status {
            JIT_EXIT_RETURNED => "Returned",
            JIT_EXIT_OVERFLOW => "Overflow",
            JIT_EXIT_DIV_ZERO => "DivZero",
            JIT_EXIT_UNCAUGHT_EXCEPTION => "UncaughtException",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_is_16_bytes_8_aligned() {
        assert_eq!(std::mem::size_of::<JitExit>(), 16);
        assert_eq!(std::mem::align_of::<JitExit>(), 8);
    }

    #[test]
    fn constructors_and_names() {
        let r = JitExit::returned(5);
        assert_eq!(r.status, JIT_EXIT_RETURNED);
        assert_eq!(r.value, 5);
        assert_eq!(r.status_name(), "Returned");
        assert_eq!(JitExit::overflow().status_name(), "Overflow");
    }
}
