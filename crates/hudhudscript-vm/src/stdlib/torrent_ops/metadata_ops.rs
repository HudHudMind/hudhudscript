use hudhudscript_bytecode::shared_value::{runtime_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use super::{
    client_ops::rpc_call,
    helpers::{default_rpc_url, optional_string, require_i64, status_string},
};

pub fn torrent_list(args: &[Value16]) -> SharedResult<Value16> {
    let rpc_url = optional_string(args, 0).unwrap_or_else(default_rpc_url);

    let arguments = serde_json::json!({
        "fields": [
            "id", "name", "status", "percentDone",
            "rateDownload", "rateUpload", "totalSize"
        ]
    });

    let result = rpc_call(&rpc_url, "torrent-get", arguments)?;

    let torrents = result
        .get("torrents")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut arr: Vec<Value16> = Vec::new();
    for t in &torrents {
        let mut entry = HashMap::new();
        entry.insert(
            "id".to_string(),
            Value16::number(t.get("id").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        );
        entry.insert(
            "name".to_string(),
            Value16::string(
                t.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
        );
        let status_num = t.get("status").and_then(|v| v.as_i64()).unwrap_or(-1);
        entry.insert(
            "status".to_string(),
            Value16::string(status_string(status_num).to_string()),
        );
        entry.insert(
            "progress".to_string(),
            Value16::number(t.get("percentDone").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        );
        entry.insert(
            "download_speed".to_string(),
            Value16::number(
                t.get("rateDownload")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            ),
        );
        entry.insert(
            "upload_speed".to_string(),
            Value16::number(t.get("rateUpload").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        );
        entry.insert(
            "size".to_string(),
            Value16::number(t.get("totalSize").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        );
        arr.push(Value16::object(entry));
    }

    Ok(Value16::array(arr))
}

pub fn torrent_info(args: &[Value16]) -> SharedResult<Value16> {
    let id = require_i64(args, 0, "torrent.info")?;
    let rpc_url = optional_string(args, 1).unwrap_or_else(default_rpc_url);

    let arguments = serde_json::json!({
        "ids": [id],
        "fields": [
            "id", "name", "hashString", "status", "percentDone",
            "rateDownload", "rateUpload", "totalSize", "eta",
            "files", "peers", "trackers", "downloadDir",
            "addedDate", "doneDate", "uploadRatio", "error", "errorString"
        ]
    });

    let result = rpc_call(&rpc_url, "torrent-get", arguments)?;

    let torrents = result
        .get("torrents")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let t = match torrents.first() {
        Some(t) => t,
        None => {
            return Err(runtime_error(format!(
                "torrent.info: torrent {} not found",
                id
            )));
        }
    };

    let mut obj = HashMap::new();
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
    obj.insert(
        "hash".to_string(),
        Value16::string(
            t.get("hashString")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
    );

    let status_num = t.get("status").and_then(|v| v.as_i64()).unwrap_or(-1);
    obj.insert(
        "status".to_string(),
        Value16::string(status_string(status_num).to_string()),
    );
    obj.insert(
        "progress".to_string(),
        Value16::number(t.get("percentDone").and_then(|v| v.as_f64()).unwrap_or(0.0)),
    );
    obj.insert(
        "download_speed".to_string(),
        Value16::number(
            t.get("rateDownload")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        ),
    );
    obj.insert(
        "upload_speed".to_string(),
        Value16::number(t.get("rateUpload").and_then(|v| v.as_f64()).unwrap_or(0.0)),
    );
    obj.insert(
        "size".to_string(),
        Value16::number(t.get("totalSize").and_then(|v| v.as_f64()).unwrap_or(0.0)),
    );
    obj.insert(
        "eta".to_string(),
        Value16::number(t.get("eta").and_then(|v| v.as_f64()).unwrap_or(-1.0)),
    );
    obj.insert(
        "download_dir".to_string(),
        Value16::string(
            t.get("downloadDir")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
    );
    obj.insert(
        "upload_ratio".to_string(),
        Value16::number(t.get("uploadRatio").and_then(|v| v.as_f64()).unwrap_or(0.0)),
    );
    obj.insert(
        "error".to_string(),
        Value16::string(
            t.get("errorString")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ),
    );

    let files = t
        .get("files")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let file_values: Vec<Value16> = files
        .iter()
        .map(|f| {
            let mut fm = HashMap::new();
            fm.insert(
                "name".to_string(),
                Value16::string(
                    f.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            );
            fm.insert(
                "size".to_string(),
                Value16::number(f.get("length").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            );
            fm.insert(
                "completed".to_string(),
                Value16::number(
                    f.get("bytesCompleted")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                ),
            );
            Value16::object(fm)
        })
        .collect();
    obj.insert("files".to_string(), Value16::array(file_values));

    let peers = t
        .get("peers")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let peer_values: Vec<Value16> = peers
        .iter()
        .map(|p| {
            let mut pm = HashMap::new();
            pm.insert(
                "address".to_string(),
                Value16::string(
                    p.get("address")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            );
            pm.insert(
                "client".to_string(),
                Value16::string(
                    p.get("clientName")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            );
            pm.insert(
                "progress".to_string(),
                Value16::number(p.get("progress").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            );
            Value16::object(pm)
        })
        .collect();
    obj.insert("peers".to_string(), Value16::array(peer_values));

    let trackers = t
        .get("trackers")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let tracker_values: Vec<Value16> = trackers
        .iter()
        .map(|tr| {
            let mut tm = HashMap::new();
            tm.insert(
                "announce".to_string(),
                Value16::string(
                    tr.get("announce")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            );
            tm.insert(
                "id".to_string(),
                Value16::number(tr.get("id").and_then(|v| v.as_f64()).unwrap_or(0.0)),
            );
            Value16::object(tm)
        })
        .collect();
    obj.insert("trackers".to_string(), Value16::array(tracker_values));

    Ok(Value16::object(obj))
}
