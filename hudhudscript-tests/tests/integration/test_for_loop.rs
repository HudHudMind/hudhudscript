use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_lexer::normalize_keywords;
use hudhudscript_parser::parse;

#[test]
fn test_c_style_for_loop() {
    // C-style for loop using while (grammar doesn't support `for(let i=0; ...)` yet)
    let code = r#"
let toplam = 0;
let i = 0;
while (i < 3) {
    toplam = toplam + i;
    i = i + 1;
}
toplam;
"#;

    let ast = parse(code).expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.execute(&ast).expect("Execution failed");

    println!("Result: {:?}", result);

    // Should be 0 + 1 + 2 = 3
    assert_eq!(result.to_string(), "3");
}
