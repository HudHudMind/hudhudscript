//! HudHud Web Template — Jinja2-style template engine.
//!
//! Provides `render()`, `render_file()`, `escape()` for the Web framework.
//! Does NOT invoke the HudHudScript VM (Kural 7c).

pub mod ast;
pub mod eval;
pub mod filters;
pub mod lexer;
pub mod parser;

use eval::{eval, Context, EvalConfig};
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};
use std::collections::HashMap;

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

fn type_error(expected: &str, got: &str, context: &str) -> Error {
    Error::new(
        ErrorCode::RuntimeTypeError,
        format!("{}: expected {}, got {}", context, expected, got),
    )
}

/// `Web.render(template_str, context_obj)` → response object with rendered HTML.
pub fn render(args: &[Value16]) -> HudHudResult<Value16> {
    let template = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.render"))?;
    let ctx = args
        .get(1)
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.render:context"))?;

    let ctx_map: Context = value_obj_to_context(ctx);
    let config = EvalConfig::default();

    let tokens = lexer::lex(template);
    let mut parser = parser::Parser::new(tokens);
    let nodes = parser
        .parse()
        .map_err(|e| runtime_error(format!("Template parse error: {}", e)))?;

    let output = eval(&nodes, &ctx_map, &config)
        .map_err(|e| runtime_error(format!("Template render error: {}", e)))?;

    Ok(html_response(200, &output))
}

/// `Web.render_file(path, context_obj)` → render template from file.
/// Supports `extends`/`include` resolved relative to the template's directory.
pub fn render_file(args: &[Value16]) -> HudHudResult<Value16> {
    let path = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.render_file"))?;
    let ctx = args
        .get(1)
        .and_then(|v| v.as_object())
        .ok_or_else(|| type_error("object", "", "Web.render_file:context"))?;

    let ctx_map: Context = value_obj_to_context(ctx);

    // Resolve template root from the file path's parent directory
    let template_path = std::path::Path::new(path);
    let config = EvalConfig {
        template_root: template_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf(),
    };

    let source = std::fs::read_to_string(path)
        .map_err(|e| runtime_error(format!("Web.render_file: {}: {}", path, e)))?;

    let tokens = lexer::lex(&source);
    let mut parser = parser::Parser::new(tokens);
    let nodes = parser
        .parse()
        .map_err(|e| runtime_error(format!("Template parse error in {}: {}", path, e)))?;

    let output = eval(&nodes, &ctx_map, &config)
        .map_err(|e| runtime_error(format!("Template render error: {}", e)))?;

    Ok(html_response(200, &output))
}

/// Build a minimal HTML response object.
fn html_response(status: u16, body: &str) -> Value16 {
    let mut headers: hudhudscript_bytecode::ObjMap = hudhudscript_bytecode::ObjMap::default();
    let mut obj = hudhudscript_bytecode::ObjMap::default();
    obj.insert("status".to_string(), Value16::number(status as f64));
    obj.insert("body".to_string(), Value16::string(body.to_string()));
    obj.insert("content_type".to_string(), Value16::string("text/html; charset=utf-8".to_string()));
    obj.insert("headers".to_string(), Value16::object(headers));
    obj.insert("cookies".to_string(), Value16::array(vec![]));
    Value16::object(obj)
}

/// `Web.escape(str)` → HTML-escaped string.
pub fn escape(args: &[Value16]) -> HudHudResult<Value16> {
    let s = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| type_error("string", "", "Web.escape"))?;
    Ok(Value16::string(filters::html_escape(s)))
}

/// Convert a Value16 object to a Context (hudhudscript_bytecode::ObjMap).
fn value_obj_to_context(obj: &hudhudscript_bytecode::ObjMap) -> Context {
    let mut ctx = Context::default();
    for (k, v) in obj {
        ctx.insert(k.clone(), v.clone());
    }
    ctx
}

// ── Unit tests ────────────────────────────────────────────────────────

