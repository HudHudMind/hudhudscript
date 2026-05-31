//! Public API tests for hudhudscript-linter —
//! LintConfig, RuleConfig, Severity, LintDiagnostic, lint(), lint_default().

use hudhudscript_ast::*;
use hudhudscript_linter::{lint, lint_default, LintConfig, LintDiagnostic, RuleConfig, Severity};

// ── helpers ───────────────────────────────────────────────────────────────────

fn span() -> Span {
    Span::new(Position::new(1, 1, 0), Position::new(1, 10, 9))
}

fn span_at(line: usize, col: usize, offset: usize) -> Span {
    Span::new(
        Position::new(line, col, offset),
        Position::new(line, col + 5, offset + 5),
    )
}

fn num_lit(n: f64) -> Expr {
    Expr::Literal(Literal::Number(n, false), span())
}

fn str_lit(s: &str) -> Expr {
    Expr::Literal(Literal::String(s.to_string()), span())
}

fn bool_lit(b: bool) -> Expr {
    Expr::Literal(Literal::Boolean(b), span())
}

fn ident(name: &str) -> Expr {
    Expr::Identifier(name.to_string(), span())
}

// ── LintConfig defaults ───────────────────────────────────────────────────────

#[test]
fn lint_config_default_max_nesting_depth_is_4() {
    assert_eq!(LintConfig::default().max_nesting_depth, 4);
}

#[test]
fn lint_config_default_rules_map_is_empty() {
    assert!(LintConfig::default().rules.is_empty());
}

#[test]
fn lint_config_default_enables_naming_convention() {
    assert!(LintConfig::default().is_enabled("naming-convention"));
}

#[test]
fn lint_config_default_enables_unused_variable() {
    assert!(LintConfig::default().is_enabled("unused-variable"));
}

#[test]
fn lint_config_default_enables_empty_block() {
    assert!(LintConfig::default().is_enabled("empty-block"));
}

#[test]
fn lint_config_default_enables_missing_return() {
    assert!(LintConfig::default().is_enabled("missing-return"));
}

#[test]
fn lint_config_default_enables_deep_nesting() {
    assert!(LintConfig::default().is_enabled("deep-nesting"));
}

#[test]
fn lint_config_default_enables_variable_shadowing() {
    assert!(LintConfig::default().is_enabled("variable-shadowing"));
}

#[test]
fn lint_config_unknown_rule_is_enabled_by_default() {
    assert!(LintConfig::default().is_enabled("nonexistent-rule"));
}

// ── LintConfig::disable / enable ─────────────────────────────────────────────

#[test]
fn disable_rule_makes_is_enabled_false() {
    let config = LintConfig::default().disable("unused-variable");
    assert!(!config.is_enabled("unused-variable"));
}

#[test]
fn disable_does_not_affect_other_rules() {
    let config = LintConfig::default().disable("empty-block");
    assert!(config.is_enabled("unused-variable"));
    assert!(config.is_enabled("naming-convention"));
}

#[test]
fn enable_with_no_severity_override() {
    let config = LintConfig::default().enable("naming-convention", None);
    assert!(config.is_enabled("naming-convention"));
}

#[test]
fn enable_with_severity_override_error() {
    let config = LintConfig::default().enable("unused-variable", Some(Severity::Error));
    assert_eq!(
        config.severity("unused-variable", Severity::Warning),
        Severity::Error
    );
}

#[test]
fn enable_with_severity_override_info() {
    let config = LintConfig::default().enable("empty-block", Some(Severity::Info));
    assert_eq!(
        config.severity("empty-block", Severity::Warning),
        Severity::Info
    );
}

// ── LintConfig::severity ──────────────────────────────────────────────────────

#[test]
fn severity_falls_back_to_default_when_no_override() {
    let config = LintConfig::default();
    assert_eq!(
        config.severity("naming-convention", Severity::Warning),
        Severity::Warning
    );
}

#[test]
fn severity_falls_back_for_unknown_rule() {
    let config = LintConfig::default();
    assert_eq!(
        config.severity("nonexistent-rule", Severity::Info),
        Severity::Info
    );
}

#[test]
fn severity_returns_override_when_set() {
    let config = LintConfig::default().enable("naming-convention", Some(Severity::Error));
    assert_eq!(
        config.severity("naming-convention", Severity::Warning),
        Severity::Error
    );
}

// ── RuleConfig ────────────────────────────────────────────────────────────────

#[test]
fn rule_config_enabled_is_enabled_true() {
    let r = RuleConfig::enabled();
    assert!(r.enabled);
}

#[test]
fn rule_config_enabled_severity_is_none() {
    let r = RuleConfig::enabled();
    assert!(r.severity.is_none());
}

#[test]
fn rule_config_disabled_is_enabled_false() {
    let r = RuleConfig::disabled();
    assert!(!r.enabled);
}

#[test]
fn rule_config_disabled_severity_is_none() {
    let r = RuleConfig::disabled();
    assert!(r.severity.is_none());
}

// ── Severity display ──────────────────────────────────────────────────────────

#[test]
fn severity_display_info() {
    assert_eq!(format!("{}", Severity::Info), "info");
}

#[test]
fn severity_display_warning() {
    assert_eq!(format!("{}", Severity::Warning), "warning");
}

#[test]
fn severity_display_error() {
    assert_eq!(format!("{}", Severity::Error), "error");
}

// ── LintDiagnostic ────────────────────────────────────────────────────────────

#[test]
fn lint_diagnostic_new_stores_fields() {
    let d = LintDiagnostic::new("my-rule", "a message", Severity::Warning, span());
    assert_eq!(d.code, "my-rule");
    assert_eq!(d.message, "a message");
    assert_eq!(d.severity, Severity::Warning);
}

#[test]
fn lint_diagnostic_display_contains_rule_code() {
    let d = LintDiagnostic::new("test-rule", "msg", Severity::Warning, span());
    assert!(format!("{}", d).contains("test-rule"));
}

#[test]
fn lint_diagnostic_display_contains_message() {
    let d = LintDiagnostic::new("test-rule", "this is a test", Severity::Warning, span());
    assert!(format!("{}", d).contains("this is a test"));
}

#[test]
fn lint_diagnostic_display_contains_severity() {
    let d = LintDiagnostic::new("rule", "msg", Severity::Error, span());
    assert!(format!("{}", d).contains("error"));
}

#[test]
fn lint_diagnostic_display_contains_line_column() {
    let d = LintDiagnostic::new("rule", "msg", Severity::Info, span());
    let s = format!("{}", d);
    assert!(s.contains("1:1"));
}

// ── naming-convention rule ────────────────────────────────────────────────────

#[test]
fn agent_pascal_case_no_naming_diagnostic() {
    let stmts = vec![Stmt::Decl(Decl::Agent {
        name: "MyAgent".to_string(),
        fields: vec![],
        span: span(),
    })];
    let diags = lint_default(&stmts);
    assert!(!diags.iter().any(|d| d.code == "naming-convention"));
}

#[test]
fn agent_snake_case_triggers_naming_convention() {
    let stmts = vec![Stmt::Decl(Decl::Agent {
        name: "my_agent".to_string(),
        fields: vec![],
        span: span(),
    })];
    let diags = lint_default(&stmts);
    assert!(diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("my_agent")));
}

#[test]
fn variable_camel_case_no_naming_diagnostic() {
    let stmts = vec![Stmt::Let {
        name: "myVar".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(!diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("myVar")));
}

#[test]
fn variable_snake_case_no_naming_diagnostic() {
    let stmts = vec![Stmt::Let {
        name: "my_var".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(!diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("my_var")));
}

#[test]
fn variable_pascal_case_triggers_naming_convention() {
    let stmts = vec![Stmt::Let {
        name: "MyVar".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("MyVar")));
}

#[test]
fn variable_screaming_snake_triggers_naming_convention() {
    let stmts = vec![Stmt::Let {
        name: "MY_VAR".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("MY_VAR")));
}

#[test]
fn variable_underscore_prefix_no_naming_warn() {
    let stmts = vec![Stmt::Let {
        name: "_private".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(!diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("_private")));
}

#[test]
fn class_pascal_case_no_naming_warn() {
    let stmts = vec![Stmt::Class(ClassDecl {
        is_abstract: false,
        name: "MyClass".to_string(),
        parent: None,
        type_params: vec![],
        implements: vec![],
        members: vec![],
        span: span(),
    })];
    let diags = lint_default(&stmts);
    assert!(!diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("MyClass")));
}

#[test]
fn class_snake_case_triggers_naming_convention() {
    let stmts = vec![Stmt::Class(ClassDecl {
        is_abstract: false,
        name: "my_class".to_string(),
        parent: None,
        type_params: vec![],
        implements: vec![],
        members: vec![],
        span: span(),
    })];
    let diags = lint_default(&stmts);
    assert!(diags
        .iter()
        .any(|d| d.code == "naming-convention" && d.message.contains("my_class")));
}

// ── unused-variable rule ──────────────────────────────────────────────────────

#[test]
fn unused_variable_triggers_warn() {
    let stmts = vec![Stmt::Let {
        name: "unused".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(diags.iter().any(|d| d.code == "unused-variable"));
}

#[test]
fn used_variable_no_unused_warn() {
    let s = span();
    let stmts = vec![
        Stmt::Let {
            name: "x".to_string(),
            value: num_lit(1.0),
            span: s,
        },
        Stmt::Expr(ident("x")),
    ];
    let diags = lint_default(&stmts);
    assert!(!diags.iter().any(|d| d.code == "unused-variable"));
}

#[test]
fn underscore_prefix_variable_not_flagged_as_unused() {
    let stmts = vec![Stmt::Let {
        name: "_ignored".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(!diags
        .iter()
        .any(|d| d.code == "unused-variable" && d.message.contains("_ignored")));
}

#[test]
fn disabled_unused_variable_rule_produces_no_diagnostic() {
    let stmts = vec![Stmt::Let {
        name: "unused".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let config = LintConfig::default().disable("unused-variable");
    let diags = lint(&stmts, &config);
    assert!(!diags.iter().any(|d| d.code == "unused-variable"));
}

// ── empty-block rule ──────────────────────────────────────────────────────────

#[test]
fn empty_function_body_triggers_empty_block() {
    let stmts = vec![Stmt::Function {
        name: "foo".to_string(),
        params: vec![],
        body: vec![],
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span: span(),
    }];
    let diags = lint_default(&stmts);
    assert!(diags.iter().any(|d| d.code == "empty-block"));
}

#[test]
fn non_empty_function_no_empty_block_warn() {
    let s = span();
    let stmts = vec![Stmt::Function {
        name: "foo".to_string(),
        params: vec![],
        body: vec![Stmt::Return {
            value: None,
            span: s,
        }],
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span: s,
    }];
    let diags = lint_default(&stmts);
    assert!(!diags.iter().any(|d| d.code == "empty-block"));
}

// ── missing-return rule ───────────────────────────────────────────────────────

#[test]
fn inconsistent_return_triggers_missing_return() {
    let s = span();
    let stmts = vec![Stmt::Function {
        name: "bar".to_string(),
        params: vec![],
        body: vec![
            Stmt::If {
                condition: bool_lit(true),
                then_branch: Box::new(Stmt::Return {
                    value: Some(num_lit(1.0)),
                    span: s,
                }),
                else_branch: None,
                span: s,
            },
            // falls through without return
            Stmt::Expr(num_lit(0.0)),
        ],
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span: s,
    }];
    let diags = lint_default(&stmts);
    assert!(diags.iter().any(|d| d.code == "missing-return"));
}

#[test]
fn consistent_return_no_missing_return_warn() {
    let s = span();
    let stmts = vec![Stmt::Function {
        name: "bar".to_string(),
        params: vec![],
        body: vec![Stmt::Return {
            value: Some(num_lit(1.0)),
            span: s,
        }],
        is_async: false,
        is_generator: false,
        type_params: vec![],
        span: s,
    }];
    let diags = lint_default(&stmts);
    assert!(!diags.iter().any(|d| d.code == "missing-return"));
}

// ── deep-nesting rule ─────────────────────────────────────────────────────────

#[test]
fn deep_nesting_above_threshold_triggers_warn() {
    let s = span();
    let mut inner = Stmt::Block {
        statements: vec![Stmt::Expr(Expr::Literal(Literal::Null, s))],
        span: s,
    };
    for _ in 0..5 {
        inner = Stmt::Block {
            statements: vec![inner],
            span: s,
        };
    }
    let config = LintConfig {
        max_nesting_depth: 4,
        ..Default::default()
    };
    let diags = lint(&[inner], &config);
    assert!(diags.iter().any(|d| d.code == "deep-nesting"));
}

#[test]
fn shallow_nesting_no_deep_nesting_warn() {
    let s = span();
    let stmts = vec![Stmt::Block {
        statements: vec![Stmt::Expr(num_lit(1.0))],
        span: s,
    }];
    let diags = lint_default(&stmts);
    assert!(!diags.iter().any(|d| d.code == "deep-nesting"));
}

// ── variable-shadowing rule ───────────────────────────────────────────────────

#[test]
fn variable_shadowing_triggers_warn() {
    let s1 = span_at(1, 1, 0);
    let s2 = span_at(3, 5, 20);
    let stmts = vec![
        Stmt::Let {
            name: "x".to_string(),
            value: num_lit(1.0),
            span: s1,
        },
        Stmt::Block {
            statements: vec![Stmt::Let {
                name: "x".to_string(),
                value: num_lit(2.0),
                span: s2,
            }],
            span: s2,
        },
    ];
    let diags = lint_default(&stmts);
    assert!(diags.iter().any(|d| d.code == "variable-shadowing"));
}

// ── severity override via config ──────────────────────────────────────────────

#[test]
fn severity_override_to_error_is_applied() {
    let stmts = vec![Stmt::Let {
        name: "unused".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let config = LintConfig::default().enable("unused-variable", Some(Severity::Error));
    let diags = lint(&stmts, &config);
    let d = diags.iter().find(|d| d.code == "unused-variable").unwrap();
    assert_eq!(d.severity, Severity::Error);
}

// ── lint returns sorted diagnostics ──────────────────────────────────────────

#[test]
fn lint_output_is_sorted_by_offset() {
    let s1 = span_at(1, 1, 0);
    let s2 = span_at(2, 1, 20);
    let stmts = vec![
        Stmt::Let {
            name: "b".to_string(),
            value: num_lit(2.0),
            span: s2,
        },
        Stmt::Let {
            name: "a".to_string(),
            value: num_lit(1.0),
            span: s1,
        },
    ];
    // If there are diagnostics, they should be ordered by span offset
    let diags = lint_default(&stmts);
    let offsets: Vec<usize> = diags.iter().map(|d| d.span.start.offset).collect();
    let mut sorted = offsets.clone();
    sorted.sort();
    assert_eq!(offsets, sorted);
}

// ── empty program ─────────────────────────────────────────────────────────────

#[test]
fn empty_program_produces_no_diagnostics() {
    let diags = lint_default(&[]);
    assert!(diags.is_empty());
}

// ── lint_default matches lint with default config ─────────────────────────────

#[test]
fn lint_default_equivalent_to_lint_with_default_config() {
    let stmts = vec![Stmt::Let {
        name: "unused".to_string(),
        value: num_lit(1.0),
        span: span(),
    }];
    let diags_default = lint_default(&stmts);
    let diags_explicit = lint(&stmts, &LintConfig::default());
    assert_eq!(diags_default.len(), diags_explicit.len());
    for (a, b) in diags_default.iter().zip(diags_explicit.iter()) {
        assert_eq!(a.code, b.code);
        assert_eq!(a.severity, b.severity);
    }
}
