//! Birleşik cmp+branch çekirdeğinin (G4 `cmp_rr_generic`) uyumsuz-tip
//! çözümleyicisi — birleştirilmemiş `IntCmp`'in F3 koluyla BİREBİR parite.
//!
//! v0.9.54 3. regresyonunun kökü: `cmp_rr_generic` merdiveninin sonunda
//! uyumsuz tipler (`Int` vs `InlineString` gibi) hard runtime error
//! üretiyordu; oysa aynı karşılaştırmanın birleştirilmemiş hali
//! (`IntCmp`, int_cmp.rs F3) policy ile false/true çözüyor, `values_equal`
//! ve döngü-füzyonlu `IntLeRR/IntLtRRJumpIfFalse` de hata üretmiyor.
//! v0.9.53 hatayı yalnızca tesadüfen maskeliyordu: `try` içinden `return`
//! eden callee'nin TF'i sahipliksiz yığına sızıyor, hata bu stale TF'yi
//! tüketip `catch_ip`'e — tesadüfen ana akışta döngü kuyruğuna — atlıyordu.
//! BULGU6'nın TF sahiplik budaması sızıntıyı kapatınca gerçek semantik
//! açığı yüzeye çıktı. Bu dosya açığı kapatır: Eq/Ne policy'e, sıralama
//! op'ları false'a düşer — tıpkı birleştirilmemiş yol gibi.

use hudhudscript_bytecode::Value16;

impl crate::vm::VM {
    /// `cmp_rr_generic`'in `Err(CMP_RR_INCOMPATIBLE)` sinyalini policy ile
    /// çözer. Policy yalnızca BURADA okunur — sıcak çekirdek imzası
    /// v0.9.53'teki gibi 3 argüman kalır (ABI: 6 register, stack yok).
    #[cold]
    #[inline(never)]
    pub(crate) fn cmp_rr_resolve_incompatible(&self, v1: Value16, v2: Value16, op: u8) -> bool {
        cmp_rr_incompatible(v1, v2, op, self.object_equality)
    }
}

/// Uyumsuz-tip çifti için karşılaştırma sonucu (IntCmp F3 paritesi).
///
/// H.6.3: nadir slow-path — çekirdek fonksiyonun içine gömülmez;
/// `#[cold]` + `#[inline(never)]` ile uzaklaştırılır.
///
/// op: 0 `<` 1 `<=` 2 `>` 3 `>=` 4 `==` 5 `!=`
#[cold]
#[inline(never)]
pub(crate) fn cmp_rr_incompatible(
    v1: Value16,
    v2: Value16,
    op: u8,
    policy: crate::vm::config_types::ObjectEquality,
) -> bool {
    let eq = match policy {
        crate::vm::config_types::ObjectEquality::Identity => {
            // IntCmp F3/Identity: payload (pointer kimliği) karşılaştırması.
            let (ptr1, ptr2) = (v1.split_tag().1, v2.split_tag().1);
            ptr1 == ptr2
        }
        crate::vm::config_types::ObjectEquality::Never => false,
        crate::vm::config_types::ObjectEquality::Deep => v1.values_equal(&v2),
    };
    match op {
        4 => eq,
        5 => !eq,
        // Sıralama op'ları uyumsuz tiplerde tanımsız → false
        // (IntLeRR/IntLtRRJumpIfFalse'ın mevcut semantiğiyle uyumlu).
        _ => false,
    }
}
