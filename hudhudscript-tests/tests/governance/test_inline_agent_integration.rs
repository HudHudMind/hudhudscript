use chrono::Utc;
use hudhudscript_governance::agent_integration::{
    validate_community_agents, validate_council_agents, validate_swarm_agents,
    AgentMembershipTracker, AgentRegistry,
};
use hudhudscript_governance::*;
use std::collections::HashMap;

#[test]
fn test_agent_registry() {
    let mut registry = AgentRegistry::new();

    assert!(registry.register_agent("agent1".to_string()).is_ok());
    assert!(registry.agent_exists(&"agent1".to_string()));
    assert!(!registry.agent_exists(&"agent2".to_string()));

    assert!(registry.register_agent("agent1".to_string()).is_err()); // Duplicate
}

#[test]
fn test_agent_validation() {
    let mut registry = AgentRegistry::new();
    registry.register_agent("agent1".to_string()).unwrap();
    registry.register_agent("agent2".to_string()).unwrap();

    let valid_agents = vec!["agent1".to_string(), "agent2".to_string()];
    assert!(registry.validate_agents(&valid_agents).is_ok());

    let invalid_agents = vec!["agent1".to_string(), "agent3".to_string()];
    assert!(registry.validate_agents(&invalid_agents).is_err());
}

#[test]
fn test_council_validation() {
    let mut registry = AgentRegistry::new();
    registry.register_agent("agent1".to_string()).unwrap();
    registry.register_agent("agent2".to_string()).unwrap();

    let council = Council {
        id: "council.1".to_string(),
        name: "Test Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![
            AgentMember {
                agent_id: "agent1".to_string(),
                role: AgentRole::Member,
                joined_at: Utc::now(),
                permissions: vec![],
            },
            AgentMember {
                agent_id: "agent2".to_string(),
                role: AgentRole::Member,
                joined_at: Utc::now(),
                permissions: vec![],
            },
        ],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    assert!(validate_council_agents(&council, &registry).is_ok());

    let invalid_council = Council {
        id: "council.2".to_string(),
        name: "Invalid Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![
            AgentMember {
                agent_id: "agent1".to_string(),
                role: AgentRole::Member,
                joined_at: Utc::now(),
                permissions: vec![],
            },
            AgentMember {
                agent_id: "agent3".to_string(), // Not registered
                role: AgentRole::Member,
                joined_at: Utc::now(),
                permissions: vec![],
            },
        ],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    assert!(validate_council_agents(&invalid_council, &registry).is_err());
}

#[test]
fn test_swarm_validation() {
    let mut registry = AgentRegistry::new();
    registry.register_agent("agent1".to_string()).unwrap();
    registry.register_agent("agent2".to_string()).unwrap();

    let swarm = Swarm {
        id: "swarm.1".to_string(),
        name: "Test Swarm".to_string(),
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        coordination_strategy: CoordinationStrategy::Parallel,
        shared_state: Default::default(),
    };

    assert!(validate_swarm_agents(&swarm, &registry).is_ok());

    let invalid_swarm = Swarm {
        id: "swarm.2".to_string(),
        name: "Invalid Swarm".to_string(),
        agents: vec!["agent1".to_string(), "agent3".to_string()],
        coordination_strategy: CoordinationStrategy::Parallel,
        shared_state: Default::default(),
    };

    assert!(validate_swarm_agents(&invalid_swarm, &registry).is_err());
}

#[test]
fn test_community_validation() {
    let mut registry = AgentRegistry::new();
    registry.register_agent("agent1".to_string()).unwrap();
    registry.register_agent("agent2".to_string()).unwrap();

    let culture = CommunityCulture {
        values: vec!["collaboration".to_string()],
        norms: vec!["respect".to_string()],
        communication_style: CommunicationStyle::Collaborative,
    };

    let community = Community {
        id: "comm.1".to_string(),
        name: "Test Community".to_string(),
        members: vec!["agent1".to_string(), "agent2".to_string()],
        councils: vec![],
        shared_resources: vec![],
        culture,
    };

    assert!(validate_community_agents(&community, &registry).is_ok());
}

#[test]
fn test_membership_tracker() {
    let mut tracker = AgentMembershipTracker::new();

    let council = Council {
        id: "council.1".to_string(),
        name: "Test Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![AgentMember {
            agent_id: "agent1".to_string(),
            role: AgentRole::Member,
            joined_at: Utc::now(),
            permissions: vec![],
        }],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    tracker.track_council(&council);

    let councils = tracker.get_agent_councils(&"agent1".to_string());
    assert_eq!(councils.len(), 1);
    assert_eq!(councils[0], "council.1");
}

#[test]
fn test_agent_role_query() {
    let mut tracker = AgentMembershipTracker::new();

    let council = Council {
        id: "council.1".to_string(),
        name: "Test Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![AgentMember {
            agent_id: "agent1".to_string(),
            role: AgentRole::Judge,
            joined_at: Utc::now(),
            permissions: vec![],
        }],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    tracker.track_council(&council);

    let role = tracker.get_agent_role_in_council(
        &"agent1".to_string(),
        &"council.1".to_string(),
        &council,
    );
    assert_eq!(role, Some("Judge".to_string()));

    let no_role = tracker.get_agent_role_in_council(
        &"agent2".to_string(),
        &"council.1".to_string(),
        &council,
    );
    assert_eq!(no_role, None);
}

#[test]
fn test_unregister_agent() {
    let mut registry = AgentRegistry::new();
    registry.register_agent("agent1".to_string()).unwrap();
    assert!(registry.agent_exists(&"agent1".to_string()));

    registry.unregister_agent(&"agent1".to_string()).unwrap();
    assert!(!registry.agent_exists(&"agent1".to_string()));
}

#[test]
fn test_unregister_nonexistent_agent() {
    let mut registry = AgentRegistry::new();
    let result = registry.unregister_agent(&"agent1".to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_get_all_agents() {
    let mut registry = AgentRegistry::new();
    registry.register_agent("agent1".to_string()).unwrap();
    registry.register_agent("agent2".to_string()).unwrap();

    let all = registry.get_all_agents();
    assert_eq!(all.len(), 2);
    assert!(all.contains(&"agent1".to_string()));
    assert!(all.contains(&"agent2".to_string()));
}

#[test]
fn test_get_all_agents_empty() {
    let registry = AgentRegistry::new();
    assert!(registry.get_all_agents().is_empty());
}

#[test]
fn test_tracker_swarm() {
    let mut tracker = AgentMembershipTracker::new();

    let swarm = Swarm {
        id: "swarm.1".to_string(),
        name: "Test Swarm".to_string(),
        agents: vec!["agent1".to_string(), "agent2".to_string()],
        coordination_strategy: CoordinationStrategy::Parallel,
        shared_state: Default::default(),
    };

    tracker.track_swarm(&swarm);

    let swarms = tracker.get_agent_swarms(&"agent1".to_string());
    assert_eq!(swarms.len(), 1);
    assert_eq!(swarms[0], "swarm.1");

    let swarms2 = tracker.get_agent_swarms(&"agent2".to_string());
    assert_eq!(swarms2.len(), 1);

    assert!(tracker.is_member_of_any(&"agent1".to_string()));
}

#[test]
fn test_tracker_community() {
    let mut tracker = AgentMembershipTracker::new();

    let culture = CommunityCulture {
        values: vec!["collab".to_string()],
        norms: vec!["respect".to_string()],
        communication_style: CommunicationStyle::Collaborative,
    };

    let community = Community {
        id: "comm.1".to_string(),
        name: "Test Community".to_string(),
        members: vec!["agent1".to_string()],
        councils: vec![],
        shared_resources: vec![],
        culture,
    };

    tracker.track_community(&community);

    let communities = tracker.get_agent_communities(&"agent1".to_string());
    assert_eq!(communities.len(), 1);
    assert_eq!(communities[0], "comm.1");

    assert!(tracker.is_member_of_any(&"agent1".to_string()));
}

#[test]
fn test_tracker_empty_queries() {
    let tracker = AgentMembershipTracker::new();
    assert!(tracker.get_agent_councils(&"nobody".to_string()).is_empty());
    assert!(tracker.get_agent_swarms(&"nobody".to_string()).is_empty());
    assert!(tracker
        .get_agent_communities(&"nobody".to_string())
        .is_empty());
}

#[test]
fn test_get_agent_role_wrong_council() {
    let tracker = AgentMembershipTracker::new();
    let council = Council {
        id: "council.1".to_string(),
        name: "Test Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![AgentMember {
            agent_id: "agent1".to_string(),
            role: AgentRole::Judge,
            joined_at: Utc::now(),
            permissions: vec![],
        }],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    // Query with wrong council ID
    let role = tracker.get_agent_role_in_council(
        &"agent1".to_string(),
        &"council.999".to_string(),
        &council,
    );
    assert!(role.is_none());
}

#[test]
fn test_is_member_of_any() {
    let mut tracker = AgentMembershipTracker::new();

    assert!(!tracker.is_member_of_any(&"agent1".to_string()));

    let council = Council {
        id: "council.1".to_string(),
        name: "Test Council".to_string(),
        constitution_id: "cons.1".to_string(),
        members: vec![AgentMember {
            agent_id: "agent1".to_string(),
            role: AgentRole::Member,
            joined_at: Utc::now(),
            permissions: vec![],
        }],
        rules: vec![],
        state: CouncilState {
            active: true,
            metadata: HashMap::new(),
        },
        governance_model: None,
    };

    tracker.track_council(&council);

    assert!(tracker.is_member_of_any(&"agent1".to_string()));
    assert!(!tracker.is_member_of_any(&"agent2".to_string()));
}
