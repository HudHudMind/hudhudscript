//! Variant C — NonNull<AnyObjData<T>> + AtomicUsize (manual, no Arc).
//
// Rune's actual design: refcount is IN the header, pointer is raw
// `NonNull`.  Allocation via `Box::leak`, reclaimed on final drop.
// Pays one atomic op on clone / drop (same as Arc) but saves the Arc
// strong/weak split (Rune doesn't use weak refs).

use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(C)]
pub struct AnyObjDataC<T: ?Sized> {
    pub refcount: AtomicUsize,
    pub vtable: &'static AnyObjVtableC,
    pub data: T,
}

#[repr(C)]
pub struct AnyObjVtableC {
    pub drop_and_free: unsafe fn(NonNull<AnyObjDataC<()>>),
    pub deep_clone: unsafe fn(NonNull<AnyObjDataC<()>>) -> NonNull<AnyObjDataC<()>>,
}

unsafe fn drop_string_c(p: NonNull<AnyObjDataC<()>>) {
    let boxed: Box<AnyObjDataC<String>> = Box::from_raw(p.as_ptr().cast());
    drop(boxed);
}

unsafe fn drop_array_c(p: NonNull<AnyObjDataC<()>>) {
    let boxed: Box<AnyObjDataC<Vec<ReprManual>>> = Box::from_raw(p.as_ptr().cast());
    drop(boxed);
}

unsafe fn clone_string_c(p: NonNull<AnyObjDataC<()>>) -> NonNull<AnyObjDataC<()>> {
    let src: &AnyObjDataC<String> = &*p.as_ptr().cast();
    alloc_string_c(src.data.clone())
}

unsafe fn clone_array_c(p: NonNull<AnyObjDataC<()>>) -> NonNull<AnyObjDataC<()>> {
    let src: &AnyObjDataC<Vec<ReprManual>> = &*p.as_ptr().cast();
    alloc_array_c(src.data.clone())
}

static STRING_VTABLE_C: AnyObjVtableC = AnyObjVtableC {
    drop_and_free: drop_string_c,
    deep_clone: clone_string_c,
};

static ARRAY_VTABLE_C: AnyObjVtableC = AnyObjVtableC {
    drop_and_free: drop_array_c,
    deep_clone: clone_array_c,
};

unsafe fn alloc_string_c(s: String) -> NonNull<AnyObjDataC<()>> {
    let b = Box::new(AnyObjDataC {
        refcount: AtomicUsize::new(1),
        vtable: &STRING_VTABLE_C,
        data: s,
    });
    let raw = Box::into_raw(b);
    NonNull::new_unchecked(raw.cast())
}

unsafe fn alloc_array_c(v: Vec<ReprManual>) -> NonNull<AnyObjDataC<()>> {
    let b = Box::new(AnyObjDataC {
        refcount: AtomicUsize::new(1),
        vtable: &ARRAY_VTABLE_C,
        data: v,
    });
    let raw = Box::into_raw(b);
    NonNull::new_unchecked(raw.cast())
}

#[derive(Clone, Copy)]
pub enum InlineC {
    Null,
    Bool(bool),
    Number(f64),
    Int(i64),
}

pub enum ReprManual {
    Inline(InlineC),
    Any(NonNull<AnyObjDataC<()>>),
}

// SAFETY: AnyObjDataC is behind an atomic refcount; payloads are Send+Sync
// for our microbench (String, Vec<ReprManual>).
unsafe impl Send for ReprManual {}
unsafe impl Sync for ReprManual {}

impl Clone for ReprManual {
    #[inline]
    fn clone(&self) -> Self {
        match self {
            ReprManual::Inline(i) => ReprManual::Inline(*i),
            ReprManual::Any(p) => {
                unsafe {
                    let header: &AnyObjDataC<()> = &*p.as_ptr();
                    header.refcount.fetch_add(1, Ordering::Relaxed);
                }
                ReprManual::Any(*p)
            }
        }
    }
}

impl Drop for ReprManual {
    #[inline]
    fn drop(&mut self) {
        if let ReprManual::Any(p) = self {
            unsafe {
                let header: &AnyObjDataC<()> = &*p.as_ptr();
                if header.refcount.fetch_sub(1, Ordering::Release) == 1 {
                    std::sync::atomic::fence(Ordering::Acquire);
                    (header.vtable.drop_and_free)(*p);
                }
            }
        }
    }
}

pub fn c_make_string(s: &str) -> ReprManual {
    unsafe { ReprManual::Any(alloc_string_c(s.to_string())) }
}

pub fn c_make_array(n: usize) -> ReprManual {
    let v: Vec<ReprManual> = (0..n)
        .map(|i| ReprManual::Inline(InlineC::Number(i as f64)))
        .collect();
    unsafe { ReprManual::Any(alloc_array_c(v)) }
}

pub fn c_make_number(x: f64) -> ReprManual {
    ReprManual::Inline(InlineC::Number(x))
}

pub fn c_dispatch(v: &ReprManual) -> u64 {
    match v {
        ReprManual::Inline(_) => 1,
        ReprManual::Any(p) => unsafe {
            let _ = (*p.as_ptr()).vtable as *const _;
            2
        },
    }
}
