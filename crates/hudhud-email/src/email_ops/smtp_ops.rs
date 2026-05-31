//! SMTP email sending — msmtp and sendmail backends.

use std::collections::HashMap;
use hudhudscript_bytecode::Value16;
use hudhudscript_errors::HudHudResult;

use super::helpers::{obj_str, obj_str_opt, runtime_error, type_error};

pub fn email_send(args: &[Value16]) -> HudHudResult<Value16> {
    if args.is_empty() {
        return Err(runtime_error(
            "email.send() requires 1 argument: config object",
        ));
    }

    let config = args[0]
        .as_object()
        .ok_or_else(|| type_error("object", args[0].type_name_str(), "email.send config"))?;

    let to = obj_str(config, "to", "email.send")?;
    let from = obj_str(config, "from", "email.send")?;
    let subject = obj_str(config, "subject", "email.send")?;
    let body = obj_str(config, "body", "email.send")?;

    if let Some(smtp_host) = config.get("smtp_host").and_then(|v| v.as_str()) {
        let smtp_port = config
            .get("smtp_port")
            .and_then(|v| v.as_number())
            .map(|n| n as u16)
            .unwrap_or(587);
        let smtp_user = obj_str_opt(config, "smtp_user").unwrap_or_default();
        let smtp_pass = obj_str_opt(config, "smtp_pass").unwrap_or_default();

        let message = format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            from, to, subject, body
        );

        let mut cmd = std::process::Command::new("msmtp");
        cmd.arg("--host").arg(smtp_host);
        cmd.arg("--port").arg(smtp_port.to_string());
        if !smtp_user.is_empty() {
            cmd.arg("--auth=plain");
            cmd.arg("--user").arg(&smtp_user);
            cmd.arg("--passwordeval")
                .arg(format!("echo '{}'", smtp_pass));
            cmd.arg("--tls=on");
        }
        cmd.arg(&to);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            runtime_error(format!(
                "Failed to spawn msmtp: {}. Install msmtp or use email.send_simple()",
                e
            ))
        })?;

        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            let _ = stdin.write_all(message.as_bytes());
        }

        let output = child
            .wait_with_output()
            .map_err(|e| runtime_error(format!("msmtp error: {}", e)))?;

        let mut result = HashMap::new();
        result.insert("ok".to_string(), Value16::bool_(output.status.success()));
        result.insert(
            "message".to_string(),
            Value16::string(String::from_utf8_lossy(&output.stderr).to_string()),
        );
        Ok(Value16::object(result))
    } else {
        send_via_sendmail(&from, &to, &subject, &body)
    }
}

pub fn email_send_simple(args: &[Value16]) -> HudHudResult<Value16> {
    if args.len() < 3 {
        return Err(runtime_error(
            "email.send_simple() requires 3 arguments: to, subject, body",
        ));
    }
    let to = args[0]
        .as_str()
        .ok_or_else(|| type_error("string", args[0].type_name_str(), "email.send_simple to"))?
        .to_string();
    let subject = args[1]
        .as_str()
        .ok_or_else(|| type_error("string", args[1].type_name_str(), "email.send_simple subject"))?
        .to_string();
    let body = args[2]
        .as_str()
        .ok_or_else(|| type_error("string", args[2].type_name_str(), "email.send_simple body"))?
        .to_string();
    send_via_sendmail("", &to, &subject, &body)
}

pub(super) fn send_via_sendmail(
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> HudHudResult<Value16> {
    let message = if from.is_empty() {
        format!(
            "To: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            to, subject, body
        )
    } else {
        format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            from, to, subject, body
        )
    };

    let mut cmd = std::process::Command::new("sendmail");
    cmd.arg("-t");
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        runtime_error(format!(
            "Failed to spawn sendmail: {}. Ensure sendmail or a compatible MTA is installed",
            e
        ))
    })?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        let _ = stdin.write_all(message.as_bytes());
    }

    let output = child
        .wait_with_output()
        .map_err(|e| runtime_error(format!("sendmail error: {}", e)))?;

    let mut result = HashMap::new();
    result.insert("ok".to_string(), Value16::bool_(output.status.success()));
    result.insert(
        "message".to_string(),
        Value16::string(String::from_utf8_lossy(&output.stderr).to_string()),
    );
    Ok(Value16::object(result))
}
