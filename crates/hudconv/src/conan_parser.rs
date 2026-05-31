use regex::Regex;
use std::path::Path;

/// A parsed Conan project extracted from conanfile.py.
#[derive(Debug, Clone, Default)]
pub struct ConanProject {
    pub name: String,
    pub version: String,
    pub requires: Vec<String>,
    pub tool_requires: Vec<String>,
    pub generators: Vec<String>,
    pub options: Vec<(String, Vec<String>)>,
}

/// Parse a conanfile.py file and return a `ConanProject`.
pub fn parse_conan(path: &Path) -> anyhow::Result<ConanProject> {
    let content = std::fs::read_to_string(path)?;
    parse_conan_content(&content)
}

/// Parse conanfile.py content from a string.
pub fn parse_conan_content(content: &str) -> anyhow::Result<ConanProject> {
    let mut project = ConanProject::default();

    parse_name(content, &mut project);
    parse_version(content, &mut project);
    parse_requires(content, &mut project);
    parse_tool_requires(content, &mut project);
    parse_generators(content, &mut project);
    parse_options(content, &mut project);

    Ok(project)
}

fn parse_name(content: &str, project: &mut ConanProject) {
    let re = Regex::new(r#"name\s*=\s*["']([^"']+)["']"#).unwrap();
    if let Some(caps) = re.captures(content) {
        project.name = caps[1].to_string();
    }
}

fn parse_version(content: &str, project: &mut ConanProject) {
    let re = Regex::new(r#"version\s*=\s*["']([^"']+)["']"#).unwrap();
    if let Some(caps) = re.captures(content) {
        project.version = caps[1].to_string();
    }
}

/// Extract quoted strings from a Python tuple/list expression.
pub fn extract_quoted_strings(text: &str) -> Vec<String> {
    let re = Regex::new(r#"["']([^"']+)["']"#).unwrap();
    re.captures_iter(text).map(|c| c[1].to_string()).collect()
}

fn parse_requires(content: &str, project: &mut ConanProject) {
    // Match: requires = ("pkg/ver", ...) or requires = "pkg/ver"
    // Also match self.requires("pkg/ver") method calls
    let re_assign =
        Regex::new(r#"(?m)^\s*requires\s*=\s*(\([\s\S]*?\)|"[^"]*"|'[^']*'|\[[\s\S]*?\])"#)
            .unwrap();
    if let Some(caps) = re_assign.captures(content) {
        project.requires = extract_quoted_strings(&caps[1]);
    }

    // Also capture self.requires(...) calls
    let re_method = Regex::new(r#"self\.requires\(\s*["']([^"']+)["']"#).unwrap();
    for caps in re_method.captures_iter(content) {
        let dep = caps[1].to_string();
        if !project.requires.contains(&dep) {
            project.requires.push(dep);
        }
    }
}

fn parse_tool_requires(content: &str, project: &mut ConanProject) {
    let re_assign =
        Regex::new(r#"(?m)^\s*tool_requires\s*=\s*(\([\s\S]*?\)|"[^"]*"|'[^']*'|\[[\s\S]*?\])"#)
            .unwrap();
    if let Some(caps) = re_assign.captures(content) {
        project.tool_requires = extract_quoted_strings(&caps[1]);
    }

    let re_method = Regex::new(r#"self\.tool_requires\(\s*["']([^"']+)["']"#).unwrap();
    for caps in re_method.captures_iter(content) {
        let dep = caps[1].to_string();
        if !project.tool_requires.contains(&dep) {
            project.tool_requires.push(dep);
        }
    }
}

fn parse_generators(content: &str, project: &mut ConanProject) {
    // generators = "CMakeDeps", "CMakeToolchain"
    // or generators = ("CMakeDeps", "CMakeToolchain")
    let re = Regex::new(r#"(?m)^\s*generators\s*=\s*(.+)"#).unwrap();
    if let Some(caps) = re.captures(content) {
        project.generators = extract_quoted_strings(&caps[1]);
    }
}

fn parse_options(content: &str, project: &mut ConanProject) {
    // options = {"shared": [True, False], "fPIC": [True, False]}
    let re = Regex::new(r#"(?m)^\s*options\s*=\s*\{([\s\S]*?)\}"#).unwrap();
    if let Some(caps) = re.captures(content) {
        let inner = &caps[1];
        // Parse individual option entries: "key": [val1, val2, ...]
        let opt_re = Regex::new(r#"["'](\w+)["']\s*:\s*\[([^\]]*)\]"#).unwrap();
        for opt_caps in opt_re.captures_iter(inner) {
            let key = opt_caps[1].to_string();
            let values_str = &opt_caps[2];
            let values: Vec<String> = values_str
                .split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            project.options.push((key, values));
        }
    }
}
