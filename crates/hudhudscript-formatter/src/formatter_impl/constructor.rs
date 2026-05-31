use super::*;

impl Formatter {
    /// Create a new formatter with default config
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
            current_indent: 0,
        }
    }

    /// Create a new formatter with custom config
    pub fn with_config(config: FormatterConfig) -> Self {
        Self {
            config,
            current_indent: 0,
        }
    }

    /// Format a program (list of statements)
    pub fn format_program(&mut self, stmts: &[Stmt]) -> String {
        let mut output = String::new();

        for (i, stmt) in stmts.iter().enumerate() {
            output.push_str(&self.format_stmt(stmt));

            // Add blank line between top-level declarations
            if i < stmts.len() - 1 && matches!(stmt, Stmt::Decl(_)) {
                output.push('\n');
            }
        }

        output
    }
}
