//! Fallback adapter for unsupported CI/CD providers.
//!
//! Returns honest errors instead of fake success. This adapter is used
//! when the requested provider (e.g., "netlify", "heroku") has no real
//! implementation yet.

use crate::*;

pub struct StubAdapter {
    name: String,
}

impl StubAdapter {
    pub fn new(provider: String) -> Self {
        Self {
            name: format!("unsupported({})", provider),
        }
    }
}

impl DeployAdapter for StubAdapter {
    fn generate(&self, plan: &DeployPlan) -> Result<Vec<DeployArtifact>, DeployError> {
        Err(DeployError::AdapterError(format!(
            "No deploy adapter for '{}'. App '{}' cannot generate artifacts. \
             Supported adapters: github, gitlab, docker, vercel, k8s",
            self.name, plan.app_name
        )))
    }

    fn deploy(&self, plan: &DeployPlan) -> Result<DeployResult, DeployError> {
        Err(DeployError::AdapterError(format!(
            "No deploy adapter for '{}'. App '{}' cannot be deployed. \
             Supported adapters: github, gitlab, docker, vercel, k8s",
            self.name, plan.app_name
        )))
    }

    fn rollback(&self, app_name: &str) -> Result<(), DeployError> {
        Err(DeployError::AdapterError(format!(
            "No deploy adapter for '{}'. App '{}' cannot be rolled back.",
            self.name, app_name
        )))
    }

    fn name(&self) -> &str {
        &self.name
    }
}
