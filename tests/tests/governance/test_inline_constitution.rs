use hudhudscript_governance::constitution::{ConstitutionError, ConstitutionManager};
use hudhudscript_governance::*;

#[test]
fn test_constitution_creation() {
    let manager = ConstitutionManager::new(
        "cons.1".to_string(),
        "Test Constitution".to_string(),
        Some("Description".to_string()),
    );

    assert_eq!(manager.current().id, "cons.1");
    assert_eq!(manager.current().name, "Test Constitution");
    assert_eq!(manager.current_version(), 1);
    assert_eq!(manager.available_versions(), vec![1]);
}

#[test]
fn test_constitution_modification() {
    let mut manager =
        ConstitutionManager::new("cons.1".to_string(), "Original Name".to_string(), None);

    let version = manager.modify(|constitution| {
        constitution.name = "Modified Name".to_string();
    });

    assert_eq!(version, 2);
    assert_eq!(manager.current_version(), 2);
    assert_eq!(manager.current().name, "Modified Name");
}

#[test]
fn test_version_history() {
    let mut manager = ConstitutionManager::new("cons.1".to_string(), "Version 1".to_string(), None);

    manager.modify(|c| c.name = "Version 2".to_string());
    manager.modify(|c| c.name = "Version 3".to_string());

    assert_eq!(manager.available_versions(), vec![1, 2, 3]);

    let v1 = manager.get_version(1).unwrap();
    assert_eq!(v1.name, "Version 1");

    let v2 = manager.get_version(2).unwrap();
    assert_eq!(v2.name, "Version 2");

    let v3 = manager.get_version(3).unwrap();
    assert_eq!(v3.name, "Version 3");
}

#[test]
fn test_rollback() {
    let mut manager = ConstitutionManager::new("cons.1".to_string(), "Version 1".to_string(), None);

    manager.modify(|c| c.name = "Version 2".to_string());
    manager.modify(|c| c.name = "Version 3".to_string());

    assert_eq!(manager.current_version(), 3);

    manager.rollback(1).unwrap();
    assert_eq!(manager.current_version(), 1);
    assert_eq!(manager.current().name, "Version 1");
}

#[test]
fn test_rollback_invalid_version() {
    let mut manager = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);

    let result = manager.rollback(99);
    assert!(matches!(result, Err(ConstitutionError::InvalidVersion(99))));
}

#[test]
fn test_rollback_previous() {
    let mut manager = ConstitutionManager::new("cons.1".to_string(), "Version 1".to_string(), None);

    manager.modify(|c| c.name = "Version 2".to_string());
    manager.modify(|c| c.name = "Version 3".to_string());

    let version = manager.rollback_previous().unwrap();
    assert_eq!(version, 2);
    assert_eq!(manager.current().name, "Version 2");
}

#[test]
fn test_rollback_previous_at_version_1() {
    let mut manager = ConstitutionManager::new("cons.1".to_string(), "Test".to_string(), None);

    let result = manager.rollback_previous();
    assert!(matches!(result, Err(ConstitutionError::NoPreviousVersion)));
}
