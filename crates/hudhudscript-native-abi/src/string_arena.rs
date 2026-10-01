//! String interning ve arena katmanı (v0.9.41) — O(n²) bellek patlamasının
//! kalıcı çözümü. K-Nucleotide deseni: döngüde 300.000 kez substring(j, j+k)
//! çağrılır; 4^k_unique farklı sonuç vardır ama her çağrı yeni alloc yapar.
//! Leak-on-exit sözleşmesiyle bu alloc'lar birikir → 8 GB.
//!
//! Çözüm iki katmanlı:
//! 1. SHORT_STRING_INTERN: ≤ 32 baytlık stringler için thread-local cache.
//!    Aynı içerik → aynı pointer (sıfır yeni alloc). K-Nucleotide'de
//!    300.000 substring → 84 cache girişi.
//! 2. STRING_ARENA: bump allocator (2 MiB slab'lar). Tüm string alloc'ları
//!    arena'dan gelir; slab dolunca yenisi açılır (GC gelene kadar).
//!    BigInt arena ile aynı desen.
//!
//! ASCII_CHARS zaten tek-karakter için bunu yapıyor — bu genelleme.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_char;

const INTERN_MAX: usize = 32;
#[doc(hidden)]
pub const ARENA_SLAB: usize = 2 * 1024 * 1024; // 2 MiB

thread_local! {
    /// ≤ 32 baytlık stringler için içerik→pointer intern tablosu.
    /// Key: [u8; 32] (stack'te — heap alloc YOK, v0.9.42 hız fix).
    static SHORT_STRING_INTERN: RefCell<HashMap<[u8; 32], usize>> =
        RefCell::new(HashMap::new());

    /// String bump arena — slab'lar leak-on-exit.
    static ARENA: RefCell<Vec<Slab>> = RefCell::new(Vec::new());

    /// Arena bellek zarfı (min, max): tek seferlik O(1) dış sınır kontrolü.
    static ARENA_BOUNDS: std::cell::Cell<(usize, usize)> =
        const { std::cell::Cell::new((usize::MAX, 0)) };

    /// Arena istatistikleri (HUDHUD_STRING_TRACE için).
    static ARENA_STATS: RefCell<(usize, usize, usize)> =
        const { RefCell::new((0, 0, 0)) }; // (alloc_count, intern_hits, arena_bytes)
}

struct Slab {
    data: Vec<u8>,
    offset: usize,
}

impl Slab {
    fn new() -> Self {
        Slab {
            data: vec![0u8; ARENA_SLAB],
            offset: 0,
        }
    }

    fn alloc(&mut self, len: usize) -> Option<*mut c_char> {
        // 16-byte hizalama (H.6 cache-line disiplini)
        let aligned = (self.offset + 15) & !15;
        if aligned + len + 1 <= self.data.len() {
            let ptr = unsafe { self.data.as_mut_ptr().add(aligned) };
            self.offset = aligned + len + 1;
            Some(ptr as *mut c_char)
        } else {
            None
        }
    }
}

/// Arena'dan string alloc (NUL sonlandırmalı). Büyük stringler (> slab/2)
/// doğrudan sistem alloc'una gider (arena'da yer israfı önlemek için).
#[doc(hidden)]
pub unsafe fn arena_alloc(len: usize) -> *mut c_char {
    ARENA_STATS.with(|s| s.borrow_mut().0 += 1);

    if len + 1 > ARENA_SLAB / 2 {
        // Büyük string: doğrudan Rust alloc (Vec::leak deseni)
        let mut v = Vec::with_capacity(len + 1);
        v.resize(len + 1, 0);
        let p = v.leak().as_mut_ptr() as *mut c_char;
        ARENA_STATS.with(|s| s.borrow_mut().2 += len + 1);
        return p;
    }

    ARENA.with(|a| {
        let mut slabs = a.borrow_mut();
        // Mevcut slab'ta yer var mı?
        if let Some(slab) = slabs.last_mut() {
            if let Some(p) = slab.alloc(len) {
                ARENA_STATS.with(|s| s.borrow_mut().2 += len + 1 + 15);
                return p;
            }
        }
        // Yeni slab aç
        let mut slab = Slab::new();
        let start = slab.data.as_ptr() as usize;
        let end = start + ARENA_SLAB;
        ARENA_BOUNDS.with(|b| {
            let (min, max) = b.get();
            b.set((min.min(start), max.max(end)));
        });
        let p = slab.alloc(len).expect("yeni slab yetersiz");
        ARENA_STATS.with(|s| s.borrow_mut().2 += ARENA_SLAB);
        slabs.push(slab);
        p
    })
}

/// Kısa string intern: aynı içerik → aynı pointer (sıfır alloc).
/// `data` NUL sonlandırmalı raw baytlardır. Sonuç her zaman geçerli bir
/// C string pointer'dır (arena'dan veya cache'den).
#[doc(hidden)]
pub unsafe fn intern_or_alloc(data: &[u8], len: usize) -> *mut c_char {
    if len <= INTERN_MAX {
        // Stack'te sabit boyutlu key — heap alloc YOK (v0.9.42)
        let mut key = [0u8; INTERN_MAX];
        std::ptr::copy_nonoverlapping(data.as_ptr(), key.as_mut_ptr(), len);
        // İlk bakış: cache hit (borrow — lock yok)
        let cached = SHORT_STRING_INTERN.with(|c| c.borrow().get(&key).copied());
        if let Some(p) = cached {
            ARENA_STATS.with(|s| s.borrow_mut().1 += 1);
            return p as *mut c_char;
        }
        // Cache'te yok: arena'dan alloc + cache'e kaydet
        let p = arena_alloc(len);
        if !p.is_null() {
            std::ptr::copy_nonoverlapping(data.as_ptr(), p as *mut u8, len);
            *p.add(len) = 0;
            SHORT_STRING_INTERN.with(|c| {
                c.borrow_mut().insert(key, p as usize);
            });
        }
        return p;
    }
    // Uzun string: arena ama intern YOK (cache belleği sınırlı)
    let p = arena_alloc(len);
    if !p.is_null() {
        std::ptr::copy_nonoverlapping(data.as_ptr(), p as *mut u8, len);
        *p.add(len) = 0;
    }
    p
}

/// Pointer'ın string arena slab'larının içinde olup olmadığını kontrol
/// eder (arena pointer'ları tekil serbest bırakılamaz — hudhud_string_free
/// bunu kullanır).
#[doc(hidden)]
pub fn is_arena_pointer(ptr: usize) -> bool {
    let (min, max) = ARENA_BOUNDS.with(|b| b.get());
    if ptr < min || ptr >= max {
        return false;
    }
    ARENA.with(|a| {
        a.borrow().iter().any(|slab| {
            let start = slab.data.as_ptr() as usize;
            ptr >= start && ptr < start + ARENA_SLAB
        })
    })
}

/// HUDHUD_STRING_TRACE=1 ile arena istatistiklerini stderr'e yazar.
pub fn trace_stats() {
    if std::env::var("HUDHUD_STRING_TRACE").is_ok() {
        ARENA_STATS.with(|s| {
            let (allocs, hits, bytes) = *s.borrow();
            eprintln!(
                "[string_arena] alloc={} intern_hits={} arena_bytes={} ({:.1} MiB)",
                allocs, hits, bytes, bytes as f64 / 1048576.0
            );
        });
    }
}

