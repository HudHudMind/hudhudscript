// P3b: Integration test — verify registry lookup works after compilation.

#[cfg(test)]
mod tests {
    use hudhudscript_compiler::Compiler;
    use hudhudscript_parser::parse;

    #[test]
    fn function_registry_has_compiled_function() {
        let ast = parse("fn add1(x) { return x + 1; } print(add1(41));").expect("parse");
        let mut compiler = Compiler::new();
        // After compilation, the registry should contain "add1"
        let bc = compiler.compile(&ast).expect("compile");

        // Check that add1 is in the compiled bytecode
        let funcs = bc.functions.borrow();
        let names = bc.function_names.borrow();
        assert!(
            names.contains_key("add1"),
            "function_names should contain 'add1' after compilation. Keys: {:?}",
            names.keys().collect::<Vec<_>>()
        );
        assert!(
            funcs.len() >= 1,
            "should have at least 1 function, got {}",
            funcs.len()
        );
    }
}
