use super::*;
impl Compiler {
    /// If the expression is an arrow function, compile it as a FunctionChunk
    /// and return a Function value. Otherwise, convert to a const value.
    pub(super) fn compile_session_hook(
        &mut self,
        decl_name: &str,
        hook_name: &str,
        hook_expr: &Expr,
    ) -> CompileResult<Value16> {
        match hook_expr {
            Expr::ArrowFunction {
                params,
                body,
                is_async,
                ..
            } => {
                use hudhudscript_ast::ArrowFunctionBody;
                let chunk_name = format!("{}::session::{}", decl_name, hook_name);
                let param_names: Vec<String> = params.clone();
                let stmts = match body {
                    ArrowFunctionBody::Block(stmts) => stmts.clone(),
                    ArrowFunctionBody::Expression(expr) => {
                        vec![hudhudscript_ast::Stmt::Return {
                            value: Some(*expr.clone()),
                            span: hudhudscript_ast::Span::default(),
                        }]
                    }
                };
                // P4: set current function name for call-site type lookup
                self.current_function_name = Some(chunk_name.clone());
                let chunk =
                    self.compile_function_body_async(param_names.clone(), &stmts, *is_async)?;
                self.current_function_name = None;
                let chunk_arc = Arc::new(chunk);
                // P3a: register for compiler-side inlining
                self.inline_function_chunks.insert(chunk_name.clone(), Arc::clone(&chunk_arc));
                self.bytecode
                    .add_function(chunk_name.clone(), chunk_arc);
                Ok(Value16::function(FunctionData {
                    name: chunk_name.clone(),
                    params: param_names,
                    chunk_name,
                    captures: Default::default(),
                }))
            }
            _ => Ok(self.expr_to_const_value(hook_expr)),
        }
    }

    /// Compile a function body (a list of statements) into a FunctionChunk.
    pub(super) fn compile_function_body(
        &mut self,
        params: Vec<String>,
        body: &[Stmt],
    ) -> CompileResult<FunctionChunk> {
        self.compile_function_body_async(params, body, false)
    }

    /// Compile a function body with async flag.
    pub(super) fn compile_function_body_async(
        &mut self,
        params: Vec<String>,
        body: &[Stmt],
        is_async: bool,
    ) -> CompileResult<FunctionChunk> {
        self.compile_function_body_named_async(params, None, body, is_async)
    }

    /// CROSS-1 (TCO): Compile a function body tagged with the
    /// function's name so self-tail-call detection can emit
    /// `TailCall` in the shared `Stmt::Return` path.
    ///
    /// Delegates to `compile_function_chunk_with` — the canonical
    /// function compilation context shared with loop/chain codegen.
    pub(super) fn compile_function_body_named_async(
        &mut self,
        params: Vec<String>,
        fn_name: Option<String>,
        body: &[Stmt],
        is_async: bool,
    ) -> CompileResult<FunctionChunk> {
        let body = body.to_vec();
        self.compile_function_chunk_with(params, fn_name, is_async, |compiler: &mut Compiler| -> CompileResult<()> {
            for stmt in &body {
                compiler.compile_stmt(stmt)?;
            }
            Ok(())
        })
    }
}
