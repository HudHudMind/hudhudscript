//! Tests extracted from hudhudscript-modules/src/module.rs

use hudhudscript_modules::module::{Export, ExportKind, Module, ModuleValue};
use std::path::PathBuf;

#[test]
fn test_module_creation() {
    let module = Module::new(
        PathBuf::from("test.hudhud"),
        "let x = 1".to_string(),
        vec![],
    );

    assert_eq!(module.id, PathBuf::from("test.hudhud"));
    assert!(!module.executed);
    assert!(module.exports.is_empty());
    assert!(module.imports.is_empty());
}

#[test]
fn test_module_exports() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    module.add_export(
        "foo".to_string(),
        Export {
            kind: ExportKind::Named(vec!["foo".to_string()]),
            source: None,
        },
    );

    assert_eq!(module.export_names(), vec!["foo"]);
    assert!(module.get_export("foo").is_some());
}

#[test]
fn test_module_imports() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    module.add_import("./other.hudhud".to_string(), vec!["bar".to_string()]);

    assert_eq!(module.import_paths(), vec!["./other.hudhud"]);
}

#[test]
fn test_module_execution() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    assert!(!module.executed);
    module.mark_executed();
    assert!(module.executed);
}

#[test]
fn test_module_exported_values() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    module.add_exported_value(
        "greet".to_string(),
        ModuleValue::Function {
            name: "greet".to_string(),
            params: vec!["name".to_string()],
            body: vec![],
        },
    );

    let val = module.get_exported_value("greet");
    assert!(val.is_some());
    match val.unwrap() {
        ModuleValue::Function { name, params, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(params, &["name".to_string()]);
        }
        _ => panic!("Expected Function variant"),
    }

    assert!(module.get_exported_value("nonexistent").is_none());
}

#[test]
fn test_module_value_variable() {
    let val = ModuleValue::Variable {
        name: "x".to_string(),
        value: "42".to_string(),
    };
    match &val {
        ModuleValue::Variable { name, value } => {
            assert_eq!(name, "x");
            assert_eq!(value, "42");
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_module_value_agent() {
    let val = ModuleValue::Agent {
        name: "MyAgent".to_string(),
        decl: "{...}".to_string(),
    };
    match &val {
        ModuleValue::Agent { name, decl } => {
            assert_eq!(name, "MyAgent");
            assert_eq!(decl, "{...}");
        }
        _ => panic!("Expected Agent"),
    }
}

#[test]
fn test_module_value_tool() {
    let val = ModuleValue::Tool {
        name: "search".to_string(),
        decl: "{}".to_string(),
    };
    match &val {
        ModuleValue::Tool { name, decl } => {
            assert_eq!(name, "search");
            assert_eq!(decl, "{}");
        }
        _ => panic!("Expected Tool"),
    }
}

#[test]
fn test_export_kind_variants() {
    let named = ExportKind::Named(vec!["foo".to_string(), "bar".to_string()]);
    assert_eq!(
        named,
        ExportKind::Named(vec!["foo".to_string(), "bar".to_string()])
    );

    let default = ExportKind::Default("main".to_string());
    assert_eq!(default, ExportKind::Default("main".to_string()));

    let wildcard = ExportKind::Wildcard;
    assert_eq!(wildcard, ExportKind::Wildcard);
}

#[test]
fn test_export_with_source() {
    let export = Export {
        kind: ExportKind::Wildcard,
        source: Some("./other".to_string()),
    };
    assert_eq!(export.source, Some("./other".to_string()));
    assert_eq!(export.kind, ExportKind::Wildcard);
}

#[test]
fn test_module_get_export_not_found() {
    let module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);
    assert!(module.get_export("does_not_exist").is_none());
}

#[test]
fn test_module_source_preserved() {
    let src = "let x = 42\nprint(x)".to_string();
    let module = Module::new(PathBuf::from("test.hudhud"), src.clone(), vec![]);
    assert_eq!(module.source, src);
}

#[test]
fn test_module_multiple_imports() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);
    module.add_import("./a".to_string(), vec!["foo".to_string()]);
    module.add_import(
        "./b".to_string(),
        vec!["bar".to_string(), "baz".to_string()],
    );

    let paths = module.import_paths();
    assert_eq!(paths.len(), 2);
    assert!(paths.contains(&"./a".to_string()));
    assert!(paths.contains(&"./b".to_string()));
}

#[test]
fn test_export_kind_serialization() {
    let named = ExportKind::Named(vec!["foo".to_string()]);
    let json = serde_json::to_string(&named).unwrap();
    let deserialized: ExportKind = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, named);

    let default_export = ExportKind::Default("main".to_string());
    let json = serde_json::to_string(&default_export).unwrap();
    let deserialized: ExportKind = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, default_export);

    let wildcard = ExportKind::Wildcard;
    let json = serde_json::to_string(&wildcard).unwrap();
    let deserialized: ExportKind = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, wildcard);
}

#[test]
fn test_module_value_serialization() {
    let val = ModuleValue::Variable {
        name: "x".to_string(),
        value: "hello".to_string(),
    };
    let json = serde_json::to_string(&val).unwrap();
    let deserialized: ModuleValue = serde_json::from_str(&json).unwrap();
    match deserialized {
        ModuleValue::Variable { name, value } => {
            assert_eq!(name, "x");
            assert_eq!(value, "hello");
        }
        _ => panic!("Expected Variable"),
    }
}

#[test]
fn test_module_add_multiple_exports() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    module.add_export(
        "foo".to_string(),
        Export {
            kind: ExportKind::Named(vec!["foo".to_string()]),
            source: None,
        },
    );
    module.add_export(
        "bar".to_string(),
        Export {
            kind: ExportKind::Default("bar".to_string()),
            source: None,
        },
    );
    module.add_export(
        "all".to_string(),
        Export {
            kind: ExportKind::Wildcard,
            source: Some("./other".to_string()),
        },
    );

    assert_eq!(module.export_names().len(), 3);
    assert!(module.get_export("foo").is_some());
    assert!(module.get_export("bar").is_some());
    assert!(module.get_export("all").is_some());
    assert!(module.get_export("nonexistent").is_none());
}

#[test]
fn test_module_export_override() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    module.add_export(
        "foo".to_string(),
        Export {
            kind: ExportKind::Named(vec!["foo".to_string()]),
            source: None,
        },
    );
    // Override with different kind
    module.add_export(
        "foo".to_string(),
        Export {
            kind: ExportKind::Default("foo".to_string()),
            source: None,
        },
    );

    assert_eq!(module.export_names().len(), 1);
    let export = module.get_export("foo").unwrap();
    assert_eq!(export.kind, ExportKind::Default("foo".to_string()));
}

#[test]
fn test_module_multiple_exported_values() {
    let mut module = Module::new(PathBuf::from("test.hudhud"), "".to_string(), vec![]);

    module.add_exported_value(
        "greet".to_string(),
        ModuleValue::Function {
            name: "greet".to_string(),
            params: vec!["name".to_string()],
            body: vec![],
        },
    );
    module.add_exported_value(
        "PI".to_string(),
        ModuleValue::Variable {
            name: "PI".to_string(),
            value: "3.14159".to_string(),
        },
    );
    module.add_exported_value(
        "Helper".to_string(),
        ModuleValue::Agent {
            name: "Helper".to_string(),
            decl: "{}".to_string(),
        },
    );
    module.add_exported_value(
        "search".to_string(),
        ModuleValue::Tool {
            name: "search".to_string(),
            decl: "{}".to_string(),
        },
    );

    assert!(module.get_exported_value("greet").is_some());
    assert!(module.get_exported_value("PI").is_some());
    assert!(module.get_exported_value("Helper").is_some());
    assert!(module.get_exported_value("search").is_some());
    assert!(module.get_exported_value("missing").is_none());
}

#[test]
fn test_module_function_serialization() {
    let val = ModuleValue::Function {
        name: "add".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        body: vec![],
    };
    let json = serde_json::to_string(&val).unwrap();
    let deserialized: ModuleValue = serde_json::from_str(&json).unwrap();
    match deserialized {
        ModuleValue::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params, vec!["a", "b"]);
        }
        _ => panic!("Expected Function"),
    }
}

#[test]
fn test_module_agent_serialization() {
    let val = ModuleValue::Agent {
        name: "MyAgent".to_string(),
        decl: "{\"model\":\"gpt-4\"}".to_string(),
    };
    let json = serde_json::to_string(&val).unwrap();
    let deserialized: ModuleValue = serde_json::from_str(&json).unwrap();
    match deserialized {
        ModuleValue::Agent { name, decl } => {
            assert_eq!(name, "MyAgent");
            assert!(decl.contains("gpt-4"));
        }
        _ => panic!("Expected Agent"),
    }
}

#[test]
fn test_module_tool_serialization() {
    let val = ModuleValue::Tool {
        name: "fetch".to_string(),
        decl: "{}".to_string(),
    };
    let json = serde_json::to_string(&val).unwrap();
    let deserialized: ModuleValue = serde_json::from_str(&json).unwrap();
    match deserialized {
        ModuleValue::Tool { name, decl } => {
            assert_eq!(name, "fetch");
            assert_eq!(decl, "{}");
        }
        _ => panic!("Expected Tool"),
    }
}

#[test]
fn test_export_serialization_roundtrip() {
    let export = Export {
        kind: ExportKind::Named(vec!["a".to_string(), "b".to_string()]),
        source: Some("./lib".to_string()),
    };
    let json = serde_json::to_string(&export).unwrap();
    let deserialized: Export = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.kind,
        ExportKind::Named(vec!["a".to_string(), "b".to_string()])
    );
    assert_eq!(deserialized.source, Some("./lib".to_string()));
}

#[test]
fn test_module_empty_exports_and_imports() {
    let module = Module::new(PathBuf::from("empty.hudhud"), "".to_string(), vec![]);
    assert!(module.export_names().is_empty());
    assert!(module.import_paths().is_empty());
    assert!(module.exported_values.is_empty());
    assert!(!module.executed);
}
