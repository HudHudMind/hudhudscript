//! CI/CD adapter implementations
//!
//! Adapters generate deployment artifacts for various platforms:
//! - GitHub Actions (YAML workflows)
//! - GitLab CI (.gitlab-ci.yml)
//! - Jenkins (Jenkinsfile)
//! - Docker (Dockerfile)
//! - Kubernetes (manifests)
//! - Vercel
//! - Custom adapters (user-defined)

pub mod docker;
pub mod github;
pub mod k8s;
pub mod unsupported;
pub mod vercel;

use crate::{DeployAdapter, DeployError};

/// Available CI/CD adapters
#[derive(Debug, Clone)]
pub enum Adapter {
    GitHub,
    GitLab,
    Jenkins,
    Docker,
    Kubernetes,
    Vercel,
    Custom(String),
}

impl Adapter {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "github" => Some(Adapter::GitHub),
            "gitlab" => Some(Adapter::GitLab),
            "jenkins" => Some(Adapter::Jenkins),
            "docker" => Some(Adapter::Docker),
            "kubernetes" | "k8s" => Some(Adapter::Kubernetes),
            "vercel" => Some(Adapter::Vercel),
            _ => Some(Adapter::Custom(s.to_string())),
        }
    }
}

/// Create an adapter for the given provider
pub fn create_adapter(adapter: &Adapter) -> Result<Box<dyn DeployAdapter>, DeployError> {
    match adapter {
        Adapter::GitHub => Ok(Box::new(github::GitHubAdapter::new())),
        Adapter::Docker => Ok(Box::new(docker::DockerAdapter::new())),
        Adapter::Vercel => Ok(Box::new(vercel::VercelAdapter::new())),
        Adapter::Kubernetes => Ok(Box::new(k8s::K8sAdapter::new())),
        _ => Ok(Box::new(unsupported::StubAdapter::new(format!(
            "{:?}",
            adapter
        )))),
    }
}
