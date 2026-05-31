//! Comprehensive public API tests for hudhudscript-modules

use hudhudscript_modules::{
    parse_gguf_header, Export, ExportKind, GgufMetadata, GgufQuantization, GraphError, HfClient,
    ImportGraph, ModelEntry, ModelFormat, ModelManager, ModelManagerError, ModelProvider, Module,
    ModuleLoader, ModuleResolver, OllamaClient,
};
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════
// ImportGraph
// ═══════════════════════════════════════════════════════════════════

#[test]
fn import_graph_new_is_empty() {
    let g = ImportGraph::new();
    assert!(g.check_cycles().is_ok());
}

#[test]
fn import_graph_default_is_empty() {
    let g = ImportGraph::default();
    assert!(g.check_cycles().is_ok());
}

#[test]
fn import_graph_add_module_no_cycle() {
    let mut g = ImportGraph::new();
    g.add_module(PathBuf::from("a.hudhud"));
    assert!(g.check_cycles().is_ok());
}

#[test]
fn import_graph_add_dependency_linear() {
    let mut g = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    g.add_module(a.clone());
    g.add_module(b.clone());
    g.add_dependency(a, b);
    assert!(g.check_cycles().is_ok());
}

#[test]
fn import_graph_cycle_detected() {
    let mut g = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    g.add_module(a.clone());
    g.add_module(b.clone());
    g.add_dependency(a.clone(), b.clone());
    g.add_dependency(b, a);
    assert!(g.check_cycles().is_err());
}

#[test]
fn import_graph_self_cycle() {
    let mut g = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    g.add_module(a.clone());
    g.add_dependency(a.clone(), a);
    assert!(g.check_cycles().is_err());
}

#[test]
fn import_graph_topological_sort_linear() {
    let mut g = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    let c = PathBuf::from("c.hudhud");
    g.add_module(a.clone());
    g.add_module(b.clone());
    g.add_module(c.clone());
    g.add_dependency(a.clone(), b.clone());
    g.add_dependency(b.clone(), c.clone());
    let sorted = g.topological_sort().unwrap();
    let pos_a = sorted.iter().position(|x| x == &a).unwrap();
    let pos_b = sorted.iter().position(|x| x == &b).unwrap();
    let pos_c = sorted.iter().position(|x| x == &c).unwrap();
    assert!(pos_a < pos_b);
    assert!(pos_b < pos_c);
}

#[test]
fn import_graph_topological_sort_cycle_error() {
    let mut g = ImportGraph::new();
    let a = PathBuf::from("a.hudhud");
    let b = PathBuf::from("b.hudhud");
    g.add_module(a.clone());
    g.add_module(b.clone());
    g.add_dependency(a.clone(), b.clone());
    g.add_dependency(b, a);
    assert!(g.topological_sort().is_err());
}

#[test]
fn import_graph_diamond_no_cycle() {
    let mut g = ImportGraph::new();
    let (a, b, c, d) = (
        PathBuf::from("a"),
        PathBuf::from("b"),
        PathBuf::from("c"),
        PathBuf::from("d"),
    );
    g.add_module(a.clone());
    g.add_module(b.clone());
    g.add_module(c.clone());
    g.add_module(d.clone());
    g.add_dependency(a.clone(), b.clone());
    g.add_dependency(a, c.clone());
    g.add_dependency(b, d.clone());
    g.add_dependency(c, d);
    assert!(g.check_cycles().is_ok());
    assert_eq!(g.topological_sort().unwrap().len(), 4);
}

#[test]
fn import_graph_topo_sort_single_module() {
    let mut g = ImportGraph::new();
    g.add_module(PathBuf::from("only"));
    assert_eq!(g.topological_sort().unwrap().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// Module
// ═══════════════════════════════════════════════════════════════════

#[test]
fn module_new_defaults() {
    let m = Module::new(PathBuf::from("test.hudhud"), "let x = 1".into(), vec![]);
    assert_eq!(m.id, PathBuf::from("test.hudhud"));
    assert_eq!(m.source, "let x = 1");
    assert!(!m.executed);
    assert!(m.exports.is_empty());
    assert!(m.imports.is_empty());
    assert!(m.exported_values.is_empty());
}

#[test]
fn module_add_export_named() {
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    m.add_export(
        "foo".into(),
        Export {
            kind: ExportKind::Named(vec!["foo".into()]),
            source: None,
        },
    );
    assert!(m.get_export("foo").is_some());
    assert!(m.get_export("bar").is_none());
}

#[test]
fn module_add_export_default() {
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    m.add_export(
        "default".into(),
        Export {
            kind: ExportKind::Default("myFunc".into()),
            source: None,
        },
    );
    assert_eq!(
        m.get_export("default").unwrap().kind,
        ExportKind::Default("myFunc".into())
    );
}

#[test]
fn module_add_export_wildcard_with_source() {
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    m.add_export(
        "*".into(),
        Export {
            kind: ExportKind::Wildcard,
            source: Some("./other".into()),
        },
    );
    assert_eq!(m.get_export("*").unwrap().source, Some("./other".into()));
}

#[test]
fn module_add_import_and_paths() {
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    m.add_import("./utils".into(), vec!["helper".into(), "log".into()]);
    assert!(m.import_paths().contains(&"./utils".to_string()));
}

#[test]
fn module_export_names_count() {
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    m.add_export(
        "a".into(),
        Export {
            kind: ExportKind::Named(vec!["a".into()]),
            source: None,
        },
    );
    m.add_export(
        "b".into(),
        Export {
            kind: ExportKind::Named(vec!["b".into()]),
            source: None,
        },
    );
    assert_eq!(m.export_names().len(), 2);
}

#[test]
fn module_mark_executed() {
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    assert!(!m.executed);
    m.mark_executed();
    assert!(m.executed);
}

#[test]
fn module_exported_values_add_get() {
    use hudhudscript_modules::module::ModuleValue;
    let mut m = Module::new(PathBuf::from("m.hudhud"), String::new(), vec![]);
    m.add_exported_value(
        "x".into(),
        ModuleValue::Variable {
            name: "x".into(),
            value: "42".into(),
        },
    );
    assert!(m.get_exported_value("x").is_some());
    assert!(m.get_exported_value("y").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// ExportKind variants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn export_kind_named_eq() {
    assert_eq!(
        ExportKind::Named(vec!["x".into()]),
        ExportKind::Named(vec!["x".into()])
    );
}

#[test]
fn export_kind_default_eq() {
    assert_eq!(
        ExportKind::Default("f".into()),
        ExportKind::Default("f".into())
    );
}

#[test]
fn export_kind_wildcard_eq() {
    assert_eq!(ExportKind::Wildcard, ExportKind::Wildcard);
}

#[test]
fn export_kind_ne_across_variants() {
    assert_ne!(ExportKind::Wildcard, ExportKind::Named(vec![]));
    assert_ne!(
        ExportKind::Default("a".into()),
        ExportKind::Default("b".into())
    );
}

#[test]
fn export_serde_roundtrip() {
    let export = Export {
        kind: ExportKind::Named(vec!["a".into(), "b".into()]),
        source: Some("./mod".into()),
    };
    let json = serde_json::to_string(&export).unwrap();
    let deserialized: Export = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, export);
}

// ═══════════════════════════════════════════════════════════════════
// ModuleResolver
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolver_resolve_relative_existing() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("mod.hudhud"), "").unwrap();
    let r = ModuleResolver::new(dir.path().to_path_buf());
    assert!(r.resolve("./mod").unwrap().exists());
}

#[test]
fn resolver_resolve_not_found() {
    let dir = tempfile::TempDir::new().unwrap();
    let r = ModuleResolver::new(dir.path().to_path_buf());
    assert!(r.resolve("./nonexistent").is_err());
}

#[test]
fn resolver_add_search_path_finds_lib() {
    let dir = tempfile::TempDir::new().unwrap();
    let lib_dir = tempfile::TempDir::new().unwrap();
    std::fs::write(lib_dir.path().join("mylib.hudhud"), "").unwrap();
    let mut r = ModuleResolver::new(dir.path().to_path_buf());
    r.add_search_path(lib_dir.path().to_path_buf());
    assert!(r.resolve("mylib").is_ok());
}

#[test]
fn resolver_resolve_parent_relative() {
    let dir = tempfile::TempDir::new().unwrap();
    let sub = dir.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(dir.path().join("root.hudhud"), "").unwrap();
    let r = ModuleResolver::new(sub);
    assert!(r.resolve("../root").is_ok());
}

#[test]
fn resolver_absolute_like_path_not_found() {
    let resolver = ModuleResolver::new(PathBuf::from("/nonexistent"));
    assert!(resolver.resolve("some_pkg").is_err());
}

// ═══════════════════════════════════════════════════════════════════
// ModuleLoader (async)
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn loader_load_simple() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("test.hudhud"), "let x = 1").unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    let m = loader.load("./test").await.unwrap();
    assert_eq!(m.source, "let x = 1");
}

#[tokio::test]
async fn loader_cache_hit() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("c.hudhud"), "").unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    loader.load("./c").await.unwrap();
    assert!(loader.get_cached("./c").await.is_some());
}

#[tokio::test]
async fn loader_cache_miss() {
    let dir = tempfile::TempDir::new().unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    assert!(loader.get_cached("./missing").await.is_none());
}

#[tokio::test]
async fn loader_clear_cache_removes_entries() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("x.hudhud"), "").unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    loader.load("./x").await.unwrap();
    loader.clear_cache().await;
    assert!(loader.get_cached("./x").await.is_none());
}

#[tokio::test]
async fn loader_not_found_error() {
    let dir = tempfile::TempDir::new().unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    assert!(loader.load("./nope").await.is_err());
}

#[tokio::test]
async fn loader_load_returns_same_on_second_call() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(dir.path().join("twice.hudhud"), "let a = 1").unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    let m1 = loader.load("./twice").await.unwrap();
    let m2 = loader.load("./twice").await.unwrap();
    assert_eq!(m1.id, m2.id);
    assert_eq!(m1.source, m2.source);
}

#[tokio::test]
async fn loader_clear_then_reload_gets_new_content() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("reload.hudhud");
    std::fs::write(&file, "let a = 1").unwrap();
    let loader = ModuleLoader::new(dir.path().to_path_buf());
    loader.load("./reload").await.unwrap();
    loader.clear_cache().await;
    std::fs::write(&file, "let a = 2").unwrap();
    let m = loader.load("./reload").await.unwrap();
    assert_eq!(m.source, "let a = 2");
}

// ═══════════════════════════════════════════════════════════════════
// GraphError Display
// ═══════════════════════════════════════════════════════════════════

#[test]
fn graph_error_circular_display() {
    assert!(GraphError::CircularDependency("a->b".into())
        .to_string()
        .contains("Circular"));
}

#[test]
fn graph_error_not_found_display() {
    assert!(GraphError::ModuleNotFound("foo".into())
        .to_string()
        .contains("not found"));
}

// ═══════════════════════════════════════════════════════════════════
// GGUF types
// ═══════════════════════════════════════════════════════════════════

#[test]
fn gguf_quantization_from_u32_all_variants() {
    assert_eq!(GgufQuantization::from(0), GgufQuantization::F32);
    assert_eq!(GgufQuantization::from(1), GgufQuantization::F16);
    assert_eq!(GgufQuantization::from(2), GgufQuantization::Q4_0);
    assert_eq!(GgufQuantization::from(3), GgufQuantization::Q4_1);
    assert_eq!(GgufQuantization::from(6), GgufQuantization::Q5_0);
    assert_eq!(GgufQuantization::from(7), GgufQuantization::Q5_1);
    assert_eq!(GgufQuantization::from(8), GgufQuantization::Q8_0);
    assert_eq!(GgufQuantization::from(999), GgufQuantization::Unknown(999));
}

#[test]
fn gguf_quantization_display_all() {
    assert_eq!(GgufQuantization::Q4_0.to_string(), "Q4_0");
    assert_eq!(GgufQuantization::Q4_1.to_string(), "Q4_1");
    assert_eq!(GgufQuantization::Q5_0.to_string(), "Q5_0");
    assert_eq!(GgufQuantization::Q5_1.to_string(), "Q5_1");
    assert_eq!(GgufQuantization::Q8_0.to_string(), "Q8_0");
    assert_eq!(GgufQuantization::F16.to_string(), "F16");
    assert_eq!(GgufQuantization::F32.to_string(), "F32");
    assert_eq!(GgufQuantization::Unknown(42).to_string(), "Unknown(42)");
}

#[test]
fn gguf_metadata_clone_debug() {
    let meta = GgufMetadata {
        architecture: "llama".into(),
        context_length: 4096,
        quantization: GgufQuantization::Q4_0,
        embedding_length: 4096,
        vocab_size: 32000,
        file_size: 1024,
    };
    let cloned = meta.clone();
    assert_eq!(cloned.architecture, "llama");
    assert!(format!("{:?}", meta).contains("llama"));
}

#[test]
fn gguf_parse_too_short() {
    assert!(parse_gguf_header(&[0u8; 10]).is_err());
}

#[test]
fn gguf_parse_invalid_magic() {
    let mut data = vec![0u8; 64];
    data[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
    assert!(parse_gguf_header(&data).is_err());
}

#[test]
fn gguf_parse_unsupported_version() {
    let mut data = vec![0u8; 64];
    data[0..4].copy_from_slice(&0x46475547u32.to_le_bytes());
    data[4..8].copy_from_slice(&99u32.to_le_bytes());
    assert!(parse_gguf_header(&data).is_err());
}

// ═══════════════════════════════════════════════════════════════════
// HfClient
// ═══════════════════════════════════════════════════════════════════

#[test]
fn hf_client_new_custom_url() {
    let c = HfClient::new("https://example.com", None);
    assert_eq!(c.base_url, "https://example.com");
    assert!(c.auth_token.is_none());
}

#[test]
fn hf_client_public_defaults() {
    let c = HfClient::public();
    assert_eq!(c.base_url, "https://huggingface.co");
}

#[test]
fn hf_client_with_auth_token() {
    let c = HfClient::new("https://hf.co", Some("tok_123".into()));
    assert_eq!(c.auth_token, Some("tok_123".into()));
}

#[test]
fn hf_client_download_url_format() {
    let c = HfClient::public();
    let url = c.download_url("TheBloke/Llama-2-7B-GGUF", "model.gguf");
    assert!(url.contains("TheBloke/Llama-2-7B-GGUF"));
    assert!(url.contains("resolve/main"));
    assert!(url.contains("model.gguf"));
}

#[test]
fn hf_client_clone() {
    let c = HfClient::public();
    let c2 = c.clone();
    assert_eq!(c2.base_url, c.base_url);
}

// ═══════════════════════════════════════════════════════════════════
// OllamaClient
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ollama_client_new_custom() {
    assert_eq!(
        OllamaClient::new("http://remote:8080").base_url,
        "http://remote:8080"
    );
}

#[test]
fn ollama_client_local_default() {
    assert_eq!(OllamaClient::local().base_url, "http://localhost:11434");
}

#[test]
fn ollama_client_clone() {
    let c = OllamaClient::local();
    let c2 = c.clone();
    assert_eq!(c2.base_url, c.base_url);
}

// ═══════════════════════════════════════════════════════════════════
// ModelManager
// ═══════════════════════════════════════════════════════════════════

#[test]
fn model_manager_new_empty() {
    let mm = ModelManager::new("/tmp/models");
    assert!(mm.list().is_empty());
    assert_eq!(mm.disk_usage(), 0);
}

#[test]
fn model_manager_register_and_get() {
    let mut mm = ModelManager::new("/tmp/models");
    mm.register(ModelEntry {
        name: "llama2".into(),
        version: "1.0.0".into(),
        provider: ModelProvider::HuggingFace,
        path: PathBuf::from("/tmp/l"),
        size: 4_000_000_000,
        format: ModelFormat::Gguf,
    })
    .unwrap();
    assert_eq!(
        mm.get("llama2").unwrap().provider,
        ModelProvider::HuggingFace
    );
}

#[test]
fn model_manager_register_duplicate_error() {
    let mut mm = ModelManager::new("/tmp");
    let entry = ModelEntry {
        name: "m".into(),
        version: "1".into(),
        provider: ModelProvider::Local,
        path: PathBuf::from("/tmp/m"),
        size: 100,
        format: ModelFormat::Bin,
    };
    mm.register(entry.clone()).unwrap();
    assert!(mm.register(entry).is_err());
}

#[test]
fn model_manager_remove_success() {
    let mut mm = ModelManager::new("/tmp");
    mm.register(ModelEntry {
        name: "m".into(),
        version: "1".into(),
        provider: ModelProvider::Ollama,
        path: PathBuf::from("/tmp/m"),
        size: 500,
        format: ModelFormat::SafeTensors,
    })
    .unwrap();
    assert_eq!(mm.remove("m").unwrap().name, "m");
    assert!(mm.get("m").is_err());
}

#[test]
fn model_manager_remove_not_found() {
    let mut mm = ModelManager::new("/tmp");
    assert!(mm.remove("nonexistent").is_err());
}

#[test]
fn model_manager_disk_usage_sum() {
    let mut mm = ModelManager::new("/tmp");
    for i in 0..3 {
        mm.register(ModelEntry {
            name: format!("m{i}"),
            version: "1".into(),
            provider: ModelProvider::Local,
            path: PathBuf::from("/tmp/x"),
            size: 100,
            format: ModelFormat::Bin,
        })
        .unwrap();
    }
    assert_eq!(mm.disk_usage(), 300);
}

#[test]
fn model_manager_list_count() {
    let mut mm = ModelManager::new("/tmp");
    mm.register(ModelEntry {
        name: "a".into(),
        version: "1".into(),
        provider: ModelProvider::Local,
        path: PathBuf::from("/tmp/a"),
        size: 10,
        format: ModelFormat::Other("onnx".into()),
    })
    .unwrap();
    assert_eq!(mm.list().len(), 1);
}

#[test]
fn model_format_variants_eq() {
    assert_eq!(ModelFormat::Gguf, ModelFormat::Gguf);
    assert_eq!(ModelFormat::SafeTensors, ModelFormat::SafeTensors);
    assert_ne!(ModelFormat::Gguf, ModelFormat::Bin);
}

#[test]
fn model_provider_variants_eq() {
    assert_eq!(ModelProvider::HuggingFace, ModelProvider::HuggingFace);
    assert_ne!(ModelProvider::HuggingFace, ModelProvider::Local);
}

#[test]
fn model_manager_check_disk_space_small() {
    assert!(ModelManager::new("/tmp").check_disk_space(1).is_ok());
}

#[test]
fn model_manager_error_display_not_found() {
    assert!(ModelManagerError::NotFound("x".into())
        .to_string()
        .contains("not found"));
}

#[test]
fn model_manager_error_display_already_exists() {
    assert!(ModelManagerError::AlreadyExists("x".into())
        .to_string()
        .contains("already"));
}

#[test]
fn model_manager_get_not_found() {
    let mm = ModelManager::new("/tmp");
    assert!(mm.get("no_such_model").is_err());
}
