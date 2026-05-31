use crate::common::{CliError, HudHudConfig};
use hudhudscript_compiler::{Bytecode, Compiler};
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_formatter::Formatter;
use hudhudscript_mcp::{McpClient, TransportConfig};
use hudhudscript_parser::{parse, parse_with_recovery};
use hudhudscript_vm::{OutputLocale, VM};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub fn detect_locale(source: &str) -> &'static str {
    // Check for Japanese characters (Hiragana, Katakana, Kanji)
    let has_japanese = source.chars().any(|c| {
        matches!(c,
            '\u{3040}'..='\u{309F}' |  // Hiragana
            '\u{30A0}'..='\u{30FF}' |  // Katakana
            '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
            '\u{3400}'..='\u{4DBF}'    // CJK Extension A
        )
    });

    if has_japanese {
        return "ja";
    }

    // Check for Arabic characters
    let has_arabic = source
        .chars()
        .any(|c| matches!(c, '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}'));

    if has_arabic {
        return "ar";
    }

    "default"
}

pub fn show_version(detailed: bool) {
    println!("HudHudScript v{}", env!("CARGO_PKG_VERSION"));

    if detailed {
        println!();
        println!("Build information:");
        println!("  Package: {}", env!("CARGO_PKG_NAME"));
        println!("  Authors: {}", env!("CARGO_PKG_AUTHORS"));
        println!();
        println!("Features:");
        println!("  - Multi-language support (English, Turkish, Japanese, Arabic)");
        println!("  - MCP protocol integration");
        println!("  - Layer-based agent orchestration");
        println!("  - Built-in VCS (branch/merge)");
        println!("  - Intent-based programming");
        println!("  - Japanese kanji numeral support");
        println!("  - Arabic-Indic numeral support");
    }
}

pub fn show_info() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              HudHudScript - System Information                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    // Version Information
    println!("📦 Version Information:");
    println!("   Version:        {}", env!("CARGO_PKG_VERSION"));
    println!("   Package:        {}", env!("CARGO_PKG_NAME"));
    println!("   Authors:        {}", env!("CARGO_PKG_AUTHORS"));
    println!("   Repository:     {}", env!("CARGO_PKG_REPOSITORY"));
    println!();

    // Build Information
    println!("🔨 Build Information:");
    println!("   Rust Version:   {}", env!("CARGO_PKG_RUST_VERSION"));
    println!("   Target:         {}", std::env::consts::ARCH);
    println!("   OS:             {}", std::env::consts::OS);
    println!("   Family:         {}", std::env::consts::FAMILY);
    println!();

    // Language Support
    println!("🌍 Language Support:");
    println!("   ✓ English (en)     - Subject-Verb-Object (SVO)");
    println!("   ✓ Turkish (tr)     - Subject-Object-Verb (SOV)");
    println!("   ✓ Japanese (ja)    - SOV + Particles + Kanji numerals");
    println!("   ✓ Arabic (ar)      - VSO + RTL + Arabic-Indic numerals");
    println!("   ✓ Spanish (es)     - SVO");
    println!("   ✓ Portuguese (pt)  - SVO");
    println!("   ✓ French (fr)      - SVO");
    println!("   ✓ German (de)      - V2 word order");
    println!("   ✓ Russian (ru)     - Free word order");
    println!("   ✓ Chinese (zh)     - SVO + Characters");
    println!("   + 13 more languages");
    println!();

    // Features
    println!("✨ Features:");
    println!("   ✓ Multi-language parser (23 languages)");
    println!("   ✓ Full interpreter with provider system");
    println!("   ✓ Bytecode compiler & VM");
    println!("   ✓ MCP protocol integration (90+ servers)");
    println!("   ✓ AI provider support (OpenAI, Anthropic, Ollama)");
    println!("   ✓ Governance system (constitutions, laws, councils)");
    println!("   ✓ Token optimization (80-95% savings)");
    println!("   ✓ Async/await support");
    println!("   ✓ Module system");
    println!("   ✓ Arrow functions & template strings");
    println!();

    // Provider Status
    println!("🤖 AI Provider Status:");
    let openai_available = std::env::var("OPENAI_API_KEY").is_ok();
    let anthropic_available = std::env::var("ANTHROPIC_API_KEY").is_ok();
    let ollama_url =
        std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    println!(
        "   OpenAI:         {}",
        if openai_available {
            "✓ Available"
        } else {
            "✗ Not configured (set OPENAI_API_KEY)"
        }
    );
    println!(
        "   Anthropic:      {}",
        if anthropic_available {
            "✓ Available"
        } else {
            "✗ Not configured (set ANTHROPIC_API_KEY)"
        }
    );
    println!("   Ollama:         ⚠ Check connection ({})", ollama_url);
    println!();

    // MCP Configuration
    println!("🔌 MCP Configuration:");
    let user_config = dirs::home_dir().map(|mut p| {
        p.push(".kiro/settings/mcp.json");
        p
    });
    let workspace_config = std::path::PathBuf::from(".kiro/settings/mcp.json");

    if let Some(ref path) = user_config {
        println!(
            "   User config:    {} ({})",
            if path.exists() {
                "✓ Found"
            } else {
                "✗ Not found"
            },
            path.display()
        );
    }
    println!(
        "   Workspace:      {} ({})",
        if workspace_config.exists() {
            "✓ Found"
        } else {
            "✗ Not found"
        },
        workspace_config.display()
    );
    println!();

    // System Paths
    println!("📁 System Paths:");
    if let Some(home) = dirs::home_dir() {
        println!("   Home:           {}", home.display());
    }
    if let Ok(current) = std::env::current_dir() {
        println!("   Current:        {}", current.display());
    }
    println!();

    // Test Status
    println!("🧪 Test Status:");
    println!("   Run `cargo test` to see current test results.");
    println!();

    // Documentation
    println!("📚 Documentation:");
    println!("   README:         https://github.com/HudHudMind/hudhudscript");
    println!("   Quickstart:     QUICKSTART.md");
    println!("   Examples:       examples/");
    println!();

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  For more information: hudhudscript --help                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
}
