// Tests for new language features
// break, continue, switch-case, try-catch-finally, throw

use hudhudscript_ast::Stmt;
use hudhudscript_parser::parse;

#[test]
fn test_break_statement() {
    let source = r#"
        var i = 0;
        while (i < 10) {
            if (i == 5) {
                break;
            }
            i = i + 1;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse break statement: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed");
}

#[test]
fn test_continue_statement() {
    let source = r#"
        var i = 0;
        while (i < 10) {
            i = i + 1;
            if (i % 2 == 0) {
                continue;
            }
            print(i);
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse continue statement: {:?}",
        result.err()
    );
}

#[test]
fn test_switch_case() {
    let source = r#"
        var day = 3;
        switch (day) {
            case 1:
                print("Monday");
            case 2:
                print("Tuesday");
            case 3:
                print("Wednesday");
            default:
                print("Other");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse switch statement: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert!(stmts.len() >= 2, "Expected at least 2 statements");

    // Check if second statement is a switch
    if let Stmt::Switch { cases, default, .. } = &stmts[1] {
        assert_eq!(cases.len(), 3, "Expected 3 cases");
        assert!(default.is_some(), "Expected default clause");
    } else {
        panic!("Expected Switch statement, got {:?}", stmts[1]);
    }
}

#[test]
fn test_try_catch() {
    let source = r#"
        try {
            var result = riskyOperation();
            print(result);
        } catch (error) {
            print("Error: " + error);
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse try-catch: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed");

    if let Stmt::Try { catch_clause, .. } = &stmts[0] {
        assert!(catch_clause.is_some(), "Expected catch clause");
    } else {
        panic!("Expected Try statement");
    }
}

#[test]
fn test_try_catch_finally() {
    let source = r#"
        try {
            print("Trying...");
        } catch (error) {
            print("Error: " + error);
        } finally {
            print("Cleanup");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse try-catch-finally: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    if let Stmt::Try {
        catch_clause,
        finally_block,
        ..
    } = &stmts[0]
    {
        assert!(catch_clause.is_some(), "Expected catch clause");
        assert!(finally_block.is_some(), "Expected finally block");
    } else {
        panic!("Expected Try statement");
    }
}

#[test]
fn test_throw_statement() {
    let source = r#"
        throw "Something went wrong";
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse throw statement: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    assert!(!stmts.is_empty(), "No statements parsed");

    if let Stmt::Throw { .. } = &stmts[0] {
        // Success
    } else {
        panic!("Expected Throw statement");
    }
}

#[test]
fn test_turkish_switch() {
    let source = r#"
        var gün = 3;
        seç (gün) {
            durum 1:
                yazdır("Pazartesi");
            durum 2:
                yazdır("Salı");
            varsayılan:
                yazdır("Diğer");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish switch: {:?}",
        result.err()
    );
}

#[test]
fn test_turkish_try_catch() {
    let source = r#"
        dene {
            yazdır("Deneniyor...");
        } yakala (hata) {
            yazdır("Hata: " + hata);
        } sonunda {
            yazdır("Temizlik");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish try-catch: {:?}",
        result.err()
    );
}

#[test]
fn test_turkish_break_continue() {
    let source = r#"
        var i = 0;
        iken (i < 10) {
            (i == 5) ise {
                kır;
            }
            (i % 2 == 0) ise {
                devam;
            }
            i = i + 1;
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse Turkish break/continue: {:?}",
        result.err()
    );
}

#[test]
fn test_nested_switch() {
    let source = r#"
        var x = 1;
        var y = 2;
        switch (x) {
            case 1:
                switch (y) {
                    case 2:
                        print("x=1, y=2");
                    default:
                        print("x=1, y=other");
                }
            default:
                print("x=other");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse nested switch: {:?}",
        result.err()
    );
}

#[test]
fn test_switch_without_default() {
    let source = r#"
        var value = 5;
        switch (value) {
            case 1:
                print("One");
            case 2:
                print("Two");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse switch without default: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    if let Stmt::Switch { default, .. } = &stmts[1] {
        assert!(default.is_none(), "Expected no default clause");
    }
}

#[test]
fn test_try_without_catch() {
    let source = r#"
        try {
            print("Trying...");
        } finally {
            print("Cleanup");
        }
    "#;

    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse try without catch: {:?}",
        result.err()
    );

    let stmts = result.unwrap();
    if let Stmt::Try {
        catch_clause,
        finally_block,
        ..
    } = &stmts[0]
    {
        assert!(catch_clause.is_none(), "Expected no catch clause");
        assert!(finally_block.is_some(), "Expected finally block");
    }
}
