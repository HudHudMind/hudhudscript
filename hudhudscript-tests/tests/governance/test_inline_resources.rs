use hudhudscript_governance::resources::{AllocationPolicy, Resource, ResourceManager};

#[test]
fn test_resource_creation() {
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        Some("CPU cores".to_string()),
        100,
        AllocationPolicy::FairShare,
    );

    assert_eq!(resource.id, "res.1");
    assert_eq!(resource.capacity, 100);
    assert_eq!(resource.available, 100);
}

#[test]
fn test_resource_allocation() {
    let mut resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    assert!(resource.allocate(50).is_ok());
    assert_eq!(resource.available, 50);

    assert!(resource.allocate(60).is_err()); // Insufficient
    assert_eq!(resource.available, 50); // Unchanged
}

#[test]
fn test_resource_release() {
    let mut resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    resource.allocate(50).unwrap();
    assert_eq!(resource.available, 50);

    assert!(resource.release(30).is_ok());
    assert_eq!(resource.available, 80);

    assert!(resource.release(30).is_err()); // Would exceed capacity
}

#[test]
fn test_resource_manager_registration() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    assert!(manager.register_resource(resource.clone()).is_ok());
    assert!(manager.register_resource(resource).is_err()); // Duplicate
}

#[test]
fn test_access_control() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    manager.register_resource(resource).unwrap();

    assert!(manager
        .grant_access(&"res.1".to_string(), "agent1".to_string())
        .is_ok());
    assert!(manager.has_access(&"res.1".to_string(), &"agent1".to_string()));
    assert!(!manager.has_access(&"res.1".to_string(), &"agent2".to_string()));

    assert!(manager
        .revoke_access(&"res.1".to_string(), &"agent1".to_string())
        .is_ok());
    assert!(!manager.has_access(&"res.1".to_string(), &"agent1".to_string()));
}

#[test]
fn test_resource_allocation_with_access_control() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    manager.register_resource(resource).unwrap();
    manager
        .grant_access(&"res.1".to_string(), "agent1".to_string())
        .unwrap();

    // Agent with access can allocate
    assert!(manager
        .allocate_resource(&"res.1".to_string(), &"agent1".to_string(), 50)
        .is_ok());

    // Agent without access cannot allocate
    assert!(manager
        .allocate_resource(&"res.1".to_string(), &"agent2".to_string(), 10)
        .is_err());
}

#[test]
fn test_usage_tracking() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    manager.register_resource(resource).unwrap();
    manager
        .grant_access(&"res.1".to_string(), "agent1".to_string())
        .unwrap();

    manager
        .allocate_resource(&"res.1".to_string(), &"agent1".to_string(), 30)
        .unwrap();
    manager
        .allocate_resource(&"res.1".to_string(), &"agent1".to_string(), 20)
        .unwrap();

    let usage = manager.get_agent_usage(&"agent1".to_string());
    assert_eq!(usage.len(), 2);

    let total = manager.get_total_usage(&"res.1".to_string(), &"agent1".to_string());
    assert_eq!(total, 50);
}

#[test]
fn test_resource_validation() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );

    manager.register_resource(resource).unwrap();

    assert!(manager.validate_resources(&["res.1".to_string()]).is_ok());
    assert!(manager
        .validate_resources(&["res.1".to_string(), "res.2".to_string()])
        .is_err());
}

#[test]
fn test_resource_is_available() {
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );
    assert!(resource.is_available(50));
    assert!(resource.is_available(100));
    assert!(!resource.is_available(101));
}

#[test]
fn test_grant_access_nonexistent_resource() {
    let mut manager = ResourceManager::new();
    let result = manager.grant_access(&"nonexistent".to_string(), "agent1".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_grant_access_duplicate() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );
    manager.register_resource(resource).unwrap();
    manager
        .grant_access(&"res.1".to_string(), "agent1".to_string())
        .unwrap();
    let result = manager.grant_access(&"res.1".to_string(), "agent1".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already has access"));
}

#[test]
fn test_revoke_access_nonexistent_resource() {
    let mut manager = ResourceManager::new();
    let result = manager.revoke_access(&"nonexistent".to_string(), &"agent1".to_string());
    assert!(result.is_err());
}

#[test]
fn test_revoke_access_agent_without_access() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );
    manager.register_resource(resource).unwrap();
    manager
        .grant_access(&"res.1".to_string(), "agent1".to_string())
        .unwrap();
    let result = manager.revoke_access(&"res.1".to_string(), &"agent2".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("doesn't have access"));
}

#[test]
fn test_release_resource() {
    let mut manager = ResourceManager::new();
    let resource = Resource::new(
        "res.1".to_string(),
        "CPU".to_string(),
        None,
        100,
        AllocationPolicy::FairShare,
    );
    manager.register_resource(resource).unwrap();
    manager
        .grant_access(&"res.1".to_string(), "agent1".to_string())
        .unwrap();
    manager
        .allocate_resource(&"res.1".to_string(), &"agent1".to_string(), 50)
        .unwrap();
    manager
        .release_resource(&"res.1".to_string(), &"agent1".to_string(), 30)
        .unwrap();

    let usage = manager.get_resource_usage(&"res.1".to_string());
    // allocate + release = 2 records
    assert_eq!(usage.len(), 2);
}

#[test]
fn test_release_resource_nonexistent() {
    let mut manager = ResourceManager::new();
    let result = manager.release_resource(&"nonexistent".to_string(), &"agent1".to_string(), 10);
    assert!(result.is_err());
}

#[test]
fn test_get_agent_usage_empty() {
    let manager = ResourceManager::new();
    let usage = manager.get_agent_usage(&"agent1".to_string());
    assert!(usage.is_empty());
}

#[test]
fn test_get_resource_usage_empty() {
    let manager = ResourceManager::new();
    let usage = manager.get_resource_usage(&"res.1".to_string());
    assert!(usage.is_empty());
}

#[test]
fn test_get_total_usage_zero() {
    let manager = ResourceManager::new();
    let total = manager.get_total_usage(&"res.1".to_string(), &"agent1".to_string());
    assert_eq!(total, 0);
}

#[test]
fn test_resource_manager_default() {
    let manager = ResourceManager::default();
    assert!(manager.validate_resources(&[]).is_ok());
}

#[test]
fn test_allocation_policies() {
    let policies = vec![
        AllocationPolicy::FirstComeFirstServed,
        AllocationPolicy::PriorityBased,
        AllocationPolicy::FairShare,
    ];

    for policy in policies {
        let resource = Resource::new("res.1".to_string(), "CPU".to_string(), None, 100, policy);
        assert_eq!(resource.allocation_policy, policy);
    }
}
