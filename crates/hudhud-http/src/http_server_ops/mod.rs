//! Shared HTTP server builtins (Issue #602, #688).
//!
//! Single source of truth for VM and interpreter runtimes (Kural 7).
//! Binds a real TCP socket via `std::net::TcpListener`, spawns a background
//! accept loop thread, parses HTTP/1.1 requests, dispatches to registered
//! routes, and writes responses.

use helpers::*;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, HudHudResult};
use server_api_core::*;
use server_api_listen::*;

pub(crate) mod connection;
pub(crate) mod helpers;
pub(crate) mod server_api_core;
pub(crate) mod server_api_listen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpServerMethodId {
    Create,
    Route,
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Middleware,
    Listen,
    Stop,
    StaticFiles,
    AddStaticFiles,
    Websocket,
    AddWebsocket,
    Status,
    AddRoute,
    RouteResponse,
}

impl std::str::FromStr for HttpServerMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Self::Create),
            "route" => Ok(Self::Route),
            "get" => Ok(Self::Get),
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            "delete" => Ok(Self::Delete),
            "patch" => Ok(Self::Patch),
            "middleware" => Ok(Self::Middleware),
            "listen" => Ok(Self::Listen),
            "stop" => Ok(Self::Stop),
            "static_files" => Ok(Self::StaticFiles),
            "add_static_files" => Ok(Self::AddStaticFiles),
            "websocket" => Ok(Self::Websocket),
            "add_websocket" => Ok(Self::AddWebsocket),
            "status" => Ok(Self::Status),
            "add_route" => Ok(Self::AddRoute),
            "route_response" => Ok(Self::RouteResponse),
            _ => Err(runtime_error(format!("Unknown Server method: {}", s))),
        }
    }
}

impl HttpServerMethodId {
    pub fn dispatch(self, args: &[Value16]) -> HudHudResult<Value16> {
        match self {
            Self::Create => server_create(args),
            Self::Route => server_route(args),
            Self::Get => server_verb(args, "GET", "Server.get"),
            Self::Post => server_verb(args, "POST", "Server.post"),
            Self::Put => server_verb(args, "PUT", "Server.put"),
            Self::Delete => server_verb(args, "DELETE", "Server.delete"),
            Self::Patch => server_verb(args, "PATCH", "Server.patch"),
            Self::Middleware => server_middleware(args),
            Self::Listen => server_listen(args),
            Self::Stop => server_stop(args),
            Self::StaticFiles => server_static_files(args),
            Self::AddStaticFiles => server_add_static_files(args),
            Self::Websocket => server_websocket(args),
            Self::AddWebsocket => server_add_websocket(args),
            Self::Status => server_status(args),
            Self::AddRoute => server_add_route(args),
            Self::RouteResponse => server_route_response(args),
        }
    }
}

pub fn dispatch_str(method: &str, args: &[Value16]) -> HudHudResult<Value16> {
    method.parse::<HttpServerMethodId>()?.dispatch(args)
}

// ── Internal state ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct Route {
    method: String,
    pattern: String,
    handler: String,
    response_body: Option<String>,
    response_status: u16,
    response_content_type: String,
}

#[derive(Debug)]
pub(crate) struct ServerState {
    routes: Vec<Route>,
    middlewares: Vec<String>,
    static_dirs: Vec<(String, String)>,
    shutdown: bool,
}

impl ServerState {
    fn new() -> Self {
        Self {
            routes: Vec::new(),
            middlewares: Vec::new(),
            static_dirs: Vec::new(),
            shutdown: false,
        }
    }
}
