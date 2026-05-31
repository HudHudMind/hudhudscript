//! Project scaffolding — `init_project` creates library or application layout.

use colored::Colorize;

use crate::Result;

use super::Installer;

impl Installer {
    /// Initialize new project.
    ///
    /// When `library` is true the scaffolded project is a library package
    /// (no `main.hud`, adds `lib/` directory instead).
    pub async fn init_project(&self, name: &str, path: &str, library: bool) -> Result<()> {
        println!("{} Initializing project: {}", ">>".green(), name.bold());

        std::fs::create_dir_all(path)?;

        if library {
            std::fs::create_dir_all(format!("{}/lib", path))?;
            std::fs::create_dir_all(format!("{}/tests", path))?;

            let config = format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
type = "library"
description = ""
authors = []
license = "MIT"

[dependencies]
# Add your dependencies here

[mcp-servers]
# Configure MCP servers here

[ai-providers]
# Configure AI providers here
"#
            );
            std::fs::write(format!("{}/hudhud.toml", path), config)?;

            let utils = r#"// Utility module for the library
// Export functions, agents, or types for consumers.

fn greet(name) {
    return "Hello, " + name + "!"
}
"#;
            std::fs::write(format!("{}/lib/utils.hud", path), utils)?;
        } else {
            std::fs::create_dir_all(format!("{}/tests", path))?;

            let config = format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
type = "application"
entry = "main.hud"
description = ""
authors = []
license = "MIT"

[dependencies]
# Add your dependencies here

[mcp-servers]
# Configure MCP servers here

[ai-providers]
# Configure AI providers here
"#
            );
            std::fs::write(format!("{}/hudhud.toml", path), config)?;

            let main_file = r#"// Main entry point
print("Hello from HudHudScript!")
"#;
            std::fs::write(format!("{}/main.hud", path), main_file)?;

            let agent_file = r#"// Template agent
agent MyAgent {
    model = "gpt-4"
    instructions = "You are a helpful assistant."

    fn run(input) {
        return ask(input)
    }
}
"#;
            std::fs::write(format!("{}/agent.hud", path), agent_file)?;
        }

        let readme = format!("# {}\n\nA HudHudScript project.\n", name);
        std::fs::write(format!("{}/README.md", path), readme)?;

        println!("{} Project initialized successfully!", ">>".green());
        Ok(())
    }
}
