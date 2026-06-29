//! HudHud Web Framework — umbrella crate.
//!
//! Re-exports and dispatches to `hudhud-web-*` sub-crates.
//! Follows the `hudhud-url` MethodId + dispatch pattern.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}

/// Enum identifying each Web operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebMethodId {
    Serve,
    Accept,
    Respond,
    Run,
    RouteMatch,
    RouteParams,
    Static,
    Render,
    RenderFile,
    Markdown,
    Escape,
    Html,
    Json,
    Redirect,
    SetCookie,
    SessionGet,
    SessionSet,
    FromWidget,
}

impl std::str::FromStr for WebMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "serve" => Ok(Self::Serve),
            "accept" => Ok(Self::Accept),
            "respond" => Ok(Self::Respond),
            "run" => Ok(Self::Run),
            "route_match" => Ok(Self::RouteMatch),
            "route_params" => Ok(Self::RouteParams),
            "static" => Ok(Self::Static),
            "render" => Ok(Self::Render),
            "render_file" => Ok(Self::RenderFile),
            "markdown" => Ok(Self::Markdown),
            "escape" => Ok(Self::Escape),
            "html" => Ok(Self::Html),
            "json" => Ok(Self::Json),
            "redirect" => Ok(Self::Redirect),
            "set_cookie" => Ok(Self::SetCookie),
            "session_get" => Ok(Self::SessionGet),
            "session_set" => Ok(Self::SessionSet),
            "from_widget" => Ok(Self::FromWidget),
            _ => Err(runtime_error(format!("Unknown Web method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for Web operations.
/// Each arm delegates to a `hudhud-web-*` sub-crate.
pub fn dispatch(method: WebMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        WebMethodId::Serve => hudhud_web_server::serve(args),
        WebMethodId::Accept => hudhud_web_server::accept(args),
        WebMethodId::Respond => hudhud_web_server::respond(args),
        WebMethodId::RouteMatch => hudhud_web_server::route_match(args),
        WebMethodId::RouteParams => hudhud_web_server::route_params(args),
        WebMethodId::Run => hudhud_web_prefork::run(args),
        WebMethodId::Static => hudhud_web_static::serve_static(args),
        WebMethodId::Render => hudhud_web_template::render(args),
        WebMethodId::RenderFile => hudhud_web_template::render_file(args),
        WebMethodId::Escape => hudhud_web_template::escape(args),
        WebMethodId::Markdown => hudhud_web_markdown::to_html(args),
        WebMethodId::Html => hudhud_web_response::html(args),
        WebMethodId::Json => hudhud_web_response::json(args),
        WebMethodId::Redirect => hudhud_web_response::redirect(args),
        WebMethodId::SetCookie => hudhud_web_response::set_cookie(args),
        WebMethodId::SessionGet => hudhud_web_session::session_get(args),
        WebMethodId::SessionSet => hudhud_web_session::session_set(args),
        WebMethodId::FromWidget => from_widget(args),
    }
}

/// `Web.from_widget(widget)` → HTML string.
fn from_widget(args: &[Value16]) -> HudHudResult<Value16> {
    let widget_val = args.first().ok_or_else(|| {
        Error::new(
            ErrorCode::RuntimeTypeError,
            "Web.from_widget: expected widget argument".to_string(),
        )
    })?;
    Ok(Value16::string(value_to_widget_html(widget_val)))
}

/// Convert a Value16 widget object to HTML string.
fn value_to_widget_html(val: &Value16) -> String {
    if let Some(obj) = val.as_object() {
        let widget_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("Text");
        let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let label = obj.get("label").and_then(|v| v.as_str()).unwrap_or("");
        match widget_type {
            "Text" => format!("<p id=\"{}\">{}</p>", id, label),
            "Button" => format!("<button id=\"{}\">{}</button>", id, label),
            "Input" => format!("<input id=\"{}\" />", id),
            "Column" | "Row" => {
                let dir = if widget_type == "Column" { "column" } else { "row" };
                let children = obj.get("children").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(|c| value_to_widget_html(c)).collect::<Vec<_>>().join(""))
                    .unwrap_or_default();
                format!("<div id=\"{}\" style=\"display:flex;flex-direction:{}\">{}</div>", id, dir, children)
            }
            _ => format!("<div id=\"{}\">unknown widget</div>", id),
        }
    } else {
        String::new()
    }
}
