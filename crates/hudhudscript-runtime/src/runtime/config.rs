//! Runtime configuration and statistics.

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Maximum concurrent executions
    pub max_concurrent_executions: usize,

    /// Enable execution history
    pub enable_history: bool,

    /// Maximum history size
    pub max_history_size: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_executions: 100,
            enable_history: true,
            max_history_size: 10000,
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone)]
pub struct RuntimeStatistics {
    /// Total number of agents
    pub total_agents: usize,

    /// Total number of executions
    pub total_executions: usize,

    /// Number of successful executions
    pub successful_executions: usize,

    /// Number of failed executions
    pub failed_executions: usize,

    /// Success rate
    pub success_rate: f64,
}
