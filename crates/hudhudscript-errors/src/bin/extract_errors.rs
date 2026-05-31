use hudhudscript_errors::catalog::{ERROR_GROUPS, ERROR_TABLE};
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
struct ErrorTranslation {
    title: String,
    short_description: String,
    long_description: String,
    hints: Vec<String>,
}

fn generate_json(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = serde_json::Map::new();

    for entry in ERROR_TABLE.iter() {
        let translation = ErrorTranslation {
            title: entry.title.to_string(),
            short_description: entry.short_description.to_string(),
            long_description: entry.long_description.to_string(),
            hints: entry.hints.iter().map(|s| s.to_string()).collect(),
        };

        errors.insert(
            entry.short_code.to_string(),
            serde_json::to_value(translation)?,
        );
    }

    let mut catalog = serde_json::Map::new();
    catalog.insert("errors".to_string(), serde_json::Value::Object(errors));

    let mut file = File::create(output_path)?;
    let json = serde_json::to_string_pretty(&serde_json::Value::Object(catalog))?;
    file.write_all(json.as_bytes())?;

    println!(
        "JSON: {} errors written to {:?}",
        ERROR_TABLE.len(),
        output_path
    );
    Ok(())
}

fn generate_markdown(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut md = String::with_capacity(64 * 1024);

    md.push_str("# HudHudScript Error Reference\n\n");
    md.push_str(&format!(
        "This document lists all **{}** error codes across **{}** categories.\n\n",
        ERROR_TABLE.len(),
        ERROR_GROUPS.len(),
    ));
    md.push_str("Generated from the canonical error catalog (`crates/hudhudscript-errors/src/catalog.rs`).\n\n");
    md.push_str("---\n\n");

    // Table of contents
    md.push_str("## Categories\n\n");
    for (name, entries) in ERROR_GROUPS.iter() {
        md.push_str(&format!(
            "- [{}](#{}) ({} errors)\n",
            name,
            name,
            entries.len()
        ));
    }
    md.push_str("\n---\n\n");

    // Each category
    for (name, entries) in ERROR_GROUPS.iter() {
        md.push_str(&format!("## {}\n\n", name));

        for entry in entries.iter() {
            md.push_str(&format!("### {} — {}\n\n", entry.short_code, entry.title));
            md.push_str(&format!("**Code:** `{}`\n\n", entry.long_code));
            md.push_str(&format!("**Category:** {}\n\n", entry.category));
            md.push_str(&format!("{}\n\n", entry.short_description));

            if !entry.long_description.is_empty() {
                md.push_str(&format!("{}\n\n", entry.long_description));
            }

            if !entry.hints.is_empty() {
                md.push_str("**Hints:**\n\n");
                for hint in entry.hints.iter() {
                    md.push_str(&format!("- {}\n", hint));
                }
                md.push_str("\n");
            }

            if let Some(bad) = entry.example_bad {
                md.push_str("**Example (incorrect):**\n\n");
                md.push_str(&format!("```hudhudscript\n{}\n```\n\n", bad));
            }

            if let Some(good) = entry.example_good {
                md.push_str("**Example (correct):**\n\n");
                md.push_str(&format!("```hudhudscript\n{}\n```\n\n", good));
            }

            if !entry.see_also.is_empty() {
                md.push_str(&format!("**See also:** {}\n\n", entry.see_also.join(", ")));
            }

            md.push_str(&format!("*Since: {}*\n\n", entry.since_version));
            md.push_str("---\n\n");
        }
    }

    let mut file = File::create(output_path)?;
    file.write_all(md.as_bytes())?;

    println!(
        "Markdown: {} errors written to {:?}",
        ERROR_TABLE.len(),
        output_path
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Extracting {} error entries...", ERROR_TABLE.len());

    // Generate JSON catalog
    generate_json(Path::new("errors_en.json"))?;

    // Generate Markdown reference
    generate_markdown(Path::new("docs/ERROR_REFERENCE.md"))?;

    Ok(())
}
