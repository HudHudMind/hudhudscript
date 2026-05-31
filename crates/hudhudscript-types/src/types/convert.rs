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
            AstType::Parameterized { base, args } => {
                let base_type = Type::from_ast(base);
                let param_types: Vec<Type> = args.iter().map(Type::from_ast).collect();
                // Special case: Array<T> uses the dedicated Array variant
                if matches!(base_type, Type::Array(_)) && param_types.len() == 1 {
                    return Type::Array(Box::new(param_types.into_iter().next().unwrap()));
                }
                // Special case: Promise<T> uses the dedicated Promise variant
                if matches!(base_type, Type::Promise(_)) && param_types.len() == 1 {
                    return Type::Promise(Box::new(param_types.into_iter().next().unwrap()));
                }
                Type::Parameterized {
                    base: Box::new(base_type),
                    params: param_types,
                }
            }
        }
    }
}
