//! Tests extracted from hudhudscript-sandbox/src/process.rs

use hudhudscript_sandbox::{ProcessConfig, ProcessSandbox};

#[test]
fn test_command_execution() {
    let config = ProcessConfig {
        allow_commands: vec!["python".to_string(), "node".to_string()],
        deny_commands: vec!["rm".to_string(), "dd".to_string()],
        max_processes: 10,
    };

    let sandbox = ProcessSandbox::new(config);

    // Should allow allowed commands
    assert!(sandbox.check_execution("python script.py").is_ok());
    assert!(sandbox.check_execution("node app.js").is_ok());

    // Should deny denied commands
    assert!(sandbox.check_execution("rm -rf /").is_err());
    assert!(sandbox.check_execution("dd if=/dev/zero").is_err());

    // Should deny non-allowed commands
    assert!(sandbox.check_execution("bash script.sh").is_err());
}

#[test]
fn test_process_limit() {
    let config = ProcessConfig {
        allow_commands: vec!["*".to_string()],
        deny_commands: vec![],
        max_processes: 2,
    };

    let sandbox = ProcessSandbox::new(config);

    // Should allow up to max_processes
    sandbox.increment_process_count();
    assert_eq!(sandbox.get_process_count(), 1);
    assert!(sandbox.check_execution("python").is_ok());

    sandbox.increment_process_count();
    assert_eq!(sandbox.get_process_count(), 2);

    // Should deny when limit reached
    assert!(sandbox.check_execution("python").is_err());

    // Should allow after decrement
    sandbox.decrement_process_count();
    assert!(sandbox.check_execution("python").is_ok());
}

#[test]
fn test_wildcard_allow() {
    let config = ProcessConfig {
        allow_commands: vec!["*".to_string()],
        deny_commands: vec!["rm".to_string()],
        max_processes: 10,
    };

    let sandbox = ProcessSandbox::new(config);

    // Should allow any command except denied
    assert!(sandbox.check_execution("python").is_ok());
    assert!(sandbox.check_execution("node").is_ok());
    assert!(sandbox.check_execution("bash").is_ok());

    // Should still deny explicitly denied commands
    assert!(sandbox.check_execution("rm").is_err());
}

#[test]
fn test_decrement_below_zero() {
    let config = ProcessConfig {
        allow_commands: vec!["*".to_string()],
        deny_commands: vec![],
        max_processes: 10,
    };
    let sandbox = ProcessSandbox::new(config);
    assert_eq!(sandbox.get_process_count(), 0);
    sandbox.decrement_process_count(); // should not go below 0
    assert_eq!(sandbox.get_process_count(), 0);
}

#[test]
fn test_empty_command() {
    let config = ProcessConfig {
        allow_commands: vec!["*".to_string()],
        deny_commands: vec![],
        max_processes: 10,
    };
    let sandbox = ProcessSandbox::new(config);
    assert!(sandbox.check_execution("").is_ok());
}

#[test]
fn test_full_path_bypass_blocked() {
    let config = ProcessConfig {
        allow_commands: vec!["python".to_string()],
        deny_commands: vec!["rm".to_string(), "dd".to_string()],
        max_processes: 10,
    };

    let sandbox = ProcessSandbox::new(config);

    // Full path to denied command should still be denied
    assert!(sandbox.check_execution("/usr/bin/rm -rf /").is_err());
    assert!(sandbox.check_execution("/bin/dd if=/dev/zero").is_err());

    // Full path to allowed command should work
    assert!(sandbox.check_execution("/usr/bin/python script.py").is_ok());
}
