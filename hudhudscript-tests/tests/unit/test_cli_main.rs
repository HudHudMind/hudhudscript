//! Tests for the `hudhud` binary's clap CLI specification (help rendering).

use hudhudscript_cli::clap::CommandFactory;
use hudhudscript_cli::common::cli_spec::Cli;

#[test]
fn help_does_not_reduce_hudhudscript_to_mcp_orchestration() {
    let help = Cli::command().render_help().to_string();
    assert!(!help.contains("MCP-based orchestration language"));
}
