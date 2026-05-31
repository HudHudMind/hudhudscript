use chrono::Utc;
use hudhudscript_cache::CommandCache;
use hudhudscript_governance::{Action, Constitution, EnforcementLevel, Law, Rule};
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

fn constitution_id_strategy() -> impl Strategy<Value = String> {
    (1u32..1000).prop_map(|n| format!("cons.{}", n))
}

fn law_id_strategy() -> impl Strategy<Value = (String, String)> {
    (1u32..100, 1u32..100).prop_map(|(n, m)| {
        let constitution_id = format!("cons.{}", n);
        let law_id = format!("cons{}.law{}", n, m);
        (constitution_id, law_id)
    })
}

fn rule_id_strategy() -> impl Strategy<Value = String> {
    (1u32..1000).prop_map(|n| format!("rule.{}", n))
}

fn constitution_strategy(id: String) -> Constitution {
    Constitution {
        id: id.clone(),
        name: format!("Constitution {}", id),
        description: Some(format!("Description for {}", id)),
        laws: HashMap::new(),
        created_at: Utc::now(),
        version: 1,
    }
}

fn law_strategy(constitution_id: String, law_id: String) -> Law {
    Law {
        id: law_id.clone(),
        constitution_id: constitution_id.clone(),
        name: format!("Law {}", law_id),
        description: format!("Description for {}", law_id),
        enforcement_level: EnforcementLevel::Mandatory,
        conditions: vec![],
    }
}

fn rule_strategy(id: String) -> Rule {
    Rule {
        id: id.clone(),
        name: format!("Rule {}", id),
        conditions: vec![],
        actions: vec![Action::Allow],
        priority: 10,
    }
}

#[test]
fn property_cache_round_trip_constitution() {
    proptest!(|(id in constitution_id_strategy())| {
        let mut cache = CommandCache::new();
        let constitution = constitution_strategy(id.clone());

        let stored_id = cache.store_constitution(constitution.clone()).unwrap();

        let resolved = cache.resolve_constitution(&stored_id);
        prop_assert!(resolved.is_some());

        let resolved = resolved.unwrap();
        prop_assert_eq!(&resolved.id, &id);
        prop_assert_eq!(&resolved.name, &constitution.name);
        prop_assert_eq!(&resolved.description, &constitution.description);
        prop_assert_eq!(resolved.version, constitution.version);
    });
}

#[test]
fn property_cache_round_trip_law() {
    proptest!(|((constitution_id, law_id) in law_id_strategy())| {
        let mut cache = CommandCache::new();
        let law = law_strategy(constitution_id.clone(), law_id.clone());

        let stored_id = cache.store_law(law.clone()).unwrap();

        let resolved = cache.resolve_law(&stored_id);
        prop_assert!(resolved.is_some());

        let resolved = resolved.unwrap();
        prop_assert_eq!(&resolved.id, &law_id);
        prop_assert_eq!(&resolved.constitution_id, &constitution_id);
        prop_assert_eq!(&resolved.name, &law.name);
        prop_assert_eq!(&resolved.description, &law.description);
    });
}

#[test]
fn property_cache_round_trip_rule() {
    proptest!(|(id in rule_id_strategy())| {
        let mut cache = CommandCache::new();
        let rule = rule_strategy(id.clone());

        let stored_id = cache.store_rule(rule.clone()).unwrap();

        let resolved = cache.resolve_rule(&stored_id);
        prop_assert!(resolved.is_some());

        let resolved = resolved.unwrap();
        prop_assert_eq!(&resolved.id, &id);
        prop_assert_eq!(&resolved.name, &rule.name);
        prop_assert_eq!(resolved.priority, rule.priority);
    });
}

#[test]
fn property_cache_serialization_round_trip() {
    proptest!(|(
        const_ids in prop::collection::vec(constitution_id_strategy(), 1..10),
        rule_ids in prop::collection::vec(rule_id_strategy(), 1..10)
    )| {
        let mut cache = CommandCache::new();

        for id in &const_ids {
            let constitution = constitution_strategy(id.clone());
            cache.store_constitution(constitution).ok();
        }

        for id in &rule_ids {
            let rule = rule_strategy(id.clone());
            cache.store_rule(rule).ok();
        }

        let json = cache.serialize_definitions().unwrap();
        let mut restored = CommandCache::deserialize_definitions(&json).unwrap();

        for id in &const_ids {
            let resolved = restored.resolve_constitution(id);
            prop_assert!(resolved.is_some(), "Constitution {} not found after round-trip", id);
        }

        for id in &rule_ids {
            let resolved = restored.resolve_rule(id);
            prop_assert!(resolved.is_some(), "Rule {} not found after round-trip", id);
        }
    });
}

#[test]
fn property_cache_serialization_preserves_content() {
    proptest!(|(id in constitution_id_strategy())| {
        let mut cache = CommandCache::new();
        let constitution = constitution_strategy(id.clone());

        cache.store_constitution(constitution.clone()).unwrap();

        let json = cache.serialize_definitions().unwrap();
        let mut restored = CommandCache::deserialize_definitions(&json).unwrap();

        let resolved = restored.resolve_constitution(&id).unwrap();
        prop_assert_eq!(&resolved.name, &constitution.name);
        prop_assert_eq!(&resolved.description, &constitution.description);
        prop_assert_eq!(resolved.version, constitution.version);
    });
}

#[test]
fn property_constitution_id_uniqueness() {
    proptest!(|(ids in prop::collection::vec(constitution_id_strategy(), 2..20))| {
        let mut cache = CommandCache::new();
        let mut stored_ids = HashSet::new();

        for id in ids {
            let constitution = constitution_strategy(id.clone());

            match cache.store_constitution(constitution) {
                Ok(stored_id) => {
                    prop_assert!(
                        stored_ids.insert(stored_id.clone()),
                        "Duplicate ID stored: {}",
                        stored_id
                    );
                }
                Err(_) => {}
            }
        }

        prop_assert_eq!(stored_ids.len(), cache.constitutions.len());
    });
}

#[test]
fn property_law_id_uniqueness() {
    proptest!(|(law_ids in prop::collection::vec(law_id_strategy(), 2..20))| {
        let mut cache = CommandCache::new();
        let mut stored_ids = HashSet::new();

        for (constitution_id, law_id) in law_ids {
            let law = law_strategy(constitution_id, law_id.clone());

            match cache.store_law(law) {
                Ok(stored_id) => {
                    prop_assert!(
                        stored_ids.insert(stored_id.clone()),
                        "Duplicate ID stored: {}",
                        stored_id
                    );
                }
                Err(_) => {}
            }
        }

        prop_assert_eq!(stored_ids.len(), cache.laws.len());
    });
}

#[test]
fn property_rule_id_uniqueness() {
    proptest!(|(ids in prop::collection::vec(rule_id_strategy(), 2..20))| {
        let mut cache = CommandCache::new();
        let mut stored_ids = HashSet::new();

        for id in ids {
            let rule = rule_strategy(id.clone());

            match cache.store_rule(rule) {
                Ok(stored_id) => {
                    prop_assert!(
                        stored_ids.insert(stored_id.clone()),
                        "Duplicate ID stored: {}",
                        stored_id
                    );
                }
                Err(_) => {}
            }
        }

        prop_assert_eq!(stored_ids.len(), cache.rules.len());
    });
}

#[test]
fn property_duplicate_id_rejected() {
    proptest!(|(id in constitution_id_strategy())| {
        let mut cache = CommandCache::new();
        let constitution1 = constitution_strategy(id.clone());
        let constitution2 = constitution_strategy(id.clone());

        let result1 = cache.store_constitution(constitution1);
        prop_assert!(result1.is_ok());

        let result2 = cache.store_constitution(constitution2);
        prop_assert!(result2.is_err());
    });
}

#[test]
fn property_all_stored_ids_are_unique() {
    proptest!(|(
        const_ids in prop::collection::vec(constitution_id_strategy(), 1..10),
        law_ids in prop::collection::vec(law_id_strategy(), 1..10),
        rule_ids in prop::collection::vec(rule_id_strategy(), 1..10)
    )| {
        let mut cache = CommandCache::new();

        for id in &const_ids {
            let constitution = constitution_strategy(id.clone());
            cache.store_constitution(constitution).ok();
        }

        for (constitution_id, law_id) in &law_ids {
            let law = law_strategy(constitution_id.clone(), law_id.clone());
            cache.store_law(law).ok();
        }

        for id in &rule_ids {
            let rule = rule_strategy(id.clone());
            cache.store_rule(rule).ok();
        }

        let mut all_const_ids: Vec<_> = cache.constitutions.keys().collect();
        let mut all_law_ids: Vec<_> = cache.laws.keys().collect();
        let mut all_rule_ids: Vec<_> = cache.rules.keys().collect();

        all_const_ids.sort();
        all_law_ids.sort();
        all_rule_ids.sort();

        for i in 1..all_const_ids.len() {
            prop_assert_ne!(all_const_ids[i-1], all_const_ids[i]);
        }

        for i in 1..all_law_ids.len() {
            prop_assert_ne!(all_law_ids[i-1], all_law_ids[i]);
        }

        for i in 1..all_rule_ids.len() {
            prop_assert_ne!(all_rule_ids[i-1], all_rule_ids[i]);
        }
    });
}

#[test]
fn property_resolve_returns_none_for_nonexistent() {
    proptest!(|(id in constitution_id_strategy())| {
        let mut cache = CommandCache::new();

        let resolved = cache.resolve_constitution(&id);
        prop_assert!(resolved.is_none());
    });
}

#[test]
fn property_cache_size_increases_on_store() {
    proptest!(|(ids in prop::collection::vec(constitution_id_strategy(), 1..10))| {
        let mut cache = CommandCache::new();
        let initial_size = cache.constitutions.len();

        let mut expected_size = initial_size;
        for id in ids {
            let constitution = constitution_strategy(id);
            if cache.store_constitution(constitution).is_ok() {
                expected_size += 1;
            }
            prop_assert_eq!(cache.constitutions.len(), expected_size);
        }
    });
}

#[test]
fn property_serialization_is_deterministic() {
    proptest!(|(id in constitution_id_strategy())| {
        let mut cache = CommandCache::new();
        let constitution = constitution_strategy(id);

        cache.store_constitution(constitution).unwrap();

        let json1 = cache.serialize_definitions().unwrap();
        let json2 = cache.serialize_definitions().unwrap();

        prop_assert_eq!(json1, json2);
    });
}
