//! Tests for hudhudscript-mir `types` — MirType classes and Display.

use hudhudscript_mir::types::{MirType, RefKind};

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
