# hudhudscript-hudunit

HudHudScript dili için unit test framework'ü — pyunit/phpunit/pytest tarzı.
`hudunit` binary'si ve `hudhudscript_hudunit` kütüphanesini sağlar.

Tam referans belgesi (tüm bayraklar, assertion imzaları, coverage modeli,
mimari notlar ve sınırlamalar): [`docs/hudunit.md`](../../docs/hudunit.md).

## Hızlı başlangıç

```
cargo build -p hudhudscript-hudunit --bin hudunit
hudunit init        # yeni projede: src/ + tests/ + hudunit.toml iskeleti
cd projem && hudunit
```

Testler `test_` önekli fonksiyonlardır; `assert*` prelude fonksiyonlarıyla
doğrulama yapar:

```
// tests/math/test_arithmetic.hud
import { topla } from "../../src/math.hud";

// @group hizli
fn test_topla_basit() {
    assert_eq(topla(2, 3), 5);
}
```

- Uzantılar: `.hhs`, `.hud`, `.hudhud`
- Dizin yapısı = grup (`tests/math/` → `math` grubu; `hudunit tests/math`)
- `// @group adı` yorumlarıyla ek gruplar (`--group`, `--exclude-group`)
- `setup()` / `teardown()` her testten önce/sonra çalışır (taze VM, izolasyonlu)
- `ignore_test_*` atlanır, `_test_*` de atlanır
- `--coverage` satır + fonksiyon coverage'ı (test dosyaları VE import edilen
  `src/` modülleri); `--html rapor.html` self-contained HTML raporu (satır
  ısı haritası ile), `--json rapor.json` CI çıktısı, `--junit junit.xml`
  JUnit XML (GitHub/GitLab/Jenkins)
- Test içindeki `print` çıktısı yakalanır; failing testlerde (ve `-v` ile
  hepsinde) raporlanır
- `timeout_ms` aşan testler `timeout` ile başarısız işaretlenir; sonsuz
  döngülere karşı `fuel` limiti de aktiftir
- `--watch` ile dosya değişince otomatik yeniden koşum (testler + `src/` +
  `hudunit.toml` izlenir, Ctrl-C ile çıkış)
- `--fail-fast` ilk başarısızda koşumu durdurur; `-v` en yavaş 5 testi listeler
- Renkler `--no-color` veya `NO_COLOR` ortam değişkeni ile kapatılır
- Çıkış kodu: tümü geçtiyse 0, aksi halde 1

## Assertions

`assert`, `assert_true`, `assert_false`, `assert_eq`, `assert_ne`,
`assert_null`, `assert_contains`, `assert_length`, `assert_approx`,
`assert_throws(fn)` (fonksiyon bir exception atmalı). Hepsi sondan opsiyonel
bir açıklama parametresi alır: `assert_eq(topla(1, 1), 3, "toplama kontrolu")`
→ `"toplama kontrolu: assert_eq failed: expected 3, got 2"`.

## Tasarım notu

Framework lexer/parser/derleyici/VM'e dokunmaz: assertion'lar saf
HudHudScript prelude'unda `throw` ile, coverage ise derleme öncesi AST
instrumentasyonuyla (`__hudunit_mark` çağrıları) toplanır. Normal `hudhud`
çalışma zamanı hiçbir şekilde etkilenmez.

Coverage hem test dosyalarını hem de `import` ile yüklenen `src/` modüllerini
ölçer: modül çözümleyicisi, import edilen kaynağın fonksiyon gövdelerini
instrumente edip biçimlendirerek VM'in modül yükleyicisine verir (işaretleme
id'leri koşum boyunca cache'lenir). Modül fonksiyon gövdeleri test sırasında
host VM'de koştuğu için işaretler ana akışta toplanır; modülün top-level
ifadeleri (sub-VM'de koşar) instrumente edilmez.
