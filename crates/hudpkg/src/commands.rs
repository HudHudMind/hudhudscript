use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    load_lockfile, load_manifest, resolve_all, save_manifest, write_lockfile, DependencyValue,
    HudHudManifest, INSTALL_DIR, LOCKFILE, MANIFEST, VERSION,
};

pub(crate) fn cmd_init() {
    if Path::new(MANIFEST).exists() {
        println!("hudhud.toml already exists.");
        return;
    }
    let manifest = HudHudManifest::default();
    save_manifest(&manifest);
    println!("Created hudhud.toml");
}

/// `hudpkg install` — resolve all dependencies from the manifest.
pub(crate) fn cmd_install_all() {
    let manifest = load_manifest();
    if manifest.dependencies.is_empty() {
        println!("No dependencies to install.");
        return;
    }

    println!("Resolving {} dependencies...", manifest.dependencies.len());

    match resolve_all(&manifest.dependencies) {
        Ok(resolved) => {
            crate::install::install_resolved(&resolved);
            write_lockfile(&resolved);
            println!();
            println!("Installed {} packages into {}", resolved.len(), INSTALL_DIR);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// `hudpkg install <package> [version]` — add to manifest and resolve.
pub(crate) fn cmd_install_package(pkg: &str, version: Option<&str>) {
    let mut manifest = load_manifest();

    if manifest.dependencies.contains_key(pkg) && version.is_none() {
        println!("{} is already in dependencies, resolving...", pkg);
    } else {
        let ver = version.unwrap_or("*");
        manifest
            .dependencies
            .insert(pkg.to_string(), DependencyValue::Simple(ver.to_string()));
        save_manifest(&manifest);
        println!("+ {} = \"{}\" added to dependencies.", pkg, ver);
    }

    match resolve_all(&manifest.dependencies) {
        Ok(resolved) => {
            crate::install::install_resolved(&resolved);
            write_lockfile(&resolved);
            println!();
            println!("Installed {} packages into {}", resolved.len(), INSTALL_DIR);
        }
        Err(e) => {
            eprintln!("Error resolving dependencies: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_remove(pkg: &str) {
    let mut manifest = load_manifest();
    if manifest.dependencies.remove(pkg).is_some() {
        save_manifest(&manifest);
        println!("- {} removed from dependencies.", pkg);

        let install_path = PathBuf::from(INSTALL_DIR).join(pkg);
        if install_path.exists() {
            if let Err(e) = fs::remove_dir_all(&install_path) {
                eprintln!(
                    "Warning: could not remove {}: {}",
                    install_path.display(),
                    e
                );
            } else {
                println!("  Removed {}", install_path.display());
            }
        }

        if !manifest.dependencies.is_empty() {
            match resolve_all(&manifest.dependencies) {
                Ok(resolved) => write_lockfile(&resolved),
                Err(e) => eprintln!("Warning: could not update lockfile: {}", e),
            }
        } else {
            let _ = fs::remove_file(LOCKFILE);
        }
    } else {
        println!("{} is not in dependencies.", pkg);
    }
}

pub(crate) fn cmd_list() {
    let manifest = load_manifest();
    if manifest.dependencies.is_empty() {
        println!("No dependencies installed.");
        return;
    }

    println!("Dependencies (from {}):", MANIFEST);
    for (name, dep) in &manifest.dependencies {
        let status = if PathBuf::from(INSTALL_DIR).join(name).exists() {
            "installed"
        } else {
            "not installed"
        };
        println!("  {} = \"{}\" [{}]", name, dep.version_str(), status);
    }

    if let Ok(lockfile) = load_lockfile() {
        println!();
        println!("Locked versions (from {}):", LOCKFILE);
        for (name, locked) in &lockfile.packages {
            println!("  {} = {} ({})", name, locked.version, locked.source);
        }
    }
}

pub(crate) fn print_version() {
    println!("hudpkg {}", VERSION);
}

pub(crate) fn print_usage() {
    println!("hudpkg {} — HudHud Package Manager", VERSION);
    println!();
    println!("Manage dependencies for HudHudScript projects via hudhud.toml.");
    println!();
    println!("USAGE:");
    println!("  hudpkg <command> [arguments]");
    println!();
    println!("COMMANDS:");
    println!("  init                Initialize a new hudhud.toml manifest");
    println!("  install             Resolve and install all dependencies");
    println!("  install <package>   Add a package and install it (alias: add)");
    println!("  remove <package>    Remove a package from dependencies (alias: rm)");
    println!("  list                List all dependencies and status (alias: ls)");
    println!("  help                Show this help message (also: --help, -h)");
    println!("  version             Show version information (also: --version, -V)");
    println!();
    println!("PACKAGE SOURCES:");
    println!("  Packages are resolved from the local packages/ directory.");
    println!("  Each package should be a subdirectory with a hudhud.toml:");
    println!();
    println!("    packages/");
    println!("      hudhud-http/");
    println!("        hudhud.toml");
    println!("        src/");
    println!("          main.hud");
    println!("      hudhud-fs-0.2.0/");
    println!("        hudhud.toml");
    println!("        ...");
    println!();
    println!("  Installed packages are copied to .hudpkg/ and locked in hudhud.lock.");
    println!();
    println!("EXAMPLES:");
    println!("  hudpkg init                    # Start a new project");
    println!("  hudpkg install                 # Install all deps from hudhud.toml");
    println!("  hudpkg install hudhud-http     # Add and install a package");
    println!("  hudpkg install hudhud-fs ^0.2  # Add with version constraint");
    println!("  hudpkg remove hudhud-fs        # Remove a dependency");
    println!("  hudpkg list                    # Show current dependencies");
}
