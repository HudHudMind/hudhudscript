//! G5: GitHub REST API tool — reqwest wrapper.
//! Token from GITHUB_TOKEN env var.

use serde_json::Value;

/// GitHub REST API v3 client.
pub struct GithubTool {
    token: String,
}

impl GithubTool {
    pub fn new(token: Option<String>) -> Self {
        Self { token: token.unwrap_or_default() }
    }

    fn client() -> Result<reqwest::blocking::Client, String> {
        reqwest::blocking::Client::builder()
            .user_agent("HudHudScript-GithubTool/1.0")
            .build()
            .map_err(|e| format!("client: {}", e))
    }

    pub fn list_issues(&self, owner: &str, repo: &str) -> Result<Value, String> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let resp = Self::client()?
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .map_err(|e| format!("HTTP: {}", e))?;
        resp.json().map_err(|e| format!("JSON: {}", e))
    }

    pub fn create_issue(&self, owner: &str, repo: &str, title: &str, body: Option<&str>) -> Result<Value, String> {
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
        let mut payload = serde_json::json!({ "title": title });
        if let Some(b) = body { payload["body"] = serde_json::json!(b); }
        let resp = Self::client()?
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .json(&payload)
            .send()
            .map_err(|e| format!("HTTP: {}", e))?;
        resp.json().map_err(|e| format!("JSON: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_tool_creation() {
        let tool = GithubTool::new(Some("fake".into()));
        let tool2 = GithubTool::new(None);
    }
}
