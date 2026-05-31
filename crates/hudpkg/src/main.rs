//! hudpkg — HudHud package manager
//!
//! A command-line tool for managing HudHudScript project dependencies.
//! It reads and writes `hudhud.toml` manifest files and resolves packages
//! from a local `packages/` directory.
//!
//! # Usage
//!
//! ```text
//! hudpkg <command> [arguments]
//! ```
//!
//! # Commands
//!
//! - `init`              — Create a new `hudhud.toml` in the current directory
//! - `install`           — Resolve and install all dependencies from hudhud.toml
//! - `install <package>` — Add a package to dependencies and install it (alias: `add`)
//! - `remove <package>`  — Remove a package from dependencies (alias: `rm`)
//! - `list`              — List all installed dependencies (alias: `ls`)
//! - `help`              — Show help information

use std::env;

mod commands;
mod install;
mod lockfile;
mod manifest;
mod resolve;

pub(crate) use commands::*;
pub(crate) use lockfile::*;
pub(crate) use manifest::*;
pub(crate) use resolve::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MANIFEST: &str = "hudhud.toml";
const LOCKFILE: &str = "hudhud.lock";
const PACKAGES_SOURCE_DIR: &str = "packages";
const INSTALL_DIR: &str = ".hudpkg";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "init" => cmd_init(),
        "install" | "add" => {
            if args.len() >= 3 {
                let version = args.get(3).map(|s| s.as_str());
                cmd_install_package(&args[2], version);
            } else {
                cmd_install_all();
            }
        }
        "remove" | "rm" => {
            if args.len() < 3 {
                eprintln!("Error: package name required");
                eprintln!();
                eprintln!("Usage: hudpkg remove <package>");
                std::process::exit(1);
            }
            cmd_remove(&args[2]);
        }
        "list" | "ls" => cmd_list(),
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-V" => print_version(),
        other => {
            eprintln!("Error: unknown command '{}'", other);
            eprintln!();
            eprintln!("Run 'hudpkg help' to see available commands.");
            std::process::exit(1);
        }
    }
}
