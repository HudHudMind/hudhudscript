//! GitHub Actions adapter (#555, #721)
//!
//! Generates `.github/workflows/deploy.yml` from a DeployPlan,
//! writes the file to disk, and pushes to trigger the workflow.

use crate::*;
use std::path::{Path, PathBuf};
use std::process::Command;

/// GitHub Actions deploy adapter.
///
/// When `project_dir` is set, `deploy()` writes the generated workflow YAML
/// into `<project_dir>/.github/workflows/`, commits, and pushes.
#[derive(Default)]
pub struct GitHubAdapter {
    project_dir: Option<PathBuf>,
}

impl GitHubAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an adapter that will deploy into the given project directory.
    pub fn with_project_dir(project_dir: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: Some(project_dir.into()),
        }
    }

    fn target_build_step(target: &Target) -> String {
        match target.platform {
            TargetPlatform::Web => format!(
                "      - name: Build web ({})\n        run: npm run build\n",
                target.framework
            ),
            TargetPlatform::Desktop => format!(
                "      - name: Build desktop ({})\n        run: cargo tauri build\n",
                target.framework
            ),
            TargetPlatform::Mobile => format!(
                "      - name: Build mobile ({})\n        run: flutter build apk\n",
                target.framework
            ),
            TargetPlatform::Wasm => {
                "      - name: Build WASM\n        run: wasm-pack build --target web\n".to_string()
            }
            TargetPlatform::Custom(ref name) => format!(
                "      - name: Build {}\n        run: echo 'Custom build for {}'\n",
                name, name
            ),
        }
    }

    /// Run a command in the project directory, returning stdout on success.
    fn run_git(project_dir: &Path, args: &[&str]) -> Result<String, DeployError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(project_dir)
            .output()
            .map_err(|e| {
                DeployError::DeployFailed(format!("failed to run git {}: {}", args[0], e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DeployError::DeployFailed(format!(
                "git {} failed: {}",
                args[0], stderr
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Write workflow YAML, commit, push, and return (commit_sha, workflow_url).
    fn write_and_push(
        project_dir: &Path,
        plan: &DeployPlan,
        yaml: &str,
    ) -> Result<(String, String), DeployError> {
        // Ensure .github/workflows/ exists
        let workflows_dir = project_dir.join(".github/workflows");
        std::fs::create_dir_all(&workflows_dir).map_err(|e| {
            DeployError::DeployFailed(format!(
                "failed to create {}: {}",
                workflows_dir.display(),
                e
            ))
        })?;

        // Write the workflow file
        let workflow_path = workflows_dir.join("deploy.yml");
        std::fs::write(&workflow_path, yaml).map_err(|e| {
            DeployError::DeployFailed(format!(
                "failed to write {}: {}",
                workflow_path.display(),
                e
            ))
        })?;

        // Stage, commit, push
        Self::run_git(project_dir, &["add", ".github/workflows/deploy.yml"])?;
        Self::run_git(
            project_dir,
            &[
                "commit",
                "-m",
                &format!("ci: update GitHub Actions workflow for '{}'", plan.app_name),
            ],
        )?;
        Self::run_git(project_dir, &["push"])?;

        // Retrieve commit SHA
        let sha = Self::run_git(project_dir, &["rev-parse", "HEAD"])?;

        // Derive the workflow URL from the remote origin
        let remote_url =
            Self::run_git(project_dir, &["remote", "get-url", "origin"]).unwrap_or_default();
        let workflow_url = derive_workflow_url(&remote_url, &sha);

        Ok((sha, workflow_url))
    }
}

/// Best-effort conversion of a git remote URL to a GitHub Actions run URL.
fn derive_workflow_url(remote_url: &str, sha: &str) -> String {
    // Normalise SSH or HTTPS remotes to "owner/repo"
    let slug = remote_url
        .trim()
        .trim_end_matches(".git")
        .replace("git@github.com:", "")
        .replace("https://github.com/", "");

    if slug.contains('/') {
        format!(
            "https://github.com/{}/actions?query=event:push+sha:{}",
            slug, sha
        )
    } else {
        // Fallback when the remote doesn't look like GitHub
        format!("(workflow triggered by commit {})", sha)
    }
}

impl DeployAdapter for GitHubAdapter {
    fn generate(&self, plan: &DeployPlan) -> Result<Vec<DeployArtifact>, DeployError> {
        let mut yaml = String::new();
        yaml.push_str(&format!(
            "# Generated by HudHudScript deploy for '{}'\n",
            plan.app_name
        ));
        yaml.push_str("name: Deploy\n\non:\n  push:\n    branches: [main]\n\njobs:\n");

        for (i, target) in plan.targets.iter().enumerate() {
            yaml.push_str(&format!(
                "  build-{}:\n    runs-on: ubuntu-latest\n    steps:\n",
                i
            ));
            yaml.push_str("      - uses: actions/checkout@v4\n");
            yaml.push_str(&Self::target_build_step(target));
        }

        if plan.targets.is_empty() {
            yaml.push_str("  build:\n    runs-on: ubuntu-latest\n    steps:\n");
            yaml.push_str("      - uses: actions/checkout@v4\n");
            yaml.push_str("      - name: Build\n        run: echo 'No targets configured'\n");
        }

        Ok(vec![DeployArtifact {
            filename: ".github/workflows/deploy.yml".to_string(),
            content: yaml,
        }])
    }

    fn deploy(&self, plan: &DeployPlan) -> Result<DeployResult, DeployError> {
        let project_dir = match &self.project_dir {
            Some(dir) => dir.clone(),
            None => {
                return Err(DeployError::DeployFailed(
                    "GitHubAdapter: project_dir not set — use GitHubAdapter::with_project_dir()"
                        .to_string(),
                ));
            }
        };

        // Generate the workflow YAML
        let artifacts = self.generate(plan)?;
        let yaml = &artifacts[0].content;

        // Write to disk, commit, push
        let (sha, workflow_url) = Self::write_and_push(&project_dir, plan, yaml)?;

        Ok(DeployResult {
            success: true,
            url: Some(workflow_url),
            message: format!(
                "GitHub Actions workflow committed and pushed for '{}' (commit {})",
                plan.app_name, sha
            ),
        })
    }

    fn rollback(&self, app_name: &str) -> Result<(), DeployError> {
        let project_dir = match &self.project_dir {
            Some(dir) => dir.clone(),
            None => {
                return Err(DeployError::RollbackFailed(
                    "GitHubAdapter: project_dir not set".to_string(),
                ));
            }
        };

        // Revert the last workflow commit
        Self::run_git(&project_dir, &["revert", "--no-edit", "HEAD"])?;
        Self::run_git(&project_dir, &["push"])?;

        println!(
            "[github] Rollback for '{}' — reverted last workflow commit",
            app_name
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "github"
    }
}
