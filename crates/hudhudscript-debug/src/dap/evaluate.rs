use super::*;

impl DapServer {
    pub(super) fn handle_evaluate<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let args: EvaluateArguments = match &request.arguments {
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            None => {
                let resp = self.make_error_response(request, "missing arguments");
                return Self::write_message(writer, &resp);
            }
        };

        // Try multiple resolution strategies:
        // 1. Direct variable lookup in scope
        // 2. Watch expression lookup
        // 3. Dot-path access (e.g. "obj.field") by searching variable store
        let result = self.evaluate_expression(&args.expression);

        match result {
            Some((value, ty)) => {
                let resp = self.make_response(
                    request,
                    true,
                    Some(serde_json::json!({
                        "result": value,
                        "type": ty,
                        "variablesReference": 0,
                    })),
                );
                Self::write_message(writer, &resp)
            }
            None => {
                let resp = self.make_error_response(
                    request,
                    &format!("cannot evaluate '{}'", args.expression),
                );
                Self::write_message(writer, &resp)
            }
        }
    }

    pub(super) fn handle_set_exception_breakpoints<W: Write>(
        &mut self,
        request: &DapRequest,
        writer: &mut W,
    ) -> io::Result<()> {
        let args: SetExceptionBreakpointsArguments = match &request.arguments {
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            None => {
                let resp = self.make_error_response(request, "missing arguments");
                return Self::write_message(writer, &resp);
            }
        };

        // Clear existing exception breakpoints first.
        let old_ids: Vec<_> = self
            .debugger
            .breakpoints()
            .iter()
            .filter(|bp| bp.is_exception())
            .map(|bp| bp.id)
            .collect();
        for id in old_ids {
            self.debugger.remove_breakpoint(id);
        }

        // The "all" filter means break on all exceptions.
        if args.filters.contains(&"all".to_string()) {
            self.debugger.set_break_on_all_exceptions(true);
        } else {
            self.debugger.set_break_on_all_exceptions(false);
        }

        // The "uncaught" filter adds a catch-all exception breakpoint.
        if args.filters.contains(&"uncaught".to_string()) {
            self.debugger.add_exception_breakpoint(None);
        }

        let resp = self.make_response(request, true, None);
        Self::write_message(writer, &resp)
    }

    // ---------------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------------

    /// Evaluate an expression string against the current debug state.
    ///
    /// Resolution order:
    /// 1. Direct variable name lookup in the debugger's scope variables
    /// 2. Watch expression last-known value
    /// 3. Dot-path access: for `a.b.c`, look up `a` in the variable store
    ///    and search nested children (if stored at higher variablesReferences)
    /// 4. Numeric/string literal passthrough
    pub fn evaluate_expression(&self, expression: &str) -> Option<(String, String)> {
        let expr = expression.trim();

        // 1. Direct scope variable lookup
        if let Some(var) = self.debugger.inspect(expr) {
            return Some((var.value.clone(), var.ty.clone()));
        }

        // 2. Watch expression lookup
        if let Some(val) = self.debugger.get_watch_value(expr) {
            return Some((val.to_string(), "watch".to_string()));
        }

        // 3. Dot-path: search the variable store for nested access
        if expr.contains('.') {
            let parts: Vec<&str> = expr.splitn(2, '.').collect();
            if parts.len() == 2 {
                let root = parts[0];
                let rest = parts[1];
                // Find the root variable in variable_store reference 1 (local scope)
                if let Some(vars) = self.variable_store.get(&1) {
                    for var in vars {
                        if var.name == root && var.variables_reference > 0 {
                            // Look for `rest` in the nested reference
                            if let Some(nested) = self.variable_store.get(&var.variables_reference)
                            {
                                // Recurse for deeper paths
                                if rest.contains('.') {
                                    let sub_parts: Vec<&str> = rest.splitn(2, '.').collect();
                                    for nvar in nested {
                                        if nvar.name == sub_parts[0] && nvar.variables_reference > 0
                                        {
                                            if let Some(deep) =
                                                self.variable_store.get(&nvar.variables_reference)
                                            {
                                                if sub_parts.len() == 2 {
                                                    for dvar in deep {
                                                        if dvar.name == sub_parts[1] {
                                                            return Some((
                                                                dvar.value.clone(),
                                                                dvar.ty.clone(),
                                                            ));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    for nvar in nested {
                                        if nvar.name == rest {
                                            return Some((nvar.value.clone(), nvar.ty.clone()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Literal passthrough for simple values
        if expr == "true" {
            return Some(("true".to_string(), "boolean".to_string()));
        }
        if expr == "false" {
            return Some(("false".to_string(), "boolean".to_string()));
        }
        if expr == "null" || expr == "nil" {
            return Some(("null".to_string(), "null".to_string()));
        }
        if expr.parse::<f64>().is_ok() {
            return Some((expr.to_string(), "number".to_string()));
        }
        if (expr.starts_with('"') && expr.ends_with('"'))
            || (expr.starts_with('\'') && expr.ends_with('\''))
        {
            let inner = &expr[1..expr.len() - 1];
            return Some((inner.to_string(), "string".to_string()));
        }

        None
    }

    /// Populate the variable store from the debugger's own scope variables.
    ///
    /// This is called automatically by `send_stopped_event` when no external
    /// variables are provided, ensuring the `variables` request always works
    /// when the debugger has scope data.
    pub fn populate_variables_from_scope(&mut self) {
        let scope_vars = self.debugger.scope_variables();
        if scope_vars.is_empty() {
            return;
        }
        let dap_vars: Vec<Variable> = scope_vars
            .iter()
            .map(|sv| Variable {
                name: sv.name.clone(),
                value: sv.value.clone(),
                ty: sv.ty.clone(),
                variables_reference: 0,
            })
            .collect();
        if !dap_vars.is_empty() {
            self.variable_store.insert(1, dap_vars);
        }
    }

    pub(super) fn current_source_json(&self) -> Value {
        match self.debugger.current_location() {
            Some((file, _)) => serde_json::json!({
                "path": file,
                "name": file,
            }),
            None => serde_json::json!(null),
        }
    }
}
