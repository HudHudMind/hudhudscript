use crate::vm::VM;
use hudhudscript_bytecode::error::compile_codes;
use hudhudscript_bytecode::error::CompileResult;
use hudhudscript_bytecode::Value16;

impl crate::vm::VM {
    pub(crate) fn call_toml_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_serial::toml_ops::TomlMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── YAML methods (v0.4.38 — #650) ─────────────────────────────────

    pub(crate) fn call_yaml_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_serial::yaml_ops::YamlMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── CSV methods (v0.4.38 — #650) ──────────────────────────────────

    pub(crate) fn call_csv_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_serial::csv_ops::CsvMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── INI methods (v0.4.38 — #650) ──────────────────────────────────

    pub(crate) fn call_ini_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_serial::ini_ops::IniMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── Base64 methods (v0.4.38 — #651) ───────────────────────────────

    pub(crate) fn call_base64_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_encoding::Base64MethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── Hex methods (v0.4.38 — #651) ──────────────────────────────────

    pub(crate) fn call_hex_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_encoding::HexMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── URL encoding methods (v0.4.38 — #651) ────────────────────────

    pub(crate) fn call_url_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_encoding::UrlMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── UUID methods (v0.4.38 — #652) ─────────────────────────────────

    pub(crate) fn call_uuid_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_encoding::UuidMethodId>()?;
            id.dispatch(&args)
        }
    }

    // ── Path methods (v0.4.38 — #663) ──────────────────────────────────

    pub(crate) fn call_path_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_fs::path::dispatch(method, &args)
    }

    // ── Temp methods (v0.4.38 — #664) ──────────────────────────────────

    pub(crate) fn call_temp_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_fs::temp::dispatch(method, &args)
    }

    // ── URLParser methods (v0.4.38 — #665) ─────────────────────────────

    pub(crate) fn call_url_parser_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        {
            let id = method.parse::<hudhud_url::url_parser::UrlParserMethodId>()?;
            hudhud_url::url_parser::dispatch(id, &args)
        }
    }

    // ── Glob methods (v0.4.38 — #666) ──────────────────────────────────

    pub(crate) fn call_glob_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        hudhud_fs::glob_ops::dispatch(method, &args)
    }

    /// Set module method dispatch (Set.new) — Issue #653
    pub(crate) fn call_set_module_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        match method {
            "new" => {
                if args.is_empty() {
                    Ok(Value16::set(Vec::new()))
                } else if let Some(arr) = args[0].as_array() {
                    let mut items = Vec::new();
                    for v in arr {
                        if !items.iter().any(|x| self.values_equal(x, v)) {
                            items.push(v.clone());
                        }
                    }
                    Ok(Value16::set(items))
                } else {
                    Ok(Value16::set(vec![args[0].clone()]))
                }
            }
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown Set method: {}",
                method
            ))),
        }
    }

    /// Map module method dispatch (Map.new) — Issue #654
    pub(crate) fn call_map_module_method(
        &self,
        method: &str,
        args: Vec<Value16>,
    ) -> CompileResult<Value16> {
        match method {
            "new" => {
                if args.is_empty() {
                    Ok(Value16::map(Vec::new()))
                } else if let Some(arr) = args[0].as_array() {
                    let mut pairs: Vec<(Value16, Value16)> = Vec::new();
                    for item in arr {
                        if let Some(pair) = item.as_array() {
                            if pair.len() >= 2 {
                                pairs.retain(|(k, _): &(Value16, Value16)| {
                                    !self.values_equal(k, &pair[0])
                                });
                                pairs.push((pair[0].clone(), pair[1].clone()));
                            }
                        }
                    }
                    Ok(Value16::map(pairs))
                } else if let Some(obj) = args[0].as_object() {
                    let pairs: Vec<(Value16, Value16)> = obj
                        .iter()
                        .map(|(k, v)| (Value16::string(k.clone()), v.clone()))
                        .collect();
                    Ok(Value16::map(pairs))
                } else {
                    Ok(Value16::map(Vec::new()))
                }
            }
            _ => Err(compile_codes::runtime_error(format!(
                "Unknown Map method: {}",
                method
            ))),
        }
    }
}
