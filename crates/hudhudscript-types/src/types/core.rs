//! Core type system — the [`Type`] enum and its `Display` implementation.

use std::collections::HashMap;
use std::fmt;

/// Type representation in the type system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// String type
    String,
    /// Number type (f64)
    Number,
    /// Boolean type
    Boolean,
    /// Null type
    Null,
    /// Any type (top type)
    Any,
    /// Array type with element type
    Array(Box<Type>),
    /// Object type with property types
    Object(HashMap<String, Type>),
    /// Function type with parameter types and return type
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    /// Tool type with server and tool name
    Tool { server: String, tool_name: String },
    /// Resource type with server and URI
    Resource { server: String, uri: String },
    /// Server type
    Server,
    /// Generic type variable
    Generic(String),
    /// Parameterized type (e.g. Array<Number>, Map<String, Number>)
    Parameterized { base: Box<Type>, params: Vec<Type> },
    /// Union type (multiple possible types)
    Union(Vec<Type>),
    /// Promise type (for async operations)
    Promise(Box<Type>),
    /// Class type with name and optional parent class name
    Class {
        name: String,
        parent: Option<String>,
    },
    /// Instance of a class
    Instance(String),
}

impl Type {
    /// Human-readable name of this type (used by Display)
    pub(crate) fn type_name(&self) -> String {
        match self {
            Type::String => "String".to_string(),
            Type::Number => "Number".to_string(),
            Type::Boolean => "Boolean".to_string(),
            Type::Null => "Null".to_string(),
            Type::Any => "Any".to_string(),
            Type::Server => "Server".to_string(),
            Type::Array(elem) => format!("Array<{}>", elem),
            Type::Function {
                params,
                return_type,
            } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) => {}", params_str, return_type)
            }
            Type::Promise(inner) => format!("Promise<{}>", inner),
            Type::Object(_) => "Object".to_string(),
            Type::Tool { server, tool_name } => format!("Tool({}.{})", server, tool_name),
            Type::Resource { server, uri } => format!("Resource({}.{})", server, uri),
            Type::Generic(name) => name.clone(),
            Type::Parameterized { base, params } => {
                let params_str = params
                    .iter()
                    .map(|p| format!("{}", p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", base, params_str)
            }
            Type::Union(types) => types
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(" | "),
            Type::Class { name, .. } => format!("class {}", name),
            Type::Instance(name) => name.clone(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.type_name())
    }
}
