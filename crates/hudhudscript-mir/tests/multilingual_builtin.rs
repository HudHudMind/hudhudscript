//! M1 (çok dilli yerleşikler, v0.9.34) regresyon testleri.
//!
//! Yerelleştirilmiş print adları (yazdır, اطبع, 打印, …) MIR'e inerken
//! kanonik "print" helper'ına çözümlenmelidir; modül fonksiyonu aynı adı
//! taşıyorsa KULLANICI FONKSİYONU önceliklidir.

use hudhudscript_mir::{lower_module_typed, specialize_module, MirModule, MirInst};
use hudhudscript_parser::parse;
use hudhudscript_types::lower_module_with_init;

fn mir_of(src: &str) -> MirModule {
    let ast = parse(src).expect("parse");
    let mut hir = lower_module_with_init(&ast).expect("hir");
    specialize_module(&mut hir);
    let ptys = hudhudscript_mir::param_infer::infer_module_param_types(&hir);
    lower_module_typed(&hir, &ptys).expect("mir")
}

fn init_insts(mir: &MirModule) -> Vec<&MirInst> {
    let f = mir
        .functions
        .iter()
        .find(|f| f.name.as_ref() == "_hudhud_init")
        .expect("_hudhud_init bulunmalı");
    f.blocks.iter().flat_map(|b| b.insts.iter()).collect()
}

#[test]
fn localized_print_lowers_to_print_helper() {
    for name in ["yazdır", "yaz", "اطبع", "書く", "출력", "drucken", "打印", "in_ra"] {
        let mir = mir_of(&format!("{name}(42)"));
        let insts = init_insts(&mir);
        assert!(
            insts.iter().any(|i| matches!(
                i,
                MirInst::CallNative { helper: hudhudscript_mir::RuntimeHelperId::Print, .. }
            )),
            "{name}: kanonik print helper'ına inmedi"
        );
    }
}

#[test]
fn unknown_still_rejected() {
    // Bilinmeyen ad yerleşik değildir — lowering hatası korunmalı
    let ast = parse("fogsgon(1)").expect("parse");
    let hir = lower_module_with_init(&ast).expect("hir");
    let mut hir2 = hir;
    specialize_module(&mut hir2);
    let ptys = hudhudscript_mir::param_infer::infer_module_param_types(&hir2);
    assert!(lower_module_typed(&hir2, &ptys).is_err());
}

#[test]
fn user_function_priority_over_alias() {
    // Kullanıcı 'yaz' adında fonksiyon tanımlarsa takma ad DEĞIL o çağrılır
    let mir = mir_of("fn yaz(n) { return n * 2 }\nlet r = yaz(21)\nprint(r)");
    let insts = init_insts(&mir);
    assert!(
        insts.iter().any(|i| matches!(i, MirInst::CallStatic { .. })),
        "kullanıcı fonksiyonu CallStatic ile çağrılmalı"
    );
}
