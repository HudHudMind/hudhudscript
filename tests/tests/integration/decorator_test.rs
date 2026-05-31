// Tests for decorator annotations on subjects: @ai, @payment, @cloud, @hudhud, @custom

use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;

#[test]
fn test_simple_decorator_on_subject() {
    let source = r#"
        @ai
        subject AIHelper {
            state ready: true
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse @ai subject: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    assert!(!stmts.is_empty());
    if let Stmt::Decl(Decl::Subject {
        name, decorators, ..
    }) = &stmts[0]
    {
        assert_eq!(name, "AIHelper");
        assert_eq!(decorators.len(), 1);
        assert_eq!(decorators[0].name, "ai");
    } else {
        panic!("Expected Subject declaration, got {:?}", stmts[0]);
    }
}

#[test]
fn test_multiple_decorators_on_subject() {
    let source = r#"
        @ai
        @payment
        @cloud
        subject EcommerceService {
            state balance: 0
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse multi-decorator subject: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    if let Stmt::Decl(Decl::Subject {
        name, decorators, ..
    }) = &stmts[0]
    {
        assert_eq!(name, "EcommerceService");
        assert_eq!(decorators.len(), 3);
        assert_eq!(decorators[0].name, "ai");
        assert_eq!(decorators[1].name, "payment");
        assert_eq!(decorators[2].name, "cloud");
    } else {
        panic!("Expected Subject declaration");
    }
}

#[test]
fn test_decorator_with_params() {
    let source = r#"
        @ai(provider: "openai", model: "gpt-4")
        subject SmartAgent {
            state active: true
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse decorator with params: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    if let Stmt::Decl(Decl::Subject { decorators, .. }) = &stmts[0] {
        assert_eq!(decorators.len(), 1);
        assert_eq!(decorators[0].name, "ai");
        assert_eq!(decorators[0].params.len(), 2);
        assert_eq!(decorators[0].params[0].0, "provider");
        assert_eq!(decorators[0].params[1].0, "model");
    } else {
        panic!("Expected Subject declaration");
    }
}

#[test]
fn test_hudhud_ecosystem_decorator() {
    let source = r#"
        @hudhud
        subject HudHudCore has CoreService {
            state version: "1.0",
            can process_request
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse @hudhud decorator: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    if let Stmt::Decl(Decl::Subject {
        name,
        decorators,
        roles,
        capabilities,
        ..
    }) = &stmts[0]
    {
        assert_eq!(name, "HudHudCore");
        assert_eq!(decorators.len(), 1);
        assert_eq!(decorators[0].name, "hudhud");
        assert_eq!(roles, &["CoreService"]);
        assert_eq!(capabilities, &["process_request"]);
    } else {
        panic!("Expected Subject declaration");
    }
}

#[test]
fn test_custom_decorator_with_mixed_params() {
    let source = r#"
        @custom(type: "webhook", endpoint: "https://api.example.com")
        @cloud(region: "us-east-1")
        subject WebhookHandler has HTTPService {
            state request_count: 0,
            uses http_provider via rest_api,
            can handle_request
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse custom decorator: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    if let Stmt::Decl(Decl::Subject {
        name,
        decorators,
        uses,
        ..
    }) = &stmts[0]
    {
        assert_eq!(name, "WebhookHandler");
        assert_eq!(decorators.len(), 2);
        assert_eq!(decorators[0].name, "custom");
        assert_eq!(decorators[0].params.len(), 2);
        assert_eq!(decorators[1].name, "cloud");
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].0, "http_provider");
        assert_eq!(uses[0].1, Some("rest_api".to_string()));
    } else {
        panic!("Expected Subject declaration");
    }
}

#[test]
fn test_subject_without_decorator_still_works() {
    let source = r#"
        subject SimplePlayer {
            state health: 100,
            can attack
        }
    "#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse subject without decorators: {:?}",
        result.err()
    );
    let stmts = result.unwrap();
    if let Stmt::Decl(Decl::Subject {
        name, decorators, ..
    }) = &stmts[0]
    {
        assert_eq!(name, "SimplePlayer");
        assert!(decorators.is_empty());
    } else {
        panic!("Expected Subject declaration");
    }
}
