use super::*;
use hudhudscript_ast::{Decl, Expr, Literal, Stmt};
use std::collections::HashSet;

impl Compiler {
    pub(super) fn precompute_shared_top_level(&mut self, statements: &[Stmt]) {
        let mut top = HashSet::new();
        for s in statements {
            Self::collect_top_decls(s, &mut top);
        }
        let mut shared = HashSet::new();
        for s in statements {
            Self::collect_shared(s, &top, &mut shared);
        }
        for s in statements {
            Self::collect_top_level_calls(s, &top, &mut shared);
        }
        self.shared_top_level_names = shared;
    }
    fn collect_top_decls(stmt: &Stmt, out: &mut HashSet<String>) {
        match stmt {
            Stmt::Let { name, .. } | Stmt::Const { name, .. } | Stmt::Function { name, .. } => {
                out.insert(name.clone());
            }
            Stmt::VarDecl(vd) => {
                out.insert(vd.name.clone());
            }
            Stmt::Class(cd) => {
                out.insert(cd.name.clone());
            }
            // Export wraps an inner statement; recurse to find the declared name.
            Stmt::Export { item, .. } => {
                Self::collect_top_decls(item, out);
            }
            _ => {}
        }
    }
    /// Walk the full statement tree to find function/closure/class/agent
    /// bodies.  When a body is found, walk_refs marks top-level names
    /// referenced inside it as shared.  Containers (Block/If/While/For/
    /// Try/Match/Switch) are recursed to find nested declarations.
    fn collect_shared(stmt: &Stmt, top: &HashSet<String>, out: &mut HashSet<String>) {
        match stmt {
            // Function/closure/class bodies → walk_refs on each body statement
            Stmt::Function { body, .. } => {
                for s in body {
                    Self::walk_refs(s, top, out);
                }
            }
            Stmt::Class(cd) => {
                for m in &cd.members {
                    if let hudhudscript_ast::ClassMember::Method { body, .. }
                    | hudhudscript_ast::ClassMember::Constructor { body, .. } = m
                    {
                        for s in body {
                            Self::walk_refs(s, top, out);
                        }
                    }
                }
            }
            // Containers → recurse collect_shared (functions may be nested inside)
            Stmt::Block { statements, .. } => {
                for s in statements {
                    Self::collect_shared(s, top, out);
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                Self::collect_shared(then_branch, top, out);
                if let Some(e) = else_branch {
                    Self::collect_shared(e, top, out);
                }
            }
            Stmt::While { body, .. } => Self::collect_shared(body, top, out),
            Stmt::For { body, .. } => Self::collect_shared(body, top, out),
            Stmt::ForCStyle {
                init, update, body, ..
            } => {
                if let Some(i) = init {
                    Self::collect_shared(i, top, out);
                }
                if let Some(u) = update {
                    Self::collect_shared(u, top, out);
                }
                Self::collect_shared(body, top, out);
            }
            Stmt::ForRange { body, .. } => Self::collect_shared(body, top, out),
            Stmt::Try {
                try_block,
                catch_clause,
                finally_block,
                ..
            } => {
                Self::collect_shared(try_block, top, out);
                if let Some(c) = catch_clause {
                    Self::collect_shared(&c.body, top, out);
                }
                if let Some(f) = finally_block {
                    Self::collect_shared(f, top, out);
                }
            }
            Stmt::Match { arms, .. } => {
                for a in arms {
                    for s in &a.body {
                        Self::collect_shared(s, top, out);
                    }
                }
            }
            Stmt::Switch { cases, default, .. } => {
                for c in cases {
                    for s in &c.body {
                        Self::collect_shared(s, top, out);
                    }
                }
                if let Some(d) = default {
                    for s in d {
                        Self::collect_shared(s, top, out);
                    }
                }
            }
            Stmt::Export { item, .. } => Self::collect_shared(item, top, out),
            // Statements with expressions that may contain arrow functions
            Stmt::Expr(e) => Self::walk_closures_in_expr(e, top, out),
            Stmt::Let { value, .. } | Stmt::Const { value, .. } => {
                Self::walk_closures_in_expr(value, top, out)
            }
            Stmt::VarDecl(vd) => {
                if let Some(init) = &vd.initializer {
                    Self::walk_closures_in_expr(init, top, out);
                }
            }
            Stmt::Assignment { value, .. } => Self::walk_closures_in_expr(value, top, out),
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    Self::walk_closures_in_expr(v, top, out);
                }
            }
            Stmt::Throw { value, .. } => Self::walk_closures_in_expr(value, top, out),
            Stmt::Destructure { value, .. } => Self::walk_closures_in_expr(value, top, out),
            Stmt::Perform { action, .. } => Self::walk_closures_in_expr(action, top, out),
            Stmt::Require { condition, .. } => Self::walk_closures_in_expr(condition, top, out),
            Stmt::Remember { content, .. } => Self::walk_closures_in_expr(content, top, out),
            Stmt::Recall { query, .. } => Self::walk_closures_in_expr(query, top, out),
            Stmt::Forget { target, .. } => Self::walk_closures_in_expr(target, top, out),
            Stmt::Send {
                message, target, ..
            } => {
                Self::walk_closures_in_expr(message, top, out);
                Self::walk_closures_in_expr(target, top, out);
            }
            Stmt::Spawn { args, .. } => {
                for a in args {
                    Self::walk_closures_in_expr(a, top, out);
                }
            }
            Stmt::Receive { source, .. } => Self::walk_closures_in_expr(source, top, out),
            // Decl variants
            Stmt::Decl(decl) => Self::collect_shared_decl(decl, top, out),
            // No function bodies or closures — listed explicitly for exhaustiveness
            Stmt::McpServer(_) => {}
            Stmt::Break { .. } => {}
            Stmt::Continue { .. } => {}
            Stmt::Import { .. } => {}
            Stmt::Trait { .. } => {} // trait methods are signatures only (no bodies)
            Stmt::EnumDecl { .. } => {}
            Stmt::Despawn { .. } => {}
        }
    }
    /// Helper for Stmt::Decl — finds function bodies and closures in Decl variants.
    fn collect_shared_decl(
        decl: &hudhudscript_ast::Decl,
        top: &HashSet<String>,
        out: &mut HashSet<String>,
    ) {
        use hudhudscript_ast::Decl;
        match decl {
            // Bodies → walk_refs
            Decl::AgentAction { body, .. }
            | Decl::Ability { body, .. }
            | Decl::Effect { body, .. }
            | Decl::Step { body, .. } => {
                for s in body {
                    Self::walk_refs(s, top, out);
                }
            }
            Decl::Loop { items, .. } => {
                for item in items {
                    if let hudhudscript_ast::stmt::decl::LoopItemAst::InlineStep(s) = item {
                        Self::walk_refs_decl(s, top, out);
                    }
                }
            }
            Decl::Gate { .. } => {}
            Decl::Chain { links, .. } => {
                for link in links {
                    if let Some(inline) = &link.inline_loop {
                        Self::walk_refs_decl(inline, top, out);
                    }
                }
            }
            Decl::AttachStep { .. }
            | Decl::AttachLoop { .. }
            | Decl::RunLoop { .. }
            | Decl::RunChain { .. } => {}
            // Field expressions → walk_closures_in_expr (may contain arrow functions)
            Decl::Agent { fields, .. }
            | Decl::Action { fields, .. }
            | Decl::Tool { fields, .. }
            | Decl::Resource { fields, .. }
            | Decl::Provider { fields, .. }
            | Decl::Governance { fields, .. }
            | Decl::Relation { fields, .. }
            | Decl::Role { fields, .. }
            | Decl::Store { fields, .. }
            | Decl::Entity { fields, .. }
            | Decl::StateMachine { fields, .. }
            | Decl::Event { fields, .. }
            | Decl::Contract { fields, .. }
            | Decl::Treaty { fields, .. }
            | Decl::Music { fields, .. }
            | Decl::Deploy { fields, .. } => {
                for (_, v) in fields {
                    Self::walk_closures_in_expr(v, top, out);
                }
            }
            // Subject has multiple expression vectors
            Decl::Subject {
                states,
                perception,
                fields,
                memory,
                ..
            } => {
                for (_, v) in states
                    .iter()
                    .chain(perception.iter())
                    .chain(fields.iter())
                    .chain(memory.iter())
                {
                    Self::walk_closures_in_expr(v, top, out);
                }
            }
            // Protocol and Strategy have session: Vec<(String, Expr)>
            Decl::Protocol { session, .. } | Decl::Strategy { session, .. } => {
                for (_, v) in session {
                    Self::walk_closures_in_expr(v, top, out);
                }
            }
            // Law has rules: Vec<Expr>
            Decl::Law { rules, .. } => {
                for r in rules {
                    Self::walk_closures_in_expr(r, top, out);
                }
            }
            // Rule has conditions with value: Expr
            Decl::Rule { conditions, .. } => {
                for c in conditions {
                    Self::walk_closures_in_expr(&c.value, top, out);
                }
            }
            // Constitution has laws with rules: Vec<Expr>
            Decl::Constitution { laws, .. } => {
                for law in laws {
                    for r in &law.rules {
                        Self::walk_closures_in_expr(r, top, out);
                    }
                }
            }
            // No expressions — listed explicitly for exhaustiveness
            Decl::Import { .. } => {}
            Decl::Council { .. } => {}
            Decl::Swarm { .. } => {}
            Decl::Community { .. } => {}
            Decl::Compose { .. } => {}
            Decl::UiApp { .. } => {}
        }
    }
    /// Find and walk arrow-function bodies inside an expression WITHOUT
    /// marking top-level identifiers (unlike mark_expr).  Used by
    /// collect_shared on top-level expressions that may contain closures.
    fn walk_closures_in_expr(expr: &Expr, top: &HashSet<String>, out: &mut HashSet<String>) {
        match expr {
            Expr::Literal(_, _) => {}
            Expr::Identifier(_, _) => {}
            Expr::Binary { left, right, .. } => {
                Self::walk_closures_in_expr(left, top, out);
                Self::walk_closures_in_expr(right, top, out);
            }
            Expr::Unary { expr: e, .. } => Self::walk_closures_in_expr(e, top, out),
            Expr::Call { callee, args, .. } => {
                Self::walk_closures_in_expr(callee, top, out);
                for a in args {
                    Self::walk_closures_in_expr(a, top, out);
                }
            }
            Expr::Perform { action, .. } => Self::walk_closures_in_expr(action, top, out),
            Expr::Recall { query, .. } => Self::walk_closures_in_expr(query, top, out),
            Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
                Self::walk_closures_in_expr(object, top, out)
            }
            Expr::Index { object, index, .. } => {
                Self::walk_closures_in_expr(object, top, out);
                Self::walk_closures_in_expr(index, top, out);
            }
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
                ..
            } => {
                Self::walk_closures_in_expr(condition, top, out);
                Self::walk_closures_in_expr(true_expr, top, out);
                Self::walk_closures_in_expr(false_expr, top, out);
            }
            Expr::Array { elements, .. } => {
                for e in elements {
                    Self::walk_closures_in_expr(e, top, out);
                }
            }
            Expr::Object { properties, .. } => {
                for (_, v) in properties {
                    Self::walk_closures_in_expr(v, top, out);
                }
            }
            Expr::TemplateString { parts, .. } => {
                for p in parts {
                    if let hudhudscript_ast::TemplateStringPart::Interpolation(e) = p {
                        Self::walk_closures_in_expr(e, top, out);
                    }
                }
            }
            Expr::ArrowFunction { body, .. } => match body {
                hudhudscript_ast::ArrowFunctionBody::Block(stmts) => {
                    for s in stmts {
                        Self::walk_refs(s, top, out);
                    }
                }
                hudhudscript_ast::ArrowFunctionBody::Expression(e) => {
                    Self::mark_expr(e, top, out);
                }
            },
            Expr::Await { expr: e, .. } => Self::walk_closures_in_expr(e, top, out),
            Expr::New { args, .. } => {
                for a in args {
                    Self::walk_closures_in_expr(a, top, out);
                }
            }
            Expr::This(_) => {}
            Expr::Spread { expr: e, .. } => Self::walk_closures_in_expr(e, top, out),
            Expr::Yield { value, .. } => {
                if let Some(v) = value {
                    Self::walk_closures_in_expr(v, top, out);
                }
            }
            Expr::Spawn { args, .. } => {
                for a in args {
                    Self::walk_closures_in_expr(a, top, out);
                }
            }
            Expr::ViewAs { instance, .. } => Self::walk_closures_in_expr(instance, top, out),
        }
    }
    fn collect_top_level_calls(stmt: &Stmt, top: &HashSet<String>, out: &mut HashSet<String>) {
        match stmt {
            Stmt::Block { statements, .. } => {
                for s in statements {
                    Self::collect_top_level_calls(s, top, out);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                Self::mark_call_expr(condition, top, out);
                Self::collect_top_level_calls(body, top, out);
            }
            Stmt::For { iterable, body, .. } => {
                Self::mark_call_expr(iterable, top, out);
                Self::collect_top_level_calls(body, top, out);
            }
            Stmt::ForCStyle {
                init,
                condition,
                update,
                body,
                ..
            } => {
                if let Some(i) = init {
                    Self::collect_top_level_calls(i, top, out);
                }
                if let Some(c) = condition {
                    Self::mark_call_expr(c, top, out);
                }
                if let Some(u) = update {
                    Self::collect_top_level_calls(u, top, out);
                }
                Self::collect_top_level_calls(body, top, out);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                Self::mark_call_expr(condition, top, out);
                Self::collect_top_level_calls(then_branch, top, out);
                if let Some(e) = else_branch {
                    Self::collect_top_level_calls(e, top, out);
                }
            }
            Stmt::Assignment { value, .. }
            | Stmt::Let { value, .. }
            | Stmt::Const { value, .. }
            | Stmt::Throw { value, .. } => Self::mark_call_expr(value, top, out),
            Stmt::VarDecl(vd) => {
                if let Some(init) = &vd.initializer {
                    Self::mark_call_expr(init, top, out);
                }
            }
            Stmt::Destructure { value, .. } | Stmt::Expr(value) => {
                Self::mark_call_expr(value, top, out);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    Self::mark_call_expr(value, top, out);
                }
            }
            Stmt::Export { item, .. } => Self::collect_top_level_calls(item, top, out),
            _ => {}
        }
    }
    /// Walk ALL statement expression fields inside a function body.
    fn walk_refs(stmt: &Stmt, top: &HashSet<String>, out: &mut HashSet<String>) {
        match stmt {
            Stmt::Block { statements, .. } => {
                for s in statements {
                    Self::walk_refs(s, top, out);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                Self::mark_expr(condition, top, out);
                Self::walk_refs(body, top, out);
            }
            Stmt::For {
                variable: _,
                iterable,
                body,
                ..
            } => {
                Self::mark_expr(iterable, top, out);
                Self::walk_refs(body, top, out);
            }
            Stmt::ForCStyle {
                init,
                condition,
                update,
                body,
                ..
            } => {
                if let Some(i) = init {
                    Self::walk_refs(i, top, out);
                }
                if let Some(c) = condition {
                    Self::mark_expr(c, top, out);
                }
                if let Some(u) = update {
                    Self::walk_refs(u, top, out);
                }
                Self::walk_refs(body, top, out);
            }
            Stmt::ForRange {
                start,
                stop,
                step,
                body,
                ..
            } => {
                Self::mark_expr(start, top, out);
                Self::mark_expr(stop, top, out);
                if let Some(s) = step {
                    Self::mark_expr(s, top, out);
                }
                Self::walk_refs(body, top, out);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                Self::mark_expr(condition, top, out);
                Self::walk_refs(then_branch, top, out);
                if let Some(e) = else_branch {
                    Self::walk_refs(e, top, out);
                }
            }
            Stmt::Function { body, .. } => {
                for s in body {
                    Self::walk_refs(s, top, out);
                }
            }
            Stmt::Class(cd) => {
                for m in &cd.members {
                    if let hudhudscript_ast::ClassMember::Method { body, .. }
                    | hudhudscript_ast::ClassMember::Constructor { body, .. } = m
                    {
                        for s in body {
                            Self::walk_refs(s, top, out);
                        }
                    }
                }
            }
            Stmt::Match { value, arms, .. } => {
                Self::mark_expr(value, top, out);
                for a in arms {
                    for s in &a.body {
                        Self::walk_refs(s, top, out);
                    }
                }
            }
            Stmt::Switch {
                value,
                cases,
                default,
                ..
            } => {
                Self::mark_expr(value, top, out);
                for c in cases {
                    Self::mark_expr(&c.value, top, out);
                    for s in &c.body {
                        Self::walk_refs(s, top, out);
                    }
                }
                if let Some(d) = default {
                    for s in d {
                        Self::walk_refs(s, top, out);
                    }
                }
            }
            Stmt::Try {
                try_block,
                catch_clause,
                finally_block,
                ..
            } => {
                Self::walk_refs(try_block, top, out);
                if let Some(c) = catch_clause {
                    Self::walk_refs(&c.body, top, out);
                }
                if let Some(f) = finally_block {
                    Self::walk_refs(f, top, out);
                }
            }
            Stmt::Assignment { target, value, .. } => {
                Self::mark_id(target, top, out);
                Self::mark_expr(value, top, out);
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    Self::mark_expr(v, top, out);
                }
            }
            Stmt::Throw { value, .. } => {
                Self::mark_expr(value, top, out);
            }
            Stmt::Let { value, .. } | Stmt::Const { value, .. } => {
                Self::mark_expr(value, top, out);
            }
            Stmt::VarDecl(vd) => {
                if let Some(init) = &vd.initializer {
                    Self::mark_expr(init, top, out);
                }
            }
            Stmt::Destructure { value, .. } => {
                Self::mark_expr(value, top, out);
            }
            Stmt::Expr(e) => Self::mark_expr(e, top, out),
            Stmt::Perform { action, .. } => {
                Self::mark_expr(action, top, out);
            }
            Stmt::Require { condition, .. } => {
                Self::mark_expr(condition, top, out);
            }
            Stmt::Remember { content, .. } => {
                Self::mark_expr(content, top, out);
            }
            Stmt::Recall { query, .. } => {
                Self::mark_expr(query, top, out);
            }
            Stmt::Forget { target, .. } => {
                Self::mark_expr(target, top, out);
            }
            Stmt::Send {
                message, target, ..
            } => {
                Self::mark_expr(message, top, out);
                Self::mark_expr(target, top, out);
            }
            Stmt::Spawn { args, .. } => {
                for a in args {
                    Self::mark_expr(a, top, out);
                }
            }
            Stmt::Receive { source, .. } => {
                Self::mark_expr(source, top, out);
            }
            Stmt::Decl(decl) => Self::walk_refs_decl(decl, top, out),
            // No expression fields — listed explicitly for exhaustiveness
            Stmt::Break { .. } => {}
            Stmt::Continue { .. } => {}
            Stmt::Import { .. } => {}
            Stmt::Export { item, .. } => Self::walk_refs(item, top, out),
            Stmt::Trait { .. } => {}
            Stmt::EnumDecl { .. } => {}
            Stmt::Despawn { .. } => {}
            Stmt::McpServer(_) => {}
        }
    }
    /// Helper for walk_refs on Stmt::Decl — marks top-level references in Decl.
    fn walk_refs_decl(
        decl: &hudhudscript_ast::Decl,
        top: &HashSet<String>,
        out: &mut HashSet<String>,
    ) {
        use hudhudscript_ast::Decl;
        match decl {
            Decl::AgentAction { body, .. }
            | Decl::Ability { body, .. }
            | Decl::Effect { body, .. }
            | Decl::Step { body, .. } => {
                for s in body {
                    Self::walk_refs(s, top, out);
                }
            }
            Decl::Loop { items, .. } => {
                for item in items {
                    if let hudhudscript_ast::stmt::decl::LoopItemAst::InlineStep(s) = item {
                        Self::walk_refs_decl(s, top, out);
                    }
                }
            }
            Decl::Gate { .. } => {}
            Decl::Chain { links, .. } => {
                for link in links {
                    if let Some(inline) = &link.inline_loop {
                        Self::walk_refs_decl(inline, top, out);
                    }
                }
            }
            Decl::AttachStep { .. }
            | Decl::AttachLoop { .. }
            | Decl::RunLoop { .. }
            | Decl::RunChain { .. } => {}
            Decl::Agent { fields, .. }
            | Decl::Action { fields, .. }
            | Decl::Tool { fields, .. }
            | Decl::Resource { fields, .. }
            | Decl::Provider { fields, .. }
            | Decl::Governance { fields, .. }
            | Decl::Relation { fields, .. }
            | Decl::Role { fields, .. }
            | Decl::Store { fields, .. }
            | Decl::Entity { fields, .. }
            | Decl::StateMachine { fields, .. }
            | Decl::Event { fields, .. }
            | Decl::Contract { fields, .. }
            | Decl::Treaty { fields, .. }
            | Decl::Music { fields, .. }
            | Decl::Deploy { fields, .. } => {
                for (_, v) in fields {
                    Self::mark_expr(v, top, out);
                }
            }
            Decl::Subject {
                states,
                perception,
                fields,
                memory,
                ..
            } => {
                for (_, v) in states
                    .iter()
                    .chain(perception.iter())
                    .chain(fields.iter())
                    .chain(memory.iter())
                {
                    Self::mark_expr(v, top, out);
                }
            }
            Decl::Protocol { session, .. } | Decl::Strategy { session, .. } => {
                for (_, v) in session {
                    Self::mark_expr(v, top, out);
                }
            }
            Decl::Law { rules, .. } => {
                for r in rules {
                    Self::mark_expr(r, top, out);
                }
            }
            Decl::Rule { conditions, .. } => {
                for c in conditions {
                    Self::mark_expr(&c.value, top, out);
                }
            }
            Decl::Constitution { laws, .. } => {
                for law in laws {
                    for r in &law.rules {
                        Self::mark_expr(r, top, out);
                    }
                }
            }
            Decl::Import { .. } => {}
            Decl::Council { .. } => {}
            Decl::Swarm { .. } => {}
            Decl::Community { .. } => {}
            Decl::Compose { .. } => {}
            Decl::UiApp { .. } => {}
        }
    }
    fn mark_id(expr: &Expr, top: &HashSet<String>, out: &mut HashSet<String>) {
        match expr {
            Expr::Identifier(name, _) => {
                if top.contains(name) {
                    out.insert(name.clone());
                }
            }
            Expr::Member { object, .. } => Self::mark_id(object, top, out),
            Expr::Index { object, index, .. } => {
                Self::mark_id(object, top, out);
                Self::mark_expr(index, top, out);
            }
            _ => {}
        }
    }
    fn mark_expr(expr: &Expr, top: &HashSet<String>, out: &mut HashSet<String>) {
        match expr {
            Expr::Literal(_, _) => {}
            Expr::Identifier(name, _) => {
                if top.contains(name) {
                    out.insert(name.clone());
                }
            }
            Expr::ArrowFunction { body, .. } => match body {
                hudhudscript_ast::ArrowFunctionBody::Block(stmts) => {
                    for s in stmts {
                        Self::walk_refs(s, top, out);
                    }
                }
                hudhudscript_ast::ArrowFunctionBody::Expression(e) => {
                    Self::mark_expr(e, top, out);
                }
            },
            Expr::Call { callee, args, .. } => {
                Self::mark_expr(callee, top, out);
                for a in args {
                    Self::mark_expr(a, top, out);
                }
            }
            Expr::Binary { left, right, .. } => {
                Self::mark_expr(left, top, out);
                Self::mark_expr(right, top, out);
            }
            Expr::Unary { expr: e, .. } | Expr::Await { expr: e, .. } => {
                Self::mark_expr(e, top, out)
            }
            Expr::Perform { action, .. } => Self::mark_expr(action, top, out),
            Expr::Recall { query, .. } => Self::mark_expr(query, top, out),
            Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
                Self::mark_expr(object, top, out)
            }
            Expr::Index { object, index, .. } => {
                Self::mark_expr(object, top, out);
                Self::mark_expr(index, top, out);
            }
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
                ..
            } => {
                Self::mark_expr(condition, top, out);
                Self::mark_expr(true_expr, top, out);
                Self::mark_expr(false_expr, top, out);
            }
            Expr::Array { elements, .. } => {
                for e in elements {
                    Self::mark_expr(e, top, out);
                }
            }
            Expr::Object { properties, .. } => {
                for (_, val) in properties {
                    Self::mark_expr(val, top, out);
                }
            }
            Expr::TemplateString { parts, .. } => {
                for p in parts {
                    if let hudhudscript_ast::TemplateStringPart::Interpolation(e) = p {
                        Self::mark_expr(e, top, out);
                    }
                }
            }
            Expr::New { args, .. } => {
                for a in args {
                    Self::mark_expr(a, top, out);
                }
            }
            Expr::This(_) => {}
            Expr::Spread { expr: e, .. } => Self::mark_expr(e, top, out),
            Expr::Yield { value, .. } => {
                if let Some(v) = value {
                    Self::mark_expr(v, top, out);
                }
            }
            Expr::Spawn { args, .. } => {
                for a in args {
                    Self::mark_expr(a, top, out);
                }
            }
            Expr::ViewAs { instance, .. } => Self::mark_expr(instance, top, out),
        }
    }
    fn mark_call_expr(expr: &Expr, top: &HashSet<String>, out: &mut HashSet<String>) {
        match expr {
            Expr::Call { callee, args, .. } => {
                if let Expr::Identifier(name, _) = callee.as_ref() {
                    if top.contains(name) {
                        out.insert(name.clone());
                    }
                }
                Self::mark_call_expr(callee, top, out);
                for arg in args {
                    Self::mark_call_expr(arg, top, out);
                }
            }
            Expr::Binary { left, right, .. } => {
                Self::mark_call_expr(left, top, out);
                Self::mark_call_expr(right, top, out);
            }
            Expr::Unary { expr, .. } | Expr::Await { expr, .. } | Expr::Spread { expr, .. } => {
                Self::mark_call_expr(expr, top, out);
            }
            Expr::Member { object, .. } | Expr::OptionalMember { object, .. } => {
                Self::mark_call_expr(object, top, out);
            }
            Expr::Index { object, index, .. } => {
                Self::mark_call_expr(object, top, out);
                Self::mark_call_expr(index, top, out);
            }
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
                ..
            } => {
                Self::mark_call_expr(condition, top, out);
                Self::mark_call_expr(true_expr, top, out);
                Self::mark_call_expr(false_expr, top, out);
            }
            Expr::Array { elements, .. } => {
                for elem in elements {
                    Self::mark_call_expr(elem, top, out);
                }
            }
            Expr::Object { properties, .. } => {
                for (_, value) in properties {
                    Self::mark_call_expr(value, top, out);
                }
            }
            _ => {}
        }
    }
}
