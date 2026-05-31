//! Vercel adapter (#557)
//!
//! Generates vercel.json configuration from a DeployPlan.

use crate::*;

#[derive(Default)]
pub struct VercelAdapter;

impl VercelAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl DeployAdapter for VercelAdapter {
    fn generate(&self, plan: &DeployPlan) -> Result<Vec<DeployArtifact>, DeployError> {
        let has_wasm = plan
            .targets
            .iter()
            .any(|t| matches!(t.platform, TargetPlatform::Wasm));

        let vercel_json = if has_wasm {
            format!(
                "{{\n  \"$schema\": \"https://openapi.vercel.sh/vercel.json\",\n  \
                \"name\": \"{}\",\n  \
                \"buildCommand\": \"wasm-pack build --target web\",\n  \
                \"outputDirectory\": \"pkg\",\n  \
                \"framework\": null\n}}\n",
                plan.app_name
            )
        } else {
            format!(
                "{{\n  \"$schema\": \"https://openapi.vercel.sh/vercel.json\",\n  \
                \"name\": \"{}\",\n  \
                \"framework\": \"nextjs\"\n}}\n",
                plan.app_name
            )
        };

        Ok(vec![DeployArtifact {
            filename: "vercel.json".to_string(),
            content: vercel_json,
        }])
    }

    fn deploy(&self, plan: &DeployPlan) -> Result<DeployResult, DeployError> {
        Ok(DeployResult {
            success: true,
            url: Some(format!("https://{}.vercel.app", plan.app_name)),
            message: format!("Vercel config generated for '{}'", plan.app_name),
        })
    }

    fn rollback(&self, app_name: &str) -> Result<(), DeployError> {
        println!("[vercel] Rollback: vercel rollback for '{}'", app_name);
        Ok(())
    }

    fn name(&self) -> &str {
        "vercel"
    }
}
