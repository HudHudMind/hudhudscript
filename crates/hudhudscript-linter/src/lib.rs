//! HudHudScript Linter
//!
//! Static analysis tool for HudHudScript programs. Takes a parsed AST
//! (`Vec<Stmt>`) and produces a list of [`LintDiagnostic`]s.
//!
//! # Supported Rules
//!
//! | Code | Rule | Default Severity | Default |
//! |------|------|-----------------|---------|
//! | `naming-convention` | Agent/subject names should be PascalCase; variables camelCase or snake_case | Warning | enabled |
//! | `unused-variable` | Variable declared but never referenced | Warning | enabled |
//! | `empty-block` | Empty function/agent/subject body | Warning | enabled |
//! | `missing-return` | Function with inconsistent return paths | Warning | enabled |
//! | `deep-nesting` | Block nesting deeper than threshold | Warning | enabled |
//! | `variable-shadowing` | Variable shadows an outer scope variable | Info | enabled |
//! | `empty-catch` | Catch block with empty body (swallowed error) | Warning | enabled |
//! | `const-reassign` | Assignment to a `const` variable | Error | enabled |
//! | `prefer-const` | `let` variable never reassigned — could be `const` | Info | enabled |
//! | `no-print` | `print()` / `println()` call in code | Warning | **disabled** |
//!
//! # Configuration
//!
//! Place a `.hudlint` JSON file in the project root to customise rules:
//!
//! ```json
//! {
//!   "rules": {
//!     "no-print": { "enabled": true },
//!     "prefer-const": { "enabled": false },
//!     "deep-nesting": { "enabled": true, "severity": "Error" }
//!   },
//!   "max_nesting_depth": 5
//! }
//! ```

mod config;
mod diagnostic;
mod rules;
mod walker;

pub use config::{LintConfig, RuleConfig, Severity};
pub use diagnostic::LintDiagnostic;
pub use rules::naming_convention::{is_camel_or_snake, is_pascal_case};

use hudhudscript_ast::Stmt;
use std::path::Path;

/// Lint a parsed HudHudScript program.
///
/// Returns a list of diagnostics sorted by source position.
pub fn lint(stmts: &[Stmt], config: &LintConfig) -> Vec<LintDiagnostic> {
    let mut ctx = walker::LintContext::new(config);
    walker::walk_stmts(&mut ctx, stmts, 0);

    // Post-walk analysis
    ctx.check_unused_variables();
    ctx.check_prefer_const();

    let mut diagnostics = ctx.into_diagnostics();
    diagnostics.sort_by_key(|d| d.span.start.offset);
    diagnostics
}

/// Lint with default configuration.
pub fn lint_default(stmts: &[Stmt]) -> Vec<LintDiagnostic> {
    lint(stmts, &LintConfig::default())
}

/// Load a [`LintConfig`] from a `.hudlint` JSON file at the given directory.
///
/// If the file does not exist, returns the default configuration.
/// If the file exists but is malformed, returns an error.
pub fn load_config(project_root: &Path) -> Result<LintConfig, String> {
    let config_path = project_root.join(".hudlint");
    if !config_path.exists() {
        return Ok(LintConfig::default());
    }
    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("failed to read .hudlint: {e}"))?;
    let mut config: LintConfig =
        serde_json::from_str(&contents).map_err(|e| format!("invalid .hudlint JSON: {e}"))?;
    // The `no-print` rule is disabled by default; only enable it if the user
    // explicitly sets it in the config file. If the parsed config does not
    // mention it, insert the disabled default.
    config
        .rules
        .entry("no-print".to_string())
        .or_insert_with(RuleConfig::disabled);
    Ok(config)
}

/// Convenience: load config from project root + lint.
pub fn lint_with_config_file(
    stmts: &[Stmt],
    project_root: &Path,
) -> Result<Vec<LintDiagnostic>, String> {
    let config = load_config(project_root)?;
    Ok(lint(stmts, &config))
}
