//! HudHudScript Testing Framework
//!
//! This crate provides the built-in testing framework for HudHudScript,
//! including assertions, test suites with lifecycle hooks, a configurable
//! test runner, mock/stub utilities, and coverage tracking structures.

pub mod assertion;
pub mod coverage;
pub mod framework;
pub mod mock;
pub mod runner;

pub use assertion::{Assertion, AssertionError};
pub use coverage::{CoverageReport, FileCoverage};
pub use framework::{LifecycleHook, Test, TestFramework, TestResult, TestSuite};
pub use mock::{EventMock, FunctionMock, MockCall, MockEvent, ProviderMock};
pub use runner::{RunResult, SuiteResult, TestFilter, TestRunner, Verbosity};
