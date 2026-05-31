//! Tests for ResourceCache.

use hudhudscript_resources::{CachedResource, ResourceCache, ResourceContent, ResourceMetadata};
use std::time::{Duration, SystemTime};

fn make_cached(uri: &str, text: &str) -> CachedResource {
    let now = SystemTime::now();
    CachedResource {
        metadata: ResourceMetadata {
            uri: uri.into(),
            name: "Test".into(),
            description: None,
            mime_type: Some("text/plain".into()),
            server: "test".into(),
            discovered_at: now,
            last_accessed: None,
            access_count: 0,
            tags: vec![],
            etag: None,
        },
        content: ResourceContent::Text(text.into()),
        cached_at: now,
        etag: None,
    }
}

#[test]
fn cache_new_is_empty() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    assert_eq!(cache.size(), 0);
}

#[test]
fn cache_put_and_get() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    let uri = "file:///data.txt".to_string();
    let resource = make_cached(&uri, "hello");
    cache.put(uri.clone(), resource);
    assert_eq!(cache.size(), 1);

    let cached = cache.get(&uri);
    assert!(cached.is_some());
    match cached.unwrap().content {
        ResourceContent::Text(t) => assert_eq!(t, "hello"),
        _ => panic!("expected Text"),
    }
}

#[test]
fn cache_get_missing() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    assert!(cache.get("nonexistent").is_none());
}

#[test]
fn cache_remove() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    let uri = "file:///remove_me.txt".to_string();
    cache.put(uri.clone(), make_cached(&uri, "bye"));
    assert_eq!(cache.size(), 1);

    cache.remove(&uri);
    assert_eq!(cache.size(), 0);
    assert!(cache.get(&uri).is_none());
}

#[test]
fn cache_clear() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    for i in 0..5 {
        let uri = format!("file:///data{}.txt", i);
        cache.put(uri.clone(), make_cached(&uri, "data"));
    }
    assert_eq!(cache.size(), 5);

    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn cache_multiple_entries() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    cache.put("a".into(), make_cached("a", "alpha"));
    cache.put("b".into(), make_cached("b", "beta"));
    cache.put("c".into(), make_cached("c", "gamma"));
    assert_eq!(cache.size(), 3);
    assert!(cache.get("a").is_some());
    assert!(cache.get("b").is_some());
    assert!(cache.get("c").is_some());
}

#[test]
fn cache_overwrite() {
    let cache = ResourceCache::new(Duration::from_secs(300));
    let uri = "file:///overwrite.txt".to_string();
    cache.put(uri.clone(), make_cached(&uri, "old"));
    cache.put(uri.clone(), make_cached(&uri, "new"));
    assert_eq!(cache.size(), 1);
    match cache.get(&uri).unwrap().content {
        ResourceContent::Text(t) => assert_eq!(t, "new"),
        _ => panic!("expected Text"),
    }
}
