//! Tests for hudhudscript-runtime provider timeout default logic
//! (moved from src/provider/types.rs).

use hudhudscript_runtime::provider::types::DEFAULT_PROVIDER_TIMEOUT_SECS;

fn get_effective_timeout(req: Option<u64>, cfg: Option<u64>) -> u64 {
    req.or(cfg).unwrap_or(DEFAULT_PROVIDER_TIMEOUT_SECS)
}

#[test]
fn test_provider_timeout_effective_logic() {
    assert_eq!(get_effective_timeout(Some(300), Some(120)), 300);
    assert_eq!(get_effective_timeout(None, Some(300)), 300);
    assert_eq!(get_effective_timeout(Some(180), None), 180);
    assert_eq!(
        get_effective_timeout(None, None),
        DEFAULT_PROVIDER_TIMEOUT_SECS
    );
}
