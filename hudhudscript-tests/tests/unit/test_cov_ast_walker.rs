//! Coverage tests for the statement child walker
//! (`crates/hudhudscript-ast/src/visitor/stmt_walker.rs`).
//!
//! Each test records the exact enter/leave visit sequence produced by
//! `walk_stmts`/`walk_stmt` (parsed source when practical, public AST
//! constructors otherwise) and compares it with the expected sequence —
//! pinning child order, leaf behavior, and Stop/SkipChildren semantics.

use hudhudscript_ast::visitor::*;
use hudhudscript_ast::{
    AccessModifier, BinaryOp, CatchClause, ClassDecl, ClassMember, Decl, EnumVariant, Expr,
    ImportKind, Literal, MatchArm, MatchPattern, McpServerDecl, McpToolDef, OwnershipMode, Pattern,
    ServerConfig, Span, Stmt, SwitchCase, TransportType, UiNode, UiScreenDecl, VarDecl,
};
use hudhudscript_parser::parse;

// ── recording visitor ────────────────────────────────────────────────

/// Classify a statement for the visit log. Exhaustive on purpose.
#[rustfmt::skip]
fn stmt_kind(s: &Stmt) -> &'static str {
    match s {
        Stmt::Decl(_) => "Decl", Stmt::McpServer(_) => "McpServer", Stmt::VarDecl(_) => "VarDecl",
        Stmt::Let { .. } => "Let", Stmt::Const { .. } => "Const", Stmt::Assignment { .. } => "Assignment",
        Stmt::If { .. } => "If", Stmt::While { .. } => "While", Stmt::For { .. } => "For",
        Stmt::ForCStyle { .. } => "ForCStyle", Stmt::ForRange { .. } => "ForRange",
        Stmt::Block { .. } => "Block", Stmt::Return { .. } => "Return", Stmt::Break { .. } => "Break",
        Stmt::Continue { .. } => "Continue", Stmt::Switch { .. } => "Switch", Stmt::Try { .. } => "Try",
        Stmt::Throw { .. } => "Throw", Stmt::Expr(_) => "Expr", Stmt::Import { .. } => "Import",
        Stmt::Export { .. } => "Export", Stmt::Function { .. } => "Function",
        Stmt::Trait { .. } => "Trait", Stmt::Destructure { .. } => "Destructure",
        Stmt::Class(_) => "Class", Stmt::Match { .. } => "Match", Stmt::EnumDecl { .. } => "EnumDecl",
        Stmt::Spawn { .. } => "Spawn", Stmt::Despawn { .. } => "Despawn", Stmt::Send { .. } => "Send",
        Stmt::Receive { .. } => "Receive", Stmt::Require { .. } => "Require",
        Stmt::Perform { .. } => "Perform", Stmt::Remember { .. } => "Remember",
        Stmt::Recall { .. } => "Recall", Stmt::Forget { .. } => "Forget",
    }
}

/// Classify an expression, embedding its payload so sequences stay specific.
fn expr_kind(e: &Expr) -> String {
    match e {
        Expr::Literal(Literal::String(s), _) => format!("lit:\"{s}\""),
        Expr::Literal(Literal::Int(i), _) => format!("lit:{i}"),
        Expr::Literal(Literal::Number(f, _), _) => format!("lit:{f}"),
        Expr::Identifier(name, _) => format!("id:{name}"),
        Expr::Binary { .. } => "bin".to_string(),
        _ => "expr".to_string(),
    }
}

fn decl_kind(d: &Decl) -> &'static str {
    match d {
        Decl::UiApp { .. } => "UiApp",
        _ => "decl",
    }
}

/// Records every visit/leave event in traversal order, with optional
/// Stop / SkipChildren triggers for control-flow tests.
struct Rec {
    stop_stmt: Option<&'static str>,
    skip_stmt: Option<&'static str>,
    stop_expr: Option<&'static str>,
    log: Vec<String>,
}

impl AstVisitor for Rec {
    fn visit_stmt(&mut self, s: &Stmt) -> VisitControl {
        let k = stmt_kind(s);
        if self.stop_stmt == Some(k) {
            self.log.push(format!("STOP>stmt:{k}"));
            return VisitControl::Stop;
        }
        if self.skip_stmt == Some(k) {
            self.log.push(format!("SKIP>stmt:{k}"));
            return VisitControl::SkipChildren;
        }
        self.log.push(format!(">stmt:{k}"));
        VisitControl::Continue
    }
    fn leave_stmt(&mut self, s: &Stmt) {
        self.log.push(format!("<stmt:{}", stmt_kind(s)));
    }
    fn visit_expr(&mut self, e: &Expr) -> VisitControl {
        let k = expr_kind(e);
        if self.stop_expr == Some(k.as_str()) {
            self.log.push(format!("STOP>expr:{k}"));
            return VisitControl::Stop;
        }
        self.log.push(format!(">expr:{k}"));
        VisitControl::Continue
    }
    fn leave_expr(&mut self, e: &Expr) {
        self.log.push(format!("<expr:{}", expr_kind(e)));
    }
    fn visit_decl(&mut self, d: &Decl) -> VisitControl {
        self.log.push(format!(">decl:{}", decl_kind(d)));
        VisitControl::Continue
    }
    fn leave_decl(&mut self, d: &Decl) {
        self.log.push(format!("<decl:{}", decl_kind(d)));
    }
}

#[rustfmt::skip]
impl Rec {
    fn new() -> Self { Rec { stop_stmt: None, skip_stmt: None, stop_expr: None, log: Vec::new() } }
}

/// Walk `stmts` with a fresh recorder and return the joined event log.
#[rustfmt::skip]
fn record(stmts: &[Stmt]) -> String {
    let mut v = Rec::new();
    walk_stmts(&mut v, stmts);
    v.log.join(" ")
}

// ── AST construction helpers ─────────────────────────────────────────

#[rustfmt::skip]
fn sp() -> Span { Span::default() }
#[rustfmt::skip]
fn ilit(n: i64) -> Expr { Expr::Literal(Literal::Int(n), sp()) }
#[rustfmt::skip]
fn slit(s: &str) -> Expr { Expr::Literal(Literal::String(s.into()), sp()) }
#[rustfmt::skip]
fn id(name: &str) -> Expr { Expr::Identifier(name.into(), sp()) }
#[rustfmt::skip]
fn brk() -> Stmt { Stmt::Break { span: sp() } }
#[rustfmt::skip]
fn cnt() -> Stmt { Stmt::Continue { span: sp() } }
#[rustfmt::skip]
fn block(stmts: Vec<Stmt>) -> Stmt { Stmt::Block { statements: stmts, span: sp() } }
#[rustfmt::skip]
fn let_stmt(name: &str, value: Expr) -> Stmt { Stmt::Let { name: name.into(), value, span: sp() } }

// ── parsed source ────────────────────────────────────────────────────

#[test]
fn walk_parsed_if_else_condition_then_else_order() {
    let stmts = parse("if (a) { let x = 1; } else { let y = 2; }").unwrap();
    assert_eq!(stmts.len(), 1);
    assert_eq!(
        record(&stmts),
        ">stmt:If >expr:id:a <expr:id:a \
         >stmt:Block >stmt:Let >expr:lit:1 <expr:lit:1 <stmt:Let <stmt:Block \
         >stmt:Block >stmt:Let >expr:lit:2 <expr:lit:2 <stmt:Let <stmt:Block <stmt:If"
    );
}

// ── simple, leaf and binding arms ────────────────────────────────────

#[test]
#[rustfmt::skip]
fn walk_simple_arms_visit_their_expressions_in_order() {
    let bin = Expr::Binary { left: Box::new(id("a")), op: BinaryOp::Add, right: Box::new(id("b")), span: sp() };
    let func = Stmt::Function { name: "f".into(), params: vec![], body: vec![let_stmt("x", ilit(1))],
        is_async: false, is_generator: false, type_params: vec![], span: sp() };
    let stmts = vec![
        Stmt::Assignment { target: id("x"), value: ilit(5), span: sp() },
        Stmt::Expr(bin),
        Stmt::Return { value: Some(ilit(1)), span: sp() },
        Stmt::Return { value: None, span: sp() },
        func,
    ];
    assert_eq!(record(&stmts),
        ">stmt:Assignment >expr:id:x <expr:id:x >expr:lit:5 <expr:lit:5 <stmt:Assignment \
         >stmt:Expr >expr:bin >expr:id:a <expr:id:a >expr:id:b <expr:id:b <expr:bin <stmt:Expr \
         >stmt:Return >expr:lit:1 <expr:lit:1 <stmt:Return \
         >stmt:Return <stmt:Return \
         >stmt:Function >stmt:Let >expr:lit:1 <expr:lit:1 <stmt:Let <stmt:Function");
}

#[test]
#[rustfmt::skip]
fn walk_leaf_and_binding_arms_visit_initializers_only_when_present() {
    let var = |name: &str, init: Option<Expr>| Stmt::VarDecl(VarDecl { name: name.into(),
        is_const: false, type_annotation: None, initializer: init, ownership: OwnershipMode::Owned, span: sp() });
    let stmts = vec![
        brk(),
        cnt(),
        Stmt::Import { path: "m".into(), imports: ImportKind::Named(vec!["a".into()]), span: sp() },
        Stmt::Trait { name: "T".into(), type_params: vec![], methods: vec![], span: sp() },
        Stmt::EnumDecl { name: "E".into(),
            variants: vec![EnumVariant { name: "A".into(), fields: vec![], span: sp() }], span: sp() },
        var("v", Some(ilit(7))),
        var("w", None),
        let_stmt("x", ilit(1)),
        Stmt::Const { name: "c".into(), value: ilit(2), span: sp() },
        Stmt::Destructure { pattern: Pattern::Identifier("a".into()), value: id("cfg"),
            is_const: false, span: sp() },
        Stmt::Export { item: Box::new(let_stmt("x", ilit(9))), source: None, span: sp() },
    ];
    assert_eq!(record(&stmts),
        ">stmt:Break <stmt:Break >stmt:Continue <stmt:Continue \
         >stmt:Import <stmt:Import >stmt:Trait <stmt:Trait >stmt:EnumDecl <stmt:EnumDecl \
         >stmt:VarDecl >expr:lit:7 <expr:lit:7 <stmt:VarDecl \
         >stmt:VarDecl <stmt:VarDecl \
         >stmt:Let >expr:lit:1 <expr:lit:1 <stmt:Let \
         >stmt:Const >expr:lit:2 <expr:lit:2 <stmt:Const \
         >stmt:Destructure >expr:id:cfg <expr:id:cfg <stmt:Destructure \
         >stmt:Export >stmt:Let >expr:lit:9 <expr:lit:9 <stmt:Let <stmt:Export");
}

// ── loop arms ────────────────────────────────────────────────────────

#[test]
#[rustfmt::skip]
fn walk_loop_arms_visit_controlling_expressions_before_bodies() {
    let stmts = vec![
        Stmt::While { condition: id("ready"), body: Box::new(block(vec![brk()])), span: sp() },
        Stmt::For { variable: "i".into(), iterable: id("items"),
            body: Box::new(block(vec![cnt()])), span: sp() },
        Stmt::ForCStyle { init: Some(Box::new(let_stmt("i", ilit(0)))), condition: Some(id("ready")),
            update: Some(Box::new(Stmt::Assignment { target: id("i"), value: ilit(1), span: sp() })),
            body: Box::new(block(vec![brk()])), span: sp() },
        Stmt::ForRange { start: ilit(0), stop: ilit(10), step: Some(ilit(2)),
            body: Box::new(block(vec![brk()])), span: sp() },
    ];
    assert_eq!(record(&stmts),
        ">stmt:While >expr:id:ready <expr:id:ready \
         >stmt:Block >stmt:Break <stmt:Break <stmt:Block <stmt:While \
         >stmt:For >expr:id:items <expr:id:items \
         >stmt:Block >stmt:Continue <stmt:Continue <stmt:Block <stmt:For \
         >stmt:ForCStyle >stmt:Let >expr:lit:0 <expr:lit:0 <stmt:Let \
         >expr:id:ready <expr:id:ready \
         >stmt:Assignment >expr:id:i <expr:id:i >expr:lit:1 <expr:lit:1 <stmt:Assignment \
         >stmt:Block >stmt:Break <stmt:Break <stmt:Block <stmt:ForCStyle \
         >stmt:ForRange >expr:lit:0 <expr:lit:0 >expr:lit:10 <expr:lit:10 >expr:lit:2 <expr:lit:2 \
         >stmt:Block >stmt:Break <stmt:Break <stmt:Block <stmt:ForRange");
}

// ── switch, try and match arms ───────────────────────────────────────

#[test]
#[rustfmt::skip]
fn walk_switch_visits_value_cases_then_default() {
    let stmt = Stmt::Switch { value: id("x"), default: Some(vec![Stmt::Throw { value: id("e"), span: sp() }]), span: sp(),
        cases: vec![
            SwitchCase { value: ilit(1), body: vec![brk()], span: sp() },
            SwitchCase { value: ilit(2), body: vec![cnt()], span: sp() },
        ] };
    assert_eq!(record(&[stmt]),
        ">stmt:Switch >expr:id:x <expr:id:x \
         >expr:lit:1 <expr:lit:1 >stmt:Break <stmt:Break \
         >expr:lit:2 <expr:lit:2 >stmt:Continue <stmt:Continue \
         >stmt:Throw >expr:id:e <expr:id:e <stmt:Throw <stmt:Switch");
}

#[test]
#[rustfmt::skip]
fn walk_try_visits_try_catch_finally_in_order() {
    let stmt = Stmt::Try { span: sp(),
        try_block: Box::new(block(vec![Stmt::Expr(id("a"))])),
        catch_clause: Some(CatchClause { param: "e".into(), span: sp(),
            body: Box::new(block(vec![Stmt::Throw { value: id("err"), span: sp() }])) }),
        finally_block: Some(Box::new(block(vec![brk()]))) };
    assert_eq!(record(&[stmt]),
        ">stmt:Try \
         >stmt:Block >stmt:Expr >expr:id:a <expr:id:a <stmt:Expr <stmt:Block \
         >stmt:Block >stmt:Throw >expr:id:err <expr:id:err <stmt:Throw <stmt:Block \
         >stmt:Block >stmt:Break <stmt:Break <stmt:Block <stmt:Try");
}

#[test]
#[rustfmt::skip]
fn walk_match_visits_value_then_arm_guards_and_bodies() {
    let stmt = Stmt::Match { value: id("x"), span: sp(), arms: vec![
        MatchArm { pattern: MatchPattern::Literal(Literal::Int(1)), guard: Some(id("positive")),
            body: vec![brk()], span: sp() },
        MatchArm { pattern: MatchPattern::Wildcard, guard: None, body: vec![cnt()], span: sp() },
    ] };
    assert_eq!(record(&[stmt]),
        ">stmt:Match >expr:id:x <expr:id:x \
         >expr:id:positive <expr:id:positive >stmt:Break <stmt:Break \
         >stmt:Continue <stmt:Continue <stmt:Match");
}

// ── class and UI declaration arms ────────────────────────────────────

#[test]
#[rustfmt::skip]
fn walk_class_members_in_declaration_order() {
    let class = ClassDecl { name: "C".into(), parent: None, is_abstract: false, type_params: vec![],
        implements: vec![], span: sp(),
        members: vec![
            ClassMember::Field { access: AccessModifier::Private, is_static: false, name: "n".into(),
                initializer: Some(ilit(1)), span: sp() },
            ClassMember::Method { access: AccessModifier::Public, is_static: false, name: "m".into(),
                params: vec![], body: vec![Stmt::Return { value: Some(ilit(2)), span: sp() }], span: sp() },
            ClassMember::Constructor { params: vec![], body: vec![brk()], span: sp() },
        ] };
    assert_eq!(record(&[Stmt::Class(class)]),
        ">stmt:Class >expr:lit:1 <expr:lit:1 \
         >stmt:Return >expr:lit:2 <expr:lit:2 <stmt:Return \
         >stmt:Break <stmt:Break <stmt:Class");
}

#[test]
#[rustfmt::skip]
fn walk_ui_decl_statement_reaches_screen_body_expressions() {
    let decl = Decl::UiApp { name: "A".into(), entry_screen: Some("S".into()), components: vec![],
        span: sp(), screens: vec![UiScreenDecl { name: "S".into(), params: vec![], span: sp(),
            body: vec![UiNode::Var { name: "v".into(), value: ilit(3), span: sp() }] }] };
    assert_eq!(record(&[Stmt::Decl(decl)]),
        ">stmt:Decl >decl:UiApp >expr:lit:3 <expr:lit:3 <decl:UiApp <stmt:Decl");
}

// ── SOP, RAG and MCP server arms ─────────────────────────────────────

#[test]
#[rustfmt::skip]
fn walk_sop_and_rag_arms_visit_their_expression_children() {
    let stmts = vec![
        Stmt::Spawn { subject_name: "Player".into(), args: vec![ilit(1), ilit(2)], span: sp() },
        Stmt::Despawn { name: "hero".into(), span: sp() },
        Stmt::Send { message: Box::new(id("m")), target: Box::new(id("t")), span: sp() },
        Stmt::Receive { variable: "msg".into(), source: Box::new(id("src")), span: sp() },
        Stmt::Require { condition: Box::new(id("ok")), span: sp() },
        Stmt::Perform { action: Box::new(id("act")), span: sp() },
        Stmt::Remember { content: Box::new(slit("note")), store_name: Some("mem".into()), span: sp() },
        Stmt::Recall { query: Box::new(slit("q")), store_name: None, span: sp() },
        Stmt::Forget { target: Box::new(slit("id1")), store_name: Some("mem".into()), span: sp() },
    ];
    assert_eq!(record(&stmts),
        ">stmt:Spawn >expr:lit:1 <expr:lit:1 >expr:lit:2 <expr:lit:2 <stmt:Spawn \
         >stmt:Despawn <stmt:Despawn \
         >stmt:Send >expr:id:m <expr:id:m >expr:id:t <expr:id:t <stmt:Send \
         >stmt:Receive >expr:id:src <expr:id:src <stmt:Receive \
         >stmt:Require >expr:id:ok <expr:id:ok <stmt:Require \
         >stmt:Perform >expr:id:act <expr:id:act <stmt:Perform \
         >stmt:Remember >expr:lit:\"note\" <expr:lit:\"note\" <stmt:Remember \
         >stmt:Recall >expr:lit:\"q\" <expr:lit:\"q\" <stmt:Recall \
         >stmt:Forget >expr:lit:\"id1\" <expr:lit:\"id1\" <stmt:Forget");
}

#[test]
#[rustfmt::skip]
fn walk_mcp_server_visits_fields_then_tool_bodies() {
    let decl = McpServerDecl { name: "srv".into(), span: sp(),
        config: ServerConfig { transport: TransportType::Stdio, command: None, args: vec![],
            url: None, auth: None },
        fields: vec![("url".into(), slit("http://x")), ("command".into(), slit("node"))],
        tools: vec![McpToolDef { name: "t".into(), params: vec![], body: vec![brk()], span: sp() }] };
    assert_eq!(record(&[Stmt::McpServer(decl)]),
        ">stmt:McpServer >expr:lit:\"http://x\" <expr:lit:\"http://x\" \
         >expr:lit:\"node\" <expr:lit:\"node\" \
         >stmt:Break <stmt:Break <stmt:McpServer");
}

// ── control-flow semantics (Stop / SkipChildren) ─────────────────────

#[test]
fn walk_stmt_stop_and_skip_children_semantics() {
    // Stop on visit: traversal halts, children and leave are skipped.
    let stmt = block(vec![brk()]);
    let mut g = Rec {
        stop_stmt: Some("Block"),
        ..Rec::new()
    };
    assert_eq!(walk_stmt(&mut g, &stmt), VisitControl::Stop);
    assert_eq!(g.log.join(" "), "STOP>stmt:Block");
    // SkipChildren on visit: children and leave are skipped, siblings continue.
    let mut g = Rec {
        skip_stmt: Some("Block"),
        ..Rec::new()
    };
    assert_eq!(walk_stmt(&mut g, &stmt), VisitControl::Continue);
    assert_eq!(g.log.join(" "), "SKIP>stmt:Block");
}

#[test]
#[rustfmt::skip]
fn walk_stop_inside_condition_stops_whole_if() {
    let stmt = Stmt::If { condition: id("a"), span: sp(),
        then_branch: Box::new(block(vec![brk()])),
        else_branch: Some(Box::new(block(vec![cnt()]))) };
    let mut g = Rec { stop_expr: Some("id:a"), ..Rec::new() };
    assert_eq!(walk_stmt(&mut g, &stmt), VisitControl::Stop);
    assert_eq!(g.log.join(" "), ">stmt:If STOP>expr:id:a");
}

#[test]
fn walk_stmts_stops_after_first_halted_statement() {
    let stmts = vec![let_stmt("x", ilit(1)), let_stmt("y", ilit(2))];
    let mut g = Rec {
        stop_stmt: Some("Let"),
        ..Rec::new()
    };
    walk_stmts(&mut g, &stmts);
    assert_eq!(g.log.join(" "), "STOP>stmt:Let");
}
