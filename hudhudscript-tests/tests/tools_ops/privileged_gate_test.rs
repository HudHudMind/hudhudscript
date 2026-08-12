// The `sudo` gate on the firewall and apt modules must deny by default.
//
// `firewall_ops::dispatch` and `apt_ops::dispatch` shell out to `sudo ufw …` and
// `sudo apt-get …`. They take no policy context, and the coverage tests in
// test_deep_dispatch.rs call them directly — so `cargo test` really executed
// `sudo ufw reset --force`, `sudo ufw disable` and `sudo apt-get install -y curl`
// against the developer's machine. Each of those tests then sat for over a minute
// waiting on a password prompt; with passwordless sudo they would have silently
// reconfigured the firewall and installed packages.
//
// `hudhudscript_bytecode::privileged_ops` now guards every one of those call
// sites, off by default, and the runtime opts in only when `[runtime]
// allow_privileged = true`.
//
// These tests deliberately never call `allow_privileged_ops()`: flipping that
// process-global inside a test binary would re-arm sudo for every other test in
// the same process. Deny-by-default is the property worth pinning.
use hudhudscript_bytecode::privileged_ops;

#[test]
fn privileged_ops_denied_by_default() {
    assert!(
        !privileged_ops::privileged_ops_allowed(),
        "privilege escalation must be off unless the runtime grants it"
    );
}

#[test]
fn firewall_enable_is_refused_without_the_grant() {
    let err = hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Enable,
        &[],
    )
    .expect_err("firewall.enable must not reach sudo without the grant");
    let msg = format!("{err}");
    assert!(
        msg.contains("sudo") || msg.contains("privilege"),
        "the error should say why it was refused; got: {msg}"
    );
}

#[test]
fn firewall_reset_is_refused_without_the_grant() {
    // The most destructive one: `sudo ufw reset --force` used to run for real.
    assert!(
        hudhud_firewall::firewall_ops::dispatch(
            hudhud_firewall::firewall_ops::ScriptMethodId::Reset,
            &[],
        )
        .is_err(),
        "firewall.reset must not reach sudo without the grant"
    );
}

#[test]
fn firewall_status_is_refused_without_the_grant() {
    // Read-only, but still `sudo ufw status verbose`.
    assert!(hudhud_firewall::firewall_ops::dispatch(
        hudhud_firewall::firewall_ops::ScriptMethodId::Status,
        &[],
    )
    .is_err());
}

#[test]
fn apt_install_is_refused_without_the_grant() {
    let err = hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Install,
        &[hudhudscript_bytecode::Value16::string("curl".to_string())],
    )
    .expect_err("apt.install must not reach sudo without the grant");
    let msg = format!("{err}");
    assert!(
        msg.contains("sudo") || msg.contains("privilege"),
        "the error should say why it was refused; got: {msg}"
    );
}

#[test]
fn apt_remove_and_update_are_refused_without_the_grant() {
    assert!(hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Remove,
        &[hudhudscript_bytecode::Value16::string("unused-pkg".to_string())],
    )
    .is_err());
    assert!(hudhud_apt::apt_ops::dispatch(
        hudhud_apt::apt_ops::ScriptMethodId::Update,
        &[],
    )
    .is_err());
}
