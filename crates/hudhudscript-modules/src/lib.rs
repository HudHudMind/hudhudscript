//! Module system for HudHudScript
//!
//! This crate provides module loading, resolution, and caching:
//! - Module loading from files
//! - Import/export declarations
//! - Module path resolution
//! - Circular dependency detection
//! - Module caching
//! - Model download & artifact management (HuggingFace, Ollama, GGUF)

pub mod gguf;
pub mod graph;
pub mod huggingface;
pub mod loader;
pub mod model_manager;
pub mod module;
pub mod ollama;
pub mod resolver;

pub use gguf::{
    parse_header as parse_gguf_header, GgufError, GgufMetadata, GgufQuantization, GgufValue,
    GGUF_MAGIC, GGUF_TYPE_ARRAY, GGUF_TYPE_BOOL, GGUF_TYPE_FLOAT32, GGUF_TYPE_FLOAT64,
    GGUF_TYPE_INT16, GGUF_TYPE_INT32, GGUF_TYPE_INT64, GGUF_TYPE_INT8, GGUF_TYPE_STRING,
    GGUF_TYPE_UINT16, GGUF_TYPE_UINT32, GGUF_TYPE_UINT64, GGUF_TYPE_UINT8,
};
pub use graph::{GraphError, ImportGraph};
pub use huggingface::{value_to_hf_model, HfClient, HfError, HfFile, HfModel};
pub use loader::{ModuleLoader, ModuleLoaderError};
pub use model_manager::{ModelEntry, ModelFormat, ModelManager, ModelManagerError, ModelProvider};
pub use module::{Export, ExportKind, Module, ModuleId};
pub use ollama::{
    value_to_manifest, value_to_ollama_model, OllamaClient, OllamaError, OllamaLayer,
    OllamaManifest, OllamaModel,
};
pub use resolver::{ModuleResolver, ResolverError};
