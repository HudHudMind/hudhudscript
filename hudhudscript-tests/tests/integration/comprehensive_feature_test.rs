//! Comprehensive Feature Test Suite
//! Tests ALL claimed features to verify what actually works

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_parser::parse;
#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_english_basic_syntax() {
        let code = r#"
            var x = 10;
            var y = 20;
            var sum = x + y;
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "English basic syntax should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_turkish_basic_syntax() {
        let code = r#"
            değişken x = 10;
            değişken y = 20;
            değişken toplam = x + y;
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Turkish basic syntax should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_japanese_basic_syntax() {
        let code = r#"
            hensuu x = 10;
            hensuu y = 20;
            hensuu goukei = x + y;
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Japanese basic syntax should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_english_agent_declaration() {
        let code = r#"
            agent DataAnalyst {
                model: "gpt-4",
                temperature: 0.7
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "English agent declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_turkish_agent_declaration() {
        let code = r#"
            ajan VeriAnalist {
                model: "gpt-4",
                temperature: 0.7
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Turkish agent declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_japanese_agent_declaration() {
        let code = r#"
            eijento DeetaAnalisuto {
                model: "gpt-4",
                temperature: 0.7
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Japanese agent declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_english_use_statement() {
        let code = "use postgres as db;";
        let result = parse(code);
        assert!(
            result.is_ok(),
            "English use statement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_turkish_use_statement() {
        let code = "postgres sunucusunu db olarak kullan;";
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Turkish use statement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_japanese_use_statement() {
        let code = "postgres wo db toshite tsukau;";
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Japanese use statement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_if_statement_english() {
        let code = r#"
            if (x > 10) {
                var result = "big";
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "English if statement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_if_statement_turkish() {
        let code = r#"
            eğer (x > 10) {
                değişken sonuc = "büyük";
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Turkish if statement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_while_loop_english() {
        let code = r#"
            while (x < 10) {
                var x = x + 1;
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "English while loop should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_for_loop_english() {
        let code = r#"
            for (item in items) {
                var x = item;
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "English for loop should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_function_call() {
        let code = r#"
            var result = someFunction(arg1, arg2);
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Function call should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_array_literal() {
        let code = r#"
            var arr = [1, 2, 3, 4, 5];
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Array literal should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_object_literal() {
        let code = r#"
            var obj = {
                name: "test",
                value: 42
            };
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Object literal should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_template_string() {
        let code = r#"
            var name = "World";
            var greeting = `Hello, ${name}!`;
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Template string should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_arrow_function() {
        let code = r#"
            var add = (x, y) => x + y;
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Arrow function should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_constitution_declaration() {
        let code = r#"
            constitution Standards {
                name: "Security Standards",
                version: "1.0"
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Constitution declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_council_declaration() {
        let code = r#"
            council ReviewCouncil {
                name: "Code Review Council",
                members: []
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Council declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_try_catch() {
        let code = r#"
            try {
                var result = riskyOperation();
            } catch (error) {
                var msg = "Error occurred";
            }
        "#;
        let result = parse(code);
        assert!(result.is_ok(), "Try-catch should parse: {:?}", result.err());
    }

    #[test]
    fn test_switch_statement() {
        let code = r#"
            switch (value) {
                case 1:
                    var x = "one";
                case 2:
                    var x = "two";
                default:
                    var x = "other";
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Switch statement should parse: {:?}",
            result.err()
        );
    }
}

#[cfg(test)]
mod interpreter_tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let code = r#"
            var x = 10;
            var y = 20;
            var sum = x + y;
        "#;
        // This will likely fail - interpreter not fully implemented
        let ast = parse(code).expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(&ast);
        // We expect this to work
        assert!(
            result.is_ok(),
            "Basic arithmetic should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_variable_assignment() {
        let code = r#"
            var x = 42;
        "#;
        let ast = parse(code).expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(&ast);
        assert!(
            result.is_ok(),
            "Variable assignment should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_string_operations() {
        let code = r#"
            var name = "World";
            var greeting = "Hello";
        "#;
        let ast = parse(code).expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_program(&ast);
        assert!(
            result.is_ok(),
            "String operations should work: {:?}",
            result.err()
        );
    }
}

#[cfg(test)]
mod mcp_integration_tests {
    use super::*;

    #[cfg(feature = "integration-tests")]
    #[test]
    fn test_mcp_server_declaration() {
        let code = r#"
            mcp github_server {
                server: "github",
                config: { token: "test" }
            }
        "#;
        let result = parse(code);
        // This will likely fail
        assert!(result.is_ok(), "MCP server declaration should parse");
    }

    #[test]
    fn test_mcp_call_syntax() {
        // Test that mcp.call() syntax parses and executes (will fail at runtime without server)
        let code = r#"
            var result = mcp.call("github", "list_repos", {});
        "#;
        let result = parse(code);
        assert!(result.is_ok(), "MCP call should parse: {:?}", result.err());

        // Also test that it fails gracefully when server not registered
        let ast = result.unwrap();
        let mut interpreter = Interpreter::new();
        let exec_result = interpreter.eval_program(&ast);
        assert!(
            exec_result.is_err(),
            "Should fail when server not registered"
        );
    }
}

#[cfg(test)]
mod governance_tests {
    use super::*;

    #[test]
    fn test_law_declaration() {
        let code = r#"
            law SecurityLaw {
                name: "Security Requirements",
                enforcement: "mandatory"
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Law declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_swarm_declaration() {
        let code = r#"
            swarm WorkerSwarm {
                name: "Worker Swarm",
                strategy: "parallel"
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Swarm declaration should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_community_declaration() {
        let code = r#"
            community DevCommunity: {
                name: "Developer Community",
                culture: {}
            }
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Community declaration should parse: {:?}",
            result.err()
        );
    }
}

#[cfg(test)]
mod async_tests {
    use super::*;

    #[test]
    #[ignore] // Depends on v0.4.8: async function parser support
    fn test_async_function() {
        let code = r#"
            async function fetchData() {
                data result = await fetch("url");
                return result;
            }
        "#;
        let result = parse(code);
        assert!(result.is_ok(), "Async function should parse");
    }
}

#[cfg(test)]
mod module_tests {
    use super::*;

    #[test]
    fn test_import_statement() {
        let code = r#"
            import { helper } from "./utils";
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Import statement should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_export_statement() {
        let code = r#"
            export var value = 42;
        "#;
        let result = parse(code);
        assert!(
            result.is_ok(),
            "Export statement should parse: {:?}",
            result.err()
        );
    }
}
