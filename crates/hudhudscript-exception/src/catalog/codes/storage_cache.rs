use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum StorageCacheExceptionCode {
    /// E0016 — Constitution Entry Missing From Cache
    CacheConstitutionNotFound = 16,
    /// E0017 — Cache Entry Failed To Deserialize
    CacheDeserializationError = 17,
    /// E0018 — Identical Content Already Stored Under Another Key
    CacheDuplicateContent = 18,
    /// E0019 — Cache Id Collision Between Different Contents
    CacheIdCollision = 19,
    /// E0020 — Law Entry Missing From Cache
    CacheLawNotFound = 20,
    /// E0021 — Cache Capacity Quota Exceeded
    CacheQuotaExceeded = 21,
    /// E0022 — Rule Entry Missing From Cache
    CacheRuleNotFound = 22,
    /// E0023 — Cache Value Failed To Serialize
    CacheSerializationError = 23,
}
