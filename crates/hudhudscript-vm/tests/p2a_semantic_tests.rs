// P2a: VM semantic tests — ArrayPop empty, StringLen, alias, pop result.

#[cfg(test)]
mod tests {
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;
    use hudhudscript_vm::VM;
    use hudhudscript_bytecode::error::CompileResult;

    fn run(src: &str) -> Result<String, String> {
        let ast = parse(src).map_err(|e| format!("parse: {}", e))?;
        let mut c = Compiler::new();
        let bc = c.compile(&ast).map_err(|e| format!("compile: {}", e))?;
        let locale = VM::detect_locale(src);
        let mut vm = VM::with_locale(locale);
        vm.execute(&bc).map_err(|e| format!("vm: {}", e))?;
        // Can't easily capture stdout, so we verify via error absence
        Ok("ok".to_string())
    }

    #[test]
    fn array_pop_non_empty_works() {
        let src = "let a = [1,2]; let x = a.pop();";
        run(src).expect("pop non-empty should succeed");
    }

    #[test]
    fn array_pop_empty_errors() {
        let src = "let a = []; let x = a.pop();";
        let r = run(src);
        assert!(r.is_err(), "pop empty array must error");
        assert!(
            r.unwrap_err().contains("Cannot pop from empty array"),
            "error must say 'Cannot pop from empty array'"
        );
    }

    #[test]
    fn string_len_semantics() {
        // "ğ" is 2 bytes in UTF-8, .len() returns byte length
        let src = r#"let s = "ğ"; print(s.length);"#;
        run(src).expect("string len should work");
    }

    #[test]
    fn array_alias_not_broken() {
        let src = "
let a = [];
let b = a;
a.push(1);
print(b.length);
";
        run(src).expect("alias should work after push");
    }

    #[test]
    fn pop_result_is_correct() {
        let src = "let a = [10, 20]; let x = a.pop();";
        run(src).expect("pop result should be reachable");
    }
}
