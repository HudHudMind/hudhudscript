//! Tab completion for the HudHudScript REPL.

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};

/// Tab-completion helper for the HudHudScript REPL.
///
/// Completes HudHudScript keywords, built-in function names, built-in module
/// names, and REPL meta-commands.  Pulls built-in names from the shared-builtins
/// registry so additions there are automatically available in the REPL.
pub struct HudCompleter {
    /// All completable words (keywords + builtins + REPL commands).
    candidates: Vec<String>,
}

impl Default for HudCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl HudCompleter {
    pub fn new() -> Self {
        let mut candidates: Vec<String> = Vec::with_capacity(128);

        // ── Language keywords (English) ─────────────────────────────────
        // These mirror the lexer keyword list and the parser grammar.
        let keywords: &[&str] = &[
            // Variables & declarations
            "let",
            "const",
            "var",
            "set",
            // Control flow
            "if",
            "else",
            "while",
            "for",
            "return",
            "break",
            "continue",
            "switch",
            "case",
            "default",
            "match",
            // Functions
            "function",
            "fn",
            // Async
            "async",
            "await",
            // Error handling
            "try",
            "catch",
            "finally",
            "throw",
            // OOP
            "class",
            "extends",
            "new",
            "this",
            "super",
            "self",
            "public",
            "private",
            "protected",
            "static",
            "constructor",
            // Trait / interface
            "trait",
            "interface",
            "implements",
            // Module system
            "import",
            "export",
            "from",
            "as",
            "use",
            // Agent / MCP
            "agent",
            "task",
            "tool",
            "resource",
            "mcp",
            "server",
            "config",
            "provider",
            "model",
            "call",
            // Data & state
            "data",
            "state",
            "statemachine",
            "enum",
            // Literals
            "true",
            "false",
            "null",
            // SOP
            "subject",
            "spawn",
            "send",
            "receive",
            // Events
            "event",
            "on",
            "trigger",
            "when",
            // Governance
            "constitution",
            "law",
            "rule",
            "council",
            "swarm",
            "community",
            "contract",
            // Flow
            "flow",
            "dataflow",
            "parallel",
            "sequential",
            "execute",
        ];
        candidates.extend(keywords.iter().map(|s| (*s).to_string()));

        // ── Built-in global functions (from shared-builtins registry) ───
        for member in hudhudscript_bytecode::registry::BUILTIN_GLOBALS {
            let name = member.name.to_string();
            if !candidates.contains(&name) {
                candidates.push(name);
            }
        }

        // ── Built-in module names (Math, JSON, Array, …) ───────────────
        for module in hudhudscript_bytecode::registry::BUILTIN_MODULES.iter() {
            let name = module.name.to_string();
            if !candidates.contains(&name) {
                candidates.push(name);
            }
        }

        // ── REPL meta-commands ──────────────────────────────────────────
        let repl_cmds: &[&str] = &[
            ":help", ":quit", ":exit", ":clear", ":history", ":env", ":type",
        ];
        candidates.extend(repl_cmds.iter().map(|s| (*s).to_string()));

        candidates.sort();
        candidates.dedup();

        Self { candidates }
    }
}

impl Completer for HudCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find the start of the current word (identifier or `:` command).
        let line_to_pos = &line[..pos];
        let start = line_to_pos
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = &line_to_pos[start..];
        if prefix.is_empty() {
            return Ok((start, Vec::new()));
        }

        let matches: Vec<Pair> = self
            .candidates
            .iter()
            .filter(|c| c.starts_with(prefix))
            .map(|c| Pair {
                display: c.clone(),
                replacement: c.clone(),
            })
            .collect();

        Ok((start, matches))
    }
}

impl Hinter for HudCompleter {
    type Hint = String;
}

impl Highlighter for HudCompleter {}
impl Validator for HudCompleter {}
impl Helper for HudCompleter {}
