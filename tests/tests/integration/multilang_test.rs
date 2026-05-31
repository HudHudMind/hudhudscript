//! Multi-language parser integration tests

use hudhudscript_ast::{Decl, Stmt};
use hudhudscript_parser::parse;

#[test]
fn test_english_import() {
    let source = "use postgres as db;";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse English import: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Decl(Decl::Import { module, alias, .. }) => {
            assert_eq!(module, "postgres");
            assert_eq!(alias.as_ref().unwrap(), "db");
        }
        _ => panic!("Expected Import statement, got: {:?}", stmts[0]),
    }
}

#[test]
fn test_turkish_import() {
    let source = "postgres sunucusunu db olarak kullan;";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish import: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Decl(Decl::Import { module, alias, .. }) => {
            assert_eq!(module, "postgres");
            assert_eq!(alias.as_ref().unwrap(), "db");
        }
        _ => panic!("Expected Import statement, got: {:?}", stmts[0]),
    }
}

#[test]
fn test_japanese_import() {
    // Native Japanese: postgres を db として 使う
    let source = "postgres を db として 使う;";
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Japanese import: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Decl(Decl::Import { module, alias, .. }) => {
            assert_eq!(module, "postgres");
            assert_eq!(alias.as_ref().unwrap(), "db");
        }
        _ => panic!("Expected Import statement, got: {:?}", stmts[0]),
    }
}

#[test]
fn test_all_three_languages_produce_same_ast() {
    let english = "use postgres as db;";
    let turkish = "postgres sunucusunu db olarak kullan;";
    let japanese = "postgres を db として 使う;";

    let en_ast = parse(english).unwrap();
    let tr_ast = parse(turkish).unwrap();
    let ja_ast = parse(japanese).unwrap();

    // All should have same structure
    assert_eq!(en_ast.len(), 1);
    assert_eq!(tr_ast.len(), 1);
    assert_eq!(ja_ast.len(), 1);

    // Extract module and alias from each
    let extract = |stmt: &Stmt| -> (String, String) {
        match stmt {
            Stmt::Decl(Decl::Import { module, alias, .. }) => {
                (module.clone(), alias.as_ref().unwrap().clone())
            }
            _ => panic!("Expected Import statement"),
        }
    };

    let (en_mod, en_alias) = extract(&en_ast[0]);
    let (tr_mod, tr_alias) = extract(&tr_ast[0]);
    let (ja_mod, ja_alias) = extract(&ja_ast[0]);

    // All should be identical
    assert_eq!(en_mod, tr_mod);
    assert_eq!(tr_mod, ja_mod);
    assert_eq!(en_alias, tr_alias);
    assert_eq!(tr_alias, ja_alias);

    println!("✅ All three languages produce identical AST!");
    println!("   Module: {}, Alias: {}", en_mod, en_alias);
}

#[test]
fn test_english_agent() {
    let source = r#"
        agent DataAnalyst {
            model: "gpt-4",
            temperature: 0.7
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse English agent: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Decl(Decl::Agent { name, fields, .. }) => {
            assert_eq!(name, "DataAnalyst");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected Agent declaration"),
    }
}

#[test]
fn test_turkish_agent() {
    let source = r#"
        ajan VeriAnalist {
            model: "gpt-4",
            temperature: 0.7
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish agent: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Decl(Decl::Agent { name, fields, .. }) => {
            assert_eq!(name, "VeriAnalist");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected Agent declaration"),
    }
}

#[test]
fn test_japanese_agent() {
    // Native Japanese: エージェント (agent)
    let source = r#"
        エージェント DeetaAnalisuto {
            model: "gpt-4",
            temperature: 0.7
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Japanese agent: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::Decl(Decl::Agent { name, fields, .. }) => {
            assert_eq!(name, "DeetaAnalisuto");
            assert_eq!(fields.len(), 2);
        }
        _ => panic!("Expected Agent declaration"),
    }
}
