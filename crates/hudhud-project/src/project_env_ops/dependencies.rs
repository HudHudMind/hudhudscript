use std::path::Path;

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::{make_dep, read_file, require_string};

pub fn dependencies(args: &[Value16]) -> HudHudResult<Value16> {
    let dir = require_string(args, 0, "project.dependencies")?;
    let base = Path::new(&dir);

    let pkg_json = base.join("package.json");
    if pkg_json.exists() {
        return parse_package_json(&pkg_json);
    }
    let req_txt = base.join("requirements.txt");
    if req_txt.exists() {
        return parse_requirements_txt(&req_txt);
    }
    let cargo_toml = base.join("Cargo.toml");
    if cargo_toml.exists() {
        return parse_cargo_toml(&cargo_toml);
    }
    let go_mod = base.join("go.mod");
    if go_mod.exists() {
        return parse_go_mod(&go_mod);
    }
    let gemfile = base.join("Gemfile");
    if gemfile.exists() {
        return parse_gemfile(&gemfile);
    }

    Ok(Value16::array(vec![]))
}

fn parse_package_json(path: &Path) -> HudHudResult<Value16> {
    let content = read_file(path, "project.dependencies")?;
    let mut deps = Vec::new();

    for section in ["dependencies", "devDependencies"] {
        let needle = format!("\"{}\"", section);
        if let Some(start) = content.find(&needle) {
            if let Some(brace_start) = content[start..].find('{') {
                let block_start = start + brace_start;
                let mut depth: i32 = 0;
                let mut block_end = block_start;
                for (i, ch) in content[block_start..].char_indices() {
                    match ch {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                block_end = block_start + i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let block = &content[block_start..block_end];
                let mut chars = block.chars().peekable();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        let mut key = String::new();
                        for c in chars.by_ref() {
                            if c == '"' {
                                break;
                            }
                            key.push(c);
                        }
                        let mut found_colon = false;
                        for c in chars.by_ref() {
                            if c == ':' {
                                found_colon = true;
                            }
                            if found_colon && c == '"' {
                                break;
                            }
                        }
                        if found_colon {
                            let mut val = String::new();
                            for c in chars.by_ref() {
                                if c == '"' {
                                    break;
                                }
                                val.push(c);
                            }
                            if !key.is_empty() {
                                deps.push(make_dep(&key, &val));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Value16::array(deps))
}

fn parse_requirements_txt(path: &Path) -> HudHudResult<Value16> {
    let content = read_file(path, "project.dependencies")?;
    let mut deps = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
            continue;
        }
        let separators = ["==", ">=", "<=", "~=", "!=", ">", "<"];
        let mut found = false;
        for sep in separators {
            if let Some((name, ver)) = line.split_once(sep) {
                deps.push(make_dep(name.trim(), ver.trim()));
                found = true;
                break;
            }
        }
        if !found {
            deps.push(make_dep(line, "*"));
        }
    }
    Ok(Value16::array(deps))
}

fn parse_cargo_toml(path: &Path) -> HudHudResult<Value16> {
    let content = read_file(path, "project.dependencies")?;
    let mut deps = Vec::new();
    let mut in_deps_section = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with('[') {
            in_deps_section = trimmed == "[dependencies]" || trimmed == "[dev-dependencies]";
            continue;
        }
        if in_deps_section {
            if trimmed.is_empty() {
                continue;
            }
            if let Some((name_part, rest_part)) = trimmed.split_once('=') {
                let name = name_part.trim();
                let rest = rest_part.trim();
                if rest.starts_with('"') {
                    let ver = rest.trim_matches('"');
                    deps.push(make_dep(name, ver));
                } else if rest.starts_with('{') {
                    if let Some(vs) = rest.find("version") {
                        let after = &rest[vs + 7..];
                        if let Some(q1) = after.find('"') {
                            let after_q1 = &after[q1 + 1..];
                            if let Some(q2) = after_q1.find('"') {
                                let ver = &after_q1[..q2];
                                deps.push(make_dep(name, ver));
                            }
                        }
                    } else {
                        deps.push(make_dep(name, "*"));
                    }
                }
            }
        }
    }
    Ok(Value16::array(deps))
}

fn parse_go_mod(path: &Path) -> HudHudResult<Value16> {
    let content = read_file(path, "project.dependencies")?;
    let mut deps = Vec::new();
    let mut in_require = false;

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("require (") || trimmed == "require (" {
            in_require = true;
            continue;
        }
        if trimmed == ")" {
            in_require = false;
            continue;
        }
        if in_require {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push(make_dep(parts[0], parts[1]));
            }
        } else if trimmed.starts_with("require ") {
            let rest = trimmed.strip_prefix("require ").unwrap_or("");
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                deps.push(make_dep(parts[0], parts[1]));
            }
        }
    }
    Ok(Value16::array(deps))
}

fn parse_gemfile(path: &Path) -> HudHudResult<Value16> {
    let content = read_file(path, "project.dependencies")?;
    let mut deps = Vec::new();
    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if !trimmed.starts_with("gem ") {
            continue;
        }
        let rest = &trimmed[4..];
        let parts: Vec<&str> = rest.split(',').collect();
        if let Some(name_part) = parts.first() {
            let name = name_part
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            let version = parts
                .get(1)
                .map(|v| {
                    v.trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .trim()
                        .to_string()
                })
                .unwrap_or_else(|| "*".to_string());
            deps.push(make_dep(&name, &version));
        }
    }
    Ok(Value16::array(deps))
}
