use hudhudscript_bytecode::shared_value::SharedResult;
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use super::{
    client_ops::rpc_call,
    helpers::{default_rpc_url, ok_message, optional_string, require_i64, require_string},
};

pub fn torrent_add(args: &[Value16]) -> SharedResult<Value16> {
    let torrent_url = require_string(args, 0, "torrent.add")?;
    let rpc_url = optional_string(args, 1).unwrap_or_else(default_rpc_url);

    let arguments = serde_json::json!({ "filename": torrent_url });

    let result = rpc_call(&rpc_url, "torrent-add", arguments)?;

    let added = result
        .get("torrent-added")
        .or_else(|| result.get("torrent-duplicate"));

    let mut obj = HashMap::new();
    obj.insert("ok".to_string(), Value16::boolean(true));
    if let Some(t) = added {
        obj.insert(
            "id".to_string(),
            Value16::number(t.get("id").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        );
        obj.insert(
            "name".to_string(),
            Value16::string(
                t.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
        );
    } else {
        obj.insert("id".to_string(), Value16::number(0.0));
        obj.insert("name".to_string(), Value16::string(String::new()));
    }

    Ok(Value16::object(obj))
}

pub fn torrent_remove(args: &[Value16]) -> SharedResult<Value16> {
    let id = require_i64(args, 0, "torrent.remove")?;

    let delete_data = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

    let rpc_url = optional_string(args, 2).unwrap_or_else(default_rpc_url);

    let arguments = serde_json::json!({
        "ids": [id],
        "delete-local-data": delete_data
    });

    rpc_call(&rpc_url, "torrent-remove", arguments)?;

    Ok(ok_message(true, format!("torrent {} removed", id)))
}

pub fn torrent_pause(args: &[Value16]) -> SharedResult<Value16> {
    let id = require_i64(args, 0, "torrent.pause")?;
    let rpc_url = optional_string(args, 1).unwrap_or_else(default_rpc_url);
    let arguments = serde_json::json!({ "ids": [id] });
    rpc_call(&rpc_url, "torrent-stop", arguments)?;
    Ok(ok_message(true, format!("torrent {} paused", id)))
}

pub fn torrent_resume(args: &[Value16]) -> SharedResult<Value16> {
    let id = require_i64(args, 0, "torrent.resume")?;
    let rpc_url = optional_string(args, 1).unwrap_or_else(default_rpc_url);
    let arguments = serde_json::json!({ "ids": [id] });
    rpc_call(&rpc_url, "torrent-start", arguments)?;
    Ok(ok_message(true, format!("torrent {} resumed", id)))
}
