# Görev: Derin Benchmark, Flamegraph, Callgraph ve Callgrind Analizi

Geliştirdiğim programlama dili için kapsamlı benchmark ve profiler analizi yapmak istiyorum.

Çalışmanın amacı:

- `fib(30)` recursive benchmark’ının neden çok yavaş olduğunu anlamak
- Interpreter/VM mimarisindeki darboğazları bulmak
- Stack-based VM ile performans farklarını açıklamak
- Flamegraph, callgraph ve callgrind verilerinden gerçek bottleneck’leri çıkarmak
- Gereksiz çağrıları, allocation’ları ve runtime overhead’lerini tespit etmek
- Stack based vm bile fib(30) da daha hızlıydı, neleri yanlış yapıyoruz?

---

# KRİTİK KURALLAR

## Kesin Yasaklar

- Kaynak kodlarını ASLA değiştirme.
- Hiçbir `.rs`, `.c`, `.cpp`, `.h`, `.hpp`, `.py`, `.js`, `.ts`, `.toml`, `.json`, `.yaml` vb. dosyaya dokunma.
- Benchmark kodlarını değiştirme.
- Build scriptlerini değiştirme.
- Cargo/CMake/npm/pip yapılarını değiştirme.
- Sadece aşağıdaki Markdown dosyasını oluştur veya güncelle:

`/home/onur/HudHudMind/hudhud-script-interactive/KIMI_BENCHMARK.md`

Kod değişikliği önermende sorun yok, ancak bunlar SADECE rapor içinde öneri olarak yazılmalı.

---

# İncelenecek Dosyalar

Önce şu dosyayı oku:

`/home/onur/HudHudMind/hudhud-script-interactive/profiles/REPORT.md`

Sonra şu klasörde bulunan TÜM profiler çıktılarını analiz et:

`/home/onur/HudHudMind/hudhud-script-interactive/profiles/`

Özellikle şunları incele:

- flamegraph dosyaları
- callgraph dosyaları
- callgrind dosyaları
- benchmark sonuçları
- perf çıktıları
- profiler logları
- CPU sampling çıktıları
- instruction/cycle verileri
- recursive benchmark çıktıları

Eksik veri varsa bunu açıkça belirt.

---

# Ana Araştırma Konusu

`fib(30)` recursive benchmark çok yavaş çıktı.

Bazı durumlarda stack-based VM’den bile daha kötü performans gösteriyor.

Bunun nedenlerini ayrıntılı şekilde araştır.

Özellikle şu sorulara cevap ver:

- Recursive call overhead nereden geliyor?
- Function call maliyeti neden yüksek?
- Stack frame oluşturma/yıkma pahalı mı?
- Interpreter dispatch overhead var mı?
- AST walking maliyeti yüksek mi?
- Dynamic dispatch / virtual dispatch etkisi var mı?
- Boxed value / enum matching overhead var mı?
- Heap allocation yoğunluğu oluşuyor mu?
- GC / allocator / memory management maliyetleri var mı?
- String/hashmap lookup darboğazı oluşuyor mu?
- Environment/symbol lookup pahalı mı?
- Parser/evaluator benchmark sırasında tekrar mı çalışıyor?
- Tail-call optimization eksikliği etkiliyor mu?
- Inline eksikliği etkiliyor mu?
- Cache locality problemi var mı?
- Branch misprediction ihtimali var mı?
- Çok küçük helper fonksiyonlar aşırı mı çağrılıyor?
- Clone/copy/allocation fazlalığı var mı?
- Call count aşırı yükselen fonksiyonlar var mı?
- Self cost yüksek fonksiyonlar hangileri?
- Children/total cost yüksek çağrı zincirleri hangileri?
- Instruction count patlaması yaşanan fonksiyonlar hangileri?
- Stack-based VM neden daha hızlı kalıyor?
- Bytecode VM ile AST interpreter arasında nerede fark oluşuyor?
- Native execution ile fark tam olarak hangi katmanda oluşuyor?

---

# Flamegraph Analizi

Flamegraph’ları ayrıntılı analiz et.

Açıkla:

- Uzun kuleler neyi gösteriyor?
- Geniş yatay alanlar neyi gösteriyor?
- En geniş frame’ler hangileri?
- Hangi fonksiyonlar CPU zamanının çoğunu tüketiyor?
- Hangi çağrı zincirleri recursive patlama oluşturuyor?
- Hot path nerede oluşuyor?
- Beklenmedik geniş fonksiyonlar var mı?
- Küçük ama çok sık çağrılan helper fonksiyonlar var mı?
- Dispatch loop görünür mü?
- Allocation pattern’leri flamegraph’ta görülebiliyor mu?

---

# Callgraph Analizi

Callgraph’ları ayrıntılı analiz et.

Açıkla:

- En yoğun çağrı zincirleri hangileri?
- Recursive depth nasıl görünüyor?
- Gereksiz tekrar eden çağrılar var mı?
- Çok yüksek call count’a sahip ama az iş yapan fonksiyonlar var mı?
- Hangi parent-child ilişkileri darboğaz oluşturuyor?
- Fonksiyon katmanları gereğinden fazla mı derin?
- Dispatch katmanı recursive benchmark’ta aşırı mı çalışıyor?

---

# Callgrind Analizi

Callgrind verilerini ayrıntılı yorumla.

Özellikle:

- instruction count
- inclusive cost
- exclusive/self cost
- call count
- cycles
- branching davranışı
- cache etkileri

hakkında detaylı açıklama yap.

Şunları belirt:

- En pahalı fonksiyonlar hangileri?
- En fazla instruction tüketen path hangisi?
- Recursive explosion nerede oluşuyor?
- Instruction count neden bu kadar yükseliyor?
- VM dispatch maliyeti toplam sürenin ne kadarını oluşturuyor?
- Environment lookup toplam maliyeti ne kadar?
- Function enter/exit overhead ne kadar?
- Allocation maliyeti görünüyor mu?

---

# Özellikle Aranacak Problemler

Şunları özellikle araştır:

- bottleneck fonksiyonlar
- recursive explosion
- gereksiz stack frame oluşturma
- aşırı function call overhead
- interpreter dispatch overhead
- AST traversal maliyeti
- dynamic dispatch maliyeti
- boxed value overhead
- clone/copy overhead
- heap allocation yoğunluğu
- hashmap lookup maliyeti
- symbol table lookup maliyeti
- parser/evaluator tekrar çalışması
- GC/allocator baskısı
- branch misprediction ihtimali
- kötü cache locality
- inline edilemeyen helper fonksiyonlar
- aşırı abstraction katmanları
- gereksiz wrapper fonksiyonlar
- yüksek self-time fonksiyonlar
- yüksek children-cost çağrı zincirleri
- çok sık çağrılan tiny helper’lar
- instruction explosion
- dispatch loop darboğazı

---

# Her Bulunan Problem İçin

Her problem için ayrı başlık aç ve şunları yaz:

1. Problem nedir?
2. Nerede görülüyor?
3. Hangi profiler çıktısı bunu gösteriyor?
4. İlgili fonksiyonlar hangileri?
5. Call chain nasıl ilerliyor?
6. Self cost ne söylüyor?
7. Children/inclusive cost ne söylüyor?
8. Instruction count ne durumda?
9. `fib(30)` benchmark’ını nasıl etkiliyor?
10. Stack-based VM ile farkı nasıl açıklıyor?
11. Bu kesin bulgu mu yoksa muhtemel yorum mu?
12. Ne yapılmalı?
13. Öncelik seviyesi nedir?

---

# İstenen Rapor Yapısı

`KIMI_BENCHMARK.md` içinde şu bölümler olsun:

1. Genel Özet
2. İncelenen Dosyalar
3. Benchmark Genel Durumu
4. Flamegraph Analizi
5. Callgraph Analizi
6. Callgrind Analizi
7. Hot Path Analizi
8. Recursive Call Analizi
9. Function Call Overhead Analizi
10. Interpreter Dispatch Analizi
11. Allocation / Memory Analizi
12. Environment / Symbol Lookup Analizi
13. Stack-based VM Karşılaştırması
14. En Büyük Bottleneck’ler
15. Muhtemel Mimari Problemler
16. Ölçüm Güvenilirliği ve Eksik Veriler
17. Kod Değiştirmeden Yapılabilecek Yorumlar
18. Optimizasyon Önerileri
19. Önceliklendirilmiş Aksiyon Listesi
20. Sonuç

---

# Öneriler Bölümü

Raporun sonunda öncelik sırasıyla öneriler ver:

- Ölçüm doğrulama
- Benchmark setup maliyetini ayırma
- Parser maliyetini benchmark dışına alma
- Recursive call path sadeleştirme
- Function enter/exit maliyetini azaltma
- Dispatch maliyetini azaltma
- Environment lookup cache
- Symbol lookup cache
- Inline edilebilecek helper fonksiyonlar
- Allocation azaltma
- Clone/copy azaltma
- AST interpreter yerine bytecode path değerlendirme
- Stack frame optimizasyonu
- Tail-call optimization araştırması
- Arena allocator/pool allocator ihtimali
- Native/JIT/bytecode karşılaştırması
- Aynı benchmark’ı:
  - native
  - AST interpreter
  - bytecode VM
  - stack VM
  - threaded VM
  ile kıyaslama önerileri

---

# Çıktı Dili

Rapor Türkçe yazılsın.

Ancak:

- fonksiyon adları
- profiler terimleri
- metric isimleri
- dosya isimleri
- call stack’ler

orijinal halleriyle bırakılmalı.

Teknik detayları azaltma.
Derin ve profesyonel analiz yap.