use hudhudscript_governance::*;

#[test]
fn test_community_creation() {
    let culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    let community = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        vec!["council.1".to_string()],
        vec![],
        culture,
    )
    .unwrap();

    assert_eq!(community.id, "comm.1");
    assert_eq!(community.name, "Test Community");
    assert_eq!(community.members.len(), 2);
    assert_eq!(community.councils.len(), 1);
}

#[test]
fn test_community_duplicate_members() {
    let culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    let result = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["agent1".to_string(), "agent1".to_string()], // Duplicate
        vec![],
        vec![],
        culture,
    );

    assert!(result.is_err());
    // v0.4.47.9: CommunityError is now typed; check via Display string
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Duplicate") || err_msg.contains("duplicate"),
        "Expected duplicate-member error, got: {}",
        err_msg
    );
}

#[test]
fn test_add_member() {
    let culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    let mut community = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["agent1".to_string()],
        vec![],
        vec![],
        culture,
    )
    .unwrap();

    assert!(community.add_member("agent2".to_string()).is_ok());
    assert_eq!(community.members.len(), 2);

    // Try to add duplicate
    assert!(community.add_member("agent2".to_string()).is_err());
}

#[test]
fn test_remove_member() {
    let culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    let mut community = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        vec![],
        vec![],
        culture,
    )
    .unwrap();

    assert!(community.remove_member(&"agent1".to_string()).is_ok());
    assert_eq!(community.members.len(), 1);

    // Try to remove non-existent member
    assert!(community.remove_member(&"agent1".to_string()).is_err());
}

#[test]
fn test_is_member() {
    let culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    let community = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["agent1".to_string(), "agent2".to_string()],
        vec![],
        vec![],
        culture,
    )
    .unwrap();

    assert!(community.is_member(&"agent1".to_string()));
    assert!(community.is_member(&"agent2".to_string()));
    assert!(!community.is_member(&"agent3".to_string()));
}

#[test]
fn test_add_council() {
    let culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    let mut community = Community::new(
        "comm.1".to_string(),
        "Test Community".to_string(),
        vec!["agent1".to_string()],
        vec![],
        vec![],
        culture,
    )
    .unwrap();

    assert!(community.add_council("council.1".to_string()).is_ok());
    assert_eq!(community.councils.len(), 1);

    // Try to add duplicate
    assert!(community.add_council("council.1".to_string()).is_err());
}

#[test]
fn test_culture_management() {
    let mut culture = CommunityCulture::new(
        vec!["collaboration".to_string()],
        vec!["respect".to_string()],
        CommunicationStyle::Collaborative,
    );

    culture.add_value("transparency".to_string());
    assert_eq!(culture.values.len(), 2);

    culture.add_norm("honesty".to_string());
    assert_eq!(culture.norms.len(), 2);

    culture.set_communication_style(CommunicationStyle::Formal);
    assert_eq!(culture.communication_style, CommunicationStyle::Formal);
}

#[test]
fn test_communication_styles() {
    let styles = vec![
        CommunicationStyle::Formal,
        CommunicationStyle::Informal,
        CommunicationStyle::Technical,
        CommunicationStyle::Collaborative,
    ];

    for style in styles {
        let culture = CommunityCulture::new(vec![], vec![], style);
        assert_eq!(culture.communication_style, style);
    }
}
