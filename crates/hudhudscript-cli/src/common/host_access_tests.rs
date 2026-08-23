use super::*;

#[test]
fn default_is_deny() {
    let cfg = HostAccessConfig::default();
    assert_eq!(cfg.default, AccessDecision::Deny);
    assert_eq!(cfg.env.default, AccessDecision::Deny);
    assert_eq!(cfg.exec.default, AccessDecision::Deny);
    assert_eq!(cfg.modules.default, None);
}

#[test]
fn restrictive_is_deny() {
    let cfg = HostAccessConfig::restrictive();
    assert_eq!(cfg.default, AccessDecision::Deny);
    assert_eq!(cfg.exec.max_processes, 0);
    assert_eq!(cfg.modules.default, Some(AccessDecision::Deny));
}

#[test]
fn permissive_is_allow() {
    let cfg = HostAccessConfig::permissive();
    assert_eq!(cfg.default, AccessDecision::Allow);
    assert_eq!(cfg.exec.max_processes, 100);
    assert_eq!(cfg.modules.default, Some(AccessDecision::Allow));
    assert_eq!(cfg.modules.database, Some(AccessDecision::Deny));
}

#[test]
fn toml_round_trip() {
    let toml = r#"
default = "deny"

[env]
default = "allow"
allow = ["DEEPSEEK_API_KEY"]
deny = ["HOME"]

[exec]
default = "deny"
allow = ["python", "node"]
deny = ["rm"]
max_processes = 10

[modules]
default = "deny"
http = "allow"
process = "deny"
"#;
    let cfg: HostAccessConfig = toml::from_str(toml).expect("valid TOML");
    assert_eq!(cfg.default, AccessDecision::Deny);
    assert_eq!(cfg.env.default, AccessDecision::Allow);
    assert!(cfg.env.allow.contains(&"DEEPSEEK_API_KEY".to_string()));
    assert!(cfg.env.deny.contains(&"HOME".to_string()));
    assert_eq!(cfg.exec.default, AccessDecision::Deny);
    assert!(cfg.exec.allow.contains(&"python".to_string()));
    assert_eq!(cfg.exec.max_processes, 10);
    assert_eq!(cfg.modules.default, Some(AccessDecision::Deny));
    assert_eq!(cfg.modules.http, Some(AccessDecision::Allow));
    assert_eq!(cfg.modules.process, Some(AccessDecision::Deny));
}

#[test]
fn invalid_decision_is_parse_error() {
    let toml = r#"default = "maybe""#;
    let result: Result<HostAccessConfig, _> = toml::from_str(toml);
    assert!(result.is_err());
}

#[test]
fn merge_scalar_default_overlay_wins() {
    let base = HostAccessConfig::restrictive();
    let mut overlay = HostAccessConfig::default();
    overlay.default = AccessDecision::Allow;
    let merged = base.merge(Some(&overlay));
    assert_eq!(merged.default, AccessDecision::Allow);
    // subtables unchanged because overlay left them at default
    assert_eq!(merged.env.default, AccessDecision::Deny);
}

#[test]
fn merge_env_lists_allow_replace_deny_union() {
    let mut base = HostAccessConfig::default();
    base.env.allow = vec!["BASE_ALLOW".to_string()];
    base.env.deny = vec!["BASE_DENY".to_string()];

    let mut overlay = HostAccessConfig::default();
    overlay.env.allow = vec!["OVERLAY_ALLOW".to_string()];
    overlay.env.deny = vec!["OVERLAY_DENY".to_string()];

    let merged = base.merge(Some(&overlay));
    // allow whitelist: overlay replaces
    assert_eq!(merged.env.allow, vec!["OVERLAY_ALLOW".to_string()]);
    // deny blacklist: union of base and overlay
    assert!(merged.env.deny.contains(&"BASE_DENY".to_string()));
    assert!(merged.env.deny.contains(&"OVERLAY_DENY".to_string()));
}

#[test]
fn merge_exec_max_processes_overlay_wins_when_non_zero() {
    let mut base = HostAccessConfig::default();
    base.exec.max_processes = 5;
    let mut overlay = HostAccessConfig::default();
    overlay.exec.max_processes = 10;
    let merged = base.merge(Some(&overlay));
    assert_eq!(merged.exec.max_processes, 10);
}

#[test]
fn merge_modules_per_module_overlay_wins() {
    let mut base = HostAccessConfig::default();
    base.modules.default = Some(AccessDecision::Deny);
    base.modules.http = Some(AccessDecision::Deny);
    base.modules.tcp = Some(AccessDecision::Allow);

    let mut overlay = HostAccessConfig::default();
    overlay.modules.http = Some(AccessDecision::Allow);
    overlay.modules.process = Some(AccessDecision::Deny);

    let merged = base.merge(Some(&overlay));
    // default scalar unchanged because overlay left it at None
    assert_eq!(merged.modules.default, Some(AccessDecision::Deny));
    assert_eq!(merged.modules.http, Some(AccessDecision::Allow));
    assert_eq!(merged.modules.tcp, Some(AccessDecision::Allow));
    assert_eq!(merged.modules.process, Some(AccessDecision::Deny));
}

#[test]
fn merge_none_returns_base() {
    let base = HostAccessConfig::restrictive();
    let merged = base.merge(None);
    assert_eq!(merged.default, AccessDecision::Deny);
}

#[test]
fn to_policy_converts_decisions_and_lists() {
    let mut cfg = HostAccessConfig::default();
    cfg.default = AccessDecision::Allow;
    cfg.env.default = AccessDecision::Deny;
    cfg.env.allow = vec!["KEY".to_string()];
    cfg.exec.default = AccessDecision::Allow;
    cfg.exec.deny = vec!["rm".to_string()];
    cfg.modules.http = Some(AccessDecision::Deny);

    let policy = cfg.to_policy();
    assert!(policy.default.is_allowed());
    assert!(policy.ensure_env_read("KEY").is_ok());
    assert!(policy.ensure_env_read("OTHER").is_err());
    assert!(policy.ensure_exec_method("run").is_ok());
    assert!(policy.ensure_command_allowed("rm").is_err());
    assert!(policy.ensure_module_allowed("http").is_err());
}

#[test]
fn to_policy_default_deny_inherits_to_modules() {
    let cfg = HostAccessConfig::default(); // default Deny, modules.default None
    let policy = cfg.to_policy();
    assert!(policy.ensure_module_allowed("http").is_err());
}
