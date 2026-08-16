//! Template parser — builds AST from lexer tokens.

use super::ast::{Expr, Node};
use super::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    fn peek_ahead(&self, offset: usize) -> &Token {
        self.tokens.get(self.pos + offset).unwrap_or(&Token::Eof)
    }
    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        let t = self.peek();
        if std::mem::discriminant(t) == std::mem::discriminant(&expected) {
            self.pos += 1;
            Ok(())
        } else {
            let msg = match (t, &expected) {
                (Token::Ident(_), Token::Ident(_)) => {
                    return {
                        self.pos += 1;
                        Ok(())
                    }
                }
                (Token::String(_), Token::String(_)) => {
                    return {
                        self.pos += 1;
                        Ok(())
                    }
                }
                _ => format!("expected {:?}, got {:?}", expected, t),
            };
            Err(msg)
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::Ident(s) => Ok(s.clone()),
            t => Err(format!("expected identifier, got {:?}", t)),
        }
    }
    fn expect_string(&mut self) -> Result<String, String> {
        match self.advance() {
            Token::String(s) => Ok(s.clone()),
            t => Err(format!("expected string, got {:?}", t)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        while !matches!(self.peek(), Token::Eof) {
            nodes.push(self.parse_node()?);
        }
        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<Node, String> {
        match self.peek() {
            Token::Text(_) => {
                if let Token::Text(s) = self.advance() {
                    Ok(Node::Text(s.clone()))
                } else {
                    unreachable!()
                }
            }
            Token::VarStart => self.parse_variable(),
            Token::TagStart => self.parse_tag(),
            _ => Err(format!("unexpected token: {:?}", self.peek())),
        }
    }

    fn parse_variable(&mut self) -> Result<Node, String> {
        self.expect(Token::VarStart)?;
        let expr = self.parse_expr()?;
        self.expect(Token::VarEnd)?;
        Ok(Node::Variable(expr))
    }

    fn parse_tag(&mut self) -> Result<Node, String> {
        self.expect(Token::TagStart)?;
        let node = match self.peek() {
            Token::If => self.parse_if(),
            Token::For => self.parse_for(),
            Token::Extends => self.parse_extends(),
            Token::Block => self.parse_block(),
            Token::Include => self.parse_include(),
            Token::Comment => self.parse_comment(),
            _ => Err(format!("unknown tag: {:?}", self.peek())),
        }?;
        self.expect(Token::TagEnd)?;
        Ok(node)
    }

    fn parse_if(&mut self) -> Result<Node, String> {
        self.expect(Token::If)?;
        let cond = self.parse_expr()?;
        self.expect(Token::TagEnd)?;
        let body = self.parse_until(&[Token::Elif, Token::Else, Token::EndIf])?;
        let mut conditions = vec![(cond, body)];
        let mut else_body = Vec::new();
        loop {
            if matches!(self.peek(), Token::TagStart) {
                self.advance();
            }
            match self.peek() {
                Token::Elif => {
                    self.advance();
                    let c = self.parse_expr()?;
                    self.expect(Token::TagEnd)?;
                    let b = self.parse_until(&[Token::Elif, Token::Else, Token::EndIf])?;
                    conditions.push((c, b));
                }
                Token::Else => {
                    self.advance();
                    self.expect(Token::TagEnd)?;
                    else_body = self.parse_until(&[Token::EndIf])?; /* fall through to EndIf */
                }
                Token::EndIf => {
                    self.advance();
                    break;
                }
                _ => break,
            }
        }
        Ok(Node::If {
            conditions,
            else_body,
        })
    }

    fn parse_for(&mut self) -> Result<Node, String> {
        self.expect(Token::For)?;
        let var = self.expect_ident()?;
        self.expect(Token::In)?;
        let iter = self.parse_expr()?;
        self.expect(Token::TagEnd)?;
        let body = self.parse_until(&[Token::Else, Token::EndFor])?;
        let else_body = if matches!(self.peek(), Token::TagStart)
            && matches!(self.peek_ahead(1), Token::Else)
        {
            self.advance();
            self.advance();
            self.expect(Token::TagEnd)?;
            self.parse_until(&[Token::EndFor])?
        } else {
            Vec::new()
        };
        if matches!(self.peek(), Token::TagStart) {
            self.advance();
        }
        self.expect(Token::EndFor)?;
        Ok(Node::For {
            var,
            iter,
            body,
            else_body,
        })
    }

    fn parse_extends(&mut self) -> Result<Node, String> {
        self.expect(Token::Extends)?;
        Ok(Node::Extends(self.expect_string()?))
    }
    fn parse_block(&mut self) -> Result<Node, String> {
        self.expect(Token::Block)?;
        let name = self.expect_ident()?;
        self.expect(Token::TagEnd)?;
        let body = self.parse_until(&[Token::EndBlock])?;
        if matches!(self.peek(), Token::TagStart) {
            self.advance();
        }
        self.expect(Token::EndBlock)?;
        Ok(Node::Block { name, body })
    }
    fn parse_include(&mut self) -> Result<Node, String> {
        self.expect(Token::Include)?;
        Ok(Node::Include(self.expect_string()?))
    }
    fn parse_comment(&mut self) -> Result<Node, String> {
        self.expect(Token::Comment)?;
        while !matches!(self.peek(), Token::EndComment | Token::Eof) {
            self.advance();
        }
        if matches!(self.peek(), Token::EndComment) {
            self.advance();
        }
        Ok(Node::Text(String::new()))
    }

    fn parse_until(&mut self, stop_tokens: &[Token]) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => break,
                Token::TagStart => {
                    let next = self.peek_ahead(1);
                    let next_next = self.peek_ahead(2);
                    let should_stop = stop_tokens
                        .iter()
                        .any(|s| std::mem::discriminant(s) == std::mem::discriminant(next));
                    let is_end = matches!(
                        next,
                        Token::EndIf | Token::EndFor | Token::EndBlock | Token::EndComment
                    );
                    let is_branch = matches!(next, Token::Else | Token::Elif)
                        && matches!(next_next, Token::TagEnd);
                    if should_stop || is_end || is_branch {
                        break;
                    }
                    nodes.push(self.parse_node()?);
                }
                t if stop_tokens
                    .iter()
                    .any(|s| std::mem::discriminant(s) == std::mem::discriminant(t)) =>
                {
                    break
                }
                Token::Text(_) | Token::VarStart => {
                    nodes.push(self.parse_node()?);
                }
                _ => {
                    self.advance();
                }
            }
        }
        Ok(nodes)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Token::Or) {
            self.advance();
            left = Expr::Or(Box::new(left), Box::new(self.parse_and()?));
        }
        Ok(left)
    }
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while matches!(self.peek(), Token::And) {
            self.advance();
            left = Expr::And(Box::new(left), Box::new(self.parse_equality()?));
        }
        Ok(left)
    }
    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            match self.peek() {
                Token::Eq => {
                    self.advance();
                    left = Expr::Eq(Box::new(left), Box::new(self.parse_comparison()?));
                }
                Token::Neq => {
                    self.advance();
                    left = Expr::Neq(Box::new(left), Box::new(self.parse_comparison()?));
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_filter()?;
        loop {
            match self.peek() {
                Token::Lt => {
                    self.advance();
                    left = Expr::Lt(Box::new(left), Box::new(self.parse_filter()?));
                }
                Token::Gt => {
                    self.advance();
                    left = Expr::Gt(Box::new(left), Box::new(self.parse_filter()?));
                }
                Token::Le => {
                    self.advance();
                    left = Expr::Le(Box::new(left), Box::new(self.parse_filter()?));
                }
                Token::Ge => {
                    self.advance();
                    left = Expr::Ge(Box::new(left), Box::new(self.parse_filter()?));
                }
                _ => break,
            }
        }
        Ok(left)
    }
    fn parse_filter(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        while matches!(self.peek(), Token::Pipe) {
            self.advance();
            let name = self.expect_ident()?;
            let mut args = Vec::new();
            if matches!(self.peek(), Token::LParen) {
                self.advance();
                if !matches!(self.peek(), Token::RParen) {
                    args.push(self.parse_expr()?);
                    while matches!(self.peek(), Token::Comma) {
                        self.advance();
                        args.push(self.parse_expr()?);
                    }
                }
                self.expect(Token::RParen)?;
            }
            expr = Expr::Filter(Box::new(expr), name, args);
        }
        Ok(expr)
    }
    fn parse_primary(&mut self) -> Result<Expr, String> {
        let mut expr = match self.peek() {
            Token::Ident(_) => Expr::Ident(self.expect_ident()?),
            Token::String(_) => Expr::String(self.expect_string()?),
            Token::Number(n) => {
                let v = *n;
                self.advance();
                Expr::Number(v)
            }
            Token::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(Token::RParen)?;
                e
            }
            Token::Not => {
                self.advance();
                Expr::Not(Box::new(self.parse_primary()?))
            }
            t => return Err(format!("unexpected token in expression: {:?}", t)),
        };
        loop {
            match self.peek() {
                Token::Dot => {
                    self.advance();
                    expr = Expr::Dot(Box::new(expr), self.expect_ident()?);
                }
                Token::LBracket => {
                    self.advance();
                    let idx = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::Bracket(Box::new(expr), Box::new(idx));
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}
