//! Tests for hudhudscript-codegen-cranelift — AOT object emission
//! (moved from crates/hudhudscript-codegen-cranelift/src/aot.rs).

use std::collections::HashMap;

use hudhudscript_codegen::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_codegen_cranelift::{compile_to_object, entry_symbol};
use hudhudscript_mir::{lower_module_typed, MirModule, MirType};
use hudhudscript_parser::parse;
use hudhudscript_types::lower_module_with_init;

fn mir_of(src: &str) -> MirModule {
    let ast = parse(src).expect("parse");
    let hir = lower_module_with_init(&ast).expect("hir");
    let mut ptys: HashMap<String, HashMap<String, MirType>> = HashMap::new();
    for (name, f) in &hir.functions {
        let m: HashMap<String, MirType> =
            f.params.iter().map(|p| (p.name.clone(), MirType::I64)).collect();
        ptys.insert(name.clone(), m);
    }
    lower_module_typed(&hir, &ptys).expect("mir")
}

#[test]
fn emits_valid_object_file() {
    let mir = mir_of("print(2 + 3)");
    let target = hudhudscript_target::TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let out = std::env::temp_dir().join(format!("hudhud_aot_test_{}.o", std::process::id()));
    let symbols = compile_to_object(&mir, &ctx, &out).expect("object");
    assert!(!symbols.is_empty());
    let bytes = std::fs::read(&out).expect("read back");
    // ELF magic (Linux host) — object üretilmiş ve yazılmış
    assert!(bytes.len() > 64);
    assert_eq!(&bytes[0..4], &[0x7f, b'E', b'L', b'F']);
    assert_eq!(entry_symbol(&mir).as_deref(), Some("hudhud__hudhud_init"));
    let _ = std::fs::remove_file(&out);
}
