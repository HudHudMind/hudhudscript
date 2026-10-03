//! Coverage tests for declaration/type formatting in
//! `hudhudscript-formatter` (`formatter_impl/types.rs`).
//!
//! Every test asserts the EXACT rendered string. Tests that build the AST
//! directly pin the formatter contract independently of the parser; tests
//! that go through `hudhudscript_parser::parse` additionally assert
//! idempotency (`format(format(x)) == format(x)`) for sources whose
//! rendered output is valid HudHudScript syntax.

use hudhudscript_ast::{
    BinaryOp, ComposeMode, ComposeRule, CouncilMemberDecl, Decl, EnumVariant, Expr, LawDecl,
    Literal, Span, Stmt,
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

fn variant(name: &str, fields: &[&str]) -> EnumVariant {
    EnumVariant { name: name.to_string(), fields: fields.iter().map(|f| f.to_string()).collect(), span: Span::default() }
}

fn law(name: &str, desc: &str, level: &str, rules: Vec<Expr>) -> LawDecl {
    LawDecl { name: name.to_string(), description: desc.to_string(), enforcement_level: level.to_string(), rules, span: Span::default() }
}

fn member(agent: &str, role: &str) -> CouncilMemberDecl {
    CouncilMemberDecl { agent_id: agent.to_string(), role: role.to_string(), span: Span::default() }
}

fn import(module: &str, alias: Option<&str>) -> Stmt {
    Stmt::Decl(Decl::Import { module: module.to_string(), alias: alias.map(|a| a.to_string()), span: Span::default() })
}

// ── enum declarations (format_enum_decl) ──────────────────────────

#[test]
fn enum_source_expands_to_one_variant_per_line() {
    assert_eq!(
        fmt_src("enum Shape { Circle, Rectangle(w, h), Point }"),
        "enum Shape {\n    Circle,\n    Rectangle(w, h),\n    Point,\n}\n"
    );
    assert_idempotent("enum Shape { Circle, Rectangle(w, h), Point }");
}

#[test]
fn enum_with_empty_variant_list_renders_two_open_lines() {
    let stmt = Stmt::EnumDecl { name: "Nothing".to_string(), variants: vec![], span: Span::default() };
    assert_eq!(fmt(&[stmt]), "enum Nothing {\n}\n");
}

#[test]
fn enum_honors_configured_indent_width() {
    let stmt = Stmt::EnumDecl {
        name: "Dir".to_string(),
        variants: vec![variant("North", &[]), variant("Pair", &["a", "b"])],
        span: Span::default(),
    };
    assert_eq!(fmt_with(two_space(), &[stmt]), "enum Dir {\n  North,\n  Pair(a, b),\n}\n");
}

// ── use-imports (Decl::Import arm of format_decl) ─────────────────

#[test]
fn import_with_alias_prints_as_form() {
    assert_eq!(fmt_src("use postgres.hudhud as db;"), "use postgres.hudhud as db;\n");
    assert_idempotent("use postgres.hudhud as db;");
}

#[test]
fn import_without_alias_prints_module_only() {
    assert_eq!(fmt(&[import("fs", None)]), "use fs;\n");
}

// ── field-carrying declarations (format_decl_with_fields) ─────────

#[test]
fn agent_source_fields_get_normalized_layout() {
    assert_eq!(
        fmt_src("agent helper { model: \"gpt-4\", retries: 3 }"),
        "agent helper {\n    model: \"gpt-4\",\n    retries: 3,\n}\n"
    );
    assert_idempotent("agent helper { model: \"gpt-4\", retries: 3 }");
}

#[test]
fn declarations_without_fields_render_inline_empty_braces() {
    let agent = Stmt::Decl(Decl::Agent { name: "sentinel".into(), fields: vec![], span: Span::default() });
    let subject = Stmt::Decl(Decl::Subject {
        name: "Player".into(), decorators: vec![], of_subject: None, roles: vec![], states: vec![],
        capabilities: vec![], ability_defs: vec![], intents: vec![], uses: vec![],
        memory: vec![], perception: vec![], fields: vec![], span: Span::default(),
    });
    let store = Stmt::Decl(Decl::Store { name: "minds".into(), fields: vec![], span: Span::default() });
    let ui = Stmt::Decl(Decl::UiApp {
        name: "MyApp".into(), entry_screen: None, screens: vec![], components: vec![], span: Span::default(),
    });
    assert_eq!(fmt(&[agent]), "agent sentinel {}\n");
    assert_eq!(fmt(&[subject]), "subject Player {}\n");
    assert_eq!(fmt(&[store]), "store minds {}\n");
    assert_eq!(fmt(&[ui]), "ui MyApp {}\n");
}

#[test]
fn field_carrying_keywords_dispatch_to_their_decl_name() {
    let kv = |k: &str, v: Expr| (k.to_string(), v);
    let cases: Vec<(Decl, &str)> = vec![
        (Decl::Action { name: "escalate".into(), fields: vec![kv("delay", i(5))], span: Span::default() }, "action escalate {\n    delay: 5,\n}\n"),
        (Decl::Tool { name: "search".into(), fields: vec![kv("url", s("https://example.test"))], span: Span::default() }, "tool search {\n    url: \"https://example.test\",\n}\n"),
        (Decl::Resource { name: "gpu".into(), fields: vec![kv("count", i(2))], span: Span::default() }, "resource gpu {\n    count: 2,\n}\n"),
        (Decl::Provider { name: "openai".into(), fields: vec![kv("model", s("gpt-4"))], span: Span::default() }, "provider openai {\n    model: \"gpt-4\",\n}\n"),
        (Decl::Entity { name: "Player".into(), fields: vec![kv("health", i(100))], span: Span::default() }, "entity Player {\n    health: 100,\n}\n"),
        (Decl::StateMachine { name: "Light".into(), fields: vec![kv("initial", s("red"))], span: Span::default() }, "statemachine Light {\n    initial: \"red\",\n}\n"),
        (Decl::Event { name: "Damage".into(), fields: vec![kv("amount", i(10))], span: Span::default() }, "event Damage {\n    amount: 10,\n}\n"),
        (Decl::Contract { name: "Trade".into(), fields: vec![kv("parties", Expr::Array { elements: vec![s("A"), s("B")], span: Span::default() })], span: Span::default() }, "contract Trade {\n    parties: [\"A\", \"B\"],\n}\n"),
        (Decl::Treaty { name: "Peace".into(), fields: vec![kv("year", i(1996))], span: Span::default() }, "treaty Peace {\n    year: 1996,\n}\n"),
        (Decl::Music { kind: "tempo".into(), name: "allegro".into(), fields: vec![kv("bpm", i(120))], span: Span::default() }, "tempo allegro {\n    bpm: 120,\n}\n"),
        (Decl::Deploy { name: "prod".into(), targets: vec![], providers: vec![], fields: vec![kv("region", s("eu"))], span: Span::default() }, "deploy prod {\n    region: \"eu\",\n}\n"),
    ];
    for (decl, expected) in cases {
        assert_eq!(fmt(&[Stmt::Decl(decl)]), expected);
    }
}

#[test]
fn deeply_nested_field_expression_flattens_without_parens() {
    // ((a + b) * c) - d: format_expr renders infix without parentheses.
    let bin = |op: BinaryOp, l: Expr, r: Expr| Expr::Binary {
        left: Box::new(l),
        op,
        right: Box::new(r),
        span: Span::default(),
    };
    let expr = bin(
        BinaryOp::Sub,
        bin(BinaryOp::Mul, bin(BinaryOp::Add, ident("a"), ident("b")), ident("c")),
        ident("d"),
    );
    let decl = Decl::Tool { name: "calc".into(), fields: vec![("formula".into(), expr)], span: Span::default() };
    assert_eq!(fmt(&[Stmt::Decl(decl)]), "tool calc {\n    formula: a + b * c - d,\n}\n");
}

// ── constitution (format_constitution) ────────────────────────────

#[test]
fn constitution_renders_description_and_nested_laws() {
    let decl = Decl::Constitution {
        name: "Core".into(),
        description: Some("root charter".into()),
        laws: vec![law("L1", "first law", "mandatory", vec![s("no secrets")])],
        span: Span::default(),
    };
    // Nested law blocks close with a bare `}` (no trailing comma) —
    // format_constitution (types.rs) closes each law with `{indent}}\n`.
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "constitution Core {\n    description: \"root charter\",\n    law L1 {\n        description: \"first law\",\n        enforcement_level: \"mandatory\",\n        rules: [\"no secrets\"],\n    }\n}\n"
    );
}

#[test]
fn constitution_omits_description_and_empty_law_rules_lines() {
    let decl = Decl::Constitution {
        name: "Bare".into(),
        description: None,
        laws: vec![law("L0", "plain", "advisory", vec![])],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "constitution Bare {\n    law L0 {\n        description: \"plain\",\n        enforcement_level: \"advisory\",\n    }\n}\n"
    );
}

#[test]
fn constitution_description_only_from_source_round_trips() {
    assert_eq!(
        fmt_src("constitution core { description: \"root\" }"),
        "constitution core {\n    description: \"root\",\n}\n"
    );
    assert_idempotent("constitution core { description: \"root\" }");
}

// ── law (format_law) ──────────────────────────────────────────────

#[test]
fn law_escapes_special_characters_in_description() {
    let decl = Decl::Law {
        name: "tricky".into(),
        description: "he said \"stop\"\nback\\slash\ttab".into(),
        enforcement_level: "advisory".into(),
        rules: vec![],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "law tricky {\n    description: \"he said \\\"stop\\\"\\nback\\\\slash\\ttab\",\n    enforcement_level: \"advisory\",\n}\n"
    );
}

#[test]
fn law_from_source_defaults_enforcement_to_mandatory() {
    assert_eq!(
        fmt_src("law honesty { description: \"be truthful\", rules: [\"no lies\"] }"),
        "law honesty {\n    description: \"be truthful\",\n    enforcement_level: \"mandatory\",\n    rules: [\"no lies\"],\n}\n"
    );
    assert_idempotent("law honesty { description: \"be truthful\", rules: [\"no lies\"] }");
}

// ── council (format_council) ──────────────────────────────────────

#[test]
fn council_renders_constitution_members_and_rules() {
    let decl = Decl::Council {
        name: "SecurityCouncil".into(),
        constitution: "Core".into(),
        members: vec![member("agent1", "lead"), member("agent2", "voter")],
        rules: vec!["R1".into(), "R2".into()],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "council SecurityCouncil {\n    constitution: \"Core\",\n    members: [\n        { agent: \"agent1\", role: \"lead\" },\n        { agent: \"agent2\", role: \"voter\" },\n    ],\n    rules: [\"R1\", \"R2\"],\n}\n"
    );
}

#[test]
fn council_minimal_prints_only_constitution_line() {
    let decl = Decl::Council {
        name: "Audit".into(),
        constitution: "K".into(),
        members: vec![],
        rules: vec![],
        span: Span::default(),
    };
    assert_eq!(fmt(&[Stmt::Decl(decl)]), "council Audit {\n    constitution: \"K\",\n}\n");
}

// ── SOP-style declarations (format_decl SOP arms) ─────────────────

#[test]
fn relation_header_uses_bidirectional_arrow_and_closes_without_newline() {
    let decl = Decl::Relation {
        subject_a: "Player".into(),
        subject_b: "Merchant".into(),
        fields: vec![("trust".into(), i(50))],
        span: Span::default(),
    };
    assert_eq!(fmt(&[Stmt::Decl(decl)]), "relation Player <-> Merchant {\n    trust: 50,\n}");
}

#[test]
fn role_lists_capabilities_before_fields() {
    let decl = Decl::Role {
        name: "Fighter".into(),
        capabilities: vec!["attack".into(), "defend".into()],
        fields: vec![("power".into(), i(10))],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "role Fighter {\n    can attack,\n    can defend,\n    power: 10,\n}"
    );
}

#[test]
fn effect_indents_body_statements_one_level() {
    let decl = Decl::Effect {
        event_name: "Damage".into(),
        params: vec![],
        body: vec![
            Stmt::Let { name: "amount".into(), value: i(5), span: Span::default() },
            Stmt::Return { value: Some(ident("amount")), span: Span::default() },
        ],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "effect on Damage {\n    let amount = 5;\n    return amount;\n}"
    );
}

#[test]
fn compose_renders_every_composition_mode() {
    let decl = Decl::Compose {
        base_subject: "Hero".into(),
        rules: vec![
            ComposeRule { ability_name: "attack".into(), mode: ComposeMode::Combine(vec!["Fast".into(), "Strong".into()]) },
            ComposeRule { ability_name: "retreat".into(), mode: ComposeMode::Override("Base.retreat".into()) },
            ComposeRule { ability_name: "heal".into(), mode: ComposeMode::Before("Base.heal".into()) },
            ComposeRule { ability_name: "buff".into(), mode: ComposeMode::After("Base.buff".into()) },
        ],
        field_rules: vec![],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "compose Hero {\n  on attack: combine [Fast, Strong]\n  on retreat: override Base.retreat\n  on heal: before Base.heal\n  on buff: after Base.buff\n}"
    );
}

#[test]
fn agent_action_and_ability_render_stub_bodies() {
    let action = Decl::AgentAction {
        agent_name: "Worker".into(),
        name: "lift".into(),
        params: vec![],
        body: vec![],
        is_async: false,
        span: Span::default(),
    };
    let ability = Decl::Ability {
        name: "attack".into(),
        subject_type: None,
        params: vec![],
        body: vec![],
        span: Span::default(),
    };
    assert_eq!(fmt(&[Stmt::Decl(action)]), "action lift { ... }");
    assert_eq!(fmt(&[Stmt::Decl(ability)]), "on attack(...) { ... }");
}

// ── program layout (format_program) ───────────────────────────────

#[test]
fn top_level_decls_are_separated_by_one_blank_line() {
    let store = Stmt::Decl(Decl::Store {
        name: "minds".into(),
        fields: vec![("backend".into(), s("hnsw"))],
        span: Span::default(),
    });
    assert_eq!(fmt(&[import("fs", None), store]), "use fs;\n\nstore minds {\n    backend: \"hnsw\",\n}\n");
}

#[test]
fn enum_decl_is_not_followed_by_a_blank_line() {
    // Stmt::EnumDecl is not a Stmt::Decl, so no separator blank line is added.
    let en = Stmt::EnumDecl { name: "E".into(), variants: vec![variant("A", &[])], span: Span::default() };
    assert_eq!(fmt(&[en, import("fs", None)]), "enum E {\n    A,\n}\nuse fs;\n");
}

// ── edge inputs ───────────────────────────────────────────────────

#[test]
fn empty_program_formats_to_empty_string() {
    assert_eq!(fmt_src(""), "");
    assert_eq!(fmt_src("   \n\t\n"), "");
}

#[test]
fn unicode_names_and_strings_survive_verbatim() {
    let decl = Decl::Agent {
        name: "ajanµ".into(),
        fields: vec![("greeting".into(), s("merhaba 中文 セカイ"))],
        span: Span::default(),
    };
    assert_eq!(
        fmt(&[Stmt::Decl(decl)]),
        "agent ajanµ {\n    greeting: \"merhaba 中文 セカイ\",\n}\n"
    );
}
