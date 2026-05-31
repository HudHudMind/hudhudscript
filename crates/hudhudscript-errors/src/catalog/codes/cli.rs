use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum CliErrorCode {
    /// E0003 — CLI Argument Has Invalid Value
    ArgInvalidValue = 3,
    /// E0004 — Required CLI Argument Not Provided
    ArgMissingRequired = 4,
    /// E0005 — No Subcommand Specified
    ArgMissingSubcommand = 5,
    /// E0006 — Generic CLI Parsing Failure
    ArgOther = 6,
    /// E0007 — Unrecognized CLI Argument
    ArgUnknownArgument = 7,
}
