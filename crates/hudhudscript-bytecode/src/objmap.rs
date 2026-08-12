use crate::{SymId, Value16};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Object map — stores property values in an `FxHashMap<SymId, Value16>`.
/// This is the P3 revert: generic dictionaries use a plain hash map so
/// property insert/lookup avoids the global ShapeRegistry overhead that
/// dominated the v0.7.344 regression.

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjMap {
    inner: FxHashMap<crate::sym::SymId, Value16>,
}

impl ObjMap {
    /// Number of properties.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// P6: create with pre-allocated capacity for N properties.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: FxHashMap::with_capacity_and_hasher(cap, Default::default()),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Insert a key-value pair.  Returns the previous value if the key existed.
    #[inline]
    pub fn insert(&mut self, key: impl Into<crate::sym::SymId>, val: Value16) -> Option<Value16> {
        self.inner.insert(key.into(), val)
    }

    /// Lookup by any key type that implements `ObjMapKey` (`&str`, `&String`, `&SymId`).
    #[inline]
    pub fn get<K: ObjMapKey + ?Sized>(&self, key: &K) -> Option<&Value16> {
        key.lookup(self)
    }

    /// Mutable lookup by string key.
    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value16> {
        self.inner.get_mut(&crate::sym::SymId::from(key))
    }

    /// Check if a key exists.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(&crate::sym::SymId::from(key))
    }

    /// Remove a key, returning the previous value.
    #[inline]
    pub fn remove(&mut self, key: &str) -> Option<Value16> {
        self.inner.remove(&crate::sym::SymId::from(key))
    }

    /// All property symbols of this object.
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = crate::sym::SymId> + '_ {
        self.inner.keys().copied()
    }

    /// Iterate over `(SymId, &Value16)` pairs.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (crate::sym::SymId, &Value16)> + '_ {
        self.inner.iter().map(|(k, v)| (*k, v))
    }

    /// Iterate over mutable values only.
    #[inline]
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Value16> + '_ {
        self.inner.values_mut()
    }

    /// Iterate over `(SymId, &mut Value16)` pairs.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (crate::sym::SymId, &mut Value16)> + '_ {
        self.inner.iter_mut().map(|(k, v)| (*k, v))
    }

    /// Iterate over values.
    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Value16> + '_ {
        self.inner.values()
    }

    /// Simple entry-style shim.
    #[inline]
    pub fn entry(&mut self, key: impl Into<crate::sym::SymId>) -> ObjMapEntry<'_> {
        ObjMapEntry {
            map: self,
            key: key.into(),
        }
    }

    /// Clear all entries.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

/// Entry shim for `ObjMap::entry`.
pub struct ObjMapEntry<'a> {
    map: &'a mut ObjMap,
    key: crate::sym::SymId,
}

impl<'a> ObjMapEntry<'a> {
    #[inline]
    pub fn or_insert(self, value: Value16) -> &'a mut Value16 {
        self.or_insert_with(|| value)
    }

    #[inline]
    pub fn or_insert_with<F: FnOnce() -> Value16>(self, f: F) -> &'a mut Value16 {
        let key = self.key;
        self.map.inner.entry(key).or_insert_with(f)
    }
}

impl<K: Into<crate::sym::SymId>> Extend<(K, Value16)> for ObjMap {
    #[inline]
    fn extend<I: IntoIterator<Item = (K, Value16)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.inner.insert(k.into(), v);
        }
    }
}

impl<K: Into<crate::sym::SymId>> FromIterator<(K, Value16)> for ObjMap {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, Value16)>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }
}

impl IntoIterator for ObjMap {
    type Item = (crate::sym::SymId, Value16);
    type IntoIter = std::collections::hash_map::IntoIter<crate::sym::SymId, Value16>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a ObjMap {
    type Item = (crate::sym::SymId, &'a Value16);
    type IntoIter = std::iter::Map<
        std::collections::hash_map::Iter<'a, crate::sym::SymId, Value16>,
        fn((&crate::sym::SymId, &'a Value16)) -> (crate::sym::SymId, &'a Value16),
    >;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter().map(|(k, v)| (*k, v))
    }
}

// ── Trait for polymorphic `get` ─────────────────────────────────────────

/// Allows `ObjMap::get` to accept both `&str` and `&SymId` keys.
pub trait ObjMapKey {
    fn lookup_offset(&self, _obj: &ObjMap) -> Option<usize> {
        None
    }
    fn lookup<'a>(&self, obj: &'a ObjMap) -> Option<&'a Value16> {
        obj.inner.get(&self.to_sym_id())
    }
    fn to_sym_id(&self) -> crate::sym::SymId;
}

impl ObjMapKey for str {
    #[inline]
    fn to_sym_id(&self) -> crate::sym::SymId {
        crate::sym::SymId::from(self)
    }
}

impl ObjMapKey for crate::sym::SymId {
    #[inline]
    fn to_sym_id(&self) -> crate::sym::SymId {
        *self
    }
}

impl ObjMapKey for String {
    #[inline]
    fn to_sym_id(&self) -> crate::sym::SymId {
        crate::sym::SymId::from(self.as_str())
    }
}

// ── Convenience constructors ────────────────────────────────────────────

impl ObjMap {
    /// Create an empty object map.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an object map with a single key/value pair.
    #[inline]
    pub fn with_one(key: impl Into<crate::sym::SymId>, value: Value16) -> Self {
        let mut m = Self::default();
        m.insert(key, value);
        m
    }
}
