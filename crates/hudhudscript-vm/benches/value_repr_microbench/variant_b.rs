//! Variant B — Arc<AnyObjData<T>> (Rune-style, atomic refcount via Arc).
//
// Layout:
//   enum ReprArc { Inline(Inline), Any(Arc<AnyObjData<()>>) }
//   Inline = 8-byte payload + 1-byte tag  → 16 B with niche
//   Any    = Arc<AnyObjData<()>>          → 8 B pointer
//
// `AnyObjData<T>` carries a vtable pointer + payload; the enum sees only
// `AnyObjData<()>` so the overall ReprArc stays at 16 B.

use std::ptr::NonNull;
use std::sync::Arc;

#[repr(C)]
pub struct AnyObjVtable {
    pub drop_in_place: unsafe fn(NonNull<u8>),
    pub deep_clone: unsafe fn(NonNull<u8>) -> Arc<AnyObjData<()>>,
}

#[repr(C)]
pub struct AnyObjData<T: ?Sized> {
    pub vtable: &'static AnyObjVtable,
    pub data: T,
}

unsafe fn drop_string(p: NonNull<u8>) {
    std::ptr::drop_in_place(p.as_ptr().cast::<String>());
}

unsafe fn drop_array_b(p: NonNull<u8>) {
    std::ptr::drop_in_place(p.as_ptr().cast::<Vec<ReprArc>>());
}

unsafe fn clone_string_arc(p: NonNull<u8>) -> Arc<AnyObjData<()>> {
    let s: &String = &*p.as_ptr().cast::<String>();
    let boxed = Arc::new(AnyObjData {
        vtable: &STRING_VTABLE,
        data: s.clone(),
    });
    // SAFETY: AnyObjData<String> and AnyObjData<()> are layout-compatible
    // via the common `vtable` prefix; the `data` field is only accessed
    // through vtable fn pointers, which know the real type.
    unsafe { std::mem::transmute::<Arc<AnyObjData<String>>, Arc<AnyObjData<()>>>(boxed) }
}

unsafe fn clone_array_arc(p: NonNull<u8>) -> Arc<AnyObjData<()>> {
    let v: &Vec<ReprArc> = &*p.as_ptr().cast::<Vec<ReprArc>>();
    let boxed = Arc::new(AnyObjData {
        vtable: &ARRAY_VTABLE_B,
        data: v.clone(),
    });
    unsafe { std::mem::transmute::<Arc<AnyObjData<Vec<ReprArc>>>, Arc<AnyObjData<()>>>(boxed) }
}

static STRING_VTABLE: AnyObjVtable = AnyObjVtable {
    drop_in_place: drop_string,
    deep_clone: clone_string_arc,
};

static ARRAY_VTABLE_B: AnyObjVtable = AnyObjVtable {
    drop_in_place: drop_array_b,
    deep_clone: clone_array_arc,
};

#[derive(Clone, Copy)]
pub enum InlineB {
    Null,
    Bool(bool),
    Number(f64),
    Int(i64),
}

pub enum ReprArc {
    Inline(InlineB),
    Any(Arc<AnyObjData<()>>),
}

impl Clone for ReprArc {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            ReprArc::Inline(i) => ReprArc::Inline(*i),
            ReprArc::Any(a) => ReprArc::Any(Arc::clone(a)),
        }
    }
}

impl Drop for ReprArc {
    #[inline]
    fn drop(&mut self) {
        if let ReprArc::Any(a) = self {
            let _ = a.vtable;
        }
    }
}

pub fn b_make_string(s: &str) -> ReprArc {
    let boxed = Arc::new(AnyObjData {
        vtable: &STRING_VTABLE,
        data: s.to_string(),
    });
    let erased: Arc<AnyObjData<()>> =
        unsafe { std::mem::transmute::<Arc<AnyObjData<String>>, Arc<AnyObjData<()>>>(boxed) };
    ReprArc::Any(erased)
}

pub fn b_make_array(n: usize) -> ReprArc {
    let v: Vec<ReprArc> = (0..n)
        .map(|i| ReprArc::Inline(InlineB::Number(i as f64)))
        .collect();
    let boxed = Arc::new(AnyObjData {
        vtable: &ARRAY_VTABLE_B,
        data: v,
    });
    let erased: Arc<AnyObjData<()>> =
        unsafe { std::mem::transmute::<Arc<AnyObjData<Vec<ReprArc>>>, Arc<AnyObjData<()>>>(boxed) };
    ReprArc::Any(erased)
}

pub fn b_make_number(x: f64) -> ReprArc {
    ReprArc::Inline(InlineB::Number(x))
}

pub fn b_dispatch(v: &ReprArc) -> u64 {
    match v {
        ReprArc::Inline(_) => 1,
        ReprArc::Any(a) => {
            let _ = a.vtable as *const _;
            2
        }
    }
}
