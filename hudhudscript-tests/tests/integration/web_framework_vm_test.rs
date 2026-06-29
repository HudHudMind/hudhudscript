//! HudHud Web Framework — VM integration tests.
//!
//! Tests Web module primitives through the HudHudScript VM:
//! template rendering, response builders, session, markdown,
//! and request parsing.

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_parser::parse;

fn vm_run(code: &str, var: &str) -> Value16 {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = hudhudscript_vm::VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("VM execution failed");
    vm.get_variable(var)
        .cloned()
        .unwrap_or_else(|| panic!("variable '{}' not found", var))
}

fn vm_run_ok(code: &str) {
    let ast = parse(code).expect("parse failed");
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).expect("compile failed");
    let mut vm = hudhudscript_vm::VM::new();
    hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
    vm.execute(&bytecode).expect("VM execution failed");
}

// ── Template tests ────────────────────────────────────────────────────

#[test]
fn test_web_escape() {
    let result = vm_run(r#"var x = Web.escape("<script>alert('x')</script>");"#, "x");
    assert_eq!(
        result.as_str().unwrap(),
        "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"
    );
}

#[test]
fn test_web_render_variable() {
    let result = vm_run(
        r#"var tpl = "<h1>Merhaba {{ name }}</h1>";
var ctx = { name: "Onur" };
var x = Web.render(tpl, ctx);"#,
        "x",
    );
    let body = result
        .as_object()
        .unwrap()
        .get("body")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(body, "<h1>Merhaba Onur</h1>");
}

#[test]
fn test_web_render_if() {
    let result = vm_run(
        r#"var tpl = "{% if flag %}yes{% else %}no{% endif %}";
var ctx = { flag: true };
var x = Web.render(tpl, ctx);"#,
        "x",
    );
    let body = result
        .as_object()
        .unwrap()
        .get("body")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(body, "yes");
}

#[test]
fn test_web_render_for() {
    let result = vm_run(
        r#"var tpl = "{% for item in items %}{{ item }},{% endfor %}";
var ctx = { items: ["a", "b", "c"] };
var x = Web.render(tpl, ctx);"#,
        "x",
    );
    let body = result
        .as_object()
        .unwrap()
        .get("body")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(body, "a,b,c,");
}

// ── Response tests ────────────────────────────────────────────────────

#[test]
fn test_web_html_response() {
    let result = vm_run(r#"var x = Web.html("<h1>Hello</h1>");"#, "x");
    let obj = result.as_object().unwrap();
    assert_eq!(obj.get("status").unwrap().as_number().unwrap(), 200.0);
    assert_eq!(
        obj.get("content_type").unwrap().as_str().unwrap(),
        "text/html; charset=utf-8"
    );
}

#[test]
fn test_web_json_response() {
    let result = vm_run(r#"var x = Web.json({ ok: true });"#, "x");
    let obj = result.as_object().unwrap();
    assert_eq!(
        obj.get("content_type").unwrap().as_str().unwrap(),
        "application/json"
    );
}

#[test]
fn test_web_redirect() {
    let result = vm_run(r#"var x = Web.redirect("/login", 301);"#, "x");
    let obj = result.as_object().unwrap();
    assert_eq!(obj.get("status").unwrap().as_number().unwrap(), 301.0);
    let headers = obj.get("headers").unwrap().as_object().unwrap();
    assert_eq!(headers.get("Location").unwrap().as_str().unwrap(), "/login");
}

// ── Session tests ─────────────────────────────────────────────────────

#[test]
fn test_web_session_roundtrip() {
    let result = vm_run(
        r#"var resp = Web.html("ok");
var resp2 = Web.session_set(resp, "my-secret", { user_id: "42" });
var cookies = resp2.cookies;
var x = cookies.length;"#,
        "x",
    );
    assert_eq!(result.as_number().unwrap(), 1.0);
}

// ── Markdown tests ────────────────────────────────────────────────────

#[test]
fn test_web_markdown_heading() {
    let result = vm_run(r##"var x = Web.markdown("# Başlık");"##, "x");
    assert!(result.as_str().unwrap().contains("<h1>"));
    assert!(result.as_str().unwrap().contains("Başlık"));
}

// ── Route matching tests ──────────────────────────────────────────────

#[test]
fn test_web_route_match() {
    let result = vm_run(
        r#"var x = Web.route_match("/users/:id", "/users/42");"#,
        "x",
    );
    assert_eq!(result.as_bool().unwrap(), true);
}

#[test]
fn test_web_route_match_no_match() {
    let result = vm_run(
        r#"var x = Web.route_match("/users/:id", "/posts/42");"#,
        "x",
    );
    assert_eq!(result.as_bool().unwrap(), false);
}

#[test]
fn test_web_route_params() {
    let result = vm_run(
        r#"var x = Web.route_params("/users/:id/posts/:pid", "/users/42/posts/7");"#,
        "x",
    );
    let obj = result.as_object().unwrap();
    assert_eq!(obj.get("id").unwrap().as_str().unwrap(), "42");
    assert_eq!(obj.get("pid").unwrap().as_str().unwrap(), "7");
}

// ── Set cookie test ───────────────────────────────────────────────────

#[test]
fn test_web_set_cookie() {
    let result = vm_run(
        "var resp = Web.html(\"ok\"); var resp2 = Web.set_cookie(resp, \"token\", \"abc\", { httponly: true, path: \"/\", max_age: 3600 }); var cookies = resp2.cookies; var x = cookies[0];",
        "x",
    );
    let cookie = result.as_str().unwrap();
    assert!(cookie.contains("token=abc"));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Path=/"));
}

#[test]
fn test_web_render_filter_upper() {
    let result = vm_run(
        r#"var x = Web.render("{{ name | upper }}", { name: "onur" });"#,
        "x",
    );
    let body = result
        .as_object()
        .unwrap()
        .get("body")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(body, "ONUR");
}

#[test]
fn test_web_render_filter_default() {
    let result = vm_run(
        r#"var x = Web.render("{{ val | default('N/A') }}", { val: null });"#,
        "x",
    );
    let body = result
        .as_object()
        .unwrap()
        .get("body")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(body, "N/A");
}
