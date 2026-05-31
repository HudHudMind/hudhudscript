use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum DeployExceptionCode {
    /// E0070 — Deploy Adapter Reported Failure
    DeployAdapterError = 70,
    /// E0071 — Deploy Build Step Failed
    DeployBuildFailed = 71,
    /// E0072 — Deploy Configuration Invalid Or Missing
    DeployConfigError = 72,
    /// E0073 — Deploy Step Failed After Successful Build
    DeployDeployFailed = 73,
    /// E0074 — Deploy Rollback Could Not Complete
    DeployRollbackFailed = 74,
}
