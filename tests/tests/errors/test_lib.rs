use hudhudscript_errors::*;

// ============================================================================
// SourcePosition — construction
// ============================================================================

#[test]
fn source_position_new_fields() {
    let pos = SourcePosition::new(10, 5, 42);
    assert_eq!(pos.line, 10);
    assert_eq!(pos.column, 5);
    assert_eq!(pos.offset, 42);
    assert_eq!(pos.file_path, None);
}

#[test]
fn source_position_with_file() {
    let pos = SourcePosition::new(1, 1, 0).with_file("main.hud");
    assert_eq!(pos.file_path, Some("main.hud".to_string()));
}

#[test]
fn source_position_with_file_preserves_coords() {
    let pos = SourcePosition::new(5, 10, 50).with_file("test.hud");
    assert_eq!(pos.line, 5);
    assert_eq!(pos.column, 10);
    assert_eq!(pos.offset, 50);
}

#[test]
fn source_position_chained_with_file() {
    let pos = SourcePosition::new(1, 1, 0)
        .with_file("first.hud")
        .with_file("second.hud");
    assert_eq!(pos.file_path, Some("second.hud".to_string()));
}

// ============================================================================
// SourcePosition — Display
// ============================================================================

#[test]
fn source_position_display_without_file() {
    let pos = SourcePosition::new(10, 5, 42);
    assert_eq!(pos.to_string(), "10:5");
}

#[test]
fn source_position_display_with_file() {
    let pos = SourcePosition::new(10, 5, 42).with_file("main.hud");
    assert_eq!(pos.to_string(), "main.hud:10:5");
}

#[test]
fn source_position_display_line_one_col_one() {
    let pos = SourcePosition::new(1, 1, 0);
    assert_eq!(pos.to_string(), "1:1");
}

// ============================================================================
// SourcePosition — Equality
// ============================================================================

#[test]
fn source_position_equality() {
    let a = SourcePosition::new(1, 2, 3);
    let b = SourcePosition::new(1, 2, 3);
    let c = SourcePosition::new(2, 2, 3);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn source_position_equality_with_file() {
    let a = SourcePosition::new(1, 1, 0).with_file("a.hud");
    let b = SourcePosition::new(1, 1, 0).with_file("a.hud");
    let c = SourcePosition::new(1, 1, 0).with_file("b.hud");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn source_position_with_file_vs_without() {
    let a = SourcePosition::new(1, 1, 0);
    let b = SourcePosition::new(1, 1, 0).with_file("test.hud");
    assert_ne!(a, b);
}

// ============================================================================
// Severity — Display
// ============================================================================

#[test]
fn severity_display_error() {
    assert_eq!(Severity::Error.to_string(), "error");
}

#[test]
fn severity_display_warning() {
    assert_eq!(Severity::Warning.to_string(), "warning");
}

#[test]
fn severity_display_info() {
    assert_eq!(Severity::Info.to_string(), "info");
}

// ============================================================================
// Severity — Equality
// ============================================================================

#[test]
fn severity_equality() {
    assert_eq!(Severity::Error, Severity::Error);
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_eq!(Severity::Info, Severity::Info);
    assert_ne!(Severity::Error, Severity::Warning);
    assert_ne!(Severity::Warning, Severity::Info);
    assert_ne!(Severity::Error, Severity::Info);
}

// ============================================================================
// Diagnostic — builders
// ============================================================================

#[test]
fn diagnostic_error_builder() {
    let d = Diagnostic::error("unexpected token");
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.message, "unexpected token");
    assert!(d.position.is_none());
    assert!(d.code.is_none());
    assert_eq!(d.hints.len(), 0);
}

#[test]
fn diagnostic_warning_builder() {
    let d = Diagnostic::warning("unused variable");
    assert_eq!(d.severity, Severity::Warning);
    assert_eq!(d.message, "unused variable");
    assert!(d.position.is_none());
}

#[test]
fn diagnostic_info_builder() {
    let d = Diagnostic::info("suggestion");
    assert_eq!(d.severity, Severity::Info);
    assert_eq!(d.message, "suggestion");
    assert!(d.position.is_none());
}

// ============================================================================
// Diagnostic — chaining methods
// ============================================================================

#[test]
fn diagnostic_at_sets_position() {
    let d = Diagnostic::error("test").at(SourcePosition::new(3, 7, 20));
    assert_eq!(d.position.as_ref().unwrap().line, 3);
    assert_eq!(d.position.as_ref().unwrap().column, 7);
}

#[test]
fn diagnostic_with_code_sets_code() {
    let d = Diagnostic::error("test").with_code("E001");
    assert_eq!(d.code, Some("E001".to_string()));
}

#[test]
fn diagnostic_with_hint_adds_hint() {
    let d = Diagnostic::error("test")
        .with_hint("try this")
        .with_hint("or that");
    assert_eq!(d.hints, vec!["try this".to_string(), "or that".to_string()]);
}

#[test]
fn diagnostic_full_chain() {
    let d = Diagnostic::error("bad syntax")
        .at(SourcePosition::new(1, 5, 4).with_file("test.hud"))
        .with_code("E0001")
        .with_hint("did you mean `;`?")
        .with_hint("check the docs");
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.message, "bad syntax");
    assert_eq!(d.code, Some("E0001".to_string()));
    assert_eq!(d.hints.len(), 2);
    assert_eq!(
        d.position.as_ref().unwrap().file_path,
        Some("test.hud".to_string())
    );
}

// ============================================================================
// Diagnostic — Display
// ============================================================================

#[test]
fn diagnostic_display_bare_error() {
    let d = Diagnostic::error("bare error");
    assert_eq!(d.to_string(), "error: bare error");
}

#[test]
fn diagnostic_display_bare_warning() {
    let d = Diagnostic::warning("unused var");
    assert_eq!(d.to_string(), "warning: unused var");
}

#[test]
fn diagnostic_display_bare_info() {
    let d = Diagnostic::info("tip");
    assert_eq!(d.to_string(), "info: tip");
}

#[test]
fn diagnostic_display_with_code() {
    let d = Diagnostic::error("test").with_code("E0001");
    assert_eq!(d.to_string(), "error[E0001]: test");
}

#[test]
fn diagnostic_display_with_position() {
    let d = Diagnostic::error("test").at(SourcePosition::new(5, 10, 40));
    assert_eq!(d.to_string(), "error at 5:10: test");
}

#[test]
fn diagnostic_display_with_code_and_position() {
    let d = Diagnostic::error("unexpected token")
        .at(SourcePosition::new(1, 5, 4).with_file("test.hud"))
        .with_code("E0001")
        .with_hint("did you mean `;`?");
    let s = d.to_string();
    assert_eq!(
        s,
        "error[E0001] at test.hud:1:5: unexpected token\n  hint: did you mean `;`?"
    );
}

#[test]
fn diagnostic_display_multiple_hints() {
    let d = Diagnostic::error("test")
        .with_hint("hint1")
        .with_hint("hint2");
    let s = d.to_string();
    assert!(s.contains("hint: hint1"));
    assert!(s.contains("hint: hint2"));
}

// ============================================================================
// Diagnostic — Equality
// ============================================================================

#[test]
fn diagnostic_equality() {
    let a = Diagnostic::error("test").with_code("E001");
    let b = Diagnostic::error("test").with_code("E001");
    assert_eq!(a, b);
}

#[test]
fn diagnostic_inequality_different_message() {
    let a = Diagnostic::error("msg1");
    let b = Diagnostic::error("msg2");
    assert_ne!(a, b);
}

// ============================================================================
// HudHudError — Display for ALL variants
// ============================================================================

#[test]
fn hudhud_error_lex_display() {
    let err = HudHudError::Lex {
        message: "invalid char".to_string(),
        position: Some(SourcePosition::new(1, 3, 2)),
    };
    assert_eq!(err.to_string(), "lex error: invalid char");
}

#[test]
fn hudhud_error_parse_display() {
    let err = HudHudError::Parse {
        message: "unexpected EOF".to_string(),
        position: None,
    };
    assert_eq!(err.to_string(), "parse error: unexpected EOF");
}

#[test]
fn hudhud_error_type_display() {
    let err = HudHudError::Type {
        message: "type mismatch".to_string(),
        position: None,
    };
    assert_eq!(err.to_string(), "type error: type mismatch");
}

#[test]
fn hudhud_error_compile_display() {
    let err = HudHudError::Compile {
        message: "undefined variable".to_string(),
        position: None,
    };
    assert_eq!(err.to_string(), "compile error: undefined variable");
}

#[test]
fn hudhud_error_runtime_display() {
    let err = HudHudError::Runtime {
        message: "division by zero".to_string(),
    };
    assert_eq!(err.to_string(), "runtime error: division by zero");
}

#[test]
fn hudhud_error_diagnostics_display() {
    let d1 = Diagnostic::error("first");
    let d2 = Diagnostic::warning("second");
    let err = HudHudError::Diagnostics(vec![d1, d2]);
    assert_eq!(err.to_string(), "2 diagnostic(s)");
}

#[test]
fn hudhud_error_diagnostics_single() {
    let err = HudHudError::Diagnostics(vec![Diagnostic::error("only one")]);
    assert_eq!(err.to_string(), "1 diagnostic(s)");
}

// ============================================================================
// HudHudError — into_diagnostics for ALL variants
// ============================================================================

#[test]
fn hudhud_error_into_diagnostics_lex() {
    let err = HudHudError::Lex {
        message: "bad char".to_string(),
        position: Some(SourcePosition::new(1, 3, 2)),
    };
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_LEX"));
    assert_eq!(diags[0].position.as_ref().unwrap().line, 1);
    assert_eq!(diags[0].severity, Severity::Error);
}

#[test]
fn hudhud_error_into_diagnostics_lex_no_position() {
    let err = HudHudError::Lex {
        message: "lex issue".to_string(),
        position: None,
    };
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_LEX"));
    assert!(diags[0].position.is_none());
}

#[test]
fn hudhud_error_into_diagnostics_parse() {
    let err = HudHudError::Parse {
        message: "unexpected token".to_string(),
        position: Some(SourcePosition::new(5, 10, 40)),
    };
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_PARSE"));
    assert_eq!(diags[0].message, "unexpected token");
}

#[test]
fn hudhud_error_into_diagnostics_type() {
    let err = HudHudError::Type {
        message: "mismatch".to_string(),
        position: None,
    };
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_TYPE"));
}

#[test]
fn hudhud_error_into_diagnostics_compile() {
    let err = HudHudError::Compile {
        message: "undefined".to_string(),
        position: Some(SourcePosition::new(2, 1, 10)),
    };
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_COMPILE"));
    assert_eq!(diags[0].position.as_ref().unwrap().line, 2);
}

#[test]
fn hudhud_error_into_diagnostics_runtime() {
    let err = HudHudError::Runtime {
        message: "stack overflow".to_string(),
    };
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_RUNTIME"));
    assert_eq!(diags[0].message, "stack overflow");
}

#[test]
fn hudhud_error_into_diagnostics_passthrough() {
    let d1 = Diagnostic::error("first").with_code("E001");
    let d2 = Diagnostic::warning("second");
    let err = HudHudError::Diagnostics(vec![d1, d2]);
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].code.as_deref(), Some("E001"));
    assert_eq!(diags[1].severity, Severity::Warning);
}

#[test]
fn hudhud_error_io_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: HudHudError = io_err.into();
    let diags = err.into_diagnostics();
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code.as_deref(), Some("E_IO"));
    assert!(diags[0].message.contains("file not found"));
}

#[test]
fn hudhud_error_io_permission_denied() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let err: HudHudError = io_err.into();
    let diags = err.into_diagnostics();
    assert_eq!(diags[0].code.as_deref(), Some("E_IO"));
    assert!(diags[0].message.contains("access denied"));
}
