use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;

pub(crate) fn build_client() -> SharedResult<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| runtime_error(format!("torrent HTTP client error: {}", e)))
}

pub(crate) fn rpc_call(
    rpc_url: &str,
    method: &str,
    arguments: serde_json::Value,
) -> SharedResult<serde_json::Value> {
    let client = build_client()?;
    let body = serde_json::json!({
        "method": method,
        "arguments": arguments
    });

    let resp = client
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| runtime_error(format!("torrent RPC request failed: {}", e)))?;

    if resp.status().as_u16() == 409 {
        let session_id = resp
            .headers()
            .get("X-Transmission-Session-Id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if session_id.is_empty() {
            return Err(runtime_error(
                "torrent: 409 received but no X-Transmission-Session-Id header found",
            ));
        }

        let resp2 = client
            .post(rpc_url)
            .header("Content-Type", "application/json")
            .header("X-Transmission-Session-Id", &session_id)
            .json(&body)
            .send()
            .map_err(|e| runtime_error(format!("torrent RPC retry failed: {}", e)))?;

        if !resp2.status().is_success() {
            let status = resp2.status().as_u16();
            let text = resp2
                .text()
                .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
            return Err(runtime_error(format!(
                "torrent RPC error (HTTP {}): {}",
                status, text
            )));
        }

        let json: serde_json::Value = resp2
            .json()
            .map_err(|e| runtime_error(format!("torrent RPC: invalid JSON response: {}", e)))?;

        return check_rpc_result(json);
    }

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let text = resp
            .text()
            .unwrap_or_else(|e| format!("<failed to read response body: {}>", e));
        return Err(runtime_error(format!(
            "torrent RPC error (HTTP {}): {}",
            status, text
        )));
    }

    let json: serde_json::Value = resp
        .json()
        .map_err(|e| runtime_error(format!("torrent RPC: invalid JSON response: {}", e)))?;

    check_rpc_result(json)
}

pub(crate) fn check_rpc_result(json: serde_json::Value) -> SharedResult<serde_json::Value> {
    let result_str = json.get("result").and_then(|v| v.as_str()).unwrap_or("");
    if result_str != "success" {
        return Err(runtime_error(format!(
            "torrent RPC returned: {}",
            result_str
        )));
    }
    Ok(json
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new())))
}
