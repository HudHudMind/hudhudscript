# Yapılan Değişiklikler — `examples` → `samples` Birleştirmesi

## Amaç
`examples/` ve `samples/` klasörlerini tek bir `samples/` klasörü altında birleştirmek, `_archive` klasörünü silmek ve tüm sample'ların çalışır durumda olduğundan emin olmak.

## Yapılan İşlemler

### 1. Klasör Birleştirme ve Temizlik
- `examples/` içindeki tüm alt klasörler `samples/` altına taşındı:
  - `examples/01-basics` → `samples/01-basics`
  - `examples/02-multilingual` → `samples/02-multilingual`
  - `examples/02-sop` → `samples/02-sop`
  - `examples/03-council` → `samples/03-council`
  - `examples/04-agents` → `samples/04-agents`
  - `examples/05-advanced` → `samples/05-advanced`
  - `examples/05-algorithms` → `samples/05-algorithms`
  - `examples/06-web` → `samples/06-web`
  - `examples/07-ai-website` → `samples/07-ai-website`
  - `examples/08-video-site` → `samples/08-video-site`
  - `examples/09-loop-engineering` → `samples/09-loop-engineering`
  - `examples/config` → `samples/config`
  - `examples/_wip` → `samples/_wip`
- `examples/` klasörü tamamen kaldırıldı.
- `examples/_archive` klasörü tamamen silindi.

### 2. Çalışmayan Sample'ların Ayrılması
Parser tarafından henüz desteklenmeyen `enum_demo.hud`, çalışan sample'lar arasından çıkarıldı:
- `samples/02-sop/enum_demo.hud` → `samples/_wip/enum_demo.hud`

### 3. İç Path Referanslarının Güncellenmesi
`examples/...` path'leri içeren örnek script'ler `samples/...` olarak güncellendi:
- `samples/06-web/app.hud`
- `samples/07-ai-website/app.hud`
- `samples/08-video-site/app.hud`
- `samples/02-sop/hudfs.hudhud`
- `samples/02-sop/realm_of_hud.hudhud`

### 4. Test ve CLI Referanslarının Güncellenmesi
`examples/...` yazan test dosyaları ve CLI örnekleri `samples/...` olarak güncellendi:
- `crates/hudhudscript-cli/src/common/locale.rs` → `Examples:` yerine `Samples:` yazdırıyor.
- `crates/hudhudscript-compiler/examples/compile_test.rs` → örnek input/output path'i `samples/hello.hud` / `samples/hello.hudb` oldu.
- `hudhudscript-tests/Cargo.toml` → test target adı ve path'i `examples` → `samples` oldu.
- `hudhudscript-tests/tests/examples/mod.rs` → `hudhudscript-tests/tests/samples/mod.rs` olarak taşındı ve `examples` referansları `samples` yapıldı.
- `hudhudscript-tests/tests/integration/examples_parse_test.rs` → `examples` → `samples` güncellemeleri.
- `hudhudscript-tests/tests/integration/examples_validation_test.rs` → `examples` → `samples` güncellemeleri; artık var olmayan `real_world_agents/*` dosyaları `_wip/real_world_agents/*` olarak işaret ediliyor.
- `hudhudscript-tests/tests/integration/web_framework_socket_test.rs` → template path'i `examples/06-web` → `samples/06-web`.
- `hudhudscript-tests/tests/loop_engineering/samples.rs` → sample path resolver workspace-relative `samples/09-loop-engineering` yapıldı.
- `hudhudscript-tests/tests/loop_engineering/semantic.rs` → `include_str!` path'leri `samples/09-loop-engineering` olarak güncellendi.
- `hudhudscript-tests/tests/loop_engineering/storage_contract.rs` → storage guard path `samples/09-loop-engineering` oldu.
- `hudhudscript-tests/tests/sop/sop_npc_rpg_test.rs` → `include_str!` path'i `samples/05-advanced/sop_npc_rpg.hud` oldu.

## Doğrulama

### Syntax Check
`hudhud check` ile `samples/` içindeki 113 dosya (_wip hariç) başarıyla doğrulandı:

```bash
PASS: 113, FAIL: 0
```

### Çalışma Testleri
Aşağıdaki sample'lar `hudhud run` ile başarıyla çalıştırıldı:
- `samples/hello.hud`
- `samples/hello_world.hhs`
- `samples/04_loops.hhs`
- `samples/09_prime.hhs`
- `samples/10_gcd.hhs`
- `samples/conditionals.hud`
- `samples/fibonacci.hud`
- `samples/functions.hud`
- `samples/loops.hud`
- `samples/oop_class.hud`
- `samples/09-loop-engineering/01_simple_done.hud`
- `samples/sop_npc_rpg.hud`

### Cargo Test
Aşağıdaki test target'leri başarıyla geçti:
- `cargo test -p hudhudscript-tests --test samples`
- `cargo test -p hudhudscript-tests --test loop_engineering` (86 test)
- `cargo test -p hudhudscript-tests --test sop` (11 test)
- `cargo run -p hudhudscript-compiler --example compile_test`

### Derleme Uyarıları
`cargo test` sırasında çok sayıda `unused import` uyarısı var, ancak bunlar önceden var olan teknik borçtur ve bu değişiklikle doğrudan ilişkili değildir.

## Etkilenen Dosyalar (Özet)
- Silinen: tüm `examples/` ağacı ve `examples/_archive`
- Taşınan/eklenen: tüm `samples/` ağacı
- Güncellenen Rust dosyaları: yukarıda listelenen test ve CLI kaynak dosyaları
- Güncellenen örnek scriptler: path referansları içeren `.hud` / `.hudhud` dosyaları
