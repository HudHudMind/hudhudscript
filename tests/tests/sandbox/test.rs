//! Sandbox integration tests — restored after the interpreter
//! retirement.  Uses the VM-backed `vm_interpreter::Interpreter` shim,
//! which forwards `Interpreter::with_sandbox(config)` to
//! `VM::with_sandbox_config(config)`.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_sandbox::{
    FileSystemConfig, NetworkConfig, ProcessConfig, ResourceConfig, SandboxConfig,
};

#[test]
fn test_interpreter_with_sandbox_creation() {
    let config = SandboxConfig {
        filesystem: FileSystemConfig::default_restrictive(),
        network: NetworkConfig::default_restrictive(),
        process: ProcessConfig::default_restrictive(),
        resources: ResourceConfig::default_restrictive(),
    };

    let _interpreter = Interpreter::with_sandbox(config);
    // If we can create it, the test passes
}

#[test]
fn test_interpreter_without_sandbox_creation() {
    let _interpreter = Interpreter::new();
}

#[test]
fn test_permissive_sandbox_creation() {
    let config = SandboxConfig::default_permissive();
    let _interpreter = Interpreter::with_sandbox(config);
}

#[test]
fn test_restrictive_sandbox_creation() {
    let config = SandboxConfig::default_restrictive();
    let _interpreter = Interpreter::with_sandbox(config);
}

#[test]
fn test_sandbox_config_variants() {
    let restrictive = SandboxConfig::default_restrictive();
    let permissive = SandboxConfig::default_permissive();

    let _interp1 = Interpreter::with_sandbox(restrictive);
    let _interp2 = Interpreter::with_sandbox(permissive);
}
