//! MCP server declaration compilation for `CompileTarget`.

use super::*;

impl Compiler {

    pub fn compile_mcp_server(
        &mut self,
        mcp_decl: &hudhudscript_ast::McpServerDecl,
    ) -> CompileResult<()> {
        let config = &mcp_decl.config;
        let default_span = hudhudscript_ast::Span::default();
        let transport_str = match config.transport {
            hudhudscript_ast::TransportType::Stdio => "stdio",
            hudhudscript_ast::TransportType::SSE => "sse",
        };
        let mut fields: Vec<(String, Expr)> = vec![(
            "transport".to_string(),
            Expr::Literal(Literal::String(transport_str.to_string()), default_span),
        )];
        if let Some(ref cmd) = config.command {
            fields.push((
                "command".to_string(),
                Expr::Literal(Literal::String(cmd.clone()), default_span),
            ));
        }
        if !config.args.is_empty() {
            fields.push((
                "args".to_string(),
                Expr::Array {
                    elements: config
                        .args
                        .iter()
                        .map(|a| Expr::Literal(Literal::String(a.clone()), default_span))
                        .collect(),
                    span: default_span,
                },
            ));
        }
        if let Some(ref url) = config.url {
            fields.push((
                "url".to_string(),
                Expr::Literal(Literal::String(url.clone()), default_span),
            ));
        }
        if let Some(ref auth) = config.auth {
            let auth_type_str = match auth.auth_type {
                hudhudscript_ast::AuthType::Bearer => "bearer",
                hudhudscript_ast::AuthType::Basic => "basic",
                hudhudscript_ast::AuthType::ApiKey => "api_key",
            };
            let mut auth_fields: Vec<(String, Expr)> = vec![(
                "auth_type".to_string(),
                Expr::Literal(Literal::String(auth_type_str.to_string()), default_span),
            )];
            if let Some(ref token) = auth.token {
                auth_fields.push((
                    "token".to_string(),
                    Expr::Literal(Literal::String(token.clone()), default_span),
                ));
            }
            if let Some(ref username) = auth.username {
                auth_fields.push((
                    "username".to_string(),
                    Expr::Literal(Literal::String(username.clone()), default_span),
                ));
            }
            if let Some(ref password) = auth.password {
                auth_fields.push((
                    "password".to_string(),
                    Expr::Literal(Literal::String(password.clone()), default_span),
                ));
            }
            fields.push((
                "auth".to_string(),
                Expr::Object {
                    properties: auth_fields.into_iter().collect(),
                    span: default_span,
                },
            ));
        }
        self.compile_decl_as_object("mcp_server", &mcp_decl.name, &fields)
    }
}
