//! M1 — stdio MCP: sync köprü üzerinden client ÖMRÜ ve ardışık tools/call.
//!
//! Kök neden (v0.8.217): `block_on_provider` her çağrıda GEÇİCİ tokio
//! runtime kuruyordu; `connect_initialized` response_loop'u (Child süreci +
//! stdout okuyucusunun sahibi StdioRecvHalf) o runtime'a spawn ediyordu.
//! `block_on` dönünce runtime düşer → task ölür → sunucunun stdout'u
//! kapanır → sunucu çıkar → İLK `tools/call` "boru kapalı" (os error 232).
//!
//! Bu testler ÜRETİMİN aynı yolunu kullanır: SYNC bağlam (tokio::test
//! DEĞİL — tokio::test kalıcı runtime sağlayıp bug'ı maskeler) +
//! `create_mcp_client_from_config` + `dispatch_mcp_tool_call` köprüsüyle
//! aynı `block_on_provider` zinciri.
use hudhudscript_vm::mcp_dispatch::{create_mcp_client_from_config, McpTransportKind};
use hudhudscript_vm::VM;

fn fixture_bin() -> String {
    env!("CARGO_BIN_EXE_mcp_fixture_server").to_string()
}

#[test]
fn m1_stdio_client_survives_sync_bridge_two_calls() {
    // 1) Bağlan + initialize — üretimdeki sync köprüden.
    let client = create_mcp_client_from_config(
        "Yerel",
        McpTransportKind::Stdio,
        Some(&fixture_bin()),
        &[],
        None,
        false,
    )
    .expect("connect+initialize sync köprüden geçmeli");

    // 2) M1 kabulü: aynı client'la ARKA ARKAYA İKİ tools/call çalışmalı.
    //    (Fix öncesi ilk çağrı "boru kapalı" ile ölüyordu.)
    let vm = VM::new();
    let _ = vm; // VM yalnız köprü ortamının üretimle aynı olduğunu belgeler.
    for (i, text) in ["merhaba", "ikinci"].iter().enumerate() {
        let args = serde_json::json!({ "text": text });
        let c = client.clone();
        let name = "echo".to_string();
        let result = hudhudscript_vm::provider_bridge_block_on(async move {
            c.call_tool(name, Some(args)).await.map_err(|e| e.to_string())
        });
        let response = result.unwrap_or_else(|e| panic!("{}. tools/call başarısız: {}", i + 1, e));
        let text_out = response
            .content
            .first()
            .and_then(|c| match c {
                hudhudscript_mcp::protocol::Content::Text { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        assert!(
            text_out.contains(text),
            "{}. çağrı echo içeriği dönmeli, got: {:?}",
            i + 1,
            text_out
        );
    }
}
