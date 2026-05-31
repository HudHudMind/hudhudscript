//! Process sandbox

use crate::{ProcessConfig, Result, SandboxError};

pub struct ProcessSandbox {
    config: ProcessConfig,
    active_processes: std::sync::Arc<std::sync::Mutex<usize>>,
}

impl ProcessSandbox {
    pub fn new(config: ProcessConfig) -> Self {
        Self {
            config,
            active_processes: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// Check if process execution is allowed
    pub fn check_execution(&self, command: &str) -> Result<()> {
        // Extract command name (first word), then get just the binary name
        // from potential full path like /usr/bin/rm -> rm
        let cmd_word = command.split_whitespace().next().unwrap_or(command);
        let cmd_name = std::path::Path::new(cmd_word)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(cmd_word);

        // Check deny list first — match both full path and binary name
        for denied in &self.config.deny_commands {
            if cmd_name == denied || cmd_word == denied {
                return Err(SandboxError::ProcessDenied(format!(
                    "Command '{}' is explicitly denied",
                    cmd_name
                )));
            }
        }

        // Check allow list — match both full path and binary name
        let allowed = self
            .config
            .allow_commands
            .iter()
            .any(|allowed| allowed == "*" || allowed == cmd_name || allowed == cmd_word);

        if !allowed {
            return Err(SandboxError::ProcessDenied(format!(
                "Command '{}' is not allowed",
                cmd_name
            )));
        }

        // Check process limit
        let count = self.active_processes.lock().unwrap();
        if *count >= self.config.max_processes {
            return Err(SandboxError::ProcessDenied(format!(
                "Maximum process limit ({}) reached",
                self.config.max_processes
            )));
        }

        Ok(())
    }

    /// Increment active process count
    pub fn increment_process_count(&self) {
        let mut count = self.active_processes.lock().unwrap();
        *count += 1;
    }

    /// Decrement active process count
    pub fn decrement_process_count(&self) {
        let mut count = self.active_processes.lock().unwrap();
        if *count > 0 {
            *count -= 1;
        }
    }

    /// Get current process count
    pub fn get_process_count(&self) -> usize {
        *self.active_processes.lock().unwrap()
    }
}
