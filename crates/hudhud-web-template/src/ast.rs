//! Template AST node types.

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    String(String),
    Number(f64),
    Dot(Box<Expr>, String),
    Bracket(Box<Expr>, Box<Expr>),
    Filter(Box<Expr>, String, Vec<Expr>),
    Not(Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Variable(Expr),
    If {
        conditions: Vec<(Expr, Vec<Node>)>,
        else_body: Vec<Node>,
    },
    For {
        var: String,
        iter: Expr,
        body: Vec<Node>,
        else_body: Vec<Node>,
    },
    Extends(String),
    Block {
        name: String,
        body: Vec<Node>,
    },
    Include(String),
}
