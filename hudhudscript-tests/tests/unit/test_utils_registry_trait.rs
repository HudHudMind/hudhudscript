use hudhudscript_utils::registry_trait::Registry;
use std::collections::HashMap;

// Simple test implementation
struct SimpleRegistry {
    items: HashMap<String, String>,
}

impl SimpleRegistry {
    fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}

impl Registry<String, String> for SimpleRegistry {
    type Error = String;

    fn register(&mut self, key: String, value: String) -> Result<(), String> {
        self.items.insert(key, value);
        Ok(())
    }

    fn unregister(&mut self, key: &String) -> Result<Option<String>, String> {
        Ok(self.items.remove(key))
    }

    fn get(&self, key: &String) -> Option<&String> {
        self.items.get(key)
    }

    fn keys(&self) -> Vec<&String> {
        self.items.keys().collect()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

#[test]
fn test_registry_basic() {
    let mut reg = SimpleRegistry::new();
    assert!(reg.is_empty());

    reg.register("foo".to_string(), "bar".to_string()).unwrap();
    assert_eq!(reg.len(), 1);
    assert!(reg.contains(&"foo".to_string()));
    assert_eq!(reg.get(&"foo".to_string()), Some(&"bar".to_string()));
}

#[test]
fn test_registry_unregister() {
    let mut reg = SimpleRegistry::new();
    reg.register("x".to_string(), "y".to_string()).unwrap();

    let removed = reg.unregister(&"x".to_string()).unwrap();
    assert_eq!(removed, Some("y".to_string()));
    assert!(reg.is_empty());
}

#[test]
fn test_registry_unregister_missing() {
    let mut reg = SimpleRegistry::new();
    let removed = reg.unregister(&"nope".to_string()).unwrap();
    assert_eq!(removed, None);
}
