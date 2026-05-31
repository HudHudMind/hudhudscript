//! HudHudScript External Test Suite
//!
//! Tests are in tests/ directory.
//! Code is in ../hudhud-script/crates/

/// Shared VM test harness for script-driven tests.
///
/// All tests in this crate drive scripts through the VM (the only runtime
/// after the interpreter-crate retirement).  The helpers below package the
/// boilerplate `parse → compile → new VM → register stdlib → execute` so
/// individual test files can focus on the source + assertion.
///
/// Kural 7 (single source of truth): the stdlib registration itself lives
/// in `hudhudscript_vm::register_vm_stdlib_modules`, shared with the CLI.
pub mod harness {
    use hudhudscript_bytecode::Value16;
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;
    use hudhudscript_vm::{register_vm_stdlib_modules, VM};

    /// Any error bubbling out of parse / compile / execute, formatted for
    /// test-panic messages (tests normally `.unwrap()` on success).
    pub type HarnessError = String;

    /// Build a VM with the full stdlib module surface registered and run
    /// the given source script.  Returns the VM on success so callers can
    /// inspect globals via `vm.get_variable_owned(name)`.
    pub fn run_script(source: &str) -> Result<VM, HarnessError> {
        let ast = parse(source).map_err(|e| format!("parse error: {:?}", e))?;
        let mut compiler = Compiler::new();
        let bytecode = compiler
            .compile(&ast)
            .map_err(|e| format!("compile error: {:?}", e))?;
        let mut vm = VM::new();
        register_vm_stdlib_modules(&mut vm);
        vm.execute(&bytecode)
            .map_err(|e| format!("execute error: {:?}", e))?;
        Ok(vm)
    }

    /// Convenience: run the script and fetch a single variable as an owned
    /// `Value`.  Panics with a descriptive message on any failure — the
    /// intent is to replace `interp.get_variable(var).unwrap()` one-liners
    /// without adding ceremony to every test body.
    pub fn run_script_get(source: &str, var: &str) -> Value16 {
        let vm = run_script(source).unwrap_or_else(|e| panic!("run_script failed: {}", e));
        vm.get_variable_owned(var)
            .unwrap_or_else(|| panic!("variable `{}` not found after execution", var))
    }

    /// Run a script and assert it completes without error.  Ignores any
    /// globals — used for tests that care only about a side-effect such as
    /// elapsed time (sleep / delay).
    pub fn run_script_ok(source: &str) {
        run_script(source).unwrap_or_else(|e| panic!("run_script failed: {}", e));
    }
}

/// VM-backed shim for the retired `hudhudscript_interpreter::Interpreter` API.
///
/// Legacy tests drove scripts through `Interpreter::new() → .execute(&ast) →
/// .get_variable(name)`.  After the interpreter-crate retirement the VM is
/// the only runtime, so this shim exposes the same surface while delegating
/// to `hudhudscript_vm::VM`:
///
/// - `Interpreter::new()` builds a VM with the full stdlib registered.
/// - `execute(&[Stmt])` / `eval_program(&[Stmt])` compile + run the AST and
///   return the value of the last expression statement (or `Value16::null()`
///   when the program has no trailing expression, matching the interpreter).
/// - `get_variable(name)` returns `Result<Value16, RuntimeError>` — the same
///   shape the interpreter used, so assertion helpers stay byte-identical
///   (Kural 1).
///
/// This shim lives in the test crate, not in production, so it can't be used
/// to bring back the old runtime by the back door.  Its *only* purpose is
/// to keep preserved test bodies compiling after the import swap.
pub mod vm_interpreter {
    use hudhudscript_ast::Stmt;
    use hudhudscript_bytecode::Value16;
    use hudhudscript_compiler::Compiler;
    use hudhudscript_errors::Error as ErrorInner;
    use hudhudscript_governance::enforcement::EvaluationContext;
    use hudhudscript_governance::Constitution;
    use hudhudscript_mcp::{McpClient, Tool as McpToolDefinition};
    use hudhudscript_runtime::provider::ProviderRegistry;
    use hudhudscript_tools::ToolRegistry;
    use hudhudscript_vm::{register_vm_stdlib_modules, VM};
    use std::sync::Arc;

    /// Legacy `RuntimeError` re-export — the unified `hudhudscript_errors::Error`
    /// (the interpreter-core type alias was identical).
    pub type RuntimeError = ErrorInner;

    /// Legacy `runtime_codes` re-exports — tests that constructed errors
    /// via the interpreter-core helpers (`custom`, `type_error`, …) keep
    /// compiling against the shim.  Each constructor builds the same
    /// `hudhudscript_errors::Error` shape.
    pub mod runtime_codes {
        use hudhudscript_errors::{Error, ErrorCode};

        pub fn custom(s: impl Into<String>) -> Error {
            Error::new(ErrorCode::RuntimeCustom, s.into())
        }

        pub fn type_error(
            expected: impl Into<String>,
            found: impl Into<String>,
            operation: impl Into<String>,
        ) -> Error {
            let expected = expected.into();
            let found = found.into();
            let operation = operation.into();
            Error::new(
                ErrorCode::RuntimeTypeError,
                format!(
                    "Type error in {}: expected {}, found {}",
                    operation, expected, found
                ),
            )
            .with_context("expected", expected)
            .with_context("found", found)
            .with_context("operation", operation)
        }
    }

    /// Thin wrapper over `hudhudscript_vm::VM` with the interpreter's
    /// method names.
    pub struct Interpreter {
        /// Public so tests that need the raw VM surface can reach in.
        pub vm: VM,
    }

    impl Default for Interpreter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Interpreter {
        /// Build a fresh VM with the full stdlib module surface registered.
        ///
        /// Uses `VM::permissive()` — the interpreter historically ran
        /// with sandbox disabled by default, so preserved tests that
        /// invoke `this.call(...)` (network-reliant provider dispatch)
        /// don't trip the VM's stricter default.  Tests that want an
        /// explicit sandbox go through [`Interpreter::with_sandbox`].
        pub fn new() -> Self {
            let mut vm = VM::permissive();
            register_vm_stdlib_modules(&mut vm);
            Self { vm }
        }

        /// Builder: equivalent to the interpreter's
        /// `Interpreter::new().with_fuel(n)`.
        pub fn with_fuel(mut self, fuel: u64) -> Self {
            self.vm.with_fuel(fuel);
            self
        }

        /// Associated constructor: build an `Interpreter` with a
        /// sandboxed VM pre-configured from
        /// `hudhudscript_sandbox::SandboxConfig`.  Mirrors the legacy
        /// `Interpreter::with_sandbox(config)` API.
        pub fn with_sandbox(config: hudhudscript_sandbox::SandboxConfig) -> Self {
            let mut vm = VM::with_sandbox_config(config);
            register_vm_stdlib_modules(&mut vm);
            Self { vm }
        }

        /// Forward-to-VM helpers — one-line shims so restored test
        /// bodies stay byte-identical (Kural 1).
        pub fn set_provider_registry(&mut self, registry: Arc<ProviderRegistry>) {
            self.vm.set_provider_registry(registry);
        }

        pub fn provider_registry(&self) -> Option<Arc<ProviderRegistry>> {
            self.vm.provider_registry()
        }

        pub fn set_tool_registry(&mut self, registry: Arc<ToolRegistry>) {
            self.vm.set_tool_registry(registry);
        }

        pub fn tool_registry(&self) -> Option<Arc<ToolRegistry>> {
            self.vm.tool_registry()
        }

        pub fn register_mcp_client(&mut self, name: String, client: Arc<McpClient>) {
            self.vm.register_mcp_client(name, client);
        }

        pub fn get_mcp_client(&self, name: &str) -> Option<Arc<McpClient>> {
            self.vm.get_mcp_client(name)
        }

        pub fn set_mcp_tool_definitions(&self, server_name: String, tools: Vec<McpToolDefinition>) {
            self.vm.set_mcp_tool_definitions(server_name, tools);
        }

        pub fn get_mcp_tool_definitions(
            &self,
            server_name: &str,
        ) -> Option<Vec<McpToolDefinition>> {
            self.vm.get_mcp_tool_definitions(server_name)
        }

        pub fn has_active_constitution(&self) -> bool {
            self.vm.has_active_constitution()
        }

        pub fn get_active_constitution(&self) -> Option<Constitution> {
            self.vm.get_active_constitution()
        }

        pub fn check_constitution_compliance(
            &self,
            context: &EvaluationContext,
        ) -> Result<(), RuntimeError> {
            self.vm.check_constitution_compliance(context)
        }

        pub fn remaining_fuel(&self) -> Option<u64> {
            self.vm.remaining_fuel_opt()
        }

        pub fn reset_fuel(&mut self) {
            self.vm.reset_fuel();
        }

        /// Compile the given statements and execute them on the VM.
        ///
        /// Returns the last expression-statement value (matching the legacy
        /// interpreter behaviour) or `Value16::null()` if the program has no
        /// trailing expression.  Implemented by rewriting a final
        /// `Stmt::Expr(e)` into `Stmt::Let { name: "__last_expr", value: e }`
        /// so the value becomes available via `get_variable_owned` after
        /// the VM returns.
        pub fn execute(&mut self, statements: &[Stmt]) -> Result<Value16, RuntimeError> {
            let mut owned: Vec<Stmt> = statements.to_vec();
            let had_tail_expr = match owned.last() {
                Some(Stmt::Expr(_)) => true,
                _ => false,
            };
            if had_tail_expr {
                if let Some(Stmt::Expr(expr)) = owned.pop() {
                    let span = hudhudscript_ast::Span::new(
                        hudhudscript_ast::Position::new(1, 1, 0),
                        hudhudscript_ast::Position::new(1, 1, 0),
                    );
                    owned.push(Stmt::Let {
                        name: "__last_expr".to_string(),
                        value: expr,
                        span,
                    });
                }
            }

            let mut compiler = Compiler::new();
            let bytecode = compiler.compile(&owned)?;
            eprintln!("INTERPRETER BYTECODE:");
            for (i, instr) in bytecode.instructions.iter().enumerate() {
                eprintln!("  {}: {:?}", i, instr);
            }
            self.vm.execute(&bytecode)?;
            if had_tail_expr {
                Ok(self
                    .vm
                    .get_variable_owned("__last_expr")
                    .unwrap_or(Value16::null()))
            } else {
                Ok(Value16::null())
            }
        }

        /// Alias for `execute` — some legacy tests used `eval_program`.
        pub fn eval_program(&mut self, statements: &[Stmt]) -> Result<Value16, RuntimeError> {
            self.execute(statements)
        }

        /// Look up a variable by name.  Mirrors the interpreter's
        /// `Result<Value16, RuntimeError>` return shape — tests that did
        /// `.get_variable(name).unwrap()` or `.is_ok()` stay byte-identical.
        pub fn get_variable(&self, name: &str) -> Result<Value16, RuntimeError> {
            self.vm.get_variable_owned(name).ok_or_else(|| {
                hudhudscript_errors::Error::new(
                    hudhudscript_errors::ErrorCode::RuntimeUndefinedVariable,
                    format!("Undefined variable: {}", name),
                )
                .with_context("name", name.to_string())
            })
        }
    }
}
