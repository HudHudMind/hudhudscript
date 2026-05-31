//! systemd service and timer file generator.

use std::collections::HashMap;

/// Restart policy for the systemd unit.
#[derive(Debug, Clone, Default)]
pub enum RestartPolicy {
    #[default]
    OnFailure,
    Always,
    No,
    OnAbnormal,
    OnAbort,
    OnWatchdog,
}

impl RestartPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RestartPolicy::OnFailure => "on-failure",
            RestartPolicy::Always => "always",
            RestartPolicy::No => "no",
            RestartPolicy::OnAbnormal => "on-abnormal",
            RestartPolicy::OnAbort => "on-abort",
            RestartPolicy::OnWatchdog => "on-watchdog",
        }
    }
}

/// Configuration for a systemd `.service` unit.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub description: String,
    pub exec_start: String,
    pub user: Option<String>,
    pub group: Option<String>,
    pub working_dir: Option<String>,
    pub restart_policy: RestartPolicy,
    pub environment: HashMap<String, String>,
    /// Additional [Service] directives (e.g. LimitNOFILE, ProtectSystem).
    pub extra_service: HashMap<String, String>,
}

impl ServiceConfig {
    /// Create a new service config with the minimum required fields.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        exec_start: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            exec_start: exec_start.into(),
            user: None,
            group: None,
            working_dir: None,
            restart_policy: RestartPolicy::default(),
            environment: HashMap::new(),
            extra_service: HashMap::new(),
        }
    }

    /// Generate the `.service` unit file content.
    pub fn generate_unit(&self) -> String {
        let mut out = String::new();

        // [Unit]
        out.push_str("[Unit]\n");
        out.push_str(&format!("Description={}\n", self.description));
        out.push_str("After=network.target\n");
        out.push('\n');

        // [Service]
        out.push_str("[Service]\n");
        out.push_str("Type=simple\n");
        out.push_str(&format!("ExecStart={}\n", self.exec_start));
        out.push_str(&format!("Restart={}\n", self.restart_policy.as_str()));

        if let Some(ref user) = self.user {
            out.push_str(&format!("User={}\n", user));
        }
        if let Some(ref group) = self.group {
            out.push_str(&format!("Group={}\n", group));
        }
        if let Some(ref wd) = self.working_dir {
            out.push_str(&format!("WorkingDirectory={}\n", wd));
        }

        for (key, value) in &self.environment {
            out.push_str(&format!("Environment=\"{}={}\"\n", key, value));
        }

        for (key, value) in &self.extra_service {
            out.push_str(&format!("{}={}\n", key, value));
        }

        out.push('\n');

        // [Install]
        out.push_str("[Install]\n");
        out.push_str("WantedBy=multi-user.target\n");

        out
    }

    /// Generate a `.timer` unit file that triggers this service at `interval`.
    ///
    /// `interval` uses systemd calendar/monotonic syntax, e.g. `"hourly"`,
    /// `"*-*-* 03:00:00"`, `"15min"`.
    pub fn generate_timer(&self, interval: &str) -> String {
        let mut out = String::new();

        out.push_str("[Unit]\n");
        out.push_str(&format!("Description=Timer for {}\n", self.description));
        out.push('\n');

        out.push_str("[Timer]\n");
        out.push_str(&format!("OnCalendar={}\n", interval));
        out.push_str("Persistent=true\n");
        out.push_str(&format!("Unit={}.service\n", self.name));
        out.push('\n');

        out.push_str("[Install]\n");
        out.push_str("WantedBy=timers.target\n");

        out
    }
}
