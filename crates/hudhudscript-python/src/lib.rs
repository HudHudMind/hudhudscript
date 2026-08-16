use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

mod convert;
mod script;

/// Result of code execution
#[pyclass]
#[derive(Clone)]
pub struct ExecutionResult {
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub output: String,
    #[pyo3(get)]
    pub error: Option<String>,
    #[pyo3(get)]
    pub execution_time_ms: u64,
}

#[pymethods]
impl ExecutionResult {
    fn __repr__(&self) -> String {
        format!(
            "ExecutionResult(success={}, output='{}', error={:?}, time={}ms)",
            self.success, self.output, self.error, self.execution_time_ms
        )
    }
}

/// Result of syntax check
#[pyclass]
#[derive(Clone)]
pub struct CheckResult {
    #[pyo3(get)]
    pub valid: bool,
    #[pyo3(get)]
    pub errors: Vec<String>,
    #[pyo3(get)]
    pub warnings: Vec<String>,
}

#[pymethods]
impl CheckResult {
    fn __repr__(&self) -> String {
        format!(
            "CheckResult(valid={}, errors={}, warnings={})",
            self.valid,
            self.errors.len(),
            self.warnings.len()
        )
    }
}

/// Result of compilation
#[pyclass]
#[derive(Clone)]
pub struct CompileResult {
    #[pyo3(get)]
    pub success: bool,
    #[pyo3(get)]
    pub bytecode: Option<Vec<u8>>,
    #[pyo3(get)]
    pub error: Option<String>,
}

#[pymethods]
impl CompileResult {
    fn __repr__(&self) -> String {
        format!(
            "CompileResult(success={}, bytecode_size={}, error={:?})",
            self.success,
            self.bytecode.as_ref().map(|b| b.len()).unwrap_or(0),
            self.error
        )
    }
}

/// Check HudHudScript code syntax
#[pyfunction]
fn check(code: &str) -> PyResult<CheckResult> {
    use hudhudscript_parser::parse;

    let mut errors = Vec::new();
    let warnings = Vec::new();

    // Parse (includes lexing and parsing)
    match parse(code) {
        Ok(_ast) => Ok(CheckResult {
            valid: true,
            errors,
            warnings,
        }),
        Err(e) => {
            errors.push(format!("Parse error: {:?}", e));
            Ok(CheckResult {
                valid: false,
                errors,
                warnings,
            })
        }
    }
}

/// Compile HudHudScript code to bytecode
#[pyfunction]
fn compile(code: &str) -> PyResult<CompileResult> {
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;

    // Parse
    let ast = match parse(code) {
        Ok(ast) => ast,
        Err(e) => {
            return Ok(CompileResult {
                success: false,
                bytecode: None,
                error: Some(format!("Parse error: {:?}", e)),
            });
        }
    };

    // Compile to bytecode
    let mut compiler = Compiler::new();
    let bytecode = match compiler.compile(&ast) {
        Ok(bc) => bc,
        Err(e) => {
            return Ok(CompileResult {
                success: false,
                bytecode: None,
                error: Some(format!("Compile error: {:?}", e)),
            });
        }
    };

    // Serialize bytecode
    let bytes = match bytecode.to_bytes() {
        Ok(b) => b,
        Err(e) => {
            return Ok(CompileResult {
                success: false,
                bytecode: None,
                error: Some(format!("Serialization error: {:?}", e)),
            });
        }
    };

    Ok(CompileResult {
        success: true,
        bytecode: Some(bytes),
        error: None,
    })
}

/// Run HudHudScript code via compile + VM (Faz 4 migration).
#[pyfunction]
fn run(code: &str, timeout_ms: Option<u64>) -> PyResult<ExecutionResult> {
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;
    use hudhudscript_vm::VM;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(30_000));

    // Parse before spawning (fast, no timeout needed)
    let ast = match parse(code) {
        Ok(ast) => ast,
        Err(e) => {
            return Ok(ExecutionResult {
                success: false,
                output: String::new(),
                error: Some(format!("Parse error: {:?}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            });
        }
    };

    // Compile up-front (fast, fails before thread spawn on any syntax issue)
    let mut compiler = Compiler::new();
    let bytecode = match compiler.compile(&ast) {
        Ok(b) => b,
        Err(e) => {
            return Ok(ExecutionResult {
                success: false,
                output: String::new(),
                error: Some(format!("Compile error: {:?}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            });
        }
    };

    // Run execution in a dedicated thread so we can enforce a wall-clock timeout.
    let (tx, rx) = mpsc::channel::<Result<(String, String), String>>();

    thread::spawn(move || {
        let mut vm = VM::new();

        hudhud_print::print_ops::start_capture();
        let exec_result = vm.execute(&bytecode);
        let captured_output = hudhud_print::print_ops::stop_capture().unwrap_or_default();

        let payload = match exec_result {
            Ok(()) => {
                // VM::execute returns () — for implicit last-expression
                // output, scripts should call print(). Captured stdout is
                // the primary channel. If nothing printed, return "".
                Ok((captured_output, String::new()))
            }
            Err(e) => Err(format!("Runtime error: {:?}", e)),
        };

        let _ = tx.send(payload);
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok((output, _))) => Ok(ExecutionResult {
            success: true,
            output,
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
        }),
        Ok(Err(err)) => Ok(ExecutionResult {
            success: false,
            output: String::new(),
            error: Some(err),
            execution_time_ms: start.elapsed().as_millis() as u64,
        }),
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(ExecutionResult {
            success: false,
            output: String::new(),
            error: Some(format!(
                "Execution timed out after {}ms",
                timeout.as_millis()
            )),
            execution_time_ms: start.elapsed().as_millis() as u64,
        }),
        Err(mpsc::RecvTimeoutError::Disconnected) => Ok(ExecutionResult {
            success: false,
            output: String::new(),
            error: Some("Execution thread terminated unexpectedly".to_string()),
            execution_time_ms: start.elapsed().as_millis() as u64,
        }),
    }
}

/// Run HudHudScript code from file
#[pyfunction]
fn run_file(path: &str, timeout_ms: Option<u64>) -> PyResult<ExecutionResult> {
    use std::fs;

    let code = fs::read_to_string(path)
        .map_err(|e| PyRuntimeError::new_err(format!("Failed to read file: {}", e)))?;

    run(&code, timeout_ms)
}

/// Get HudHudScript version
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// HudHudScript Python Module
#[pymodule]
fn hudhudscript(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add_function(wrap_pyfunction!(compile, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(run_file, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    m.add_class::<ExecutionResult>()?;
    m.add_class::<CheckResult>()?;
    m.add_class::<CompileResult>()?;
    m.add_class::<script::Script>()?;

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
