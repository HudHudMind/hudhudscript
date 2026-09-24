//! MIR core types and ID newtypes.

use std::fmt;

/// Machine-level type lattice. `Generic` is the dynamic lane (HudValue
/// ABI); everything else stays unboxed in native registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Unit,
    /// GC-managed reference kinds (§17).
    Ref(RefKind),
    /// Dynamic value — crosses the HudValue ABI (§10/§11).
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefKind {
    Object,
    String,
    Array,
    Function,
    Agent,
}

impl MirType {
    pub fn is_numeric(self) -> bool {
        matches!(
            self,
            MirType::I8
                | MirType::I16
                | MirType::I32
                | MirType::I64
                | MirType::U8
                | MirType::U16
                | MirType::U32
                | MirType::U64
                | MirType::F32
                | MirType::F64
        )
    }

    pub fn is_integer(self) -> bool {
        self.is_numeric() && !matches!(self, MirType::F32 | MirType::F64)
    }

    pub fn is_float(self) -> bool {
        matches!(self, MirType::F32 | MirType::F64)
    }
}

impl fmt::Display for MirType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MirType::I8 => "i8",
            MirType::I16 => "i16",
            MirType::I32 => "i32",
            MirType::I64 => "i64",
            MirType::U8 => "u8",
            MirType::U16 => "u16",
            MirType::U32 => "u32",
            MirType::U64 => "u64",
            MirType::F32 => "f32",
            MirType::F64 => "f64",
            MirType::Bool => "bool",
            MirType::Unit => "unit",
            MirType::Ref(k) => return write!(f, "ref<{k:?}>"),
            MirType::Generic => "generic",
        };
        f.write_str(s)
    }
}

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u32);

        impl $name {
            #[inline]
            pub fn as_usize(self) -> usize {
                self.0 as usize
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_newtype!(ValueId);
id_newtype!(BlockId);
id_newtype!(LocalId);
id_newtype!(FunctionId);
id_newtype!(SymbolId);
id_newtype!(FieldId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    Unreachable,
    DivisionByZero,
    IntegerOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeHelperId {
    /// hudhud_print runtime helper identity (§11) used by CallNative.
    Print,
    /// hudhud_print_str for string values.
    PrintStr,
    /// hudhud_throw-style error signal for the overflow lane.
    ThrowOverflow,
    /// hudhud_date_millis: Date.to_millis() — epoch ms (i64).
    DateMillis,
    /// hudhud_math_sin (f64 → f64).
    MathSin,
    /// hudhud_math_sqrt (f64 → f64).
    MathSqrt,
    /// hudhud_math_cos (f64 → f64).
    MathCos,
    MathFloor,
    MathAbs,
    /// (f64, f64) → f64
    MathPow,
    MathMin,
    MathMax,
    /// hudhud_globals() → i64 handle (modül-global obje deposu)
    GlobalsHandle,
    /// hudhud_global_get(slot: i64) -> i64
    GlobalGet,
    /// hudhud_global_set(slot: i64, val: i64)
    GlobalSet,
    /// hudhud_string_to_int: string handle → i64
    StringToInt,
    /// hudhud_string_split: (s: *const c_char, delim: *const c_char) -> *mut HudArray
    StringSplit,
    /// hudhud_string_index_of: (s: *const c_char, needle: *const c_char) -> i64
    StringIndexOf,
    /// hudhud_typeof: (val: i64) -> *const c_char
    TypeOf,
    /// hudhud_string_cmp: (a: *const c_char, b: *const c_char) -> i64
    StringCmp,
    /// hudhud_string_append: (a: *mut c_char, b: *const c_char) -> *mut c_char
    StringAppend,
    /// hudhud_throw: (val: i64)
    Throw,
    /// hudhud_has_exception: () -> i64 (0 or 1)
    HasException,
    /// hudhud_catch: () -> i64
    Catch,
    /// hudhud_array_filled: (length: i64, value: i64) -> *mut HudArray
    ArrayFilled,
    /// hudhud_array_fill: (arr: *mut HudArray, length: i64, value: i64)
    ArrayFill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    pub params: &'static [MirType],
    pub ret: MirType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_classes() {
        assert!(MirType::I64.is_integer());
        assert!(MirType::I64.is_numeric());
        assert!(!MirType::I64.is_float());
        assert!(MirType::F64.is_float());
        assert!(MirType::F64.is_numeric());
        assert!(!MirType::F64.is_integer());
        assert!(!MirType::Bool.is_numeric());
    }

    #[test]
    fn type_display() {
        assert_eq!(MirType::I64.to_string(), "i64");
        assert_eq!(MirType::F64.to_string(), "f64");
        assert_eq!(MirType::Ref(RefKind::String).to_string(), "ref<String>");
        assert_eq!(MirType::Generic.to_string(), "generic");
    }
}
