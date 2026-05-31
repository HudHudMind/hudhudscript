//! External tests for hudhudscript-package::marketplace —
//! Marketplace, MarketplaceEntry, SearchQuery, SearchResult, SortBy, Category.

use chrono::Utc;
use hudhudscript_package::{
    default_page, default_per_page, Category, Marketplace, MarketplaceEntry, RegistryClient,
    SearchQuery, SearchResult, SortBy,
};

#[test]
fn test_search_query_defaults() {
    let q = SearchQuery::default();
    assert_eq!(q.page, 1);
    assert_eq!(q.per_page, 20);
    assert_eq!(q.sort_by, SortBy::Relevance);
    assert!(q.query.is_empty());
    assert!(q.tags.is_empty());
}

#[test]
fn test_marketplace_entry_serialization() {
    let entry = MarketplaceEntry {
        name: "ai-tools".to_string(),
        version: "1.2.0".to_string(),
        description: "AI utility library".to_string(),
        author: "hudhud-team".to_string(),
        downloads: 4200,
        rating: 4.5,
        tags: vec!["ai".to_string(), "tools".to_string()],
        signature: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("ai-tools"));
    assert!(json.contains("4200"));

    let deserialized: MarketplaceEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "ai-tools");
    assert_eq!(deserialized.downloads, 4200);
}

#[test]
fn test_search_result_serialization() {
    let result = SearchResult {
        entries: vec![],
        total_count: 0,
        page: 1,
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_count, 0);
    assert_eq!(deserialized.page, 1);
}

#[test]
fn test_category_serialization() {
    let cat = Category {
        name: "ai".to_string(),
        description: "AI and machine learning packages".to_string(),
        package_count: 42,
    };
    let json = serde_json::to_string(&cat).unwrap();
    let deserialized: Category = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "ai");
    assert_eq!(deserialized.package_count, 42);
}

#[test]
fn test_sort_by_variants() {
    let json = serde_json::to_string(&SortBy::Downloads).unwrap();
    assert_eq!(json, "\"downloads\"");

    let deserialized: SortBy = serde_json::from_str("\"rating\"").unwrap();
    assert_eq!(deserialized, SortBy::Rating);
}

#[test]
fn test_marketplace_creation() {
    let registry = RegistryClient::new("https://registry.hudhudscript.org").unwrap();
    let marketplace = Marketplace::new(registry);
    // Smoke test: marketplace was created without error.
    let _ = marketplace;
}

#[test]
fn test_sort_by_all_variants_serde() {
    let variants = [
        (SortBy::Relevance, "\"relevance\""),
        (SortBy::Downloads, "\"downloads\""),
        (SortBy::Rating, "\"rating\""),
        (SortBy::RecentlyUpdated, "\"recently_updated\""),
        (SortBy::Name, "\"name\""),
    ];
    for (variant, expected_json) in &variants {
        let json = serde_json::to_string(variant).unwrap();
        assert_eq!(&json, expected_json);
        let deserialized: SortBy = serde_json::from_str(&json).unwrap();
        assert_eq!(&deserialized, variant);
    }
}

#[test]
fn test_sort_by_default() {
    assert_eq!(SortBy::default(), SortBy::Relevance);
}

#[test]
fn test_search_query_custom() {
    let q = SearchQuery {
        query: "ai tools".to_string(),
        tags: vec!["ai".to_string(), "ml".to_string()],
        sort_by: SortBy::Downloads,
        page: 3,
        per_page: 50,
    };
    let json = serde_json::to_string(&q).unwrap();
    let deserialized: SearchQuery = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.query, "ai tools");
    assert_eq!(deserialized.tags.len(), 2);
    assert_eq!(deserialized.sort_by, SortBy::Downloads);
    assert_eq!(deserialized.page, 3);
    assert_eq!(deserialized.per_page, 50);
}

#[test]
fn test_marketplace_entry_with_rating() {
    let entry = MarketplaceEntry {
        name: "rated-pkg".to_string(),
        version: "2.0.0".to_string(),
        description: "Highly rated".to_string(),
        author: "dev".to_string(),
        downloads: 10000,
        rating: 4.8,
        tags: vec![],
        signature: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    assert_eq!(entry.rating, 4.8);
    assert_eq!(entry.downloads, 10000);
    assert_eq!(entry.name, "rated-pkg");
}

#[test]
fn test_default_page_and_per_page_values() {
    assert_eq!(default_page(), 1);
    assert_eq!(default_per_page(), 20);
}

#[test]
fn test_search_query_serde_with_defaults() {
    // Deserialize JSON with missing optional fields -> defaults applied
    let json = r#"{"query": "ai"}"#;
    let q: SearchQuery = serde_json::from_str(json).unwrap();
    assert_eq!(q.query, "ai");
    assert_eq!(q.page, 1);
    assert_eq!(q.per_page, 20);
    assert_eq!(q.sort_by, SortBy::Relevance);
    assert!(q.tags.is_empty());
}

#[test]
fn test_search_result_with_entries() {
    let entry = MarketplaceEntry {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        description: "A package".to_string(),
        author: "dev".to_string(),
        downloads: 100,
        rating: 3.5,
        tags: vec!["tag1".to_string()],
        signature: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let result = SearchResult {
        entries: vec![entry],
        total_count: 1,
        page: 1,
    };
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.entries.len(), 1);
    assert_eq!(deserialized.entries[0].name, "pkg");
    assert_eq!(deserialized.total_count, 1);
}

#[test]
fn test_marketplace_entry_with_signature_none() {
    let entry = MarketplaceEntry {
        name: "unsigned".to_string(),
        version: "0.1.0".to_string(),
        description: "No signature".to_string(),
        author: "anon".to_string(),
        downloads: 0,
        rating: 0.0,
        tags: vec![],
        signature: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"signature\":null"));
    let deserialized: MarketplaceEntry = serde_json::from_str(&json).unwrap();
    assert!(deserialized.signature.is_none());
}

#[test]
fn test_category_clone() {
    let cat = Category {
        name: "tools".to_string(),
        description: "Developer tools".to_string(),
        package_count: 10,
    };
    let cloned = cat.clone();
    assert_eq!(cloned.name, "tools");
    assert_eq!(cloned.package_count, 10);
}

#[test]
fn test_marketplace_entry_clone() {
    let entry = MarketplaceEntry {
        name: "pkg".to_string(),
        version: "1.0.0".to_string(),
        description: "desc".to_string(),
        author: "author".to_string(),
        downloads: 42,
        rating: 4.0,
        tags: vec!["a".to_string()],
        signature: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let cloned = entry.clone();
    assert_eq!(cloned.name, "pkg");
    assert_eq!(cloned.downloads, 42);
    assert_eq!(cloned.tags.len(), 1);
}

#[test]
fn test_marketplace_debug() {
    let registry = RegistryClient::new("https://registry.hudhudscript.org").unwrap();
    let marketplace = Marketplace::new(registry);
    let debug = format!("{:?}", marketplace);
    assert!(debug.contains("Marketplace"));
}

#[test]
fn test_sort_by_invalid_json() {
    let result: std::result::Result<SortBy, _> = serde_json::from_str("\"unknown_sort\"");
    assert!(result.is_err());
}
