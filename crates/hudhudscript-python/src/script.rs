//! PYTHON0003/0005: Persistent Script pyclass + provider injection.

use std::collections::HashMap;
use std::sync::mpsc;

use hudhudscript_bytecode::Value16;
use hudhudscript_compiler::Compiler;
use hudhudscript_vm::VM;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use crate::convert;

/// Persistent script — compiled once, called many times.
#[pyclass]
pub struct Script {
    source: String,
    providers: HashMap<String, HashMap<String, String>>,
    allow_network: bool,
}

#[pymethods]
impl Script {
    #[new]
    fn new(code: &str) -> PyResult<Self> {
        Ok(Script {
            source: code.to_string(),
            providers: HashMap::new(),
            allow_network: false,
        })
    }

    /// Set a provider programmatically.
    #[pyo3(signature = (name, **kwargs))]
    fn set_provider(&mut self, name: &str, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let mut fields = HashMap::new();
        if let Some(kw) = kwargs {
            for (k, v) in kw.iter() {
                fields.insert(k.extract::<String>()?, v.extract::<String>()?);
            }
        }
        self.providers.insert(name.to_string(), fields);
        Ok(())
    }

    /// Allow network access for provider calls.
    fn enable_network(&mut self) {
        self.allow_network = true;
    }

    /// Call a top-level function with arguments, return result as Python value.
    #[pyo3(signature = (func_name, *args))]
    fn call(&mut self, py: Python<'_>, func_name: &str, args: &Bound<'_, PyTuple>) -> PyResult<PyObject> {
        let mut hud_args = Vec::new();
        for arg in args.iter() {
            hud_args.push(convert::py_to_value(&arg)?);
        }
        let code = self.source.clone();
        let fn_name = func_name.to_string();
        let providers = self.providers.clone();
        let allow_network = self.allow_network;

        // Run in dedicated thread to avoid GIL + Send issues
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let ast = match hudhudscript_parser::parse(&code) {
                Ok(a) => a,
                Err(e) => { let _ = tx.send(Err(format!("Parse: {}", e))); return; }
            };
            let mut compiler = Compiler::new();
            let bytecode = match compiler.compile(&ast) {
                Ok(b) => b,
                Err(e) => { let _ = tx.send(Err(format!("Compile: {}", e))); return; }
            };
            let mut vm = VM::new();
            hudhudscript_vm::register_vm_stdlib_modules(&mut vm);
            vm.set_toml_providers(providers);
            if allow_network {
                vm.allow_network();
            }

            hudhud_print::print_ops::start_capture();
            let result = (|| -> Result<Value16, String> {
                vm.execute(&bytecode).map_err(|e| format!("{}", e))?;
                vm.call_public(&fn_name, &hud_args, &bytecode).map_err(|e| format!("{}", e))
            })();
            let output = hudhud_print::print_ops::stop_capture().unwrap_or_default();

            let _ = tx.send(match result {
                Ok(val) => Ok((output, Some(val))),
                Err(e) => {
                    // Return partial output even on error
                    let _ = tx.send(Ok((output, None)));
                    Err(format!("{}", e))
                }
            });
        });

        match rx.recv_timeout(std::time::Duration::from_secs(120)) {
            Ok(Ok((_output, Some(val)))) => convert::value_to_py(py, &val),
            Ok(Ok((_, None))) => Ok(py.None()),
            Ok(Err(e)) => Err(pyo3::exceptions::PyRuntimeError::new_err(e)),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err(pyo3::exceptions::PyTimeoutError::new_err("Script timed out"))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(pyo3::exceptions::PyRuntimeError::new_err("Script thread crashed"))
            }
        }
    }

    fn __repr__(&self) -> String {
        format!("Script({} chars, {} providers)", self.source.len(), self.providers.len())
    }
}
