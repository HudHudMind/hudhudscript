//! # hudhudscript-hudunit
//!
//! Unit test framework for the HudHudScript *language*: a pyunit/phpunit/pytest
//! style suite that runs `test_` functions discovered in `.hhs` / `.hud` /
//! `.hudhud` script files.
//!
//! Script authors declare tests with the `test_` naming convention and use the
//! `assert*` builtin wrappers injected by this framework:
//!
//! ```text
//! // tests/math/test_arithmetic.hud
//! # @group hizli
//! fn test_topla_basit() {
//!     assert_eq(topla(2, 3), 5);
//! }
//! ```
//!
//! The framework never modifies the lexer, parser, compiler or VM: assertions
//! are provided through a `hudunit` builtin module registered only on the
//! test runner's own VM instances, and coverage is collected by instrumenting
//! the AST *before* compilation (source files on disk are untouched).
//!
//! Main entry points:
//! - [`discover`] — file collection + `test_`/`setup`/`teardown` discovery
//! - [`groups`] — path-derived and `@group` annotation groups, selection
//! - [`runner`] — execution model (`call_public` per test, fresh VM per test)
//! - [`coverage`] — statement/function coverage via AST instrumentation
//! - [`report`] — console (colored), JSON and self-contained HTML reports

pub mod builtins;
pub mod capture;
pub mod config;
pub mod coverage;
pub mod discover;
pub mod groups;
pub mod init;
pub mod report;
pub mod runner;

pub use config::HudunitConfig;
pub use discover::{DiscoveredTest, TestFile};
pub use runner::{Outcome, RunOptions, SuiteOutcome, TestOutcome};
