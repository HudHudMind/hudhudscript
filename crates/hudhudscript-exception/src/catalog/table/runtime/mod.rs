use crate::catalog::category::ExceptionCategory;
use crate::catalog::codes::ExceptionCode;
use crate::catalog::entry::ExceptionEntry;

mod runtime_async;
pub use runtime_async::*;
pub use runtime_async::*;

mod execution;
pub use execution::*;
mod governance;
pub use governance::*;
mod variable;
pub use variable::*;
mod promise;
pub use promise::*;
mod control;
pub use control::*;
mod stm;
pub use stm::*;
pub static TABLE: &[ExceptionEntry] = &[
    ASYNC_RUNTIME_PROMISE_NOT_FOUND,
    ASYNC_RUNTIME_RUNTIME_ERROR,
    RUNTIME_AGENT_ALREADY_EXISTS,
    RUNTIME_AGENT_NOT_FOUND,
    RUNTIME_CALL_ERROR,
    RUNTIME_CUSTOM,
    RUNTIME_DIVISION_BY_ZERO,
    RUNTIME_EXECUTION_FAILED,
    RUNTIME_GOVERNANCE_VIOLATION,
    RUNTIME_IMMUTABLE_VARIABLE,
    RUNTIME_INDEX_OUT_OF_BOUNDS,
    RUNTIME_INVALID_OPERATION,
    RUNTIME_MODULE_ERROR,
    RUNTIME_OUT_OF_GAS,
    RUNTIME_PROMISE_REJECTED,
    RUNTIME_PROPERTY_NOT_FOUND,
    RUNTIME_RESOURCE_ERROR,
    RUNTIME_RETURN,
    RUNTIME_SECURITY_VIOLATION,
    RUNTIME_STATE_ERROR,
    RUNTIME_STM_MAX_RETRIES_EXCEEDED,
    RUNTIME_STM_TIMEOUT,
    RUNTIME_TASK_NOT_FOUND,
    RUNTIME_THROW,
    RUNTIME_TOOL_ERROR,
    RUNTIME_TYPE_ERROR,
    RUNTIME_UNDEFINED_VARIABLE,
    RUNTIME_UNINITIALIZED_VARIABLE,
    RUNTIME_VARIABLE_ALREADY_DEFINED,
    RUNTIME_YIELD,
];
