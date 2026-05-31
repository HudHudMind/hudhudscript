use chrono::Utc;
use hudhudscript_cache::{CacheEntryType, CacheIndex, IndexEntry};

fn make_entry(key: &str, entry_type: CacheEntryType, size: u64) -> IndexEntry {
    IndexEntry {
        key: key.to_string(),
        entry_type,
        size_bytes: size,
        created_at: Utc::now(),
        content_hash: None,
    }
}

#[test]
fn test_insert_and_get() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("cons.1", CacheEntryType::Constitution, 100));

    assert!(index.contains("cons.1"));
    assert_eq!(index.len(), 1);

    let entry = index.get("cons.1").unwrap();
    assert_eq!(entry.entry_type, CacheEntryType::Constitution);
    assert_eq!(entry.size_bytes, 100);
}

#[test]
fn test_remove() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("cons.1", CacheEntryType::Constitution, 100));

    let removed = index.remove("cons.1");
    assert!(removed.is_some());
    assert!(!index.contains("cons.1"));
    assert!(index.is_empty());
}

#[test]
fn test_total_size() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("a", CacheEntryType::Constitution, 100));
    index.insert(make_entry("b", CacheEntryType::Law, 200));
    index.insert(make_entry("c", CacheEntryType::Rule, 50));

    assert_eq!(index.total_size_bytes(), 350);
}

#[test]
fn test_count_by_type() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("cons.1", CacheEntryType::Constitution, 100));
    index.insert(make_entry("cons.2", CacheEntryType::Constitution, 100));
    index.insert(make_entry("law.1", CacheEntryType::Law, 100));
    index.insert(make_entry("rule.1", CacheEntryType::Rule, 100));

    assert_eq!(index.count_by_type(CacheEntryType::Constitution), 2);
    assert_eq!(index.count_by_type(CacheEntryType::Law), 1);
    assert_eq!(index.count_by_type(CacheEntryType::Rule), 1);
}

#[test]
fn test_summary() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("cons.1", CacheEntryType::Constitution, 100));
    index.insert(make_entry("law.1", CacheEntryType::Law, 200));
    index.insert(make_entry("rule.1", CacheEntryType::Rule, 50));

    let summary = index.summary();
    assert_eq!(summary.total_entries, 3);
    assert_eq!(summary.constitutions, 1);
    assert_eq!(summary.laws, 1);
    assert_eq!(summary.rules, 1);
    assert_eq!(summary.total_size_bytes, 350);
}

#[test]
fn test_keys_with_hash() {
    let mut index = CacheIndex::new();

    let mut entry1 = make_entry("a", CacheEntryType::Rule, 100);
    entry1.content_hash = Some("abc123".to_string());
    index.insert(entry1);

    let mut entry2 = make_entry("b", CacheEntryType::Rule, 100);
    entry2.content_hash = Some("abc123".to_string());
    index.insert(entry2);

    let mut entry3 = make_entry("c", CacheEntryType::Rule, 100);
    entry3.content_hash = Some("def456".to_string());
    index.insert(entry3);

    let mut matches = index.keys_with_hash("abc123");
    matches.sort();
    assert_eq!(matches, vec!["a", "b"]);
}

#[test]
fn test_clear() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("a", CacheEntryType::Rule, 100));
    index.insert(make_entry("b", CacheEntryType::Rule, 100));

    index.clear();
    assert!(index.is_empty());
    assert_eq!(index.total_size_bytes(), 0);
}

#[test]
fn test_cache_entry_type_display() {
    assert_eq!(CacheEntryType::Constitution.to_string(), "constitution");
    assert_eq!(CacheEntryType::Law.to_string(), "law");
    assert_eq!(CacheEntryType::Rule.to_string(), "rule");
}

#[test]
fn test_remove_nonexistent_key_returns_none() {
    let mut index = CacheIndex::new();
    assert!(index.remove("nonexistent").is_none());
}

#[test]
fn test_get_nonexistent_key_returns_none() {
    let index = CacheIndex::new();
    assert!(index.get("nonexistent").is_none());
}

#[test]
fn test_contains_false_for_missing() {
    let index = CacheIndex::new();
    assert!(!index.contains("missing"));
}

#[test]
fn test_keys_with_hash_no_matches() {
    let index = CacheIndex::new();
    let matches = index.keys_with_hash("nonexistent");
    assert_eq!(matches.len(), 0);
}

#[test]
fn test_insert_overwrites_existing() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("a", CacheEntryType::Rule, 100));
    index.insert(make_entry("a", CacheEntryType::Law, 200));

    assert_eq!(index.len(), 1);
    let entry = index.get("a").unwrap();
    assert_eq!(entry.entry_type, CacheEntryType::Law);
    assert_eq!(entry.size_bytes, 200);
}

#[test]
fn test_summary_empty_index() {
    let index = CacheIndex::new();
    let summary = index.summary();
    assert_eq!(summary.total_entries, 0);
    assert_eq!(summary.constitutions, 0);
    assert_eq!(summary.laws, 0);
    assert_eq!(summary.rules, 0);
    assert_eq!(summary.total_size_bytes, 0);
}

#[test]
fn test_keys_iterator() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("x", CacheEntryType::Rule, 10));
    index.insert(make_entry("y", CacheEntryType::Law, 20));
    let mut keys: Vec<&String> = index.keys().collect();
    keys.sort();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], "x");
    assert_eq!(keys[1], "y");
}

#[test]
fn test_iter_returns_all_entries() {
    let mut index = CacheIndex::new();
    index.insert(make_entry("a", CacheEntryType::Constitution, 50));
    index.insert(make_entry("b", CacheEntryType::Rule, 60));
    let entries: Vec<_> = index.iter().collect();
    assert_eq!(entries.len(), 2);
}
