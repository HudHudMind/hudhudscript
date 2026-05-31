//! Public API tests for hudhudscript-package::config —
//! HudhudConfig, PackageConfig, DependencySpec, McpServerSpec, NativeDependencySpec,
//! AiProviderConfig, PackageType, TOML parsing, load/save.

use hudhudscript_package::config::{
    AiProviderConfig, DependencySpec, HudhudConfig, McpServerSpec, NativeDependencySpec,
    PackageConfig, PackageType,
};
use std::collections::HashMap;

// ── PackageType ───────────────────────────────────────────────────────────────

#[test]
fn package_type_default_is_application() {
    assert_eq!(PackageType::default(), PackageType::Application);
}

#[test]
fn package_type_application_variant_exists() {
    let pt = PackageType::Application;
    assert_eq!(pt, PackageType::Application);
}

#[test]
fn package_type_library_variant_exists() {
    let pt = PackageType::Library;
    assert_eq!(pt, PackageType::Library);
}

#[test]
fn package_type_application_ne_library() {
    assert_ne!(PackageType::Application, PackageType::Library);
}

#[test]
fn package_type_serde_application_json() {
    let json = serde_json::to_string(&PackageType::Application).unwrap();
    assert_eq!(json, "\"application\"");
}

#[test]
fn package_type_serde_library_json() {
    let json = serde_json::to_string(&PackageType::Library).unwrap();
    assert_eq!(json, "\"library\"");
}

#[test]
fn package_type_deserialize_application() {
    let pt: PackageType = serde_json::from_str("\"application\"").unwrap();
    assert_eq!(pt, PackageType::Application);
}

#[test]
fn package_type_deserialize_library() {
    let pt: PackageType = serde_json::from_str("\"library\"").unwrap();
    assert_eq!(pt, PackageType::Library);
}

#[test]
fn package_type_clone() {
    let pt = PackageType::Library;
    assert_eq!(pt.clone(), PackageType::Library);
}

// ── PackageConfig defaults ────────────────────────────────────────────────────

#[test]
fn package_config_default_name() {
    assert_eq!(PackageConfig::default().name, "my-project");
}

#[test]
fn package_config_default_version() {
    assert_eq!(PackageConfig::default().version, "0.1.0");
}

#[test]
fn package_config_default_type_is_application() {
    assert_eq!(
        PackageConfig::default().package_type,
        PackageType::Application
    );
}

#[test]
fn package_config_default_entry_is_none() {
    assert!(PackageConfig::default().entry.is_none());
}

#[test]
fn package_config_default_description_is_empty() {
    assert_eq!(PackageConfig::default().description, "");
}

#[test]
fn package_config_default_license_is_mit() {
    assert_eq!(PackageConfig::default().license, "MIT");
}

#[test]
fn package_config_default_authors_is_empty() {
    assert!(PackageConfig::default().authors.is_empty());
}

#[test]
fn package_config_default_keywords_is_empty() {
    assert!(PackageConfig::default().keywords.is_empty());
}

#[test]
fn package_config_default_categories_is_empty() {
    assert!(PackageConfig::default().categories.is_empty());
}

#[test]
fn package_config_default_repository_is_empty() {
    assert_eq!(PackageConfig::default().repository, "");
}

#[test]
fn package_config_default_homepage_is_empty() {
    assert_eq!(PackageConfig::default().homepage, "");
}

// ── HudhudConfig defaults ─────────────────────────────────────────────────────

#[test]
fn hudhud_config_default_package_name() {
    let config = HudhudConfig::default();
    assert_eq!(config.package.name, "my-project");
}

#[test]
fn hudhud_config_default_package_version() {
    let config = HudhudConfig::default();
    assert_eq!(config.package.version, "0.1.0");
}

#[test]
fn hudhud_config_default_dependencies_empty() {
    assert!(HudhudConfig::default().dependencies.is_empty());
}

#[test]
fn hudhud_config_default_dev_dependencies_empty() {
    assert!(HudhudConfig::default().dev_dependencies.is_empty());
}

#[test]
fn hudhud_config_default_native_dependencies_empty() {
    assert!(HudhudConfig::default().native_dependencies.is_empty());
}

#[test]
fn hudhud_config_default_mcp_servers_empty() {
    assert!(HudhudConfig::default().mcp_servers.is_empty());
}

#[test]
fn hudhud_config_default_ai_providers_empty() {
    assert!(HudhudConfig::default().ai_providers.is_empty());
}

#[test]
fn hudhud_config_default_is_application() {
    assert!(HudhudConfig::default().is_application());
}

#[test]
fn hudhud_config_default_not_is_library() {
    assert!(!HudhudConfig::default().is_library());
}

#[test]
fn hudhud_config_cache_dir_ends_with_hudhudscript() {
    let config = HudhudConfig::default();
    let s = config.cache_dir.to_string_lossy();
    assert!(s.ends_with("hudhudscript"));
}

// ── HudhudConfig::is_application / is_library ────────────────────────────────

#[test]
fn is_application_true_for_application_type() {
    let mut config = HudhudConfig::default();
    config.package.package_type = PackageType::Application;
    assert!(config.is_application());
    assert!(!config.is_library());
}

#[test]
fn is_library_true_for_library_type() {
    let mut config = HudhudConfig::default();
    config.package.package_type = PackageType::Library;
    assert!(config.is_library());
    assert!(!config.is_application());
}

// ── TOML parsing — minimal manifest ──────────────────────────────────────────

#[test]
fn parse_minimal_manifest() {
    let toml = r#"
[package]
name = "bare"
version = "0.0.1"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.package.name, "bare");
    assert_eq!(config.package.version, "0.0.1");
    assert!(config.dependencies.is_empty());
    assert!(config.is_application());
}

#[test]
fn parse_library_manifest() {
    let toml = r#"
[package]
name = "my-lib"
version = "0.2.0"
type = "library"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert!(config.is_library());
    assert!(!config.is_application());
}

#[test]
fn parse_manifest_with_entry_point() {
    let toml = r#"
[package]
name = "app"
version = "1.0.0"
entry = "main.hud"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.package.entry.as_deref(), Some("main.hud"));
}

#[test]
fn parse_manifest_with_authors_and_keywords() {
    let toml = r#"
[package]
name = "pkg"
version = "1.0.0"
authors = ["Alice", "Bob"]
keywords = ["ai", "agent"]
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.package.authors, vec!["Alice", "Bob"]);
    assert_eq!(config.package.keywords, vec!["ai", "agent"]);
}

#[test]
fn parse_manifest_with_categories() {
    let toml = r#"
[package]
name = "pkg"
version = "1.0.0"
categories = ["tools", "agents"]
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.package.categories, vec!["tools", "agents"]);
}

// ── DependencySpec ────────────────────────────────────────────────────────────

#[test]
fn dependency_spec_simple_version() {
    let spec = DependencySpec::Simple("^1.0".to_string());
    assert_eq!(spec.version(), "^1.0");
}

#[test]
fn dependency_spec_simple_semver_range() {
    let spec = DependencySpec::Simple(">=2.0.0".to_string());
    assert_eq!(spec.version(), ">=2.0.0");
}

#[test]
fn dependency_spec_detailed_version() {
    let spec = DependencySpec::Detailed {
        version: "~2.3".to_string(),
        features: vec![],
        registry: None,
        git: None,
        branch: None,
        tag: None,
        path: None,
        optional: false,
    };
    assert_eq!(spec.version(), "~2.3");
}

#[test]
fn dependency_spec_detailed_with_features() {
    let spec = DependencySpec::Detailed {
        version: "1.0.0".to_string(),
        features: vec!["openai".to_string(), "streaming".to_string()],
        registry: None,
        git: None,
        branch: None,
        tag: None,
        path: None,
        optional: false,
    };
    assert_eq!(spec.version(), "1.0.0");
    if let DependencySpec::Detailed { features, .. } = &spec {
        assert_eq!(features.len(), 2);
        assert!(features.contains(&"openai".to_string()));
    }
}

#[test]
fn dependency_spec_detailed_optional_flag() {
    let spec = DependencySpec::Detailed {
        version: "0.1.0".to_string(),
        features: vec![],
        registry: None,
        git: None,
        branch: None,
        tag: None,
        path: None,
        optional: true,
    };
    if let DependencySpec::Detailed { optional, .. } = spec {
        assert!(optional);
    }
}

#[test]
fn dependency_spec_detailed_git_and_branch() {
    let spec = DependencySpec::Detailed {
        version: "0.1.0".to_string(),
        features: vec![],
        registry: None,
        git: Some("https://github.com/example/repo".to_string()),
        branch: Some("main".to_string()),
        tag: None,
        path: None,
        optional: false,
    };
    if let DependencySpec::Detailed { git, branch, .. } = &spec {
        assert_eq!(git.as_deref(), Some("https://github.com/example/repo"));
        assert_eq!(branch.as_deref(), Some("main"));
    }
}

#[test]
fn dependency_spec_detailed_with_tag() {
    let spec = DependencySpec::Detailed {
        version: "1.0.0".to_string(),
        features: vec![],
        registry: None,
        git: None,
        branch: None,
        tag: Some("v1.0.0".to_string()),
        path: None,
        optional: false,
    };
    if let DependencySpec::Detailed { tag, .. } = &spec {
        assert_eq!(tag.as_deref(), Some("v1.0.0"));
    }
}

#[test]
fn dependency_spec_detailed_with_path() {
    let spec = DependencySpec::Detailed {
        version: "0.1.0".to_string(),
        features: vec![],
        registry: None,
        git: None,
        branch: None,
        tag: None,
        path: Some("../local-dep".to_string()),
        optional: false,
    };
    if let DependencySpec::Detailed { path, .. } = &spec {
        assert_eq!(path.as_deref(), Some("../local-dep"));
    }
}

#[test]
fn dependency_spec_parse_simple_from_toml() {
    let toml = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
my-dep = "^1.0"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let dep = config.dependencies.get("my-dep").unwrap();
    assert_eq!(dep.version(), "^1.0");
}

#[test]
fn dependency_spec_parse_detailed_from_toml() {
    let toml = r#"
[package]
name = "test"
version = "1.0.0"

[dependencies]
ai-tools = { version = "^2.0", features = ["openai"] }
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let dep = config.dependencies.get("ai-tools").unwrap();
    assert_eq!(dep.version(), "^2.0");
    if let DependencySpec::Detailed { features, .. } = dep {
        assert_eq!(features, &["openai"]);
    }
}

// ── dev_dependencies ──────────────────────────────────────────────────────────

#[test]
fn parse_dev_dependencies() {
    let toml = r#"
[package]
name = "test-pkg"
version = "1.0.0"

[dev_dependencies]
test-framework = "^1.0"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.dev_dependencies.len(), 1);
    assert_eq!(
        config
            .dev_dependencies
            .get("test-framework")
            .unwrap()
            .version(),
        "^1.0"
    );
}

// ── NativeDependencySpec ──────────────────────────────────────────────────────

#[test]
fn native_dependency_with_path_type_version() {
    let toml = r#"
[package]
name = "test"
version = "1.0.0"

[native-dependencies]
mylib = { path = "../mylib", type = "cmake", version = "1.0.0" }
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let dep = config.native_dependencies.get("mylib").unwrap();
    assert_eq!(dep.path.as_deref(), Some("../mylib"));
    assert_eq!(dep.build_type.as_deref(), Some("cmake"));
    assert_eq!(dep.version.as_deref(), Some("1.0.0"));
}

#[test]
fn native_dependency_minimal_all_none() {
    let toml = r#"
[package]
name = "test"
version = "1.0.0"

[native-dependencies]
minimal = {}
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let dep = config.native_dependencies.get("minimal").unwrap();
    assert!(dep.path.is_none());
    assert!(dep.build_type.is_none());
    assert!(dep.version.is_none());
}

#[test]
fn native_dependency_make_build_type() {
    let dep = NativeDependencySpec {
        path: Some("./lib".to_string()),
        build_type: Some("make".to_string()),
        version: None,
    };
    assert_eq!(dep.build_type.as_deref(), Some("make"));
}

// ── McpServerSpec ─────────────────────────────────────────────────────────────

#[test]
fn mcp_server_simple_version() {
    let spec = McpServerSpec::Simple("^1.0".to_string());
    assert_eq!(spec.version(), Some("^1.0"));
}

#[test]
fn mcp_server_detailed_version_some() {
    let spec = McpServerSpec::Detailed {
        version: Some("2.0.0".to_string()),
        server: None,
        registry: None,
        config: HashMap::new(),
        disabled: false,
        auto_approve: vec![],
    };
    assert_eq!(spec.version(), Some("2.0.0"));
}

#[test]
fn mcp_server_detailed_version_none() {
    let spec = McpServerSpec::Detailed {
        version: None,
        server: None,
        registry: None,
        config: HashMap::new(),
        disabled: true,
        auto_approve: vec![],
    };
    assert_eq!(spec.version(), None);
}

#[test]
fn mcp_server_parse_simple_and_detailed() {
    let toml = r#"
[package]
name = "test"
version = "0.1.0"

[mcp-servers]
github = "^1.0"

[mcp-servers.postgres]
version = "^2.0"
server = "pg-mcp"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.mcp_servers.len(), 2);
    assert_eq!(
        config.mcp_servers.get("github").unwrap().version(),
        Some("^1.0")
    );
    assert_eq!(
        config.mcp_servers.get("postgres").unwrap().version(),
        Some("^2.0")
    );
}

#[test]
fn mcp_server_detailed_auto_approve() {
    let toml = r#"
[package]
name = "test"
version = "0.1.0"

[mcp-servers.github]
version = "^1.0"
auto_approve = ["create_issue", "list_prs"]
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let spec = config.mcp_servers.get("github").unwrap();
    if let McpServerSpec::Detailed { auto_approve, .. } = spec {
        assert_eq!(auto_approve.len(), 2);
        assert!(auto_approve.contains(&"create_issue".to_string()));
    }
}

#[test]
fn mcp_server_detailed_disabled_flag() {
    let spec = McpServerSpec::Detailed {
        version: None,
        server: None,
        registry: None,
        config: HashMap::new(),
        disabled: true,
        auto_approve: vec![],
    };
    if let McpServerSpec::Detailed { disabled, .. } = spec {
        assert!(disabled);
    }
}

// ── AiProviderConfig ──────────────────────────────────────────────────────────

#[test]
fn ai_provider_parse_with_env_key() {
    let toml = r#"
[package]
name = "test"
version = "0.1.0"

[ai-providers.openai]
provider = "openai"
model = "gpt-4"
api_key_env = "OPENAI_API_KEY"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let p = config.ai_providers.get("openai").unwrap();
    assert_eq!(p.provider.as_deref(), Some("openai"));
    assert_eq!(p.model.as_deref(), Some("gpt-4"));
    assert_eq!(p.api_key_env.as_deref(), Some("OPENAI_API_KEY"));
    assert!(p.api_key.is_none());
}

#[test]
fn ai_provider_parse_with_inline_key() {
    let toml = r#"
[package]
name = "test"
version = "0.1.0"

[ai-providers.local]
provider = "ollama"
model = "llama2"
api_key = "sk-test"
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    let p = config.ai_providers.get("local").unwrap();
    assert_eq!(p.api_key.as_deref(), Some("sk-test"));
    assert!(p.api_key_env.is_none());
}

#[test]
fn ai_provider_config_all_none_by_default() {
    let p = AiProviderConfig {
        provider: None,
        model: None,
        api_key_env: None,
        api_key: None,
        config: HashMap::new(),
    };
    assert!(p.provider.is_none());
    assert!(p.model.is_none());
    assert!(p.api_key_env.is_none());
    assert!(p.api_key.is_none());
}

// ── HudhudConfig load / save / roundtrip ─────────────────────────────────────

#[test]
fn load_nonexistent_file_returns_error() {
    let result = HudhudConfig::load("/nonexistent/path/hudhud.toml");
    assert!(result.is_err());
}

#[test]
fn load_from_path_nonexistent_returns_error() {
    let result = HudhudConfig::load_from_path(std::path::Path::new("/nonexistent/hudhud.toml"));
    assert!(result.is_err());
}

#[test]
fn config_save_and_load_roundtrip() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_str().unwrap();
    let config = HudhudConfig::default();
    config.save(path).unwrap();
    let loaded = HudhudConfig::load(path).unwrap();
    assert_eq!(loaded.package.name, "my-project");
    assert_eq!(loaded.package.version, "0.1.0");
    assert_eq!(loaded.package.package_type, PackageType::Application);
}

#[test]
fn config_serialization_contains_name() {
    let config = HudhudConfig::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    assert!(toml_str.contains("name = \"my-project\""));
}

#[test]
fn config_serialization_contains_version() {
    let config = HudhudConfig::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    assert!(toml_str.contains("version = \"0.1.0\""));
}

#[test]
fn full_manifest_parse_counts() {
    let toml = r#"
[package]
name = "my-package"
version = "1.0.0"
type = "application"
entry = "main.hud"

[dependencies]
dep-a = "^1.0"
dep-b = { version = "^2.0", features = ["feat"] }

[dev_dependencies]
test-fw = "^0.1"

[native-dependencies]
cpp-lib = { path = "../cpp", type = "cmake" }

[mcp-servers]
github = "^1.0"
postgres = "^1.0"

[ai-providers]
openai = { model = "gpt-4" }
"#;
    let config: HudhudConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.package.name, "my-package");
    assert_eq!(config.dependencies.len(), 2);
    assert_eq!(config.dev_dependencies.len(), 1);
    assert_eq!(config.native_dependencies.len(), 1);
    assert_eq!(config.mcp_servers.len(), 2);
    assert_eq!(config.ai_providers.len(), 1);
}
