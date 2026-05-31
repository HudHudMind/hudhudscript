use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExceptionCode(pub u32);

impl ExceptionCode {
    /// Look up the static entry for this code in the per-category tables.
    pub fn entry(self) -> &'static crate::catalog::entry::ExceptionEntry {
        let idx = self.0 as usize;
        match idx {
            1..=18 => &crate::catalog::table::ai::TABLE[idx - 1],
            19..=23 => &crate::catalog::table::cli::TABLE[idx - 18],
            24..=31 => &crate::catalog::table::compile::TABLE[idx - 23],
            32..=34 => &crate::catalog::table::cybernetics::TABLE[idx - 31],
            35..=76 => &crate::catalog::table::governance::TABLE[idx - 34],
            77..=79 => &crate::catalog::table::io::TABLE[idx - 76],
            80..=83 => &crate::catalog::table::lex::TABLE[idx - 79],
            84..=86 => &crate::catalog::table::localization::TABLE[idx - 83],
            87..=88 => &crate::catalog::table::lsp::TABLE[idx - 86],
            89..=98 => &crate::catalog::table::native::TABLE[idx - 88],
            99..=122 => &crate::catalog::table::orchestration::TABLE[idx - 98],
            123..=158 => &crate::catalog::table::package::TABLE[idx - 122],
            159..=164 => &crate::catalog::table::parse::TABLE[idx - 158],
            165..=168 => &crate::catalog::table::promise::TABLE[idx - 164],
            169..=178 => &crate::catalog::table::resource::TABLE[idx - 168],
            179..=212 => &crate::catalog::table::runtime::TABLE[idx - 178],
            213..=218 => &crate::catalog::table::security::TABLE[idx - 212],
            219..=239 => &crate::catalog::table::storage::TABLE[idx - 218],
            240..=262 => &crate::catalog::table::tokenomics::TABLE[idx - 239],
            263..=290 => &crate::catalog::table::tool::TABLE[idx - 262],
            291..=301 => &crate::catalog::table::type_errors::TABLE[idx - 290],
            302..=314 => &crate::catalog::table::ui::TABLE[idx - 301],
            315..=323 => &crate::catalog::table::validation::TABLE[idx - 314],
            _ => panic!("invalid ExceptionCode index: {}", idx),
        }
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
    pub fn category(self) -> crate::catalog::category::ExceptionCategory {
        self.entry().category
    }

    /// Project this exception code onto its sibling [`hudhudscript_errors::ErrorCode`].
    pub fn as_error_code(self) -> hudhudscript_errors::ErrorCode {
        hudhudscript_errors::ErrorCode(self.0)
    }
}

impl std::fmt::Display for ExceptionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let e = self.entry();
        write!(f, "[{}] {}", e.short_code, e.title)
    }
}

impl From<hudhudscript_errors::ErrorCode> for ExceptionCode {
    fn from(code: hudhudscript_errors::ErrorCode) -> Self {
        ExceptionCode(code.0)
    }
}

mod ai;
pub use ai::AiExceptionCode;

mod cli;
pub use cli::CliExceptionCode;

mod compile;
pub use compile::CompileExceptionCode;

mod cybernetics;
pub use cybernetics::CyberneticsExceptionCode;

mod database;
pub use database::DatabaseExceptionCode;

mod deploy;
pub use deploy::DeployExceptionCode;

mod event;
pub use event::EventExceptionCode;

mod governance_community;
pub use governance_community::GovernanceCommunityExceptionCode;

mod governance_constitution;
pub use governance_constitution::GovernanceConstitutionExceptionCode;

mod governance_core;
pub use governance_core::GovernanceCoreExceptionCode;

mod governance_council;
pub use governance_council::GovernanceCouncilExceptionCode;

mod governance_coup;
pub use governance_coup::GovernanceCoupExceptionCode;

mod lex;
pub use lex::LexExceptionCode;

mod localization;
pub use localization::LocalizationExceptionCode;

mod lsp;
pub use lsp::LspExceptionCode;

mod native;
pub use native::NativeExceptionCode;

mod network;
pub use network::NetworkExceptionCode;

mod orchestration;
pub use orchestration::OrchestrationExceptionCode;

mod package_core;
pub use package_core::PackageCoreExceptionCode;

mod package_gguf;
pub use package_gguf::PackageGgufExceptionCode;

mod package_graph;
pub use package_graph::PackageGraphExceptionCode;

mod package_model;
pub use package_model::PackageModelExceptionCode;

mod package_module;
pub use package_module::PackageModuleExceptionCode;

mod package_ollama;
pub use package_ollama::PackageOllamaExceptionCode;

mod package_resolver;
pub use package_resolver::PackageResolverExceptionCode;

mod parse;
pub use parse::ParseExceptionCode;

mod resource;
pub use resource::ResourceExceptionCode;

mod runtime_agent;
pub use runtime_agent::RuntimeAgentExceptionCode;

mod runtime_async;
pub use runtime_async::RuntimeAsyncExceptionCode;

mod runtime_control;
pub use runtime_control::RuntimeControlExceptionCode;

mod runtime_execution;
pub use runtime_execution::RuntimeExecutionExceptionCode;

mod runtime_governance;
pub use runtime_governance::RuntimeGovernanceExceptionCode;

mod runtime_promise;
pub use runtime_promise::RuntimePromiseExceptionCode;

mod runtime_stm;
pub use runtime_stm::RuntimeStmExceptionCode;

mod runtime_variable;
pub use runtime_variable::RuntimeVariableExceptionCode;

mod security;
pub use security::SecurityExceptionCode;

mod storage_cache;
pub use storage_cache::StorageCacheExceptionCode;

mod storage_embedding;
pub use storage_embedding::StorageEmbeddingExceptionCode;

mod storage_index;
pub use storage_index::StorageIndexExceptionCode;

mod storage_persistence;
pub use storage_persistence::StoragePersistenceExceptionCode;

mod storage_store;
pub use storage_store::StorageStoreExceptionCode;

mod tokenomics_core;
pub use tokenomics_core::TokenomicsCoreExceptionCode;

mod tokenomics_cost;
pub use tokenomics_cost::TokenomicsCostExceptionCode;

mod tokenomics_provider;
pub use tokenomics_provider::TokenomicsProviderExceptionCode;

mod tokenomics_ratelimit;
pub use tokenomics_ratelimit::TokenomicsRatelimitExceptionCode;

mod tool_approval;
pub use tool_approval::ToolApprovalExceptionCode;

mod tool_core;
pub use tool_core::ToolCoreExceptionCode;

mod tool_git;
pub use tool_git::ToolGitExceptionCode;

mod tool_http;
pub use tool_http::ToolHttpExceptionCode;

mod tool_openapi;
pub use tool_openapi::ToolOpenapiExceptionCode;

mod tool_registry;
pub use tool_registry::ToolRegistryExceptionCode;

mod type_errors;
pub use type_errors::TypeErrorsExceptionCode;

mod ui;
pub use ui::UiExceptionCode;

mod validation;
pub use validation::ValidationExceptionCode;

mod vcs;
pub use vcs::VcsExceptionCode;
