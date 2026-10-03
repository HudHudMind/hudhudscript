//! Tests for hudhudscript-codegen-gccjit — offline object emission
//! (moved from crates/hudhudscript-codegen-gccjit/src/lib.rs).
//!
//! Needs libgccjit on the host (`apt install libgccjit-12-dev` or
//! `HUDHUD_GCCJIT_LIBDIR=<dir>`); gated to Unix like the original
//! `#[cfg(all(test, unix))]` inline module.

#![cfg(unix)]

use std::collections::HashMap;

use hudhudscript_codegen::{CodegenContext, OptGoal, OptLevel};
use hudhudscript_codegen_gccjit::compile_to_object;
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
fn emits_valid_elf_object() {
    let mir = mir_of("print(2 + 3)");
    let target = hudhudscript_target::TargetSpec::host();
    let ctx = CodegenContext {
        target: &target,
        opt: OptLevel::O2,
        opt_goal: OptGoal::Speed,
        debug_info: false,
        abi_version: 1,
    };
    let out = std::env::temp_dir().join(format!("hudhud_gccjit_{}.o", std::process::id()));
    let symbols = compile_to_object(&mir, &ctx, &out).expect("gccjit object");
    assert!(!symbols.is_empty());
    let bytes = std::fs::read(&out).expect("read back");
    assert!(bytes.len() > 64);
    assert_eq!(&bytes[0..4], &[0x7f, b'E', b'L', b'F']);
    let _ = std::fs::remove_file(&out);
}
