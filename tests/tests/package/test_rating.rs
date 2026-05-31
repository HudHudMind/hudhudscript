//! External tests for hudhudscript-package::rating — Rating, RatingStore.

use hudhudscript_package::{Rating, RatingStore};

#[test]
fn test_rating_clamp() {
    let r = Rating::new("user1", "pkg", 0, None);
    assert_eq!(r.stars, 1);

    let r = Rating::new("user1", "pkg", 10, None);
    assert_eq!(r.stars, 5);

    let r = Rating::new("user1", "pkg", 3, None);
    assert_eq!(r.stars, 3);
}

#[test]
fn test_add_and_get_ratings() {
    let mut store = RatingStore::new();

    store.add_rating(Rating::new("alice", "pkg-a", 5, Some("Great!".into())));
    store.add_rating(Rating::new("bob", "pkg-a", 3, None));

    let ratings = store.get_ratings("pkg-a");
    assert_eq!(ratings.len(), 2);
}

#[test]
fn test_replace_existing_rating() {
    let mut store = RatingStore::new();

    store.add_rating(Rating::new("alice", "pkg-a", 3, None));
    store.add_rating(Rating::new(
        "alice",
        "pkg-a",
        5,
        Some("Changed my mind".into()),
    ));

    let ratings = store.get_ratings("pkg-a");
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0].stars, 5);
}

#[test]
fn test_average_rating() {
    let mut store = RatingStore::new();

    assert!(store.average_rating("pkg-a").is_none());

    store.add_rating(Rating::new("alice", "pkg-a", 4, None));
    store.add_rating(Rating::new("bob", "pkg-a", 2, None));

    let avg = store.average_rating("pkg-a").unwrap();
    assert!((avg - 3.0).abs() < f64::EPSILON);
}

#[test]
fn test_top_rated() {
    let mut store = RatingStore::new();

    store.add_rating(Rating::new("alice", "pkg-a", 5, None));
    store.add_rating(Rating::new("bob", "pkg-a", 5, None));

    store.add_rating(Rating::new("alice", "pkg-b", 3, None));

    store.add_rating(Rating::new("alice", "pkg-c", 4, None));
    store.add_rating(Rating::new("bob", "pkg-c", 4, None));

    let top = store.top_rated(2);
    assert_eq!(top.len(), 2);
    // pkg-a (avg 5.0) should be first
    assert_eq!(top[0].0, "pkg-a");
    // pkg-c (avg 4.0, 2 ratings) should beat pkg-b (avg 3.0, 1 rating)
    assert_eq!(top[1].0, "pkg-c");
}

#[test]
fn test_empty_store() {
    let store = RatingStore::new();
    assert_eq!(store.total_ratings(), 0);
    assert!(store.get_ratings("anything").is_empty());
    assert!(store.top_rated(10).is_empty());
}

#[test]
fn test_rating_serialization() {
    let rating = Rating::new("alice", "pkg-a", 4, Some("Nice package".into()));
    let json = serde_json::to_string(&rating).unwrap();
    let deserialized: Rating = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, "alice");
    assert_eq!(deserialized.stars, 4);
    assert_eq!(deserialized.review_text, Some("Nice package".to_string()));
}

#[test]
fn test_total_ratings_with_data() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("alice", "pkg-a", 5, None));
    store.add_rating(Rating::new("bob", "pkg-a", 4, None));
    store.add_rating(Rating::new("alice", "pkg-b", 3, None));
    assert_eq!(store.total_ratings(), 3);
}

#[test]
fn test_get_ratings_nonexistent_package() {
    let store = RatingStore::new();
    let ratings = store.get_ratings("nonexistent");
    assert_eq!(ratings.len(), 0);
}

#[test]
fn test_average_rating_single() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("alice", "pkg", 4, None));
    let avg = store.average_rating("pkg").unwrap();
    assert!((avg - 4.0).abs() < f64::EPSILON);
}

#[test]
fn test_top_rated_limit_larger_than_entries() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("alice", "pkg", 5, None));
    let top = store.top_rated(100);
    assert_eq!(top.len(), 1);
    assert_eq!(top[0].0, "pkg");
    assert!((top[0].1 - 5.0).abs() < f64::EPSILON);
    assert_eq!(top[0].2, 1);
}

#[test]
fn test_rating_clamp_boundary_values() {
    let r1 = Rating::new("u", "p", 1, None);
    assert_eq!(r1.stars, 1);
    let r5 = Rating::new("u", "p", 5, None);
    assert_eq!(r5.stars, 5);
}

#[test]
fn test_rating_with_review_text() {
    let r = Rating::new("user", "pkg", 3, Some("Good stuff".to_string()));
    assert_eq!(r.review_text, Some("Good stuff".to_string()));
    assert_eq!(r.user_id, "user");
    assert_eq!(r.package_name, "pkg");
}

#[test]
fn test_replace_updates_total_ratings() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("alice", "pkg", 3, None));
    assert_eq!(store.total_ratings(), 1);
    // Replace alice's rating
    store.add_rating(Rating::new("alice", "pkg", 5, None));
    assert_eq!(store.total_ratings(), 1);
}

#[test]
fn test_top_rated_tie_breaking_by_count() {
    let mut store = RatingStore::new();

    // pkg-a: avg 4.0, 3 ratings
    store.add_rating(Rating::new("u1", "pkg-a", 4, None));
    store.add_rating(Rating::new("u2", "pkg-a", 4, None));
    store.add_rating(Rating::new("u3", "pkg-a", 4, None));

    // pkg-b: avg 4.0, 1 rating
    store.add_rating(Rating::new("u1", "pkg-b", 4, None));

    let top = store.top_rated(10);
    assert_eq!(top.len(), 2);
    // pkg-a should come first (same avg, more ratings)
    assert_eq!(top[0].0, "pkg-a");
    assert_eq!(top[0].2, 3);
    assert_eq!(top[1].0, "pkg-b");
    assert_eq!(top[1].2, 1);
}

#[test]
fn test_top_rated_zero_limit() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("u1", "pkg", 5, None));
    let top = store.top_rated(0);
    assert!(top.is_empty());
}

#[test]
fn test_average_rating_multiple_packages() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("u1", "pkg-a", 5, None));
    store.add_rating(Rating::new("u2", "pkg-a", 3, None));
    store.add_rating(Rating::new("u1", "pkg-b", 1, None));

    let avg_a = store.average_rating("pkg-a").unwrap();
    let avg_b = store.average_rating("pkg-b").unwrap();
    assert!((avg_a - 4.0).abs() < f64::EPSILON);
    assert!((avg_b - 1.0).abs() < f64::EPSILON);
    // Non-existent package
    assert!(store.average_rating("pkg-c").is_none());
}

#[test]
fn test_rating_clamp_255() {
    let r = Rating::new("u", "p", 255, None);
    assert_eq!(r.stars, 5);
}

#[test]
fn test_add_rating_multiple_users_same_package() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("alice", "pkg", 5, None));
    store.add_rating(Rating::new("bob", "pkg", 4, None));
    store.add_rating(Rating::new("charlie", "pkg", 3, None));

    let ratings = store.get_ratings("pkg");
    assert_eq!(ratings.len(), 3);
    assert_eq!(store.total_ratings(), 3);
}

#[test]
fn test_rating_store_default_is_new() {
    let store = RatingStore::default();
    assert_eq!(store.total_ratings(), 0);
    assert!(store.top_rated(10).is_empty());
}

#[test]
fn test_replace_preserves_other_users() {
    let mut store = RatingStore::new();
    store.add_rating(Rating::new("alice", "pkg", 3, None));
    store.add_rating(Rating::new("bob", "pkg", 4, None));
    // Replace alice's rating
    store.add_rating(Rating::new("alice", "pkg", 5, Some("Updated".into())));

    let ratings = store.get_ratings("pkg");
    assert_eq!(ratings.len(), 2);
    // Bob's rating should still be there
    assert!(ratings.iter().any(|r| r.user_id == "bob" && r.stars == 4));
    // Alice's rating should be updated
    assert!(ratings.iter().any(|r| r.user_id == "alice" && r.stars == 5));
}
