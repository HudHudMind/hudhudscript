/* HudHud sıcak-yol bölümü (v0.9.18 — kod yerleşimi piyangosu kökten kesilir)
 *
 * native-abi'nin en çok döngü harcayan ~16 fonksiyonu kendi bölmelerinde,
 * 64 bayt hizalı ve SABİT sırada yaşar. Genel .text'e eklenen kod (yeni
 * semboller, bağımlılık değişimleri) bu bölmelerin yerini ASLA kaydıramaz.
 * Sıra = perf döngü profillerine göre (v0.9.17 ölçümleri).
 */
SECTIONS
{
.hudhud_hot ALIGN(64) :
{
  KEEP(*(.hudhud_hot.string_concat))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.string_append))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.num_add))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.num_sub))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.num_mul))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.bigint_add_trusted))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.num_cmp))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.array_get))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.array_set))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.array_push))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.string_char_at))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.string_eq))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.string_len))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.array_len))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.num_div))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.num_rem))
  . = ALIGN(64);
  KEEP(*(.hudhud_hot.date_millis))
}
}
INSERT AFTER .text;
