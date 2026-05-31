//! Expression AST nodes

use crate::Span;
use serde::{Deserialize, Serialize};

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// String literal: "hello"
    String(String),
    /// Number literal: 42, 3.14
    Number(f64, bool),
    /// Boolean literal: true, false
    Boolean(bool),
    /// Null literal
    Null,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %

    // Comparison
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=

    // Logical
    And, // &&
    Or,  // ||

    // Null coalescing (#749)
    NullCoalesce, // ??

    // Instance check (#981)
    InstanceOf, // instanceof
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Logical not: !
    Not,
    /// Negation: -
    Neg,
    /// Unary plus: +
    Plus,
    /// Postfix increment: i++ (returns old value, then adds 1)
    PostIncrement,
    /// Postfix decrement: i-- (returns old value, then subtracts 1)
    PostDecrement,
}

/// Expression AST node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value: 42, "hello", true, null
    Literal(Literal, Span),

    /// Identifier: foo, bar
    Identifier(String, Span),

    /// Binary operation: a + b, x == y
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Unary operation: !x, -y
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    /// Function/method call: foo(a, b)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    /// Perform an agent action: perform Worker.double(21)
    Perform {
        action: Box<Expr>,
        span: Span,
    },

    /// Member access: obj.field
    Member {
        object: Box<Expr>,
        property: String,
        span: Span,
    },

    /// Optional member access: obj?.field (#749)
    /// Returns Null if obj is Null, otherwise does normal member access.
    OptionalMember {
        object: Box<Expr>,
        property: String,
        span: Span,
    },

    /// Index access: arr[0]
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    /// Ternary: condition ? true_expr : false_expr
    Ternary {
        condition: Box<Expr>,
        true_expr: Box<Expr>,
        false_expr: Box<Expr>,
        span: Span,
    },

    /// Array literal: [1, 2, 3]
    Array { elements: Vec<Expr>, span: Span },

    /// Object literal: {x: 1, y: 2}
    Object {
        properties: Vec<(String, Expr)>,
        span: Span,
    },

    /// Template string: `Hello ${name}`
    TemplateString {
        parts: Vec<TemplateStringPart>,
        span: Span,
    },

    /// Arrow function: (x, y) => x + y  or  fn(x) { ... }  or  async fn(x) { ... }
    ArrowFunction {
        params: Vec<String>,
        body: ArrowFunctionBody,
        is_async: bool,
        span: Span,
    },

    /// Await expression: await promise
    Await { expr: Box<Expr>, span: Span },

    /// New expression: new ClassName(args)
    New {
        class_name: String,
        args: Vec<Expr>,
        span: Span,
    },

    /// This/self reference: this, bu, kendi
    This(Span),

    /// Spread expression: ...expr (used in array/object literals and function calls)
    Spread { expr: Box<Expr>, span: Span },

    /// Yield expression: yield value (inside generator functions) — Issue #667
    Yield {
        value: Option<Box<Expr>>,
        span: Span,
    },

    /// Spawn expression: spawn actorName or spawn actorName(args)
    /// Used in expression context: let a = spawn myActor
    Spawn {
        subject_name: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// SOP0008: view expr as ViewType — runtime perspective addition
    ViewAs {
        instance: Box<Expr>,
        view_name: String,
        span: Span,
    },
}

/// Template string part (text or interpolation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemplateStringPart {
    Text(String),
    Interpolation(Box<Expr>),
}

/// Arrow function body (expression or block)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ArrowFunctionBody {
    Expression(Box<Expr>),
    Block(Vec<crate::Stmt>),
}

impl Expr {
    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Identifier(_, span) => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Perform { span, .. } => *span,
            Expr::Member { span, .. } => *span,
            Expr::OptionalMember { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Ternary { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::Object { span, .. } => *span,
            Expr::TemplateString { span, .. } => *span,
            Expr::ArrowFunction { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::New { span, .. } => *span,
            Expr::This(span) => *span,
            Expr::Spread { span, .. } => *span,
            Expr::Yield { span, .. } => *span,
            Expr::Spawn { span, .. } => *span,
            Expr::ViewAs { span, .. } => *span,
        }
    }
}
