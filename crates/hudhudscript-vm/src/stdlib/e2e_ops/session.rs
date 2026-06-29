//! E2E session establishment.

use hudhudscript_bytecode::shared_value::{runtime_error, type_error, SharedResult};
use hudhudscript_bytecode::Value16;
use std::collections::HashMap;

use crate::stdlib::e2e_ops::helpers::{compute_shared_secret, hex_decode, require_str};

pub fn e2e_create_session(args: &[Value16]) -> SharedResult<Value16> {
    let my_keypair = match args.first() {
        Some(v) => v.as_object().ok_or_else(|| {
            type_error("object (keypair)", v.type_name_str(), "e2e.create_session")
        })?,
        None => {
            return Err(runtime_error(
                "e2e.create_session: missing my_keypair argument",
            ));
        }
    };

    let their_public_hex = require_str(args, 1, "e2e.create_session")?;

    let my_private_hex = match my_keypair.get("private_key").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            return Err(runtime_error(
                "e2e.create_session: keypair must have 'private_key' field",
            ));
        }
    };

    let my_public_hex = match my_keypair.get("public_key").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            return Err(runtime_error(
                "e2e.create_session: keypair must have 'public_key' field",
            ));
        }
    };

    let my_private = hex_decode(&my_private_hex, "e2e.create_session", "private_key")?;
    let their_public = hex_decode(their_public_hex, "e2e.create_session", "their_public")?;

    if my_private.len() != 32 || their_public.len() != 32 {
        return Err(runtime_error(
            "e2e.create_session: keys must be 32 bytes (64 hex chars)",
        ));
    }

    let shared = compute_shared_secret(&my_private, &their_public);
    let shared_hex = hex::encode(&shared);

    let mut session = hudhudscript_bytecode::ObjMap::default();
    session.insert("shared_secret".to_string(), Value16::string(shared_hex));
    session.insert("my_public_key".to_string(), Value16::string(my_public_hex));
    session.insert(
        "their_public_key".to_string(),
        Value16::string(their_public_hex.to_string()),
    );
    session.insert("established".to_string(), Value16::boolean(true));

    Ok(Value16::object(session))
}
