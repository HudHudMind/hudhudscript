#![allow(dead_code)]
use regex::Regex;
use std::path::Path;

/// A parsed C++ header file.
#[derive(Debug, Clone, Default)]
pub struct CppHeader {
    pub includes: Vec<String>,
    pub namespaces: Vec<String>,
    pub classes: Vec<CppClass>,
    pub functions: Vec<CppFunction>,
    pub enums: Vec<CppEnum>,
}

/// A C++ class or struct definition.
#[derive(Debug, Clone, Default)]
pub struct CppClass {
    pub name: String,
    pub parent: Option<String>,
    pub methods: Vec<CppFunction>,
    pub fields: Vec<CppField>,
    pub is_struct: bool,
}

/// A C++ function or method.
#[derive(Debug, Clone, Default)]
pub struct CppFunction {
    pub name: String,
    pub return_type: String,
    pub params: Vec<(String, String)>, // (type, name)
    pub is_const: bool,
    pub is_virtual: bool,
    pub is_static: bool,
}

/// A C++ field (member variable).
#[derive(Debug, Clone, Default)]
pub struct CppField {
    pub name: String,
    pub field_type: String,
    pub is_static: bool,
    pub is_const: bool,
}

/// A C++ enum definition.
#[derive(Debug, Clone, Default)]
pub struct CppEnum {
    pub name: String,
    pub variants: Vec<String>,
    pub is_class: bool,
}

/// Parse a C++ header file.
pub fn parse_header(path: &Path) -> anyhow::Result<CppHeader> {
    let content = std::fs::read_to_string(path)?;
    parse_header_content(&content)
}

/// Parse C++ header content from a string.
pub fn parse_header_content(content: &str) -> anyhow::Result<CppHeader> {
    let mut header = CppHeader::default();

    // Strip single-line and multi-line comments for cleaner parsing
    let stripped = strip_comments(content);

    parse_includes(&stripped, &mut header);
    parse_namespaces(&stripped, &mut header);
    parse_enums(&stripped, &mut header);
    parse_classes(&stripped, &mut header);
    parse_free_functions(&stripped, &mut header);

    Ok(header)
}

pub fn strip_comments(content: &str) -> String {
    // Remove multi-line comments
    let re_block = Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    let no_block = re_block.replace_all(content, "");
    // Remove single-line comments
    let re_line = Regex::new(r"//[^\n]*").unwrap();
    re_line.replace_all(&no_block, "").to_string()
}

fn parse_includes(content: &str, header: &mut CppHeader) {
    let re = Regex::new(r#"#include\s*[<"]([^>"]+)[>"]"#).unwrap();
    for caps in re.captures_iter(content) {
        header.includes.push(caps[1].to_string());
    }
}

fn parse_namespaces(content: &str, header: &mut CppHeader) {
    let re = Regex::new(r"namespace\s+(\w+)\s*\{").unwrap();
    for caps in re.captures_iter(content) {
        let ns = caps[1].to_string();
        if !header.namespaces.contains(&ns) {
            header.namespaces.push(ns);
        }
    }
}

fn parse_enums(content: &str, header: &mut CppHeader) {
    // enum class Name { A, B, C }; or enum Name { A, B, C };
    let re = Regex::new(r"enum\s+(class\s+)?(\w+)\s*\{([^}]*)\}").unwrap();
    for caps in re.captures_iter(content) {
        let is_class = caps.get(1).is_some();
        let name = caps[2].to_string();
        let body = &caps[3];

        let variants: Vec<String> = body
            .split(',')
            .map(|s| {
                // Remove value assignments like "A = 0"
                let s = s.trim();
                if let Some(idx) = s.find('=') {
                    s[..idx].trim().to_string()
                } else {
                    s.to_string()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        header.enums.push(CppEnum {
            name,
            variants,
            is_class,
        });
    }
}

fn parse_classes(content: &str, header: &mut CppHeader) {
    // Match class or struct definitions
    let re = Regex::new(
        r"(?m)(class|struct)\s+(\w+)(?:\s*:\s*(?:public|protected|private)\s+(\w+))?\s*\{",
    )
    .unwrap();

    for caps in re.captures_iter(content) {
        // Skip "enum class" matches
        let match_start = caps.get(0).unwrap().start();
        if match_start >= 5 && &content[match_start - 5..match_start] == "enum " {
            continue;
        }

        let is_struct = &caps[1] == "struct";
        let name = caps[2].to_string();
        let parent = caps.get(3).map(|m| m.as_str().to_string());

        // Find the matching closing brace
        let match_end = caps.get(0).unwrap().end();
        let body = extract_brace_body(content, match_end);

        let mut class = CppClass {
            name,
            parent,
            methods: Vec::new(),
            fields: Vec::new(),
            is_struct,
        };

        parse_class_body(&body, &mut class);
        header.classes.push(class);
    }
}

/// Extract the body between matching braces, starting after the opening brace.
pub fn extract_brace_body(content: &str, start: usize) -> String {
    let bytes = content.as_bytes();
    let mut depth = 1;
    let mut pos = start;

    while pos < bytes.len() && depth > 0 {
        match bytes[pos] {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            pos += 1;
        }
    }

    if pos <= content.len() && start <= pos {
        content[start..pos].to_string()
    } else {
        String::new()
    }
}

fn parse_class_body(body: &str, class: &mut CppClass) {
    // Parse methods and fields from the class body
    // Split by lines and process each
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty()
            || line == "public:"
            || line == "private:"
            || line == "protected:"
            || line.starts_with("friend ")
            || line.starts_with("using ")
            || line.starts_with("typedef ")
        {
            continue;
        }

        // Try to parse as a method (has parentheses)
        if let Some(func) = try_parse_method(line) {
            class.methods.push(func);
        } else if let Some(field) = try_parse_field(line) {
            class.fields.push(field);
        }
    }
}

pub fn try_parse_method(line: &str) -> Option<CppFunction> {
    // Skip template lines, preprocessor, braces, destructors
    if line.starts_with("template")
        || line.starts_with('#')
        || line == "{"
        || line == "}"
        || line.starts_with("~")
    {
        return None;
    }

    // Match: [virtual] [static] ReturnType name(params) [const] [= 0|override|final];
    let re = Regex::new(
        r"^(virtual\s+)?(static\s+)?([\w:&*<>\s]+?)\s+(\w+)\s*\(([^)]*)\)\s*(const)?\s*(?:=\s*(?:0|default|delete))?\s*(?:override|final)?\s*;?\s*$",
    ).unwrap();

    let caps = re.captures(line)?;

    let is_virtual = caps.get(1).is_some();
    let is_static = caps.get(2).is_some();
    let return_type = caps[3].trim().to_string();
    let name = caps[4].to_string();
    let params_str = caps[5].trim();
    let is_const = caps.get(6).is_some();

    // Don't treat constructors as methods (return type same as name or empty)
    if return_type.is_empty() {
        return None;
    }

    let params = parse_params(params_str);

    Some(CppFunction {
        name,
        return_type,
        params,
        is_const,
        is_virtual,
        is_static,
    })
}

pub fn parse_params(params_str: &str) -> Vec<(String, String)> {
    if params_str.is_empty() || params_str == "void" {
        return Vec::new();
    }

    params_str
        .split(',')
        .filter_map(|p| {
            let p = p.trim();
            if p.is_empty() {
                return None;
            }
            // Remove default values
            let p = if let Some(idx) = p.find('=') {
                p[..idx].trim()
            } else {
                p
            };
            // Split into type and name: last word is name, rest is type
            let parts: Vec<&str> = p.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts
                    .last()
                    .unwrap()
                    .trim_start_matches('&')
                    .trim_start_matches('*');
                let type_parts = &parts[..parts.len() - 1];
                let ty = type_parts.join(" ");
                Some((ty, name.to_string()))
            } else if parts.len() == 1 {
                // Just a type with no name (forward decl style)
                Some((parts[0].to_string(), String::new()))
            } else {
                None
            }
        })
        .collect()
}

pub fn try_parse_field(line: &str) -> Option<CppField> {
    // Skip various non-field lines
    if line.starts_with("template")
        || line.starts_with('#')
        || line.starts_with("//")
        || line == "{"
        || line == "}"
        || line.contains('(')
        || line.starts_with("~")
        || line.starts_with("enum ")
        || line.starts_with("struct ")
        || line.starts_with("class ")
    {
        return None;
    }

    // Match: [static] [const] Type name [= value];
    let re =
        Regex::new(r"^(static\s+)?(const\s+)?([\w:&*<>\s]+?)\s+(\w+)\s*(?:=\s*[^;]*)?\s*;\s*$")
            .unwrap();

    let caps = re.captures(line)?;

    let is_static = caps.get(1).is_some();
    let is_const = caps.get(2).is_some();
    let field_type = caps[3].trim().to_string();
    let name = caps[4].to_string();

    // Filter out noise
    if field_type.is_empty() || name.is_empty() {
        return None;
    }

    Some(CppField {
        name,
        field_type,
        is_static,
        is_const,
    })
}

fn parse_free_functions(content: &str, header: &mut CppHeader) {
    // Collect class/struct names to avoid matching their methods
    let class_names: Vec<&str> = header.classes.iter().map(|c| c.name.as_str()).collect();

    // Match function declarations outside of class bodies
    // This is a simplified approach: find function-looking lines at top level
    let re =
        Regex::new(r"(?m)^(?:extern\s+)?(inline\s+)?([\w:&*<>\s]+?)\s+(\w+)\s*\(([^)]*)\)\s*;")
            .unwrap();

    for caps in re.captures_iter(content) {
        let return_type = caps[2].trim().to_string();
        let name = caps[3].to_string();
        let params_str = caps[4].trim();

        // Skip if it looks like a class/struct/enum definition keyword
        if matches!(
            return_type.as_str(),
            "class" | "struct" | "enum" | "namespace" | "using" | "typedef"
        ) {
            continue;
        }

        // Skip if the name matches a known class (constructor)
        if class_names.contains(&name.as_str()) {
            continue;
        }

        let params = parse_params(params_str);

        header.functions.push(CppFunction {
            name,
            return_type,
            params,
            is_const: false,
            is_virtual: false,
            is_static: false,
        });
    }
}
