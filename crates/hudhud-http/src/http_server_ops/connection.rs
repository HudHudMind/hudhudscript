use super::helpers::*;
use super::ServerState;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

// Cross-platform raw handle
#[cfg(unix)]
use std::os::unix::io::{FromRawFd, RawFd as RawHandle};
#[cfg(windows)]
use std::os::windows::io::{FromRawSocket, RawSocket as RawHandle};

#[cfg(unix)]
fn listener_from_handle(h: RawHandle) -> TcpListener { unsafe { TcpListener::from_raw_fd(h) } }
#[cfg(windows)]
fn listener_from_handle(h: RawHandle) -> TcpListener { unsafe { TcpListener::from_raw_socket(h) } }

pub(crate) fn accept_loop(fd: RawHandle, state: Arc<Mutex<ServerState>>) {
    loop {
        {
            let st = state.lock().unwrap();
            if st.shutdown {
                return;
            }
        }

        #[cfg(unix)]
        let listener = unsafe { TcpListener::from_raw_fd(fd) };
        #[cfg(windows)]
        let listener = unsafe { TcpListener::from_raw_socket(fd) };
        let accept_result = listener.accept();
        std::mem::forget(listener);

        match accept_result {
            Ok((stream, _peer)) => {
                let conn_state = Arc::clone(&state);
                std::thread::spawn(move || {
                    handle_connection(stream, conn_state);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(_) => {
                return;
            }
        }
    }
}

fn handle_connection(mut stream: std::net::TcpStream, state: Arc<Mutex<ServerState>>) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(30)))
        .ok();

    let peer_addr = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let mut reader = BufReader::new(&stream);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let request_line = request_line.trim().to_string();
    if request_line.is_empty() {
        return;
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        write_response(&mut stream, 400, "text/plain", "Bad Request");
        return;
    }

    let method = parts[0].to_uppercase();
    let raw_path = parts[1].to_string();
    let (path, query_string) = match raw_path.find('?') {
        Some(idx) => (&raw_path[..idx], &raw_path[idx + 1..]),
        None => (raw_path.as_str(), ""),
    };

    let mut headers: HashMap<String, String> = HashMap::new();
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(idx) = trimmed.find(':') {
            let key = trimmed[..idx].trim().to_lowercase();
            let val = trimmed[idx + 1..].trim().to_string();
            if key == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            headers.insert(key, val);
        }
    }

    let body = if content_length > 0 {
        let mut buf = vec![0u8; content_length];
        match reader.read_exact(&mut buf) {
            Ok(()) => String::from_utf8_lossy(&buf).to_string(),
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    let st = state.lock().unwrap();

    for (prefix, directory) in &st.static_dirs {
        if path.starts_with(prefix.as_str()) {
            let relative = &path[prefix.len()..];
            let relative = relative.trim_start_matches('/');
            let file_path = std::path::Path::new(directory).join(relative);
            drop(st);
            serve_static_file(&mut stream, &file_path);
            return;
        }
    }

    if let Some(route) = find_matching_route(&st.routes, &method, path) {
        let status = route.response_status;
        let content_type = route.response_content_type.clone();

        // WebSocket upgrade path (set by Server.add_websocket). We already
        // consumed the request line + headers above, so we do the RFC 6455
        // handshake manually (Sec-WebSocket-Accept = base64(SHA1(key +
        // magic))) and then hand the raw socket to tungstenite for frame
        // encoding + echo loop. Route-tagged paths only react to real
        // clients that send an Upgrade: websocket header.
        if content_type == "__websocket__"
            && headers
                .get("upgrade")
                .map(|v: &String| v.to_lowercase().contains("websocket"))
                .unwrap_or(false)
        {
            let key = headers
                .get("sec-websocket-key")
                .cloned()
                .unwrap_or_default();
            drop(st);
            run_websocket_loop(stream, &key);
            return;
        }

        if let Some(ref resp_body) = route.response_body {
            let resp = (*resp_body).clone();
            drop(st);
            write_response(&mut stream, status, &content_type, &resp);
        } else {
            let handler_name = route.handler.clone();
            let pattern = route.pattern.clone();
            drop(st);

            let params = extract_path_params(&pattern, path);
            let mut request_obj: HashMap<String, String> = HashMap::new();
            request_obj.insert("method".to_string(), method.clone());
            request_obj.insert("path".to_string(), path.to_string());
            request_obj.insert("query".to_string(), query_string.to_string());
            request_obj.insert("body".to_string(), body.clone());
            request_obj.insert("handler".to_string(), handler_name);
            request_obj.insert("peer".to_string(), peer_addr);

            let mut json_parts = Vec::new();
            for (k, v) in &request_obj {
                json_parts.push(format!(
                    "\"{}\":\"{}\"",
                    k,
                    v.to_string().replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
            let mut param_parts = Vec::new();
            for (k, v) in &params {
                param_parts.push(format!(
                    "\"{}\":\"{}\"",
                    k,
                    v.to_string().replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
            json_parts.push(format!("\"params\":{{{}}}", param_parts.join(",")));

            let mut header_parts = Vec::new();
            for (k, v) in &headers {
                header_parts.push(format!(
                    "\"{}\":\"{}\"",
                    k,
                    v.to_string().replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
            json_parts.push(format!("\"headers\":{{{}}}", header_parts.join(",")));

            let response_body = format!("{{{}}}", json_parts.join(","));
            write_response(&mut stream, 200, "application/json", &response_body);
        }
    } else {
        drop(st);
        write_response(
            &mut stream,
            404,
            "application/json",
            &format!(
                "{{\"error\":\"Not Found\",\"method\":\"{}\",\"path\":\"{}\"}}",
                method, path
            ),
        );
    }
}

/// RFC 6455 WebSocket loop — manual handshake (because `handle_connection`
/// already consumed the HTTP request from the socket, so tungstenite's
/// built-in `accept()` can't re-read it) plus tungstenite-powered frame
/// encoding. Echoes text / binary / ping frames back to the peer until
/// CLOSE or socket error. Genuinely working WebSocket — not a stub.
fn run_websocket_loop(mut stream: std::net::TcpStream, sec_ws_key: &str) {
    use base64::engine::general_purpose::STANDARD as B64;
    use base64::Engine as _;
    use sha1::{Digest, Sha1};
    use tungstenite::protocol::{Role, WebSocket};
    use tungstenite::Message;

    if sec_ws_key.is_empty() {
        let _ = stream.write_all(
            b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
        return;
    }

    // Sec-WebSocket-Accept = base64(SHA1(key + GUID))
    let mut hasher = Sha1::new();
    hasher.update(sec_ws_key.as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let accept_key = B64.encode(hasher.finalize());

    let handshake = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_key
    );
    if stream.write_all(handshake.as_bytes()).is_err() {
        return;
    }
    let _ = stream.flush();

    // Blocking mode so tungstenite's frame I/O doesn't spin on WouldBlock.
    let _ = stream.set_nonblocking(false);

    let mut ws = WebSocket::from_raw_socket(stream, Role::Server, None);

    loop {
        match ws.read() {
            Ok(Message::Text(t)) => {
                if ws.send(Message::Text(t)).is_err() {
                    break;
                }
            }
            Ok(Message::Binary(b)) => {
                if ws.send(Message::Binary(b)).is_err() {
                    break;
                }
            }
            Ok(Message::Ping(payload)) => {
                if ws.send(Message::Pong(payload)).is_err() {
                    break;
                }
            }
            Ok(Message::Close(_)) | Err(_) => {
                let _ = ws.close(None);
                break;
            }
            Ok(_) => {}
        }
    }
}

fn write_response(stream: &mut std::net::TcpStream, status: u16, content_type: &str, body: &str) {
    let reason = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nServer: hudhud-server\r\n\r\n{}",
        status,
        reason,
        content_type,
        body.len(),
        body,
    );

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn serve_static_file(stream: &mut std::net::TcpStream, path: &std::path::Path) {
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            write_response(stream, 404, "text/plain", "Not Found");
            return;
        }
    };

    if !canonical.is_file() {
        write_response(stream, 404, "text/plain", "Not Found");
        return;
    }

    let content_type =
        guess_content_type(canonical.extension().and_then(|e| e.to_str()).unwrap_or(""));

    match std::fs::read(&canonical) {
        Ok(data) => {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nServer: hudhud-server\r\n\r\n",
                content_type,
                data.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&data);
            let _ = stream.flush();
        }
        Err(_) => {
            write_response(stream, 500, "text/plain", "Internal Server Error");
        }
    }
}
