use super::helpers::*;
use super::Route;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;
use std::collections::HashMap;

pub(crate) fn server_create(args: &[Value16]) -> HudHudResult<Value16> {
    let (host, port, name) = match args.first() {
        Some(v) => {
            if let Some(o) = v.as_object() {
                let host = o
                    .get("host")
                    .and_then(|x| x.as_str())
                    .unwrap_or("127.0.0.1")
                    .to_string();
                let port = o.get("port").and_then(|x| x.as_number()).unwrap_or(8080.0);
                let name = o
                    .get("name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("hudhud-server")
                    .to_string();
                (host, port, name)
            } else if let Some(n) = v.as_number() {
                ("127.0.0.1".to_string(), n, "hudhud-server".to_string())
            } else {
                ("127.0.0.1".to_string(), 8080.0, "hudhud-server".to_string())
            }
        }
        None => ("127.0.0.1".to_string(), 8080.0, "hudhud-server".to_string()),
    };

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("name".to_string(), Value16::string(name));
    result.insert("host".to_string(), Value16::string(host));
    result.insert("port".to_string(), Value16::number(port));
    result.insert("running".to_string(), Value16::bool_(false));
    result.insert("routes".to_string(), Value16::array(vec![]));
    result.insert("middleware".to_string(), Value16::array(vec![]));
    Ok(Value16::object(result))
}

pub(crate) fn server_route(args: &[Value16]) -> HudHudResult<Value16> {
    let method = require_str(args, 0, "Server.route")?.to_uppercase();
    let path = require_str(args, 1, "Server.route")?;
    let handler = args
        .get(2)
        .and_then(|v| v.as_str())
        .unwrap_or("handler")
        .to_string();
    Ok(build_route_obj(&method, &path, &handler))
}

pub(crate) fn server_verb(args: &[Value16], verb: &str, name: &str) -> HudHudResult<Value16> {
    let path = require_str(args, 0, name)?;
    let handler = args
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("handler")
        .to_string();
    Ok(build_route_obj(verb, &path, &handler))
}

pub(crate) fn server_middleware(args: &[Value16]) -> HudHudResult<Value16> {
    let name = require_str(args, 0, "Server.middleware")?;
    let opts = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| Value16::object(hudhudscript_bytecode::ObjMap::default()));

    let mut result = hudhudscript_bytecode::ObjMap::default();
    result.insert("name".to_string(), Value16::string(name));
    result.insert("options".to_string(), opts);
    result.insert("enabled".to_string(), Value16::bool_(true));
    Ok(Value16::object(result))
}

pub(crate) fn server_add_route(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_server_fd(args, "Server.add_route")?;
    let route_obj = args.get(1).and_then(|v| v.as_object()).ok_or_else(|| {
        runtime_error("Server.add_route: expected route object as second argument")
    })?;

    let method = route_obj
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_string();
    let path = route_obj
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("/")
        .to_string();
    let handler = route_obj
        .get("handler")
        .and_then(|v| v.as_str())
        .unwrap_or("handler")
        .to_string();

    let registry = server_registry().lock().unwrap();
    if let Some(state) = registry.get(&fd) {
        let mut st = state.lock().unwrap();
        st.routes.push(Route {
            method,
            pattern: path,
            handler,
            response_body: None,
            response_status: 200,
            response_content_type: "application/json".to_string(),
        });
    } else {
        return Err(runtime_error(
            "Server.add_route: server not found — did you call Server.listen() first?",
        ));
    }

    Ok(Value16::bool_(true))
}

pub(crate) fn server_route_response(args: &[Value16]) -> HudHudResult<Value16> {
    let fd = extract_server_fd(args, "Server.route_response")?;
    let method = require_str(args, 1, "Server.route_response")?.to_uppercase();
    let path = require_str(args, 2, "Server.route_response")?;

    let body = match args.get(3) {
        Some(v) => {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if v.as_object().is_some() || v.as_array().is_some() {
                crate::json::value_to_json_string(v)
            } else {
                v.display_string()
            }
        }
        None => "{}".to_string(),
    };

    let (status, content_type) = match args.get(4) {
        Some(v) => {
            if let Some(n) = v.as_number() {
                (n as u16, "application/json".to_string())
            } else if let Some(obj) = v.as_object() {
                let s = obj
                    .get("status")
                    .and_then(|x| x.as_number())
                    .map(|n| n as u16)
                    .unwrap_or(200);
                let ct = obj
                    .get("content_type")
                    .and_then(|x| x.as_str())
                    .unwrap_or("application/json")
                    .to_string();
                (s, ct)
            } else {
                (200, "application/json".to_string())
            }
        }
        None => (200, "application/json".to_string()),
    };

    let registry = server_registry().lock().unwrap();
    if let Some(state) = registry.get(&fd) {
        let mut st = state.lock().unwrap();
        st.routes.push(Route {
            method,
            pattern: path,
            handler: "__static_response__".to_string(),
            response_body: Some(body),
            response_status: status,
            response_content_type: content_type,
        });
        Ok(Value16::bool_(true))
    } else {
        Err(runtime_error(
            "Server.route_response: server not found — call Server.listen() first",
        ))
    }
}
