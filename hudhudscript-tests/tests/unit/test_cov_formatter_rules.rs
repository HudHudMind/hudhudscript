//! Coverage tests for governance-rule formatting in `hudhudscript-formatter`
//! (`formatter_impl/rules.rs`): rule, swarm, community, protocol,
//! governance and strategy declarations.
//!
//! Every test asserts the EXACT rendered string. Tests that build the AST
//! directly pin the formatter contract independently of the parser;
//! source-based tests additionally assert idempotency
//! (`format(format(x)) == format(x)`) where the rendered output is valid
//! HudHudScript syntax.

use hudhudscript_ast::{
    ActionDecl, ConditionDecl, CultureDecl, Decl, Expr, Literal, Span, Stmt,
};
use hudhudscript_formatter::{Formatter, FormatterConfig};

// ── helpers ───────────────────────────────────────────────────────

fn fmt(stmts: &[Stmt]) -> String {
    Formatter::new().format_program(stmts)
}

fn fmt_with(config: FormatterConfig, stmts: &[Stmt]) -> String {
    Formatter::with_config(config).format_program(stmts)
}

fn fmt_src(src: &str) -> String {
    let ast = hudhudscript_parser::parse(src).expect("test source must parse");
    fmt(&ast)
}

fn assert_idempotent(src: &str) {
    let once = fmt_src(src);
    assert_eq!(fmt_src(&once), once, "second format pass must be stable");
}

fn two_space() -> FormatterConfig {
    FormatterConfig { indent: "  ".to_string(), ..Default::default() }
}

fn s(v: &str) -> Expr {
    Expr::Literal(Literal::String(v.to_string()), Span::default())
}

fn i(v: i64) -> Expr {
    Expr::Literal(Literal::Int(v), Span::default())
}

fn ident(v: &str) -> Expr {
    Expr::Identifier(v.to_string(), Span::default())
}

fn decl(decl: Decl) -> Vec<Stmt> {
    vec![Stmt::Decl(decl)]
}

// ── rule (format_rule) ────────────────────────────────────────────

#[test]
fn rule_renders_priority_conditions_and_actions_blocks() {
    let d = Decl::Rule {
        name: "high_load".into(),
        priority: 5,
        conditions: vec![ConditionDecl {
            condition_type: "greater_than".into(),
            field: "cpu".into(),
            value: Expr::Member {
                object: Box::new(ident("metrics")),
                property: "cpu".into(),
                span: Span::default(),
            },
            span: Span::default(),
        }],
        actions: vec![ActionDecl {
            action_type: "notify".into(),
            params: vec![
                ("message".into(), s("overheated")),
                ("channel".into(), ident("ops")),
            ],
            span: Span::default(),
        }],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        concat!(
            "rule high_load {\n",
            "    priority: 5,\n",
            "    conditions: [\n",
            "        { type: \"greater_than\", field: \"cpu\", value: metrics.cpu },\n",
            "    ],\n",
            "    actions: [\n",
            "        { type: \"notify\", message: \"overheated\", channel: ops },\n",
            "    ],\n",
            "}\n",
        )
    );
}

#[test]
fn rule_without_conditions_or_actions_prints_priority_only() {
    let d = Decl::Rule {
        name: "idle".into(),
        conditions: vec![],
        actions: vec![],
        priority: 0,
        span: Span::default(),
    };
    assert_eq!(fmt(&decl(d)), "rule idle {\n    priority: 0,\n}\n");
}

#[test]
fn rule_condition_type_and_field_are_escaped() {
    let d = Decl::Rule {
        name: "weird".into(),
        priority: 1,
        conditions: vec![ConditionDecl {
            condition_type: "equals\"q".into(),
            field: "tag\nx".into(),
            value: i(7),
            span: Span::default(),
        }],
        actions: vec![],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        "rule weird {\n    priority: 1,\n    conditions: [\n        { type: \"equals\\\"q\", field: \"tag\\nx\", value: 7 },\n    ],\n}\n"
    );
}

#[test]
fn rule_from_source_round_trips() {
    assert_eq!(fmt_src("rule idle { priority: 3 }"), "rule idle {\n    priority: 3,\n}\n");
    assert_idempotent("rule idle { priority: 3 }");
}

#[test]
fn rule_respects_configured_indent_width() {
    let d = Decl::Rule {
        name: "r".into(),
        conditions: vec![],
        actions: vec![],
        priority: 1,
        span: Span::default(),
    };
    assert_eq!(fmt_with(two_space(), &decl(d)), "rule r {\n  priority: 1,\n}\n");
}

// ── swarm (format_swarm) ──────────────────────────────────────────

#[test]
fn swarm_prints_strategy_line_then_agents_list() {
    let d = Decl::Swarm {
        name: "fleet".into(),
        agents: vec!["a1".into(), "a2".into()],
        strategy: "competitive".into(),
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        "swarm fleet {\n    strategy: \"competitive\",\n    agents: [\"a1\", \"a2\"],\n}\n"
    );
}

#[test]
fn swarm_without_agents_omits_agents_line() {
    let d = Decl::Swarm { name: "lone".into(), agents: vec![], strategy: "sequential".into(), span: Span::default() };
    assert_eq!(fmt(&decl(d)), "swarm lone {\n    strategy: \"sequential\",\n}\n");
}

#[test]
fn swarm_source_fields_are_reordered_strategy_first() {
    assert_eq!(
        fmt_src("swarm fleet { agents: [\"a1\", \"a2\"], strategy: \"competitive\" }"),
        "swarm fleet {\n    strategy: \"competitive\",\n    agents: [\"a1\", \"a2\"],\n}\n"
    );
    assert_idempotent("swarm fleet { agents: [\"a1\", \"a2\"], strategy: \"competitive\" }");
}

#[test]
fn swarm_defaults_strategy_to_parallel_when_source_omits_it() {
    assert_eq!(
        fmt_src("swarm s { agents: [\"x\"] }"),
        "swarm s {\n    strategy: \"parallel\",\n    agents: [\"x\"],\n}\n"
    );
}

// ── community (format_community) ──────────────────────────────────

#[test]
fn community_renders_membership_and_nested_culture_block() {
    let d = Decl::Community {
        name: "guild".into(),
        members: vec!["m1".into(), "m2".into()],
        councils: vec!["c1".into()],
        culture: CultureDecl {
            values: vec!["honesty".into()],
            norms: vec!["code review".into()],
            communication_style: "formal".into(),
            span: Span::default(),
        },
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        concat!(
            "community guild {\n",
            "    members: [\"m1\", \"m2\"],\n",
            "    councils: [\"c1\"],\n",
            "    culture: {\n",
            "        values: [\"honesty\"],\n",
            "        norms: [\"code review\"],\n",
            "        communication_style: \"formal\",\n",
            "    },\n",
            "}\n",
        )
    );
}

#[test]
fn community_minimal_prints_only_communication_style() {
    let d = Decl::Community {
        name: "solo".into(),
        members: vec![],
        councils: vec![],
        culture: CultureDecl {
            values: vec![],
            norms: vec![],
            communication_style: "informal".into(),
            span: Span::default(),
        },
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        "community solo {\n    culture: {\n        communication_style: \"informal\",\n    },\n}\n"
    );
}

// ── protocol (format_protocol) ────────────────────────────────────

#[test]
fn protocol_renders_optional_fields_and_flat_session_hooks() {
    let d = Decl::Protocol {
        name: "P".into(),
        execution: Some("parallel".into()),
        governance: Some("MyGov".into()),
        timeout: Some(30.0),
        session: vec![("onStart".into(), ident("boot")), ("onError".into(), s("log"))],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        concat!(
            "protocol P {\n",
            "    execution: \"parallel\",\n",
            "    governance: MyGov,\n",
            "    timeout: 30,\n",
            "    onStart: boot,\n",
            "    onError: \"log\",\n",
            "}\n",
        )
    );
}

#[test]
fn protocol_without_optional_fields_renders_empty_body() {
    let d = Decl::Protocol {
        name: "P".into(),
        execution: None,
        governance: None,
        timeout: None,
        session: vec![],
        span: Span::default(),
    };
    assert_eq!(fmt(&decl(d)), "protocol P {\n}\n");
}

#[test]
fn protocol_timeout_renders_fractional_seconds() {
    let d = Decl::Protocol {
        name: "P".into(),
        execution: None,
        governance: None,
        timeout: Some(1.5),
        session: vec![],
        span: Span::default(),
    };
    assert_eq!(fmt(&decl(d)), "protocol P {\n    timeout: 1.5,\n}\n");
}

#[test]
fn protocol_from_source_round_trips_without_session_hooks() {
    assert_eq!(
        fmt_src("protocol P { execution: parallel, governance: MyGov, timeout: 30 }"),
        "protocol P {\n    execution: \"parallel\",\n    governance: MyGov,\n    timeout: 30,\n}\n"
    );
    assert_idempotent("protocol P { execution: parallel, governance: MyGov, timeout: 30 }");
}

// ── governance (format_governance) ────────────────────────────────

#[test]
fn governance_header_places_spaces_around_colon() {
    let d = Decl::Governance {
        name: "MyGov".into(),
        base_type: "democracy".into(),
        fields: vec![("law_flexibility".into(), s("strict"))],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        "governance MyGov : democracy {\n    law_flexibility: \"strict\",\n}\n"
    );
}

#[test]
fn governance_from_source_defaults_base_type() {
    assert_eq!(
        fmt_src("governance VotingSystem { description: \"plain\" }"),
        "governance VotingSystem : default {\n    description: \"plain\",\n}\n"
    );
    assert_idempotent("governance VotingSystem { description: \"plain\" }");
}

// ── strategy (format_strategy) ────────────────────────────────────
//
// The parser never produces Decl::Strategy (strategy-syntax parses into
// Decl::Protocol), so these arms are reachable only via a constructed AST.

#[test]
fn strategy_renders_fields_in_documented_order() {
    let d = Decl::Strategy {
        name: "S".into(),
        execution: Some("roundRobin".into()),
        governance: Some("G".into()),
        timeout: Some(1.5),
        permissions: vec!["read".into(), "write".into()],
        realm: Some("production".into()),
        session: vec![("onComplete".into(), ident("done"))],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&decl(d)),
        concat!(
            "strategy S {\n",
            "    execution: \"roundRobin\",\n",
            "    governance: G,\n",
            "    timeout: 1.5,\n",
            "    permissions: [\"read\", \"write\"],\n",
            "    realm: \"production\",\n",
            "    onComplete: done,\n",
            "}\n",
        )
    );
}

#[test]
fn strategy_minimal_renders_empty_body() {
    let d = Decl::Strategy {
        name: "S".into(),
        execution: None,
        governance: None,
        timeout: None,
        permissions: vec![],
        realm: None,
        session: vec![],
        span: Span::default(),
    };
    assert_eq!(fmt(&decl(d)), "strategy S {\n}\n");
}

// ── idempotency suite ─────────────────────────────────────────────

#[test]
fn formatting_is_idempotent_across_rule_file_constructs() {
    let sources = [
        "",
        "use postgres.hudhud as db;",
        "agent helper { model: \"gpt-4\" }",
        "enum Shape { Circle, Point }",
        "swarm s { strategy: \"parallel\", agents: [\"a\"] }",
        "governance G { description: \"d\" }",
        "rule r { priority: 2 }",
        "law l { description: \"d\", rules: [\"x\"] }",
        "constitution c { description: \"root\" }",
        "protocol p { execution: sequential, timeout: 5 }",
    ];
    for src in sources {
        let once = fmt_src(src);
        assert_eq!(fmt_src(&once), once, "unstable second pass for {src:?}");
    }
}
