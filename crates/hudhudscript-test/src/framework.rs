//! Test framework — suites, lifecycle hooks, and orchestration.

use std::time::{Duration, Instant};

/// Test result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestResult {
    Passed,
    Failed(String),
    Skipped,
}

/// Type alias for lifecycle hook closures.
pub type LifecycleHook = std::sync::Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// A single test case, optionally with an executable body.
#[derive(Clone)]
pub struct Test {
    pub name: String,
    pub result: Option<TestResult>,
    pub duration: Option<Duration>,
    /// Tags for filtering (e.g., "slow", "integration").
    pub tags: Vec<String>,
    /// Optional closure that implements the test body.
    /// Returns `Ok(())` on success or `Err(message)` on failure.
    pub body: Option<std::sync::Arc<dyn Fn() -> Result<(), String> + Send + Sync>>,
}

impl std::fmt::Debug for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Test")
            .field("name", &self.name)
            .field("result", &self.result)
            .field("duration", &self.duration)
            .field("tags", &self.tags)
            .field("body", &self.body.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Test {
    /// Create a test with no executable body (result must be set manually).
    pub fn new(name: String) -> Self {
        Self {
            name,
            result: None,
            duration: None,
            tags: Vec::new(),
            body: None,
        }
    }

    /// Create a test with an executable body.
    pub fn with_body<F>(name: String, body: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            name,
            result: None,
            duration: None,
            tags: Vec::new(),
            body: Some(std::sync::Arc::new(body)),
        }
    }

    /// Builder: attach tags to this test.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Returns true when this test carries the given tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// Execute the test body (if any) and record the result and elapsed time.
    ///
    /// If the result has already been set (e.g., to `Skipped` by the runner),
    /// the body is **not** executed. Tests without a body and without a
    /// pre-set result are marked as `Skipped`.
    pub fn run(&mut self) {
        // Respect pre-set results (e.g., filtered-out tests marked Skipped).
        if self.result.is_some() {
            return;
        }

        if let Some(body) = self.body.clone() {
            let start = Instant::now();
            let outcome = body();
            self.duration = Some(start.elapsed());
            self.result = Some(match outcome {
                Ok(()) => TestResult::Passed,
                Err(msg) => TestResult::Failed(msg),
            });
        } else {
            self.result = Some(TestResult::Skipped);
        }
    }
}

/// Test suite with lifecycle hooks.
#[derive(Clone)]
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<Test>,
    /// Total wall-clock time for the entire suite (populated after `run`).
    pub duration: Option<Duration>,

    /// Called once before any test in the suite runs.
    pub before_all: Option<LifecycleHook>,
    /// Called once after all tests in the suite have run.
    pub after_all: Option<LifecycleHook>,
    /// Called before each individual test.
    pub before_each: Option<LifecycleHook>,
    /// Called after each individual test.
    pub after_each: Option<LifecycleHook>,
}

impl std::fmt::Debug for TestSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSuite")
            .field("name", &self.name)
            .field("tests", &self.tests)
            .field("duration", &self.duration)
            .field("before_all", &self.before_all.as_ref().map(|_| "<fn>"))
            .field("after_all", &self.after_all.as_ref().map(|_| "<fn>"))
            .field("before_each", &self.before_each.as_ref().map(|_| "<fn>"))
            .field("after_each", &self.after_each.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl TestSuite {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tests: Vec::new(),
            duration: None,
            before_all: None,
            after_all: None,
            before_each: None,
            after_each: None,
        }
    }

    /// Builder: set the `before_all` hook.
    pub fn with_before_all<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        self.before_all = Some(std::sync::Arc::new(hook));
        self
    }

    /// Builder: set the `after_all` hook.
    pub fn with_after_all<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        self.after_all = Some(std::sync::Arc::new(hook));
        self
    }

    /// Builder: set the `before_each` hook.
    pub fn with_before_each<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        self.before_each = Some(std::sync::Arc::new(hook));
        self
    }

    /// Builder: set the `after_each` hook.
    pub fn with_after_each<F>(mut self, hook: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        self.after_each = Some(std::sync::Arc::new(hook));
        self
    }

    pub fn add_test(&mut self, test: Test) {
        self.tests.push(test);
    }

    /// Execute all tests in this suite, honouring lifecycle hooks.
    pub fn run(&mut self) {
        let start = Instant::now();

        // before_all
        if let Some(hook) = &self.before_all {
            if let Err(msg) = hook() {
                // If before_all fails, mark every test as failed.
                for test in &mut self.tests {
                    test.result = Some(TestResult::Failed(format!(
                        "before_all hook failed: {}",
                        msg
                    )));
                }
                self.duration = Some(start.elapsed());
                return;
            }
        }

        for test in &mut self.tests {
            // before_each
            if let Some(hook) = &self.before_each {
                if let Err(msg) = hook() {
                    test.result = Some(TestResult::Failed(format!(
                        "before_each hook failed: {}",
                        msg
                    )));
                    // still run after_each for cleanup
                    if let Some(ah) = &self.after_each {
                        let _ = ah();
                    }
                    continue;
                }
            }

            test.run();

            // after_each
            if let Some(hook) = &self.after_each {
                if let Err(msg) = hook() {
                    // If the test passed but after_each fails, mark as failed.
                    if test.result == Some(TestResult::Passed) {
                        test.result = Some(TestResult::Failed(format!(
                            "after_each hook failed: {}",
                            msg
                        )));
                    }
                }
            }
        }

        // after_all
        if let Some(hook) = &self.after_all {
            let _ = hook();
        }

        self.duration = Some(start.elapsed());
    }
}

/// Test framework
pub struct TestFramework {
    pub(crate) suites: Vec<TestSuite>,
}

impl TestFramework {
    pub fn new() -> Self {
        Self { suites: Vec::new() }
    }

    pub fn add_suite(&mut self, suite: TestSuite) {
        self.suites.push(suite);
    }

    pub fn suite_count(&self) -> usize {
        self.suites.len()
    }

    pub fn test_count(&self) -> usize {
        self.suites.iter().map(|s| s.tests.len()).sum()
    }

    /// Execute all tests in all suites and return aggregate counts.
    pub fn run_all(&mut self) -> (usize, usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for suite in &mut self.suites {
            suite.run();
            for test in &suite.tests {
                match &test.result {
                    Some(TestResult::Passed) => passed += 1,
                    Some(TestResult::Failed(_)) => failed += 1,
                    Some(TestResult::Skipped) | None => skipped += 1,
                }
            }
        }

        (passed, failed, skipped)
    }
}

impl Default for TestFramework {
    fn default() -> Self {
        Self::new()
    }
}
