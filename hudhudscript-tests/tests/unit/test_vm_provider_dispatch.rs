//! VM provider dispatch: tool-call follow-up request construction —
//! prompt override, role text replacement, and config passthrough.

use hudhudscript_vm::vm::provider_dispatch::{build_follow_up_request, ProviderCallConfig};

#[test]
fn test_build_follow_up_request() {
    let orig_config = ProviderCallConfig {
        prompt: "original prompt".to_string(),
        system_prompt: Some("original system".to_string()),
        temperature: Some(0.5),
        max_tokens: Some(100),
        timeout_secs: Some(300),
    };

    let follow_up = build_follow_up_request(
        &orig_config,
        "new prompt",
        &[],
        Some("Role text".to_string()),
    );

    assert_eq!(follow_up.prompt, "new prompt");
    assert_eq!(follow_up.system_prompt, Some("Role text".to_string()));
    assert_eq!(follow_up.temperature, Some(0.5));
    assert_eq!(follow_up.max_tokens, Some(100));
    assert_eq!(follow_up.timeout_secs, Some(300));
}
