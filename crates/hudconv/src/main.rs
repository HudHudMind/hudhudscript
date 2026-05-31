use hudconv::cmake_parser;
use hudconv::conan_parser;
use hudconv::header_parser;
use hudconv::hud_generator;
use hudconv::manifest_generator;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "hudconv",
    about = "Convert C++/CMake/Conan projects to HudHudScript",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a CMake project (CMakeLists.txt + optional conanfile.py + headers)
    Cmake {
        /// Path to the project directory containing CMakeLists.txt
        path: PathBuf,

        /// Output directory for generated files
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Preview output without writing files
        #[arg(long)]
        dry_run: bool,
    },

    /// Convert a Conan project (conanfile.py)
    Conan {
        /// Path to the project directory containing conanfile.py
        path: PathBuf,

        /// Output directory for generated files
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Preview output without writing files
        #[arg(long)]
        dry_run: bool,
    },

    /// Convert a single C++ header file
    Header {
        /// Path to the C++ header file
        file: PathBuf,

        /// Output directory for generated files
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Preview output without writing files
        #[arg(long)]
        dry_run: bool,

        /// Library name for native binding annotations
        #[arg(short, long, default_value = "native")]
        lib_name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Cmake {
            path,
            output,
            dry_run,
        } => convert_cmake(&path, output.as_deref(), dry_run),
        Commands::Conan {
            path,
            output,
            dry_run,
        } => convert_conan(&path, output.as_deref(), dry_run),
        Commands::Header {
            file,
            output,
            dry_run,
            lib_name,
        } => convert_header(&file, output.as_deref(), dry_run, &lib_name),
    }
}

fn convert_cmake(project_path: &Path, output: Option<&Path>, dry_run: bool) -> Result<()> {
    let cmake_path = project_path.join("CMakeLists.txt");
    if !cmake_path.exists() {
        anyhow::bail!("CMakeLists.txt not found in {}", project_path.display());
    }

    println!(
        "{} {}",
        "Parsing CMake project:".green().bold(),
        cmake_path.display()
    );

    let cmake_project =
        cmake_parser::parse_cmake(&cmake_path).context("Failed to parse CMakeLists.txt")?;

    println!(
        "  Project: {} (version: {})",
        cmake_project.name.cyan(),
        cmake_project
            .version
            .as_deref()
            .unwrap_or("unspecified")
            .cyan()
    );
    println!(
        "  Libraries: {}, Executables: {}, Dependencies: {}",
        cmake_project.libraries.len(),
        cmake_project.executables.len(),
        cmake_project.dependencies.len()
    );

    // Try to parse conanfile.py if present
    let conan_path = project_path.join("conanfile.py");
    let conan_project = if conan_path.exists() {
        println!(
            "{} {}",
            "Parsing Conan file:".green().bold(),
            conan_path.display()
        );
        let conan =
            conan_parser::parse_conan(&conan_path).context("Failed to parse conanfile.py")?;
        println!("  Requires: {}", conan.requires.len());
        Some(conan)
    } else {
        None
    };

    // Generate manifest
    let manifest =
        manifest_generator::generate_manifest(Some(&cmake_project), conan_project.as_ref());

    // Generate build script
    let build_script =
        manifest_generator::generate_build_script(Some(&cmake_project), conan_project.as_ref());

    // Find and convert headers
    let mut hud_files: Vec<(String, String)> = Vec::new();
    let lib_name = cmake_project
        .libraries
        .first()
        .map(|l| l.name.clone())
        .unwrap_or_else(|| cmake_project.name.clone());

    for dir in &cmake_project.include_dirs {
        let include_path = project_path.join(dir);
        if include_path.exists() {
            for entry in WalkDir::new(&include_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if is_cpp_header(path) {
                    if let Ok(header) = header_parser::parse_header(path) {
                        let hud_content = hud_generator::generate_hud(&header, &lib_name);
                        let stem = path.file_stem().unwrap().to_string_lossy();
                        hud_files.push((format!("{}.hud", stem), hud_content));
                    }
                }
            }
        }
    }

    // Also scan common header locations
    for subdir in &["include", "src", "headers"] {
        let scan_path = project_path.join(subdir);
        if scan_path.exists() && !cmake_project.include_dirs.iter().any(|d| d == *subdir) {
            for entry in WalkDir::new(&scan_path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if is_cpp_header(path) {
                    if let Ok(header) = header_parser::parse_header(path) {
                        let hud_content = hud_generator::generate_hud(&header, &lib_name);
                        let stem = path.file_stem().unwrap().to_string_lossy();
                        let filename = format!("{}.hud", stem);
                        if !hud_files.iter().any(|(n, _)| n == &filename) {
                            hud_files.push((filename, hud_content));
                        }
                    }
                }
            }
        }
    }

    if dry_run {
        println!("\n{}", "=== DRY RUN - Preview ===".yellow().bold());
        println!("\n--- hudhud.toml ---");
        println!("{}", manifest);
        println!("--- build.sh ---");
        println!("{}", build_script);
        for (name, content) in &hud_files {
            println!("--- {} ---", name);
            println!("{}", content);
        }
    } else {
        let out_dir = output.unwrap_or(project_path);
        write_file(out_dir, "hudhud.toml", &manifest)?;
        write_file(out_dir, "build.sh", &build_script)?;

        let hud_dir = out_dir.join("src");
        std::fs::create_dir_all(&hud_dir)?;
        for (name, content) in &hud_files {
            write_file(&hud_dir, name, content)?;
        }

        println!(
            "\n{} Generated {} files in {}",
            "Done!".green().bold(),
            2 + hud_files.len(),
            out_dir.display()
        );
    }

    Ok(())
}

fn convert_conan(project_path: &Path, output: Option<&Path>, dry_run: bool) -> Result<()> {
    let conan_path = project_path.join("conanfile.py");
    if !conan_path.exists() {
        anyhow::bail!("conanfile.py not found in {}", project_path.display());
    }

    println!(
        "{} {}",
        "Parsing Conan project:".green().bold(),
        conan_path.display()
    );

    let conan_project =
        conan_parser::parse_conan(&conan_path).context("Failed to parse conanfile.py")?;

    println!(
        "  Package: {}/{}",
        conan_project.name.cyan(),
        conan_project.version.cyan()
    );
    println!("  Requires: {}", conan_project.requires.len());

    let manifest = manifest_generator::generate_manifest(None, Some(&conan_project));
    let build_script = manifest_generator::generate_build_script(None, Some(&conan_project));

    if dry_run {
        println!("\n{}", "=== DRY RUN - Preview ===".yellow().bold());
        println!("\n--- hudhud.toml ---");
        println!("{}", manifest);
        println!("--- build.sh ---");
        println!("{}", build_script);
    } else {
        let out_dir = output.unwrap_or(project_path);
        write_file(out_dir, "hudhud.toml", &manifest)?;
        write_file(out_dir, "build.sh", &build_script)?;

        println!(
            "\n{} Generated 2 files in {}",
            "Done!".green().bold(),
            out_dir.display()
        );
    }

    Ok(())
}

fn convert_header(
    file_path: &Path,
    output: Option<&Path>,
    dry_run: bool,
    lib_name: &str,
) -> Result<()> {
    if !file_path.exists() {
        anyhow::bail!("Header file not found: {}", file_path.display());
    }

    println!(
        "{} {}",
        "Parsing C++ header:".green().bold(),
        file_path.display()
    );

    let header = header_parser::parse_header(file_path).context("Failed to parse header file")?;

    println!(
        "  Classes: {}, Enums: {}, Functions: {}",
        header.classes.len(),
        header.enums.len(),
        header.functions.len()
    );

    let hud_content = hud_generator::generate_hud(&header, lib_name);
    let stem = file_path.file_stem().unwrap().to_string_lossy();
    let hud_filename = format!("{}.hud", stem);

    if dry_run {
        println!("\n{}", "=== DRY RUN - Preview ===".yellow().bold());
        println!("\n--- {} ---", hud_filename);
        println!("{}", hud_content);
    } else {
        let out_dir = output.unwrap_or_else(|| file_path.parent().unwrap());
        write_file(out_dir, &hud_filename, &hud_content)?;

        println!(
            "\n{} Generated {} in {}",
            "Done!".green().bold(),
            hud_filename,
            out_dir.display()
        );
    }

    Ok(())
}

fn write_file(dir: &Path, filename: &str, content: &str) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(filename);
    std::fs::write(&path, content)?;
    println!("  {} {}", "Wrote:".green(), path.display());
    Ok(())
}

fn is_cpp_header(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("h" | "hpp" | "hxx" | "hh")
    )
}
