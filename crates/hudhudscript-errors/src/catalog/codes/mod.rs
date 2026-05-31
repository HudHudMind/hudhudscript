use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ErrorCode(pub u32);

impl ErrorCode {
    pub fn entry(self) -> &'static crate::catalog::query::ErrorEntry {
        &crate::catalog::table::ERROR_TABLE[self.0 as usize - 1]
    }
    pub fn long_code(self) -> &'static str {
        self.entry().long_code
    }
    pub fn short_code(self) -> &'static str {
        self.entry().short_code
    }
    pub fn title(self) -> &'static str {
        self.entry().title
    }
    pub fn short_description(self) -> &'static str {
        self.entry().short_description
    }
    pub fn long_description(self) -> &'static str {
        self.entry().long_description
    }
    pub fn hints(self) -> &'static [&'static str] {
        self.entry().hints
    }
    pub fn category(self) -> crate::catalog::category::ErrorCategory {
        self.entry().category
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let e = self.entry();
        write!(f, "[{}] {}", e.short_code, e.title)
    }
}

mod impl_constants;

mod ai;
pub use ai::AiErrorCode;

mod cli;
pub use cli::CliErrorCode;

mod compile;
pub use compile::CompileErrorCode;

mod cybernetics;
pub use cybernetics::CyberneticsErrorCode;

mod database;
pub use database::DatabaseErrorCode;

mod deploy;
pub use deploy::DeployErrorCode;

mod event;
pub use event::EventErrorCode;

mod governance_community;
pub use governance_community::GovernanceCommunityErrorCode;

mod governance_constitution;
pub use governance_constitution::GovernanceConstitutionErrorCode;

mod governance_core;
pub use governance_core::GovernanceCoreErrorCode;

mod governance_council;
pub use governance_council::GovernanceCouncilErrorCode;

mod governance_coup;
pub use governance_coup::GovernanceCoupErrorCode;

mod lex;
pub use lex::LexErrorCode;

mod localization;
pub use localization::LocalizationErrorCode;

mod lsp;
pub use lsp::LspErrorCode;

mod native;
pub use native::NativeErrorCode;

mod network;
pub use network::NetworkErrorCode;

mod orchestration;
pub use orchestration::OrchestrationErrorCode;

mod package_core;
pub use package_core::PackageCoreErrorCode;

mod package_gguf;
pub use package_gguf::PackageGgufErrorCode;

mod package_graph;
pub use package_graph::PackageGraphErrorCode;

mod package_model;
pub use package_model::PackageModelErrorCode;

mod package_module;
pub use package_module::PackageModuleErrorCode;

mod package_ollama;
pub use package_ollama::PackageOllamaErrorCode;

mod package_resolver;
pub use package_resolver::PackageResolverErrorCode;

mod parse;
pub use parse::ParseErrorCode;

mod resource;
pub use resource::ResourceErrorCode;

mod runtime_agent;
pub use runtime_agent::RuntimeAgentErrorCode;

mod runtime_async;
pub use runtime_async::RuntimeAsyncErrorCode;

mod runtime_control;
pub use runtime_control::RuntimeControlErrorCode;

mod runtime_execution;
pub use runtime_execution::RuntimeExecutionErrorCode;

mod runtime_governance;
pub use runtime_governance::RuntimeGovernanceErrorCode;

mod runtime_promise;
pub use runtime_promise::RuntimePromiseErrorCode;

mod runtime_stm;
pub use runtime_stm::RuntimeStmErrorCode;

mod runtime_variable;
pub use runtime_variable::RuntimeVariableErrorCode;

mod security;
pub use security::SecurityErrorCode;

mod storage_cache;
pub use storage_cache::StorageCacheErrorCode;

mod storage_embedding;
pub use storage_embedding::StorageEmbeddingErrorCode;

mod storage_index;
pub use storage_index::StorageIndexErrorCode;

mod storage_persistence;
pub use storage_persistence::StoragePersistenceErrorCode;

mod storage_store;
pub use storage_store::StorageStoreErrorCode;

mod tokenomics_core;
pub use tokenomics_core::TokenomicsCoreErrorCode;

mod tokenomics_cost;
pub use tokenomics_cost::TokenomicsCostErrorCode;

mod tokenomics_provider;
pub use tokenomics_provider::TokenomicsProviderErrorCode;

mod tokenomics_ratelimit;
pub use tokenomics_ratelimit::TokenomicsRatelimitErrorCode;

mod tool_approval;
pub use tool_approval::ToolApprovalErrorCode;

mod tool_core;
pub use tool_core::ToolCoreErrorCode;

mod tool_git;
pub use tool_git::ToolGitErrorCode;

mod tool_http;
pub use tool_http::ToolHttpErrorCode;

mod tool_openapi;
pub use tool_openapi::ToolOpenapiErrorCode;

mod tool_registry;
pub use tool_registry::ToolRegistryErrorCode;

mod type_errors;
pub use type_errors::TypeErrorCode;

mod ui;
pub use ui::UiErrorCode;

mod validation;
pub use validation::ValidationErrorCode;

mod vcs;
pub use vcs::VcsErrorCode;
