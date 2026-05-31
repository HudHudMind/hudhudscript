#![allow(dead_code)]
use regex::Regex;
use std::path::Path;

/// Target type for a CMake library or executable.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetType {
    SharedLib,
    StaticLib,
    Executable,
}

/// A parsed CMake target (library or executable).
#[derive(Debug, Clone)]
pub struct CMakeTarget {
    pub name: String,
    pub target_type: TargetType,
    pub sources: Vec<String>,
    pub link_libraries: Vec<String>,
}

/// A parsed CMake project extracted from CMakeLists.txt.
#[derive(Debug, Clone, Default)]
pub struct CMakeProject {
    pub name: String,
    pub version: Option<String>,
    pub libraries: Vec<CMakeTarget>,
    pub executables: Vec<CMakeTarget>,
    pub dependencies: Vec<String>,
    pub include_dirs: Vec<String>,
}

/// Strip CMake-style comments (lines starting with #) from content.
pub fn strip_comments(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if let Some(idx) = line.find('#') {
                // Only strip if # is not inside a string
                let before = &line[..idx];
                let open_quotes = before.chars().filter(|&c| c == '"').count();
                if open_quotes % 2 == 0 {
                    before
                } else {
                    line
                }
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse a CMakeLists.txt file and return a `CMakeProject`.
pub fn parse_cmake(path: &Path) -> anyhow::Result<CMakeProject> {
    let content = std::fs::read_to_string(path)?;
    parse_cmake_content(&content)
}

/// Parse CMake content from a string.
pub fn parse_cmake_content(content: &str) -> anyhow::Result<CMakeProject> {
    let content = strip_comments(content);
    let mut project = CMakeProject::default();

    parse_project(&content, &mut project);
    parse_libraries(&content, &mut project);
    parse_executables(&content, &mut project);
    parse_link_libraries(&content, &mut project);
    parse_find_packages(&content, &mut project);
    parse_include_directories(&content, &mut project);

    Ok(project)
}

fn parse_project(content: &str, project: &mut CMakeProject) {
    // project(Name VERSION x.y.z ...) or project(Name ...)
    let re = Regex::new(r"(?i)project\s*\(\s*(\w+)(?:\s+VERSION\s+([\d]+(?:\.[\d]+)*))?").unwrap();
    if let Some(caps) = re.captures(content) {
        project.name = caps[1].to_string();
        if let Some(ver) = caps.get(2) {
            project.version = Some(ver.as_str().to_string());
        }
    }
}

/// Extract arguments from a CMake command call, handling multi-line parentheses.
pub fn extract_command_args(content: &str, command: &str) -> Vec<String> {
    let pattern = format!(r"(?i)\b{}\s*\(", regex::escape(command));
    let re = Regex::new(&pattern).unwrap();
    let mut results = Vec::new();

    for mat in re.find_iter(content) {
        let start = mat.end(); // right after the opening paren
        let bytes = content.as_bytes();
        let mut depth = 1;
        let mut end = start;
        while end < bytes.len() && depth > 0 {
            match bytes[end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                end += 1;
            }
        }
        if depth == 0 {
            let inner = &content[start..end];
            results.push(inner.to_string());
        }
    }
    results
}

/// Split CMake arguments (whitespace/newline separated, respecting quotes).
pub fn split_cmake_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in args.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(trimmed);
                }
                current.clear();
            }
            c => {
                current.push(c);
            }
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }
    result
}

fn parse_libraries(content: &str, project: &mut CMakeProject) {
    for args_str in extract_command_args(content, "add_library") {
        let args = split_cmake_args(&args_str);
        if args.is_empty() {
            continue;
        }
        let name = args[0].clone();
        let mut target_type = TargetType::StaticLib;
        let mut source_start = 1;

        if args.len() > 1 {
            match args[1].to_uppercase().as_str() {
                "SHARED" => {
                    target_type = TargetType::SharedLib;
                    source_start = 2;
                }
                "STATIC" => {
                    target_type = TargetType::StaticLib;
                    source_start = 2;
                }
                "INTERFACE" | "OBJECT" | "MODULE" | "IMPORTED" | "ALIAS" => {
                    source_start = 2;
                }
                _ => {}
            }
        }

        let sources: Vec<String> = args[source_start..].to_vec();

        project.libraries.push(CMakeTarget {
            name,
            target_type,
            sources,
            link_libraries: Vec::new(),
        });
    }
}

fn parse_executables(content: &str, project: &mut CMakeProject) {
    for args_str in extract_command_args(content, "add_executable") {
        let args = split_cmake_args(&args_str);
        if args.is_empty() {
            continue;
        }
        let name = args[0].clone();
        let sources: Vec<String> = args[1..].to_vec();

        project.executables.push(CMakeTarget {
            name,
            target_type: TargetType::Executable,
            sources,
            link_libraries: Vec::new(),
        });
    }
}

fn parse_link_libraries(content: &str, project: &mut CMakeProject) {
    for args_str in extract_command_args(content, "target_link_libraries") {
        let args = split_cmake_args(&args_str);
        if args.len() < 2 {
            continue;
        }
        let target_name = &args[0];
        let libs: Vec<String> = args[1..]
            .iter()
            .filter(|a| {
                !matches!(
                    a.to_uppercase().as_str(),
                    "PUBLIC" | "PRIVATE" | "INTERFACE"
                )
            })
            .cloned()
            .collect();

        // Try to find the target in libraries or executables and add the libs
        let mut found = false;
        for lib in &mut project.libraries {
            if lib.name == *target_name {
                lib.link_libraries.extend(libs.clone());
                found = true;
                break;
            }
        }
        if !found {
            for exe in &mut project.executables {
                if exe.name == *target_name {
                    exe.link_libraries.extend(libs.clone());
                    found = true;
                    break;
                }
            }
        }
        // If target not found yet, still record the libraries as project-level dependencies
        if !found {
            for lib in &libs {
                if !project.dependencies.contains(lib) {
                    project.dependencies.push(lib.clone());
                }
            }
        }
    }
}

fn parse_find_packages(content: &str, project: &mut CMakeProject) {
    for args_str in extract_command_args(content, "find_package") {
        let args = split_cmake_args(&args_str);
        if args.is_empty() {
            continue;
        }
        let dep = args[0].clone();
        if !project.dependencies.contains(&dep) {
            project.dependencies.push(dep);
        }
    }
}

fn parse_include_directories(content: &str, project: &mut CMakeProject) {
    for args_str in extract_command_args(content, "target_include_directories") {
        let args = split_cmake_args(&args_str);
        // First arg is target name, then scope keywords, then dirs
        for arg in args.iter().skip(1) {
            match arg.to_uppercase().as_str() {
                "PUBLIC" | "PRIVATE" | "INTERFACE" | "SYSTEM" | "BEFORE" | "AFTER" => continue,
                _ => {
                    if !project.include_dirs.contains(arg) {
                        project.include_dirs.push(arg.clone());
                    }
                }
            }
        }
    }

    // Also parse plain include_directories()
    for args_str in extract_command_args(content, "include_directories") {
        let args = split_cmake_args(&args_str);
        for arg in &args {
            match arg.to_uppercase().as_str() {
                "SYSTEM" | "BEFORE" | "AFTER" => continue,
                _ => {
                    if !project.include_dirs.contains(arg) {
                        project.include_dirs.push(arg.clone());
                    }
                }
            }
        }
    }
}
