#[cfg(test)]
mod tests {
    use hudhud_sop::sop_ops::*;

    #[test]
    fn test_all_methods_present() {
        let required = vec!["serialize".to_string(), "deserialize".to_string()];
        let class_methods = vec![
            "serialize".to_string(),
            "deserialize".to_string(),
            "extra".to_string(),
        ];
        assert!(check_trait_implementation("Foo", "Bar", &required, &class_methods).is_ok());
    }

    #[test]
    fn test_missing_methods() {
        let required = vec!["serialize".to_string(), "deserialize".to_string()];
        let class_methods = vec!["serialize".to_string()];
        let result = check_trait_implementation("Foo", "Bar", &required, &class_methods);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["deserialize".to_string()]);
    }

    #[test]
    fn test_empty_trait() {
        let required: Vec<String> = vec![];
        let class_methods = vec!["anything".to_string()];
        assert!(check_trait_implementation("Foo", "Bar", &required, &class_methods).is_ok());
    }

    #[test]
    fn test_error_message_format() {
        let msg =
            trait_not_implemented_error("Dog", "Animal", &["speak".to_string(), "eat".to_string()]);
        assert!(msg.contains("Dog"));
        assert!(msg.contains("Animal"));
        assert!(msg.contains("speak"));
        assert!(msg.contains("eat"));
    }
}
