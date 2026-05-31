use hudhudscript_ui_core::{
    navigation::{NavigationError, Router},
    PropValue,
};
use std::collections::HashMap;

#[test]
fn test_router_navigation() {
    let mut router = Router::new(
        vec![
            "Main".to_string(),
            "Detail".to_string(),
            "Settings".to_string(),
        ],
        "Main",
    );
    assert_eq!(router.current().screen, "Main");

    router
        .navigate("Detail".to_string(), HashMap::new())
        .unwrap();
    assert_eq!(router.current().screen, "Detail");
    assert_eq!(router.history_len(), 2);
}

#[test]
fn test_router_back() {
    let mut router = Router::new(vec!["Main".to_string(), "Detail".to_string()], "Main");
    router
        .navigate("Detail".to_string(), HashMap::new())
        .unwrap();
    router.back().unwrap();
    assert_eq!(router.current().screen, "Main");
}

#[test]
fn test_router_forward() {
    let mut router = Router::new(vec!["Main".to_string(), "Detail".to_string()], "Main");
    router
        .navigate("Detail".to_string(), HashMap::new())
        .unwrap();
    router.back().unwrap();
    router.forward().unwrap();
    assert_eq!(router.current().screen, "Detail");
}

#[test]
fn test_screen_not_found() {
    let mut router = Router::new(vec!["Main".to_string()], "Main");
    let result = router.navigate("Unknown".to_string(), HashMap::new());
    assert!(result.is_err());
}

#[test]
fn test_deep_links() {
    let mut router = Router::new(vec!["Main".to_string(), "Profile".to_string()], "Main");
    router.add_deep_link("/profile".to_string(), "Profile".to_string());
    assert_eq!(
        router.resolve_deep_link("/profile"),
        Some(&"Profile".to_string())
    );
    assert_eq!(router.resolve_deep_link("/unknown"), None);
}

#[test]
fn test_navigate_with_params() {
    let mut router = Router::new(vec!["Main".to_string(), "Detail".to_string()], "Main");
    let mut params = HashMap::new();
    params.insert("id".to_string(), PropValue::Number(42.0));
    router.navigate("Detail".to_string(), params).unwrap();
    let route = router.current();
    assert_eq!(route.screen, "Detail");
    assert!(route.params.contains_key("id"));
}

#[test]
fn test_can_go_back_and_forward() {
    let mut router = Router::new(vec!["Main".to_string(), "Detail".to_string()], "Main");
    assert!(!router.can_go_back());
    assert!(!router.can_go_forward());

    router
        .navigate("Detail".to_string(), HashMap::new())
        .unwrap();
    assert!(router.can_go_back());
    assert!(!router.can_go_forward());

    router.back().unwrap();
    assert!(!router.can_go_back());
    assert!(router.can_go_forward());
}

#[test]
fn test_back_at_start_errors() {
    let mut router = Router::new(vec!["Main".to_string()], "Main");
    let result = router.back();
    assert!(result.is_err());
}

#[test]
fn test_forward_at_end_errors() {
    let mut router = Router::new(vec!["Main".to_string()], "Main");
    let result = router.forward();
    assert!(result.is_err());
}

#[test]
fn test_navigate_truncates_forward_history() {
    let mut router = Router::new(vec!["A".to_string(), "B".to_string(), "C".to_string()], "A");
    router.navigate("B".to_string(), HashMap::new()).unwrap();
    router.navigate("C".to_string(), HashMap::new()).unwrap();
    // Now at C, history: [A, B, C]
    router.back().unwrap(); // at B
    router.back().unwrap(); // at A
                            // Navigate to B again → forward history (B, C) is truncated
    router.navigate("B".to_string(), HashMap::new()).unwrap();
    assert_eq!(router.history_len(), 2); // [A, B]
    assert!(!router.can_go_forward());
}

#[test]
fn test_navigation_error_display() {
    let err1 = NavigationError::ScreenNotFound("Missing".to_string());
    assert_eq!(format!("{}", err1), "Screen not found: Missing");

    let err2 = NavigationError::NoHistory;
    assert_eq!(format!("{}", err2), "No history to navigate");

    let err3 = NavigationError::Blocked("guard denied".to_string());
    assert_eq!(format!("{}", err3), "Navigation blocked: guard denied");
}

#[test]
fn test_navigation_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(NavigationError::NoHistory);
    assert!(err.to_string().contains("No history"));
}

#[test]
fn test_deep_link_unregistered() {
    let router = Router::new(vec!["Main".to_string()], "Main");
    assert_eq!(router.resolve_deep_link("/anything"), None);
}
