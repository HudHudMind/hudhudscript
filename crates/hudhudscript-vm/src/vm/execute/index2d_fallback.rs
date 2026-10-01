//! `Index2D` hızlı yolunun (taban dizi + satır dizi + sayısal idx2 — matris
//! `a[i][k]` kalbi) uymadığı satırların çözümleyicisi — eritilmemiş
//! `(IndexArray, Index)` zincirinin satır-sementiğiyle BİREBİR parite.
//!
//! B5 (filo raporu): derleyici tabanı dizi-kanıtlı gördüğü için `lst[i][k]`
//! zincirini Index2D'ye eritiyor (fuse_super_index.rs); taban gerçekten
//! dizidir ama SATIR harita olabilir (elemanlar `{"ad": ...}` gibi nesne).
//! Index2D hızlı yolu satırın da dizi olmasını ve idx2'nin sayısal olmasını
//! şart koşuyordu — `_scratch/forensic/b5_row.hud` bu yüzden
//! `[E0039] Index2D: idx2 not numeric` ile çöküyordu.
//!
//! Eritilmemiş zincirde ikinci erişim genel `Index`'tir: satır harita ise
//! anahtarla, instance ise alan adıyla, string ise karakterle çözülür. Bu
//! dosya o semantiği hızlı yola DOKUNMAZDAN mevcut hata dallarının hedefine
//! taşır (H.6.3: nadir slow-path — `#[cold]` + `#[inline(never)]` ile
//! uzaklaştırılır; cmp_fallback.rs deseni). Matris hızlı yolu etkilenmez:
//! satır gerçekten dizi ve idx2 sayısal ise yürütme hiç buraya uğramaz.

use super::*;
use crate::vm::index_helpers::{index_i64_to_usize, numeric_index_i64};

impl VM {
    /// Index2D fast-path uyumsuzluğunun çözümleyicisi (B5).
    ///
    /// Çağrı koşulları (indexing.rs): taban dizi kanıtlanmış, idx1 sayısal,
    /// satır alındı; fakat (a) idx2 sayısal değil VEYA (b) satır dizi değil.
    ///
    /// Eritilmemiş zincir paritesi: satır genel `Index` hedefi gibi çözülür —
    /// harita (SOP subject-instance canlı-durum okuması dahil), instance
    /// alanları, string karakter erişimi. Hiçbiri uymazsa orijinal Index2D
    /// hatası üretilir.
    #[cold]
    #[inline(never)]
    pub(crate) fn index2d_row_fallback(
        &mut self,
        dst: u8,
        row: Value16,
        idx2: Value16,
        bytecode: &Bytecode,
        ip: usize,
    ) -> CompileResult<StepAction> {
        let result = if let Some(map) = row.as_object() {
            // indexing.rs `Index` harita koluyla birebir (SOP dahil).
            let key = idx2.as_string().unwrap_or_default();
            // SOP: subject instance state read via index — read from live instance
            if map.get("__type").and_then(|v| v.as_string()).as_deref()
                == Some("subject_instance")
            {
                if let Some(id) = map.get("__instance_id").and_then(|v| v.as_string()) {
                    if let Some(inst) = self.subject_instances.get(&id) {
                        if let Some(val) = inst.state.get(&key) {
                            *val
                        } else {
                            map.get(&key).copied().unwrap_or(Value16::null())
                        }
                    } else {
                        // SOP0005: despawned subject accessed
                        return Err(compile_codes::runtime_error(format!(
                            "Cannot access index '{}' on despawned subject '{}'",
                            key, id
                        )));
                    }
                } else {
                    map.get(&key).copied().unwrap_or(Value16::null())
                }
            } else {
                map.get(&key).copied().unwrap_or(Value16::null())
            }
        } else if let Some(inst) = row.as_instance_data() {
            // indexing.rs `Index` instance koluyla birebir.
            let key = idx2.as_string().unwrap_or_default();
            inst.fields.get(&key).copied().unwrap_or(Value16::null())
        } else if let Some(s) = row.as_str() {
            // indexing.rs `Index` string koluyla birebir (ASCII fast path dahil).
            #[cfg(feature = "telemetry")]
            {
                self.telemetry.string_index_count += 1;
            }
            let i = numeric_index_i64(idx2)
                .and_then(index_i64_to_usize)
                .ok_or_else(|| {
                    Self::runtime_error_with_pos("Index2D: idx2 not numeric", bytecode, ip)
                })?;
            // ASCII fast path: byte index avoids O(n) char iteration
            let bytes = s.as_bytes();
            if i < bytes.len() && bytes[i].is_ascii() {
                Value16::string_ascii(bytes[i])
            } else {
                s.chars()
                    .nth(i)
                    .map(|c| {
                        let cs = c.to_string();
                        #[cfg(feature = "telemetry")]
                        {
                            self.telemetry.string_index_clone_count += 1;
                            self.telemetry.string_index_clone_bytes += cs.len() as u64;
                        }
                        Value16::string(cs)
                    })
                    .ok_or_else(|| {
                        Self::runtime_error_with_pos(
                            format!("String index out of bounds: {}", i),
                            bytecode,
                            ip,
                        )
                    })?
            }
        } else if numeric_index_i64(idx2).is_some() {
            // Orijinal hata: idx2 sayısaldı ama satır dizi değil ve genel
            // Index semantiğiyle de çözülemiyor.
            return Err(Self::runtime_error_with_pos(
                "Index2D: row not array",
                bytecode,
                ip,
            ));
        } else {
            // Orijinal hata: idx2 sayısal değil ve satır genel Index
            // semantiğiyle de çözülemiyor.
            return Err(Self::runtime_error_with_pos(
                "Index2D: idx2 not numeric",
                bytecode,
                ip,
            ));
        };
        self.registers[dst as usize] = result;
        Ok(StepAction::Advance)
    }
}
