//! CLI argument parsing framework for .hud scripts
//!
//! Provides argument definition, parsing, help generation, and shell completion
//! script generation so that .hud scripts can declare their own CLI interface.
//!
//! # Issue #612

use std::collections::HashMap;
use std::fmt;

// ═══════════════════════════════════════════════════════════════════════════════
// Error type
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors that can occur during argument parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgError {
    /// A required argument was not provided.
    MissingRequired(String),
    /// An unknown flag/option was encountered.
    UnknownArgument(String),
    /// A value could not be converted to the expected type.
    InvalidValue {
        arg: String,
        expected: ArgType,
        got: String,
    },
    /// A subcommand was expected but not found.
    MissingSubcommand,
    /// Generic parse error.
    Other(String),
}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgError::MissingRequired(name) => {
                write!(f, "required argument '{}' not provided", name)
            }
            ArgError::UnknownArgument(name) => write!(f, "unknown argument '{}'", name),
            ArgError::InvalidValue { arg, expected, got } => {
                write!(
                    f,
                    "invalid value for '{}': expected {:?}, got '{}'",
                    arg, expected, got
                )
            }
            ArgError::MissingSubcommand => write!(f, "a subcommand is required"),
            ArgError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ArgError {}

/// Convenience alias.
pub type ArgResult<T> = Result<T, ArgError>;

// ═══════════════════════════════════════════════════════════════════════════════
// ArgType
// ═══════════════════════════════════════════════════════════════════════════════

/// The type of value an argument accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    /// UTF-8 string value.
    String,
    /// Signed 64-bit integer.
    Int,
    /// 64-bit floating-point.
    Float,
    /// Boolean flag (no value needed; presence = true).
    Bool,
    /// Comma-separated list of strings.
    List,
}

impl fmt::Display for ArgType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgType::String => write!(f, "string"),
            ArgType::Int => write!(f, "int"),
            ArgType::Float => write!(f, "float"),
            ArgType::Bool => write!(f, "bool"),
            ArgType::List => write!(f, "list"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Arg
// ═══════════════════════════════════════════════════════════════════════════════

/// A single argument definition.
#[derive(Debug, Clone)]
pub struct Arg {
    /// Canonical name (used as key in `ParsedArgs`).
    pub name: String,
    /// Optional single-character short flag (e.g. `'v'` for `-v`).
    pub short: Option<char>,
    /// Optional long flag (e.g. `"verbose"` for `--verbose`).
    pub long: Option<String>,
    /// Human-readable description shown in `--help`.
    pub description: String,
    /// Whether the argument must be provided.
    pub required: bool,
    /// Default value as a string (parsed according to `arg_type`).
    pub default_value: Option<String>,
    /// Expected value type.
    pub arg_type: ArgType,
}

impl Arg {
    /// Create a new argument with the given name and type.
    pub fn new(name: impl Into<String>, arg_type: ArgType) -> Self {
        Self {
            name: name.into(),
            short: None,
            long: None,
            description: String::new(),
            required: false,
            default_value: None,
            arg_type,
        }
    }

    /// Set the short flag character.
    pub fn short(mut self, c: char) -> Self {
        self.short = Some(c);
        self
    }

    /// Set the long flag string.
    pub fn long(mut self, s: impl Into<String>) -> Self {
        self.long = Some(s.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, s: impl Into<String>) -> Self {
        self.description = s.into();
        self
    }

    /// Mark as required.
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }

    /// Set a default value.
    pub fn default_value(mut self, v: impl Into<String>) -> Self {
        self.default_value = Some(v.into());
        self
    }

    /// Returns the display form used in help text (e.g. `-v, --verbose`).
    fn flags_display(&self) -> String {
        match (&self.short, &self.long) {
            (Some(s), Some(l)) => format!("-{}, --{}", s, l),
            (Some(s), None) => format!("-{}", s),
            (None, Some(l)) => format!("    --{}", l),
            (None, None) => format!("<{}>", self.name),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Subcommand
// ═══════════════════════════════════════════════════════════════════════════════

/// A named subcommand with its own set of arguments.
#[derive(Debug, Clone)]
pub struct Subcommand {
    /// Subcommand name (e.g. `"build"`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Arguments accepted by this subcommand.
    pub args: Vec<Arg>,
}

impl Subcommand {
    /// Create a new subcommand.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            args: Vec::new(),
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shell (completion targets)
// ═══════════════════════════════════════════════════════════════════════════════

/// Target shell for completion script generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ParsedArgs
// ═══════════════════════════════════════════════════════════════════════════════

/// The result of a successful parse. Values are stored by argument name.
#[derive(Debug, Clone)]
pub struct ParsedArgs {
    values: HashMap<String, String>,
    list_values: HashMap<String, Vec<String>>,
    subcommand: Option<(String, Box<ParsedArgs>)>,
    positional: Vec<String>,
}

impl ParsedArgs {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            list_values: HashMap::new(),
            subcommand: None,
            positional: Vec::new(),
        }
    }

    /// Get a string value by name.
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|s| s.as_str())
    }

    /// Get an integer value by name.
    pub fn get_int(&self, name: &str) -> Option<i64> {
        self.values.get(name).and_then(|s| s.parse::<i64>().ok())
    }

    /// Get a float value by name.
    pub fn get_float(&self, name: &str) -> Option<f64> {
        self.values.get(name).and_then(|s| s.parse::<f64>().ok())
    }

    /// Get a boolean value by name. Flags default to `true` when present.
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.values.get(name).map(|s| s == "true")
    }

    /// Get a list value by name.
    pub fn get_list(&self, name: &str) -> Option<&[String]> {
        self.list_values.get(name).map(|v| v.as_slice())
    }

    /// Get the matched subcommand name and its parsed args.
    pub fn subcommand(&self) -> Option<(&str, &ParsedArgs)> {
        self.subcommand
            .as_ref()
            .map(|(name, args)| (name.as_str(), args.as_ref()))
    }

    /// Get positional arguments (values that did not match any flag).
    pub fn positional(&self) -> &[String] {
        &self.positional
    }

    /// Check whether a given argument name was provided.
    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name) || self.list_values.contains_key(name)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ArgParser
// ═══════════════════════════════════════════════════════════════════════════════

/// The main argument parser for .hud script CLI definitions.
#[derive(Debug, Clone)]
pub struct ArgParser {
    /// Program name shown in help/completions.
    pub program_name: String,
    /// Program description.
    pub description: String,
    /// Program version string.
    pub version: String,
    /// Top-level arguments.
    pub args: Vec<Arg>,
    /// Registered subcommands.
    pub subcommands: Vec<Subcommand>,
}
