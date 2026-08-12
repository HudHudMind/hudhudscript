//! AST-to-type conversion — [`Type::from_ast`].

use hudhudscript_ast::TypeAnnotation as AstType;

use super::Type;

impl Type {
    /// Convert AST type annotation to Type
    pub fn from_ast(ast_type: &AstType) -> Self {
        match ast_type {
            AstType::String => Type::String,
            AstType::Number => Type::Number,
            AstType::Boolean => Type::Boolean,
            AstType::Null => Type::Null,
            AstType::Any => Type::Any,
            AstType::Tool => Type::Tool {
                server: String::new(),
                tool_name: String::new(),
            },
            AstType::Resource => Type::Resource {
                server: String::new(),
                uri: String::new(),
            },
            AstType::Server => Type::Server,
            AstType::Generic(name) => Type::Generic(name.clone()),
            AstType::Array(elem) => Type::Array(Box::new(Type::from_ast(elem))),
            AstType::Union(types) => Type::Union(types.iter().map(Type::from_ast).collect()),
            AstType::Parameterized { base, .. } => Type::from_ast(base),
        }
    }
}
