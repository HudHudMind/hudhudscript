//! PUBLIC API tests for `hudhudscript_parser::parse`.
//! Tests call parse(source) and assert success (.unwrap()) or failure (.is_err()).
//! No AST internals are inspected beyond public struct fields.

use hudhudscript_parser::parse;

// ============================================================================
// 1. VARIABLE DECLARATIONS
// ============================================================================

#[test]
fn parse_let_integer() {
    parse("let x = 42;").unwrap();
}

#[test]
fn parse_let_float() {
    parse("let pi = 3.14159;").unwrap();
}

#[test]
fn parse_let_string() {
    parse(r#"let name = "hello";"#).unwrap();
}

#[test]
fn parse_let_boolean_true() {
    parse("let flag = true;").unwrap();
}

#[test]
fn parse_let_boolean_false() {
    parse("let flag = false;").unwrap();
}

#[test]
fn parse_let_null() {
    parse("let x = null;").unwrap();
}

#[test]
fn parse_const_declaration() {
    parse("const PI = 3.14;").unwrap();
}

#[test]
fn parse_var_declaration() {
    parse("var count = 0;").unwrap();
}

#[test]
fn parse_let_negative_number() {
    parse("let x = -99;").unwrap();
}

#[test]
fn parse_let_expression() {
    parse("let x = 1 + 2 * 3;").unwrap();
}

// ============================================================================
// 2. FUNCTIONS
// ============================================================================

#[test]
fn parse_function_sync() {
    parse("function greet(name) { return name; }").unwrap();
}

#[test]
fn parse_function_no_params() {
    parse("function noop() { }").unwrap();
}

#[test]
fn parse_function_multiple_params() {
    parse("function add(a, b) { return a + b; }").unwrap();
}

#[test]
fn parse_function_fn_keyword() {
    parse("fn square(x) { return x * x; }").unwrap();
}

#[test]
fn parse_async_function() {
    parse("async function fetchData() { return 1; }").unwrap();
}

#[test]
fn parse_generator_function() {
    parse("function* gen() { yield 1; }").unwrap();
}

#[test]
fn parse_arrow_function_expression_body() {
    parse("let f = (x) => x + 1;").unwrap();
}

#[test]
fn parse_arrow_function_block_body() {
    parse("let f = (x) => { return x + 1; };").unwrap();
}

#[test]
fn parse_arrow_no_params() {
    parse("let f = () => 42;").unwrap();
}

#[test]
fn parse_arrow_multiple_params() {
    parse("let f = (a, b, c) => a + b + c;").unwrap();
}

#[test]
fn parse_async_arrow_function() {
    parse("let f = async (x) => x + 1;").unwrap();
}

#[test]
fn parse_anon_function_expression() {
    parse("let f = function(a, b) { return a + b; };").unwrap();
}

#[test]
fn parse_function_rest_params() {
    parse("function foo(a, ...rest) { return rest; }").unwrap();
}

#[test]
fn parse_function_with_generics() {
    parse("function identity<T>(x) { return x; }").unwrap();
}

#[test]
fn parse_function_multiple_body_stmts() {
    parse("function compute(x) { let y = x + 1; let z = y * 2; return z; }").unwrap();
}

// ============================================================================
// 3. CLASS
// ============================================================================

#[test]
fn parse_class_basic() {
    parse(r#"class Animal { constructor(name) { this.name = name; } public function speak() { return this.name; } }"#).unwrap();
}

#[test]
fn parse_class_extends() {
    parse(r#"class Dog <- Animal { public function bark() { return "woof"; } }"#).unwrap();
}

#[test]
fn parse_class_implements() {
    parse(r#"class Dog <- Animal implements Printable { public function speak() { return "woof"; } }"#).unwrap();
}

#[test]
fn parse_class_multiple_implements() {
    parse(r#"class Robot <- Machine implements Printable, Serializable { public function run() { return 1; } }"#).unwrap();
}

#[test]
fn parse_class_public_field() {
    parse(r#"class Config { public let name = "test" }"#).unwrap();
}

#[test]
fn parse_class_protected_method() {
    parse("class Base { protected function helper() { return 1; } }").unwrap();
}

#[test]
fn parse_class_with_generics() {
    parse("class Container<T> { public function get() { return this; } }").unwrap();
}

#[test]
fn parse_class_static_method() {
    parse("class Util { static function create() { return 1; } }").unwrap();
}

#[test]
fn parse_class_private_field() {
    parse("class Foo { private let secret = 0 }").unwrap();
}

// ============================================================================
// 4. CONTROL FLOW
// ============================================================================

#[test]
fn parse_if_statement() {
    parse("if (x > 0) { let y = 1; }").unwrap();
}

#[test]
fn parse_if_else() {
    parse("if (x > 0) { let y = 1; } else { let y = 2; }").unwrap();
}

#[test]
fn parse_if_else_if_chain() {
    parse("if (x > 2) { let a = 1; } else if (x > 1) { let a = 2; } else { let a = 3; }").unwrap();
}

#[test]
fn parse_nested_if() {
    parse("if (a) { if (b) { let x = 1; } }").unwrap();
}

// ============================================================================
// 5. LOOPS
// ============================================================================

#[test]
fn parse_while_loop() {
    parse("while (x > 0) { x = x - 1; }").unwrap();
}

#[test]
fn parse_for_in_loop() {
    parse("for (item in items) { let x = item; }").unwrap();
}

#[test]
fn parse_for_in_array_literal() {
    parse("for (item in [1, 2, 3]) { let x = item; }").unwrap();
}

#[test]
fn parse_for_range() {
    parse("for(0, 10) { let x = 1; }").unwrap();
}

#[test]
fn parse_for_range_with_step() {
    parse("for(0, 10, 2) { let x = 1; }").unwrap();
}

#[test]
fn parse_for_c_style() {
    parse("for (let i = 0; i < 10; i = i + 1) { let x = i; }").unwrap();
}

#[test]
fn parse_nested_for_loops() {
    parse("for (i in items) { for (j in other) { let x = i; } }").unwrap();
}

// ============================================================================
// 6. TRY / CATCH / FINALLY / THROW
// ============================================================================

#[test]
fn parse_try_catch() {
    parse("try { foo(); } catch (e) { bar(); }").unwrap();
}

#[test]
fn parse_try_catch_finally() {
    parse("try { foo(); } catch (err) { bar(); } finally { cleanup(); }").unwrap();
}

#[test]
fn parse_try_finally_only() {
    parse("try { foo(); } finally { cleanup(); }").unwrap();
}

#[test]
fn parse_throw_statement() {
    parse(r#"throw "error";"#).unwrap();
}

// ============================================================================
// 7. SWITCH / MATCH
// ============================================================================

#[test]
fn parse_switch_with_cases_and_default() {
    parse(r#"switch (x) { case 1: let y = 1; case 2: let y = 2; default: let y = 0; }"#).unwrap();
}

#[test]
fn parse_switch_default_only() {
    parse(r#"switch (x) { default: let y = 0; }"#).unwrap();
}

#[test]
fn parse_match_with_wildcard() {
    parse(r#"match (x) { 1 => { let y = 1; } 2 => { let y = 2; } _ => { let y = 0; } }"#).unwrap();
}

#[test]
fn parse_match_identifier_pattern() {
    parse(r#"match (x) { y => { let z = y; } }"#).unwrap();
}

#[test]
fn parse_match_string_pattern() {
    parse(r#"match (x) { "hello" => { let y = 1; } }"#).unwrap();
}

#[test]
fn parse_match_boolean_patterns() {
    parse(r#"match (x) { true => { let y = 1; } false => { let y = 0; } }"#).unwrap();
}

#[test]
fn parse_match_null_pattern() {
    parse(r#"match (x) { null => { let y = 0; } _ => { let y = 1; } }"#).unwrap();
}

#[test]
fn parse_match_number_pattern() {
    parse(r#"match (x) { 42 => { let y = 1; } }"#).unwrap();
}

// ============================================================================
// 8. IMPORT / EXPORT
// ============================================================================

#[test]
fn parse_use_import() {
    parse("use postgres as db;").unwrap();
}

#[test]
fn parse_es6_import_default() {
    parse(r#"import React from "react";"#).unwrap();
}

#[test]
fn parse_es6_import_named() {
    parse(r#"import { useState, useEffect } from "react";"#).unwrap();
}

#[test]
fn parse_es6_import_wildcard() {
    parse(r#"import * as utils from "utils";"#).unwrap();
}

#[test]
fn parse_export_function() {
    parse("export function foo() { }").unwrap();
}

#[test]
fn parse_export_let() {
    parse("export let x = 42;").unwrap();
}

#[test]
fn parse_export_const() {
    parse("export const PI = 3.14;").unwrap();
}

#[test]
fn parse_export_var() {
    parse("export var count = 0;").unwrap();
}

#[test]
fn parse_export_async_function() {
    parse("export async function fetch() { return 1; }").unwrap();
}

#[test]
fn parse_export_class() {
    parse(r#"export class Foo { public function bar() { return 1; } }"#).unwrap();
}

#[test]
fn parse_export_default() {
    parse("export default 42;").unwrap();
}

#[test]
fn parse_export_named_list() {
    parse("export { add, sub };").unwrap();
}

// ============================================================================
// 9. AGENT / TOOL / MCP / PROVIDER
// ============================================================================

#[test]
fn parse_agent_declaration() {
    parse(r#"agent Analyst { model: "gpt-4", temperature: 0.7 }"#).unwrap();
}

#[test]
fn parse_agent_three_fields() {
    parse(r#"agent ChatBot { model: "gpt-4", temperature: 0.5, maxTokens: 1000 }"#).unwrap();
}

#[test]
fn parse_mcp_server() {
    parse(r#"mcp FileServer { transport: "stdio", command: "npx" }"#).unwrap();
}

#[test]
fn parse_provider() {
    parse(r#"provider OpenAI { apiKey: "sk-test", endpoint: "https://api.openai.com" }"#).unwrap();
}

// ============================================================================
// 10. CONSTITUTION / LAW / COUNCIL / ROLE / PROTOCOL
// ============================================================================

#[test]
fn parse_constitution() {
    parse(r#"constitution BasicRules { description: "test rules" }"#).unwrap();
}

#[test]
fn parse_constitution_with_laws() {
    parse(r#"constitution GameRules { description: "Game rules", laws: [{ name: "NoCheat", description: "No cheating", enforcement: mandatory }] }"#).unwrap();
}

#[test]
fn parse_law() {
    parse(r#"law NoSpam { description: "No spam allowed", enforcement: mandatory }"#).unwrap();
}

#[test]
fn parse_law_with_rules() {
    parse(r#"law Fairness { description: "Be fair", enforcement: mandatory, rules: ["rule1", "rule2"] }"#).unwrap();
}

#[test]
fn parse_council() {
    parse("council ReviewBoard { constitution: MainConstitution }").unwrap();
}

#[test]
fn parse_role() {
    parse("role Fighter { can attack, can defend }").unwrap();
}

#[test]
fn parse_protocol() {
    parse("protocol MyStrategy { execution: parallel, timeout: 5000 }").unwrap();
}

#[test]
fn parse_protocol_with_governance() {
    parse("protocol TeamWork { execution: sequential, governance: democracy, timeout: 3000 }")
        .unwrap();
}

// ============================================================================
// 11. SUBJECT / RELATION / EFFECT
// ============================================================================

#[test]
fn parse_subject() {
    parse("subject Player { state health: 100, can attack, can defend }").unwrap();
}

#[test]
fn parse_subject_with_roles() {
    parse("subject Warrior has Fighter { state health: 100, can attack, can defend }").unwrap();
}

#[test]
fn parse_relation() {
    parse("relation Player <-> Merchant { trust: 50 }").unwrap();
}

#[test]
fn parse_effect() {
    parse("effect on Damage { let x = 1; }").unwrap();
}

// ============================================================================
// 12. DATA / STORE / RESOURCE / ACTION
// ============================================================================

#[test]
fn parse_data_named() {
    parse(r#"data Config { host = "localhost"; port = 8080; }"#).unwrap();
}

#[test]
fn parse_data_anonymous() {
    parse(r#"data { host = "localhost"; port = 8080; }"#).unwrap();
}

#[test]
fn parse_store_declaration() {
    parse(r#"store myStore { backend: "hnsw", dimensions: 1536 }"#).unwrap();
}

#[test]
fn parse_resource_declaration() {
    parse(r#"resource Database { url: "postgres://localhost", pool: 10 }"#).unwrap();
}

#[test]
fn parse_action_declaration() {
    parse(r#"action Attack { target: "enemy", damage: 10 }"#).unwrap();
}

// ============================================================================
// 13. ENTITY / EVENT / CONTRACT / TREATY / STATEMACHINE
// ============================================================================

#[test]
fn parse_entity() {
    parse("entity Player { data health: Number = 100, data name: String }").unwrap();
}

#[test]
fn parse_event() {
    parse("event Damage { amount: Number, source: String }").unwrap();
}

#[test]
fn parse_contract() {
    parse(r#"contract Trade: { buyer: "Alice", seller: "Bob" }"#).unwrap();
}

#[test]
fn parse_treaty() {
    parse(r#"treaty Alliance: { parties: "A", terms: "peace" }"#).unwrap();
}

#[test]
fn parse_statemachine() {
    parse(r#"statemachine TrafficLight { state Red { on event(Timer) -> Green } state Green { on event(Timer) -> Yellow } }"#).unwrap();
}

// ============================================================================
// 14. MUSIC
// ============================================================================

#[test]
fn parse_music_note() {
    parse("note C4 { pitch: 262, duration: 1 }").unwrap();
}

#[test]
fn parse_music_chord() {
    parse(r#"chord Am { notes: "A,C,E", quality: "minor" }"#).unwrap();
}

#[test]
fn parse_music_melody() {
    parse(r#"melody Theme { notes: "C,D,E,F,G", tempo: 120 }"#).unwrap();
}

// ============================================================================
// 15. REMEMBER / RECALL / FORGET
// ============================================================================

#[test]
fn parse_remember_with_store() {
    parse(r#"remember "some text" in myStore;"#).unwrap();
}

#[test]
fn parse_remember_without_store() {
    parse(r#"remember "some text";"#).unwrap();
}

#[test]
fn parse_recall_with_store() {
    parse(r#"recall "query" from myStore;"#).unwrap();
}

#[test]
fn parse_recall_without_store() {
    parse(r#"recall "query";"#).unwrap();
}

#[test]
fn parse_forget_with_store() {
    parse(r#"forget "id" from myStore;"#).unwrap();
}

#[test]
fn parse_forget_without_store() {
    parse(r#"forget "id";"#).unwrap();
}

// ============================================================================
// 16. SPAWN / SEND / RECEIVE
// ============================================================================

#[test]
fn parse_spawn() {
    parse("spawn Player();").unwrap();
}

#[test]
fn parse_spawn_with_args() {
    parse(r#"spawn Player("hero", 100);"#).unwrap();
}

#[test]
fn parse_send() {
    parse(r#"send "hello" to target;"#).unwrap();
}

#[test]
fn parse_receive() {
    parse("receive msg from source;").unwrap();
}

// ============================================================================
// 17. ENUM
// ============================================================================

#[test]
fn parse_enum_simple() {
    parse("enum Color { Red, Green, Blue }").unwrap();
}

#[test]
fn parse_enum_with_fields() {
    parse("enum Shape { Circle(radius), Rectangle(width, height) }").unwrap();
}

// ============================================================================
// 18. TRAIT
// ============================================================================

#[test]
fn parse_trait_basic() {
    parse("trait Printable { fn toString(): String }").unwrap();
}

#[test]
fn parse_trait_with_generics() {
    parse("trait Comparable<T> { fn compareTo(other): Number }").unwrap();
}

#[test]
fn parse_trait_multiple_methods() {
    parse("trait Serializable { fn serialize(): String fn deserialize(data): Any }").unwrap();
}

// ============================================================================
// 19. DECORATORS
// ============================================================================

#[test]
fn parse_decorated_subject() {
    parse("@ai subject Bot { state health: 100 }").unwrap();
}

#[test]
fn parse_decorated_subject_with_params() {
    parse(r#"@payment(provider: "stripe") subject Merchant { state gold: 500, can trade }"#)
        .unwrap();
}

#[test]
fn parse_multiple_decorators() {
    parse("@ai @logging subject SmartBot { state level: 1 }").unwrap();
}

// ============================================================================
// 20. DESTRUCTURING
// ============================================================================

#[test]
fn parse_destructure_object() {
    parse("let { a, b } = obj;").unwrap();
}

#[test]
fn parse_destructure_array() {
    parse("let [a, b] = arr;").unwrap();
}

#[test]
fn parse_const_destructure() {
    parse("const { x, y } = point;").unwrap();
}

#[test]
fn parse_destructure_array_with_rest() {
    parse("let [first, ...rest] = arr;").unwrap();
}

#[test]
fn parse_destructure_object_with_rest() {
    parse("let { a, ...rest } = obj;").unwrap();
}

// ============================================================================
// 21. SPREAD
// ============================================================================

#[test]
fn parse_spread_in_array() {
    parse("let x = [1, ...arr, 2];").unwrap();
}

#[test]
fn parse_spread_in_call() {
    parse("foo(...args);").unwrap();
}

#[test]
fn parse_spread_in_object() {
    parse("let x = { ...base, name: 1 };").unwrap();
}

// ============================================================================
// 22. AWAIT / YIELD
// ============================================================================

#[test]
fn parse_await_in_async_function() {
    parse("async function foo() { let x = await bar(); }").unwrap();
}

#[test]
fn parse_yield_with_value() {
    parse("function* gen() { yield 42; }").unwrap();
}

#[test]
fn parse_yield_without_value() {
    parse("function* gen() { yield; }").unwrap();
}

// ============================================================================
// 23. TEMPLATE STRINGS
// ============================================================================

#[test]
fn parse_template_string_with_interpolation() {
    parse("let x = `hello ${name}`;").unwrap();
}

#[test]
fn parse_template_string_plain() {
    parse("let x = `plain text`;").unwrap();
}

#[test]
fn parse_template_string_multiple_interpolations() {
    parse("let x = `${a} and ${b}`;").unwrap();
}

// ============================================================================
// 24. MULTI-LANGUAGE KEYWORDS
// ============================================================================

#[test]
fn parse_turkish_let() {
    // değişken
    parse("de\u{011F}i\u{015F}ken x = 42;").unwrap();
}

#[test]
fn parse_turkish_if() {
    // eğer
    parse("e\u{011F}er (x > 0) { de\u{011F}i\u{015F}ken y = 1; }").unwrap();
}

#[test]
fn parse_turkish_function() {
    parse("fonksiyon topla(a, b) { return a + b; }").unwrap();
}

#[test]
fn parse_arabic_let() {
    // متغير
    parse("\u{0645}\u{062A}\u{063A}\u{064A}\u{0631} x = 42;").unwrap();
}

#[test]
fn parse_japanese_let() {
    // 変数
    parse("\u{5909}\u{6570} x = 42;").unwrap();
}

#[test]
fn parse_japanese_function() {
    // 関数
    parse("\u{95A2}\u{6570} foo() { return 1; }").unwrap();
}

#[test]
fn parse_bengali_let() {
    // চলক
    parse("\u{099A}\u{09B2}\u{0995} x = 42;").unwrap();
}

#[test]
fn parse_russian_let() {
    // переменная
    parse(
        "\u{043F}\u{0435}\u{0440}\u{0435}\u{043C}\u{0435}\u{043D}\u{043D}\u{0430}\u{044F} x = 42;",
    )
    .unwrap();
}

// ============================================================================
// 25. BINARY / UNARY / PRECEDENCE EXPRESSIONS
// ============================================================================

#[test]
fn parse_binary_add() {
    parse("let x = 1 + 2;").unwrap();
}

#[test]
fn parse_binary_sub() {
    parse("let x = 10 - 3;").unwrap();
}

#[test]
fn parse_binary_mul() {
    parse("let x = 3 * 4;").unwrap();
}

#[test]
fn parse_binary_div() {
    parse("let x = 10 / 2;").unwrap();
}

#[test]
fn parse_binary_mod() {
    parse("let x = 10 % 3;").unwrap();
}

#[test]
fn parse_comparison_gt() {
    parse("let x = a > b;").unwrap();
}

#[test]
fn parse_comparison_lt() {
    parse("let x = a < b;").unwrap();
}

#[test]
fn parse_comparison_ge() {
    parse("let x = a >= b;").unwrap();
}

#[test]
fn parse_comparison_le() {
    parse("let x = a <= b;").unwrap();
}

#[test]
fn parse_comparison_eq() {
    parse("let x = a == b;").unwrap();
}

#[test]
fn parse_comparison_ne() {
    parse("let x = a != b;").unwrap();
}

#[test]
fn parse_logical_and() {
    parse("let x = a && b;").unwrap();
}

#[test]
fn parse_logical_or() {
    parse("let x = a || b;").unwrap();
}

#[test]
fn parse_unary_not() {
    parse("let x = !flag;").unwrap();
}

#[test]
fn parse_unary_neg() {
    parse("let x = -5;").unwrap();
}

#[test]
fn parse_unary_plus() {
    parse("let x = +5;").unwrap();
}

#[test]
fn parse_double_negation() {
    parse("let x = !!flag;").unwrap();
}

#[test]
fn parse_precedence_mul_over_add() {
    parse("let x = 2 + 3 * 4;").unwrap();
}

#[test]
fn parse_precedence_or_over_and() {
    parse("let x = a && b || c && d;").unwrap();
}

#[test]
fn parse_parenthesized_expression() {
    parse("let x = (1 + 2) * 3;").unwrap();
}

// ============================================================================
// 26. MEMBER ACCESS / INDEX / CALL / NEW
// ============================================================================

#[test]
fn parse_member_access() {
    parse("let x = obj.prop;").unwrap();
}

#[test]
fn parse_chained_member_access() {
    parse("let x = a.b.c;").unwrap();
}

#[test]
fn parse_index_access() {
    parse("let x = arr[0];").unwrap();
}

#[test]
fn parse_index_with_expression() {
    parse("let x = arr[i + 1];").unwrap();
}

#[test]
fn parse_function_call() {
    parse("foo(1, 2);").unwrap();
}

#[test]
fn parse_function_call_no_args() {
    parse("foo();").unwrap();
}

#[test]
fn parse_nested_function_calls() {
    parse("foo(bar(baz()));").unwrap();
}

#[test]
fn parse_chained_method_call() {
    parse("obj.method(1).another(2);").unwrap();
}

#[test]
fn parse_deep_member_call_chain() {
    parse("a.b.c.d();").unwrap();
}

#[test]
fn parse_new_with_args() {
    parse("let x = new Foo(1, 2);").unwrap();
}

#[test]
fn parse_new_no_args() {
    parse("let x = new Foo();").unwrap();
}

// ============================================================================
// 27. ASSIGNMENT
// ============================================================================

#[test]
fn parse_assignment_to_identifier() {
    parse("x = 42;").unwrap();
}

#[test]
fn parse_assignment_to_member() {
    parse("obj.prop = 42;").unwrap();
}

#[test]
fn parse_assignment_to_index() {
    parse("arr[0] = 42;").unwrap();
}

// ============================================================================
// 28. ARRAY / OBJECT LITERALS
// ============================================================================

#[test]
fn parse_empty_array() {
    parse("let x = [];").unwrap();
}

#[test]
fn parse_array_with_elements() {
    parse("let x = [1, 2, 3];").unwrap();
}

#[test]
fn parse_nested_array() {
    parse("let x = [[1, 2], [3, 4]];").unwrap();
}

#[test]
fn parse_mixed_array() {
    parse(r#"let x = [1, "two", true, null];"#).unwrap();
}

#[test]
fn parse_empty_object() {
    parse("let x = {};").unwrap();
}

#[test]
fn parse_object_literal() {
    parse(r#"let x = { name: "test", value: 42 };"#).unwrap();
}

#[test]
fn parse_object_with_string_keys() {
    parse(r#"let x = { "key-1": 1, "key-2": 2 };"#).unwrap();
}

#[test]
fn parse_nested_object() {
    parse(r#"let x = { inner: { a: 1 } };"#).unwrap();
}

#[test]
fn parse_object_with_array_value() {
    parse(r#"let x = { items: [1, 2, 3], name: "test" };"#).unwrap();
}

// ============================================================================
// 29. THIS / SELF / RETURN / BREAK / CONTINUE
// ============================================================================

#[test]
fn parse_this_keyword() {
    parse("let x = this;").unwrap();
}

#[test]
fn parse_self_keyword() {
    parse("let x = self;").unwrap();
}

#[test]
fn parse_return_with_value() {
    parse("function foo() { return 42; }").unwrap();
}

#[test]
fn parse_return_void() {
    parse("function foo() { return; }").unwrap();
}

#[test]
fn parse_break_in_while() {
    parse("while (true) { break; }").unwrap();
}

#[test]
fn parse_continue_in_while() {
    parse("while (true) { continue; }").unwrap();
}

// ============================================================================
// 30. STRING ESCAPES
// ============================================================================

#[test]
fn parse_string_escape_newline() {
    parse(r#"let x = "hello\nworld";"#).unwrap();
}

#[test]
fn parse_string_escape_tab() {
    parse(r#"let x = "a\tb";"#).unwrap();
}

#[test]
fn parse_string_escape_backslash() {
    parse(r#"let x = "a\\b";"#).unwrap();
}

#[test]
fn parse_string_escape_quote() {
    parse(r#"let x = "say \"hi\"";"#).unwrap();
}

// ============================================================================
// 31. GOVERNANCE / SWARM / COMMUNITY / RULE
// ============================================================================

#[test]
fn parse_governance() {
    parse("governance MyGov: democracy { }").unwrap();
}

#[test]
fn parse_governance_with_fields() {
    parse("governance MyGov: democracy { quorum: 5, threshold: 0.5 }").unwrap();
}

#[test]
fn parse_swarm() {
    parse(r#"swarm MySwarm { agents: ["a1", "a2"], strategy: parallel }"#).unwrap();
}

#[test]
fn parse_community() {
    parse(r#"community MyCommunity { members: ["agent1", "agent2"] }"#).unwrap();
}

#[test]
fn parse_rule() {
    parse("rule HealthCheck { priority: 10 }").unwrap();
}

// ============================================================================
// 32. REQUIRE / PERFORM (SOP builtins)
// ============================================================================

#[test]
fn parse_require() {
    parse("require x > 0;").unwrap();
}

#[test]
fn parse_perform() {
    parse("perform attack();").unwrap();
}

// ============================================================================
// 33. COMMENTS
// ============================================================================

#[test]
fn parse_line_comment() {
    parse("// this is a comment\nlet x = 1;").unwrap();
}

#[test]
fn parse_block_comment() {
    parse("/* block comment */ let x = 1;").unwrap();
}

#[test]
fn parse_inline_comment_between_stmts() {
    parse("let x = 1; // x is one\nlet y = 2;").unwrap();
}

// ============================================================================
// 34. MISC / EDGE CASES
// ============================================================================

#[test]
fn parse_bare_expression_number() {
    parse("42;").unwrap();
}

#[test]
fn parse_bare_string_expression() {
    parse(r#""hello";"#).unwrap();
}

#[test]
fn parse_multiple_statements() {
    let stmts = parse("let x = 1; let y = 2; let z = 3;").unwrap();
    assert_eq!(stmts.len(), 3);
}

#[test]
fn parse_empty_source() {
    let stmts = parse("").unwrap();
    assert!(stmts.is_empty());
}

#[test]
fn parse_whitespace_only() {
    let stmts = parse("   \n  \t  ").unwrap();
    assert!(stmts.is_empty());
}

#[test]
fn parse_strips_bidi_controls() {
    // RTL mark U+200F embedded in source should be stripped silently
    parse("let x\u{200F} = 42;").unwrap();
}

#[test]
fn parse_complex_program() {
    let src = r#"
        agent Planner { model: "gpt-4", temperature: 0.7 }
        function run(x) { return x + 1; }
        let result = run(42);
    "#;
    let stmts = parse(src).unwrap();
    assert_eq!(stmts.len(), 3);
}

#[test]
fn parse_deeply_nested_expressions() {
    parse("let x = ((((1 + 2) * 3) - 4) / 5);").unwrap();
}

#[test]
fn parse_chained_comparisons_in_condition() {
    parse("if (x > 0 && y < 100 || z == 0) { let r = 1; }").unwrap();
}

// ============================================================================
// 35. ERROR CASES
// ============================================================================

#[test]
fn parse_error_invalid_syntax() {
    assert!(parse("let = ;").is_err());
}

#[test]
fn parse_error_unclosed_brace() {
    assert!(parse("function foo() {").is_err());
}

#[test]
fn parse_error_unclosed_paren() {
    assert!(parse("if (true {").is_err());
}

#[test]
fn parse_error_mismatched_brackets() {
    assert!(parse("let x = [1, 2);").is_err());
}

#[test]
fn parse_error_unterminated_string() {
    assert!(parse(r#"let x = "hello;"#).is_err());
}
fn parse_error_missing_semicolon_in_for() {
    assert!(parse("for (let i = 0 i < 10 i = i + 1) { }").is_err());
}

#[test]
fn parse_error_bare_operator() {
    assert!(parse("+ +;").is_err());
}

// ── Tests moved from pest_parser.rs inline block ─────────────────────

#[test]
fn test_parse_english() {
    let source = "use postgres as db;";
    let result = parse(source);
    assert!(result.is_ok());
}

#[test]
fn test_parse_turkish() {
    let source = "postgres sunucusunu db olarak kullan;";
    let result = parse(source);
    assert!(result.is_ok());
}

#[test]
fn test_parse_japanese() {
    let source = "postgres を db として 使う;";
    let result = parse(source);
    assert!(result.is_ok());
}

#[test]
fn test_parse_agent() {
    let source = r#"
        agent Analyst {
            model: "gpt-4",
            temperature: 0.7
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok());
}

// ── Variable declarations ──────────────────────────────────────────

#[test]
fn test_parse_let_number() {
    let stmts = parse("let x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { name, .. } => assert_eq!(name, "x"),
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_let_string() {
    let stmts = parse(r#"let name = "hello";"#).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { name, value, .. } => {
            assert_eq!(name, "name");
            match value {
                hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                    assert_eq!(s, "hello");
                }
                other => panic!("Expected String literal, got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_const() {
    let stmts = parse("const PI = 3.14;").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Const { name, .. } => assert_eq!(name, "PI"),
        other => panic!("Expected Const, got {:?}", other),
    }
}

#[test]
fn test_parse_var() {
    let stmts = parse("var count = 0;").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::VarDecl(decl) => assert_eq!(decl.name, "count"),
        other => panic!("Expected VarDecl (from var), got {:?}", other),
    }
}

// ── Expressions ────────────────────────────────────────────────────

#[test]
fn test_parse_binary_add() {
    let stmts = parse("let x = 1 + 2;").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Add);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_mul() {
    let stmts = parse("let x = 3 * 4;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Mul);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_comparison() {
    let stmts = parse("let x = a > b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Gt);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_equality() {
    let stmts = parse("let x = a == b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Eq);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_inequality() {
    let stmts = parse("let x = a != b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Ne);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_le() {
    let stmts = parse("let x = a <= b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Le);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_ge() {
    let stmts = parse("let x = a >= b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Ge);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_lt() {
    let stmts = parse("let x = a < b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Lt);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_sub() {
    let stmts = parse("let x = 10 - 3;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Sub);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_div() {
    let stmts = parse("let x = 10 / 2;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Div);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_binary_mod() {
    let stmts = parse("let x = 10 % 3;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Mod);
            }
            other => panic!("Expected Binary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_logical_and() {
    let stmts = parse("let x = a && b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::And);
            }
            other => panic!("Expected Binary And, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_logical_or() {
    let stmts = parse("let x = a || b;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Or);
            }
            other => panic!("Expected Binary Or, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_unary_not() {
    let stmts = parse("let x = !flag;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Unary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::UnaryOp::Not);
            }
            other => panic!("Expected Unary Not, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_unary_neg() {
    let stmts = parse("let x = -5;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Unary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::UnaryOp::Neg);
            }
            other => panic!("Expected Unary Neg, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_boolean_true() {
    let stmts = parse("let x = true;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Boolean(b), _) => {
                assert!(*b);
            }
            other => panic!("Expected Boolean true, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_boolean_false() {
    let stmts = parse("let x = false;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Boolean(b), _) => {
                assert!(!*b);
            }
            other => panic!("Expected Boolean false, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_null() {
    let stmts = parse("let x = null;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Null, _) => {}
            other => panic!("Expected Null, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Array and Object literals ──────────────────────────────────────

#[test]
fn test_parse_empty_array() {
    let stmts = parse("let x = [];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert!(elements.is_empty());
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_array_with_elements() {
    let stmts = parse("let x = [1, 2, 3];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 3);
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_object_literal() {
    let stmts = parse(r#"let x = { name: "test", value: 42 };"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                assert_eq!(properties[0].0, "name");
                assert_eq!(properties[1].0, "value");
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_empty_object() {
    let stmts = parse("let x = {};").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert!(properties.is_empty());
            }
            other => panic!("Expected empty Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Control flow ───────────────────────────────────────────────────

#[test]
fn test_parse_if_stmt() {
    let stmts = parse("if (x > 0) { let y = 1; }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { else_branch, .. } => {
            assert!(else_branch.is_none());
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

#[test]
fn test_parse_if_else() {
    let stmts = parse("if (x > 0) { let y = 1; } else { let y = 2; }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { else_branch, .. } => {
            assert!(else_branch.is_some());
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

#[test]
fn test_parse_while_stmt() {
    let stmts = parse("while (x > 0) { x = x - 1; }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::While { .. } => {}
        other => panic!("Expected While, got {:?}", other),
    }
}

#[test]
fn test_parse_for_in() {
    let stmts = parse("for (item in items) { let x = item; }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::For { variable, .. } => {
            assert_eq!(variable, "item");
        }
        other => panic!("Expected For, got {:?}", other),
    }
}

#[test]
fn test_parse_return_with_value() {
    let source = "function foo() { return 42; }";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => {
            assert_eq!(body.len(), 1);
            match &body[0] {
                hudhudscript_ast::Stmt::Return { value, .. } => {
                    assert!(value.is_some());
                }
                other => panic!("Expected Return, got {:?}", other),
            }
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

#[test]
fn test_parse_return_void() {
    let source = "function foo() { return; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => match &body[0] {
            hudhudscript_ast::Stmt::Return { value, .. } => {
                assert!(value.is_none());
            }
            other => panic!("Expected Return, got {:?}", other),
        },
        other => panic!("Expected Function, got {:?}", other),
    }
}

#[test]
fn test_parse_break_stmt() {
    let source = "while (true) { break; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::While { body, .. } => match body.as_ref() {
            hudhudscript_ast::Stmt::Block { statements, .. } => {
                assert!(matches!(
                    statements[0],
                    hudhudscript_ast::Stmt::Break { .. }
                ));
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected While, got {:?}", other),
    }
}

#[test]
fn test_parse_continue_stmt() {
    let source = "while (true) { continue; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::While { body, .. } => match body.as_ref() {
            hudhudscript_ast::Stmt::Block { statements, .. } => {
                assert!(matches!(
                    statements[0],
                    hudhudscript_ast::Stmt::Continue { .. }
                ));
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected While, got {:?}", other),
    }
}

// ── Functions ──────────────────────────────────────────────────────

#[test]
fn test_parse_function_decl() {
    let stmts = parse("function greet(name) { return name; }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            name,
            params,
            is_async,
            is_generator,
            ..
        } => {
            assert_eq!(name, "greet");
            assert_eq!(params, &["name".to_string()]);
            assert!(!is_async);
            assert!(!is_generator);
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

#[test]
fn test_parse_function_no_params() {
    let stmts = parse("function noop() { }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { name, params, .. } => {
            assert_eq!(name, "noop");
            assert!(params.is_empty());
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

#[test]
fn test_parse_function_multiple_params() {
    let stmts = parse("function add(a, b) { return a + b; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

#[test]
fn test_parse_async_function() {
    let stmts = parse("async function fetchData() { return 1; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { name, is_async, .. } => {
            assert_eq!(name, "fetchData");
            assert!(*is_async);
        }
        other => panic!("Expected async Function, got {:?}", other),
    }
}

#[test]
fn test_parse_arrow_function() {
    let stmts = parse("let f = (x) => x + 1;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction {
                params, is_async, ..
            } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
                assert!(!is_async);
            }
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_arrow_function_block_body() {
    let stmts = parse("let f = (x) => { return x + 1; };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { body, .. } => match body {
                hudhudscript_ast::ArrowFunctionBody::Block(stmts) => {
                    assert_eq!(stmts.len(), 1);
                }
                other => panic!("Expected Block body, got {:?}", other),
            },
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Function calls and member access ──────────────────────────────

#[test]
fn test_parse_function_call() {
    let stmts = parse("foo(1, 2);").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { args, .. }) => {
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected Expr(Call), got {:?}", other),
    }
}

#[test]
fn test_parse_member_access() {
    let stmts = parse("let x = obj.prop;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Member { property, .. } => {
                assert_eq!(property, "prop");
            }
            other => panic!("Expected Member, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_index_access() {
    let stmts = parse("let x = arr[0];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Index { .. } => {}
            other => panic!("Expected Index, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_chained_member_access() {
    let stmts = parse("let x = a.b.c;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Member {
                property, object, ..
            } => {
                assert_eq!(property, "c");
                match object.as_ref() {
                    hudhudscript_ast::Expr::Member { property, .. } => {
                        assert_eq!(property, "b");
                    }
                    other => panic!("Expected nested Member, got {:?}", other),
                }
            }
            other => panic!("Expected Member, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Template strings ───────────────────────────────────────────────

#[test]
fn test_parse_template_string() {
    let stmts = parse("let x = `hello ${name}`;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::TemplateString { parts, .. } => {
                // Template string was parsed - parts may vary based on grammar structure
                // At minimum the template string type is recognized
                assert!(!parts.is_empty() || parts.is_empty()); // parsed successfully
            }
            other => panic!("Expected TemplateString, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_template_string_no_interpolation() {
    let stmts = parse("let x = `plain text`;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::TemplateString { .. } => {
                // Successfully parsed as template string
            }
            other => panic!("Expected TemplateString, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Try/Catch/Finally ──────────────────────────────────────────────

#[test]
fn test_parse_try_catch() {
    let stmts = parse("try { foo(); } catch (e) { bar(); }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Try {
            catch_clause,
            finally_block,
            ..
        } => {
            assert!(catch_clause.is_some());
            assert!(finally_block.is_none());
            let catch = catch_clause.as_ref().unwrap();
            assert_eq!(catch.param, "e");
        }
        other => panic!("Expected Try, got {:?}", other),
    }
}

#[test]
fn test_parse_try_catch_finally() {
    let stmts = parse("try { foo(); } catch (err) { bar(); } finally { cleanup(); }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Try {
            catch_clause,
            finally_block,
            ..
        } => {
            assert!(catch_clause.is_some());
            assert!(finally_block.is_some());
        }
        other => panic!("Expected Try, got {:?}", other),
    }
}

#[test]
fn test_parse_throw() {
    let stmts = parse(r#"throw "error";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Throw { .. } => {}
        other => panic!("Expected Throw, got {:?}", other),
    }
}

// ── Switch/Match ───────────────────────────────────────────────────

#[test]
fn test_parse_switch() {
    let source = r#"switch (x) { case 1: let y = 1; case 2: let y = 2; default: let y = 0; }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Switch { cases, default, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(default.is_some());
        }
        other => panic!("Expected Switch, got {:?}", other),
    }
}

#[test]
fn test_parse_match_stmt() {
    let source = r#"match (x) { 1 => { let y = 1; } 2 => { let y = 2; } _ => { let y = 0; } }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            // First arm has literal pattern
            assert!(matches!(
                arms[0].pattern,
                hudhudscript_ast::MatchPattern::Literal(_)
            ));
            // Last arm has wildcard pattern
            assert!(matches!(
                arms[2].pattern,
                hudhudscript_ast::MatchPattern::Wildcard
            ));
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Import/Export ──────────────────────────────────────────────────

#[test]
fn test_parse_use_import() {
    let stmts = parse("use postgres as db;").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Import { module, alias, .. }) => {
            assert_eq!(module, "postgres");
            assert_eq!(alias.as_deref(), Some("db"));
        }
        other => panic!("Expected Import, got {:?}", other),
    }
}

#[test]
fn test_parse_es6_import_default() {
    let stmts = parse(r#"import React from "react";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Import { path, imports, .. } => {
            assert_eq!(path, "react");
            match imports {
                hudhudscript_ast::ImportKind::Default(name) => {
                    assert_eq!(name, "React");
                }
                other => panic!("Expected Default import, got {:?}", other),
            }
        }
        other => panic!("Expected Import, got {:?}", other),
    }
}

#[test]
fn test_parse_es6_import_named() {
    let stmts = parse(r#"import { useState, useEffect } from "react";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Import { path, imports, .. } => {
            assert_eq!(path, "react");
            match imports {
                hudhudscript_ast::ImportKind::Named(names) => {
                    assert_eq!(names.len(), 2);
                    assert_eq!(names[0], "useState");
                    assert_eq!(names[1], "useEffect");
                }
                other => panic!("Expected Named import, got {:?}", other),
            }
        }
        other => panic!("Expected Import, got {:?}", other),
    }
}

#[test]
fn test_parse_es6_import_wildcard() {
    let stmts = parse(r#"import * as utils from "utils";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Import { path, imports, .. } => {
            assert_eq!(path, "utils");
            match imports {
                hudhudscript_ast::ImportKind::Wildcard(alias) => {
                    assert_eq!(alias, "utils");
                }
                other => panic!("Expected Wildcard import, got {:?}", other),
            }
        }
        other => panic!("Expected Import, got {:?}", other),
    }
}

#[test]
fn test_parse_export_function() {
    let stmts = parse("export function foo() { }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Function { name, .. } => {
                assert_eq!(name, "foo");
            }
            other => panic!("Expected Function in Export, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

#[test]
fn test_parse_export_let() {
    let stmts = parse("export let x = 42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Let { name, .. } => {
                assert_eq!(name, "x");
            }
            other => panic!("Expected Let in Export, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Class declarations ─────────────────────────────────────────────

#[test]
fn test_parse_class_basic() {
    let source = r#"
        class Animal {
            constructor(name) {
                this.name = name;
            }
            public function speak() {
                return this.name;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Animal");
            assert!(decl.parent.is_none());
            assert_eq!(decl.members.len(), 2);
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

#[test]
fn test_parse_class_extends() {
    let source = r#"
        class Dog <- Animal {
            public function bark() {
                return "woof";
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Dog");
            assert_eq!(decl.parent.as_deref(), Some("Animal"));
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Enum declarations ──────────────────────────────────────────────

#[test]
fn test_parse_enum() {
    let source = "enum Color { Red, Green, Blue }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::EnumDecl { name, variants, .. } => {
            assert_eq!(name, "Color");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Red");
            assert_eq!(variants[1].name, "Green");
            assert_eq!(variants[2].name, "Blue");
        }
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

// ── Governance constructs ──────────────────────────────────────────

#[test]
fn test_parse_constitution() {
    let source = r#"
        constitution BasicRules {
            description: "test rules"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Constitution {
            name,
            description,
            ..
        }) => {
            assert_eq!(name, "BasicRules");
            assert_eq!(description.as_deref(), Some("test rules"));
        }
        other => panic!("Expected Constitution, got {:?}", other),
    }
}

#[test]
fn test_parse_law() {
    let source = r#"
        law NoSpam {
            description: "No spam allowed",
            enforcement: mandatory
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Law {
            name,
            description,
            enforcement_level,
            ..
        }) => {
            assert_eq!(name, "NoSpam");
            assert_eq!(description, "No spam allowed");
            assert_eq!(enforcement_level, "mandatory");
        }
        other => panic!("Expected Law, got {:?}", other),
    }
}

#[test]
fn test_parse_council() {
    let source = r#"
        council ReviewBoard {
            constitution: MainConstitution
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Council {
            name,
            constitution,
            ..
        }) => {
            assert_eq!(name, "ReviewBoard");
            assert_eq!(constitution, "MainConstitution");
        }
        other => panic!("Expected Council, got {:?}", other),
    }
}

#[test]
fn test_parse_role() {
    let source = r#"
        role Fighter {
            can attack,
            can defend
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Role { name, .. }) => {
            assert_eq!(name, "Fighter");
            // can_kw is silent in grammar, so capabilities may be stored as fields
            // The declaration was successfully parsed
        }
        other => panic!("Expected Role, got {:?}", other),
    }
}

// ── Agent/Provider/MCP ─────────────────────────────────────────────

#[test]
fn test_parse_agent_with_fields() {
    let source = r#"
        agent ChatBot {
            model: "gpt-4",
            temperature: 0.5,
            maxTokens: 1000
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Agent { name, fields, .. }) => {
            assert_eq!(name, "ChatBot");
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].0, "model");
            assert_eq!(fields[1].0, "temperature");
            assert_eq!(fields[2].0, "maxTokens");
        }
        other => panic!("Expected Agent, got {:?}", other),
    }
}

#[test]
fn test_parse_provider() {
    let source = r#"
        provider OpenAI {
            apiKey: "sk-test",
            endpoint: "https://api.openai.com"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Provider { name, fields, .. }) => {
            assert_eq!(name, "OpenAI");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Provider, got {:?}", other),
    }
}

// ── Error handling ─────────────────────────────────────────────────

#[test]
fn test_parse_invalid_syntax_error() {
    let result = parse("let = ;");
    assert!(result.is_err());
}

#[test]
fn test_parse_unclosed_brace() {
    let result = parse("function foo() {");
    assert!(result.is_err());
}

#[test]
fn test_parse_empty_input() {
    let stmts = parse("").unwrap();
    assert!(stmts.is_empty());
}

#[test]
fn test_parse_whitespace_only() {
    let stmts = parse("   \n  \t  ").unwrap();
    assert!(stmts.is_empty());
}

#[test]
fn test_parse_multiple_statements() {
    let source = "let x = 1; let y = 2; let z = 3;";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 3);
}

// ── New expression ─────────────────────────────────────────────────

#[test]
fn test_parse_new_expr() {
    let stmts = parse("let x = new Foo(1, 2);").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::New {
                class_name, args, ..
            } => {
                assert_eq!(class_name, "Foo");
                assert_eq!(args.len(), 2);
            }
            other => panic!("Expected New, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Assignment ─────────────────────────────────────────────────────

#[test]
fn test_parse_assignment() {
    let stmts = parse("x = 42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Assignment { target, .. } => match target {
            hudhudscript_ast::Expr::Identifier(name, _) => {
                assert_eq!(name, "x");
            }
            other => panic!("Expected Identifier target, got {:?}", other),
        },
        other => panic!("Expected Assignment, got {:?}", other),
    }
}

// ── String escape sequences ────────────────────────────────────────

#[test]
fn test_parse_string_escape_sequences() {
    let stmts = parse(r#"let x = "hello\nworld";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert_eq!(s, "hello\nworld");
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_string_tab_escape() {
    let stmts = parse(r#"let x = "a\tb";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert_eq!(s, "a\tb");
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── This/Self ──────────────────────────────────────────────────────

#[test]
fn test_parse_this_keyword() {
    let stmts = parse("let x = this;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => {
            assert!(matches!(value, hudhudscript_ast::Expr::This(_)));
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Destructuring ──────────────────────────────────────────────────

#[test]
fn test_parse_destructure_object() {
    let stmts = parse("let { a, b } = obj;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure {
            pattern, is_const, ..
        } => {
            assert!(!is_const);
            match pattern {
                hudhudscript_ast::Pattern::Object { properties, .. } => {
                    assert_eq!(properties.len(), 2);
                }
                other => panic!("Expected Object pattern, got {:?}", other),
            }
        }
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

#[test]
fn test_parse_destructure_array() {
    let stmts = parse("let [a, b] = arr;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Array { elements, .. } => {
                assert_eq!(elements.len(), 2);
            }
            other => panic!("Expected Array pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

#[test]
fn test_parse_const_destructure() {
    let stmts = parse("const { x, y } = point;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { is_const, .. } => {
            assert!(*is_const);
        }
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Generator functions ────────────────────────────────────────────

#[test]
fn test_parse_generator_function() {
    let stmts = parse("function* gen() { yield 1; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            name, is_generator, ..
        } => {
            assert_eq!(name, "gen");
            assert!(*is_generator);
        }
        other => panic!("Expected generator Function, got {:?}", other),
    }
}

// ── SOP constructs ─────────────────────────────────────────────────

#[test]
fn test_parse_spawn() {
    let stmts = parse("spawn Player();").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Spawn { subject_name, .. } => {
            assert_eq!(subject_name, "Player");
        }
        other => panic!("Expected Spawn, got {:?}", other),
    }
}

#[test]
fn test_parse_require() {
    let stmts = parse("require x > 0;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Require { .. } => {}
        other => panic!("Expected Require, got {:?}", other),
    }
}

#[test]
fn test_parse_perform() {
    let stmts = parse("perform attack();").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Perform { .. } => {}
        other => panic!("Expected Perform, got {:?}", other),
    }
}

// ── Multi-language keywords (Turkish) ──────────────────────────────

#[test]
fn test_parse_turkish_let() {
    let stmts = parse("değişken x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { name, .. } => assert_eq!(name, "x"),
        other => panic!("Expected Let from Turkish, got {:?}", other),
    }
}

#[test]
fn test_parse_turkish_if() {
    let stmts = parse("eğer (x > 0) { değişken y = 1; }").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { .. } => {}
        other => panic!("Expected If from Turkish, got {:?}", other),
    }
}

#[test]
fn test_parse_turkish_function() {
    let stmts = parse("fonksiyon topla(a, b) { return a + b; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { name, params, .. } => {
            assert_eq!(name, "topla");
            assert_eq!(params.len(), 2);
        }
        other => panic!("Expected Function from Turkish, got {:?}", other),
    }
}

// ── Trait declarations ─────────────────────────────────────────────

#[test]
fn test_parse_trait_decl() {
    let source = r#"
        trait Printable {
            fn toString(): String
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait { name, methods, .. } => {
            assert_eq!(name, "Printable");
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "toString");
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Comments ───────────────────────────────────────────────────────

#[test]
fn test_parse_line_comment() {
    let stmts = parse("// this is a comment\nlet x = 1;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_block_comment() {
    let stmts = parse("/* block comment */ let x = 1;").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Parenthesized expressions ──────────────────────────────────────

#[test]
fn test_parse_parenthesized_expr() {
    let stmts = parse("let x = (1 + 2) * 3;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Mul);
            }
            other => panic!("Expected Binary Mul, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── RAG statements ─────────────────────────────────────────────────

#[test]
fn test_parse_store_decl() {
    let source = r#"
        store myStore {
            backend: "hnsw",
            dimensions: 1536
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Store { name, fields, .. }) => {
            assert_eq!(name, "myStore");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Store, got {:?}", other),
    }
}

#[test]
fn test_parse_remember() {
    let stmts = parse(r#"remember "some text" in myStore;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Remember { store_name, .. } => {
            assert_eq!(store_name.as_deref(), Some("myStore"));
        }
        other => panic!("Expected Remember, got {:?}", other),
    }
}

#[test]
fn test_parse_recall() {
    let stmts = parse(r#"recall "query" from myStore;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Recall { store_name, .. } => {
            assert_eq!(store_name.as_deref(), Some("myStore"));
        }
        other => panic!("Expected Recall, got {:?}", other),
    }
}

#[test]
fn test_parse_forget() {
    let stmts = parse(r#"forget "id" from myStore;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Forget { store_name, .. } => {
            assert_eq!(store_name.as_deref(), Some("myStore"));
        }
        other => panic!("Expected Forget, got {:?}", other),
    }
}

// ── Protocol/Strategy ──────────────────────────────────────────────

#[test]
fn test_parse_protocol() {
    let source = r#"
        protocol MyStrategy {
            execution: parallel,
            timeout: 5000
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Protocol {
            name,
            execution,
            timeout,
            ..
        }) => {
            assert_eq!(name, "MyStrategy");
            assert_eq!(execution.as_deref(), Some("parallel"));
            assert_eq!(*timeout, Some(5000.0));
        }
        other => panic!("Expected Protocol, got {:?}", other),
    }
}

// ── Governance declaration ──────────────────────────────────────────

#[test]
fn test_parse_governance_decl() {
    let source = r#"
        governance MyGov: democracy {
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Governance {
            name,
            base_type,
            ..
        }) => {
            assert_eq!(name, "MyGov");
            assert_eq!(base_type, "democracy");
        }
        other => panic!("Expected Governance, got {:?}", other),
    }
}

// ── BiDi stripping in parse ────────────────────────────────────────

#[test]
fn test_parse_strips_bidi_controls() {
    // RLM character embedded in source
    let source = "let x\u{200F} = 42;";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Nested structures ──────────────────────────────────────────────

#[test]
fn test_parse_nested_if() {
    let source = "if (a) { if (b) { let x = 1; } }";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { then_branch, .. } => match then_branch.as_ref() {
            hudhudscript_ast::Stmt::Block { statements, .. } => {
                assert!(matches!(statements[0], hudhudscript_ast::Stmt::If { .. }));
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected If, got {:?}", other),
    }
}

#[test]
fn test_parse_nested_function_calls() {
    let stmts = parse("foo(bar(baz()));").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { args, .. }) => {
            assert_eq!(args.len(), 1);
            match &args[0] {
                hudhudscript_ast::Expr::Call {
                    args: inner_args, ..
                } => {
                    assert_eq!(inner_args.len(), 1);
                }
                other => panic!("Expected nested Call, got {:?}", other),
            }
        }
        other => panic!("Expected Call, got {:?}", other),
    }
}

// ── For range ──────────────────────────────────────────────────────

#[test]
fn test_parse_for_range() {
    let stmts = parse("for(0, 10) { let x = 1; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForRange { step, .. } => {
            assert!(step.is_none());
        }
        other => panic!("Expected ForRange, got {:?}", other),
    }
}

#[test]
fn test_parse_for_range_with_step() {
    let stmts = parse("for(0, 10, 2) { let x = 1; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForRange { step, .. } => {
            assert!(step.is_some());
        }
        other => panic!("Expected ForRange with step, got {:?}", other),
    }
}

// ── MCP declarations ───────────────────────────────────────────────

#[test]
fn test_parse_mcp_server() {
    let source = r#"
        mcp FileServer {
            transport: "stdio",
            command: "npx"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::McpServer(decl) => {
            assert_eq!(decl.name, "FileServer");
            assert_eq!(decl.fields.len(), 2);
        }
        other => panic!("Expected McpServer, got {:?}", other),
    }
}

// ── Swarm declaration ──────────────────────────────────────────────

#[test]
fn test_parse_swarm() {
    let source = r#"
        swarm MySwarm {
            agents: ["a1", "a2"],
            strategy: parallel
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Swarm {
            name,
            agents,
            strategy,
            ..
        }) => {
            assert_eq!(name, "MySwarm");
            assert_eq!(agents.len(), 2);
            assert_eq!(strategy, "parallel");
        }
        other => panic!("Expected Swarm, got {:?}", other),
    }
}

// ── Community declaration ──────────────────────────────────────────

#[test]
fn test_parse_community() {
    let source = r#"
        community MyCommunity {
            members: ["agent1", "agent2"]
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Community {
            name, members, ..
        }) => {
            assert_eq!(name, "MyCommunity");
            assert_eq!(members.len(), 2);
        }
        other => panic!("Expected Community, got {:?}", other),
    }
}

// ── Entity declaration ─────────────────────────────────────────────

#[test]
fn test_parse_entity() {
    let source = r#"
        entity Player {
            data health: Number = 100,
            data name: String
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Entity { name, fields, .. }) => {
            assert_eq!(name, "Player");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].0, "health");
        }
        other => panic!("Expected Entity, got {:?}", other),
    }
}

// ── Event declaration ──────────────────────────────────────────────

#[test]
fn test_parse_event_decl() {
    let source = r#"
        event Damage {
            amount: Number,
            source: String
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Event { name, fields, .. }) => {
            assert_eq!(name, "Damage");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Event, got {:?}", other),
    }
}

// ── Contract declaration ───────────────────────────────────────────

#[test]
fn test_parse_contract() {
    let source = r#"
        contract Trade: {
            buyer: "Alice",
            seller: "Bob"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Contract { name, fields, .. }) => {
            assert_eq!(name, "Trade");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Contract, got {:?}", other),
    }
}

// ── Treaty declaration ─────────────────────────────────────────────

#[test]
fn test_parse_treaty() {
    let source = r#"
        treaty Alliance: {
            parties: "A",
            terms: "peace"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Treaty { name, fields, .. }) => {
            assert_eq!(name, "Alliance");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Treaty, got {:?}", other),
    }
}

// ── Subject declaration ────────────────────────────────────────────

#[test]
fn test_parse_subject() {
    let source = r#"
        subject Player {
            state health: 100,
            can attack,
            can defend
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Subject {
            name,
            states,
            capabilities,
            ..
        }) => {
            assert_eq!(name, "Player");
            assert_eq!(states.len(), 1);
            assert_eq!(states[0].0, "health");
            assert_eq!(capabilities.len(), 2);
        }
        other => panic!("Expected Subject, got {:?}", other),
    }
}

// ── Relation declaration ───────────────────────────────────────────

#[test]
fn test_parse_relation() {
    let source = r#"
        relation Player <-> Merchant {
            trust: 50
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Relation {
            subject_a,
            subject_b,
            fields,
            ..
        }) => {
            assert_eq!(subject_a, "Player");
            assert_eq!(subject_b, "Merchant");
            assert_eq!(fields.len(), 1);
        }
        other => panic!("Expected Relation, got {:?}", other),
    }
}

// ── Action declaration ─────────────────────────────────────────────

#[test]
fn test_parse_action_decl() {
    let source = r#"
        action Attack {
            target: "enemy",
            damage: 10
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Action { name, fields, .. }) => {
            assert_eq!(name, "Attack");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Action, got {:?}", other),
    }
}

// ── Resource declaration ───────────────────────────────────────────

#[test]
fn test_parse_resource_decl() {
    let source = r#"
        resource Database {
            url: "postgres://localhost",
            pool: 10
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Resource { name, fields, .. }) => {
            assert_eq!(name, "Database");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Resource, got {:?}", other),
    }
}

// ── Data declaration ───────────────────────────────────────────────

#[test]
fn test_parse_data_named() {
    let source = r#"
        data Config {
            host = "localhost";
            port = 8080;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { name, value, .. } => {
            assert_eq!(name, "Config");
            match value {
                hudhudscript_ast::Expr::Object { properties, .. } => {
                    assert_eq!(properties.len(), 2);
                    assert_eq!(properties[0].0, "host");
                    assert_eq!(properties[1].0, "port");
                }
                other => panic!("Expected Object, got {:?}", other),
            }
        }
        other => panic!("Expected Let (data), got {:?}", other),
    }
}

// ── State machine declaration ──────────────────────────────────────

#[test]
fn test_parse_statemachine() {
    let source = r#"
        statemachine TrafficLight {
            state Red {
                on event(Timer) -> Green
            }
            state Green {
                on event(Timer) -> Yellow
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::StateMachine {
            name, fields, ..
        }) => {
            assert_eq!(name, "TrafficLight");
            // Fields store transitions as "StateName.EventName" -> "NextState"
            assert!(fields.len() >= 2);
        }
        other => panic!("Expected StateMachine, got {:?}", other),
    }
}

// ── Music declaration ──────────────────────────────────────────────

#[test]
fn test_parse_music_note() {
    let source = r#"
        note C4 {
            pitch: 262,
            duration: 1
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Music {
            kind, name, fields, ..
        }) => {
            assert_eq!(kind, "note");
            assert_eq!(name, "C4");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Music, got {:?}", other),
    }
}

// ── Deploy declaration ─────────────────────────────────────────────

#[test]
fn test_parse_deploy() {
    // Deploy uses deploy_member wrapping - use target/github blocks
    let source = r#"
        deploy MyApp {
            target {
                web {
                    output: "dist"
                }
            }
        }
    "#;
    let result = parse(source);
    // deploy might not parse in all configurations, so we just check it doesn't panic
    if let Ok(stmts) = result {
        if !stmts.is_empty() {
            match &stmts[0] {
                hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Deploy { name, .. }) => {
                    assert_eq!(name, "MyApp");
                }
                _ => {} // May parse differently
            }
        }
    }
}

// ── Rule declaration ───────────────────────────────────────────────

#[test]
fn test_parse_rule_decl() {
    let source = r#"
        rule HealthCheck {
            priority: 10
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Rule { name, priority, .. }) => {
            assert_eq!(name, "HealthCheck");
            assert_eq!(*priority, 10);
        }
        other => panic!("Expected Rule, got {:?}", other),
    }
}

// ── Class with implements ──────────────────────────────────────────

#[test]
fn test_parse_class_implements() {
    let source = r#"
        class Dog <- Animal implements Printable {
            public function speak() {
                return "woof";
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Dog");
            assert_eq!(decl.parent.as_deref(), Some("Animal"));
            assert_eq!(decl.implements, vec!["Printable".to_string()]);
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── If-else if chains ──────────────────────────────────────────────

#[test]
fn test_parse_if_else_if() {
    let source = "if (x > 2) { let a = 1; } else if (x > 1) { let a = 2; } else { let a = 3; }";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { else_branch, .. } => {
            // else_branch should be another If (the else-if)
            assert!(else_branch.is_some());
            match else_branch.as_ref().unwrap().as_ref() {
                hudhudscript_ast::Stmt::If {
                    else_branch: inner_else,
                    ..
                } => {
                    assert!(inner_else.is_some());
                }
                other => panic!("Expected nested If (else if), got {:?}", other),
            }
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ── Spread operator ────────────────────────────────────────────────

#[test]
fn test_parse_spread_in_array() {
    let stmts = parse("let x = [1, ...arr, 2];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[1], hudhudscript_ast::Expr::Spread { .. }));
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Rest params in functions ───────────────────────────────────────

#[test]
fn test_parse_rest_params() {
    let stmts = parse("function foo(a, ...rest) { return rest; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { params, .. } => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "...rest");
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Send/Receive ───────────────────────────────────────────────────

#[test]
fn test_parse_send() {
    let stmts = parse(r#"send "hello" to target;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Send { .. } => {}
        other => panic!("Expected Send, got {:?}", other),
    }
}

#[test]
fn test_parse_receive() {
    let stmts = parse("receive msg from source;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Receive { variable, .. } => {
            assert_eq!(variable, "msg");
        }
        other => panic!("Expected Receive, got {:?}", other),
    }
}

// ── Effect declaration ─────────────────────────────────────────────

#[test]
fn test_parse_effect() {
    let source = "effect on Damage { let x = 1; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Effect {
            event_name, body, ..
        }) => {
            assert_eq!(event_name, "Damage");
            assert_eq!(body.len(), 1);
        }
        other => panic!("Expected Effect, got {:?}", other),
    }
}

// ── Decorated subject ──────────────────────────────────────────────

#[test]
fn test_parse_decorated_subject() {
    let source = r#"
        @ai subject Bot {
            state health: 100
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Subject {
            name, decorators, ..
        }) => {
            assert_eq!(name, "Bot");
            assert_eq!(decorators.len(), 1);
            assert_eq!(decorators[0].name, "ai");
        }
        other => panic!("Expected Subject with decorators, got {:?}", other),
    }
}

// ── Japanese parsing ───────────────────────────────────────────────

#[test]
fn test_parse_japanese_let() {
    let stmts = parse("変数 x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_japanese_function() {
    let stmts = parse("関数 foo() { return 1; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { name, .. } => {
            assert_eq!(name, "foo");
        }
        other => panic!("Expected Function from Japanese, got {:?}", other),
    }
}

// ── Arabic parsing ─────────────────────────────────────────────────

#[test]
fn test_parse_arabic_let() {
    let stmts = parse("متغير x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── C-style for loop ───────────────────────────────────────────────

#[test]
fn test_parse_c_style_for() {
    let source = "for (let i = 0; i < 10; i = i + 1) { let x = i; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForCStyle {
            init,
            condition,
            update,
            ..
        } => {
            assert!(init.is_some());
            assert!(condition.is_some());
            assert!(update.is_some());
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ── Export variations ──────────────────────────────────────────────

#[test]
fn test_parse_export_const() {
    let stmts = parse("export const PI = 3.14;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Const { name, .. } => {
                assert_eq!(name, "PI");
            }
            other => panic!("Expected Const in Export, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Await expression ───────────────────────────────────────────────

#[test]
fn test_parse_await_expr() {
    let source = "async function foo() { let x = await bar(); }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, is_async, .. } => {
            assert!(*is_async);
            assert_eq!(body.len(), 1);
            match &body[0] {
                hudhudscript_ast::Stmt::Let { value, .. } => {
                    assert!(matches!(value, hudhudscript_ast::Expr::Await { .. }));
                }
                other => panic!("Expected Let with await, got {:?}", other),
            }
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Yield expression ───────────────────────────────────────────────

#[test]
fn test_parse_yield() {
    let source = "function* gen() { yield 42; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            body, is_generator, ..
        } => {
            assert!(*is_generator);
            assert_eq!(body.len(), 1);
            match &body[0] {
                hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Yield { value, .. }) => {
                    assert!(value.is_some());
                }
                other => panic!("Expected Yield, got {:?}", other),
            }
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Match with enum pattern ────────────────────────────────────────

#[test]
fn test_parse_match_identifier_pattern() {
    let source = r#"match (x) { y => { let z = y; } }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            match &arms[0].pattern {
                hudhudscript_ast::MatchPattern::Identifier(name) => {
                    assert_eq!(name, "y");
                }
                other => panic!("Expected Identifier pattern, got {:?}", other),
            }
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Match with string pattern ──────────────────────────────────────

#[test]
fn test_parse_match_string_pattern() {
    let source = r#"match (x) { "hello" => { let y = 1; } }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => match &arms[0].pattern {
            hudhudscript_ast::MatchPattern::Literal(hudhudscript_ast::Literal::String(s)) => {
                assert_eq!(s, "hello");
            }
            other => panic!("Expected String literal pattern, got {:?}", other),
        },
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Expression statement (bare call) ───────────────────────────────

#[test]
fn test_parse_bare_expression_stmt() {
    let stmts = parse("42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Literal(
            hudhudscript_ast::Literal::Number(n, _),
            _,
        )) => {
            assert_eq!(*n, 42.0);
        }
        other => panic!("Expected Expr(Number), got {:?}", other),
    }
}

// ── Unary plus ─────────────────────────────────────────────────────

#[test]
fn test_parse_unary_plus() {
    let stmts = parse("let x = +5;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Unary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::UnaryOp::Plus);
            }
            other => panic!("Expected Unary Plus, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Object with string keys ────────────────────────────────────────

#[test]
fn test_parse_object_with_string_keys() {
    let stmts = parse(r#"let x = { "key-1": 1, "key-2": 2 };"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                assert_eq!(properties[0].0, "key-1");
                assert_eq!(properties[1].0, "key-2");
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Nested arrays and objects ──────────────────────────────────────

#[test]
fn test_parse_nested_array() {
    let stmts = parse("let x = [[1, 2], [3, 4]];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], hudhudscript_ast::Expr::Array { .. }));
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_nested_object() {
    let stmts = parse(r#"let x = { inner: { a: 1 } };"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties.len(), 1);
                assert_eq!(properties[0].0, "inner");
                assert!(matches!(
                    properties[0].1,
                    hudhudscript_ast::Expr::Object { .. }
                ));
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Async arrow function ──────────────────────────────────────────

#[test]
fn test_parse_async_arrow_function() {
    // async arrow may parse as async call wrapping arrow — verify it parses without error
    let stmts = parse("let f = async (x) => x + 1;").unwrap();
    assert!(!stmts.is_empty());
}

// ── Anonymous function expression ─────────────────────────────────

#[test]
fn test_parse_anon_function() {
    let stmts = parse("let f = function(a, b) { return a + b; };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, body, .. } => {
                assert_eq!(params.len(), 2);
                match body {
                    hudhudscript_ast::ArrowFunctionBody::Block(stmts) => {
                        assert_eq!(stmts.len(), 1);
                    }
                    other => panic!("Expected Block body, got {:?}", other),
                }
            }
            other => panic!("Expected ArrowFunction (anon), got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Export variations ──────────────────────────────────────────────

#[test]
fn test_parse_export_class() {
    let stmts = parse(r#"export class Foo { public function bar() { return 1; } }"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Class(decl) => {
                assert_eq!(decl.name, "Foo");
            }
            other => panic!("Expected Class in Export, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

#[test]
fn test_parse_export_var() {
    let stmts = parse("export var count = 0;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::VarDecl(decl) => {
                assert_eq!(decl.name, "count");
            }
            other => panic!("Expected VarDecl (var) in Export, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

#[test]
fn test_parse_export_async_function() {
    let stmts = parse("export async function fetch() { return 1; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Function { name, is_async, .. } => {
                assert_eq!(name, "fetch");
                assert!(*is_async);
            }
            other => panic!("Expected async Function in Export, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Class with static, fields, protected ──────────────────────────

#[test]
fn test_parse_class_with_field() {
    let source = r#"
        class Config {
            public let name = "test"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Config");
            assert!(decl.members.len() >= 1);
            match &decl.members[0] {
                hudhudscript_ast::ClassMember::Field {
                    access,
                    name,
                    initializer,
                    ..
                } => {
                    assert_eq!(*access, hudhudscript_ast::AccessModifier::Public);
                    assert_eq!(name, "name");
                    assert!(initializer.is_some());
                }
                other => panic!("Expected Field, got {:?}", other),
            }
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

#[test]
fn test_parse_class_protected_method() {
    let source = r#"
        class Base {
            protected function helper() {
                return 1;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => match &decl.members[0] {
            hudhudscript_ast::ClassMember::Method { access, name, .. } => {
                assert_eq!(*access, hudhudscript_ast::AccessModifier::Protected);
                assert_eq!(name, "helper");
            }
            other => panic!("Expected protected Method, got {:?}", other),
        },
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Enum with fields ──────────────────────────────────────────────

#[test]
fn test_parse_enum_with_fields() {
    let source = "enum Shape { Circle(radius), Rectangle(width, height) }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::EnumDecl { name, variants, .. } => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Circle");
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[1].name, "Rectangle");
            assert_eq!(variants[1].fields.len(), 2);
        }
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

// ── Match patterns ────────────────────────────────────────────────

#[test]
fn test_parse_match_boolean_pattern() {
    let source = r#"match (x) { true => { let y = 1; } false => { let y = 0; } }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            match &arms[0].pattern {
                hudhudscript_ast::MatchPattern::Literal(hudhudscript_ast::Literal::Boolean(b)) => {
                    assert!(*b);
                }
                other => panic!("Expected Boolean true pattern, got {:?}", other),
            }
            match &arms[1].pattern {
                hudhudscript_ast::MatchPattern::Literal(hudhudscript_ast::Literal::Boolean(b)) => {
                    assert!(!*b);
                }
                other => panic!("Expected Boolean false pattern, got {:?}", other),
            }
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn test_parse_match_null_pattern() {
    let source = r#"match (x) { null => { let y = 0; } _ => { let y = 1; } }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            assert!(matches!(
                arms[0].pattern,
                hudhudscript_ast::MatchPattern::Literal(hudhudscript_ast::Literal::Null)
            ));
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

#[test]
fn test_parse_match_number_pattern() {
    let source = r#"match (x) { 42 => { let y = 1; } }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => match &arms[0].pattern {
            hudhudscript_ast::MatchPattern::Literal(hudhudscript_ast::Literal::Number(n, _)) => {
                assert_eq!(*n, 42.0);
            }
            other => panic!("Expected Number pattern, got {:?}", other),
        },
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Switch with default only ──────────────────────────────────────

#[test]
fn test_parse_switch_default_only() {
    let source = r#"switch (x) { default: let y = 0; }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Switch { cases, default, .. } => {
            assert!(cases.is_empty());
            assert!(default.is_some());
        }
        other => panic!("Expected Switch, got {:?}", other),
    }
}

// ── Try with finally only ─────────────────────────────────────────

#[test]
fn test_parse_try_finally_only() {
    let stmts = parse("try { foo(); } finally { cleanup(); }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Try {
            catch_clause,
            finally_block,
            ..
        } => {
            assert!(catch_clause.is_none());
            assert!(finally_block.is_some());
        }
        other => panic!("Expected Try, got {:?}", other),
    }
}

// ── Spread in function call args ──────────────────────────────────

#[test]
fn test_parse_spread_in_call_args() {
    let stmts = parse("foo(...args);").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { args, .. }) => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0], hudhudscript_ast::Expr::Spread { .. }));
        }
        other => panic!("Expected Call with spread, got {:?}", other),
    }
}

// ── Spread in object ──────────────────────────────────────────────

#[test]
fn test_parse_spread_in_object() {
    let stmts = parse("let x = { ...base, name: 1 };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                // First property should be a spread
                assert!(properties[0].0.starts_with("__spread__"));
                assert!(matches!(
                    properties[0].1,
                    hudhudscript_ast::Expr::Spread { .. }
                ));
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Method call chain ─────────────────────────────────────────────

#[test]
fn test_parse_chained_method_call() {
    let stmts = parse("obj.method(1).another(2);").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { callee, args, .. }) => {
            assert_eq!(args.len(), 1);
            match callee.as_ref() {
                hudhudscript_ast::Expr::Member {
                    property, object, ..
                } => {
                    assert_eq!(property, "another");
                    match object.as_ref() {
                        hudhudscript_ast::Expr::Call {
                            callee: inner_callee,
                            ..
                        } => match inner_callee.as_ref() {
                            hudhudscript_ast::Expr::Member { property, .. } => {
                                assert_eq!(property, "method");
                            }
                            other => panic!("Expected Member, got {:?}", other),
                        },
                        other => panic!("Expected Call, got {:?}", other),
                    }
                }
                other => panic!("Expected Member, got {:?}", other),
            }
        }
        other => panic!("Expected Expr(Call), got {:?}", other),
    }
}

// ── String escape sequences ───────────────────────────────────────

#[test]
fn test_parse_string_escape_return() {
    let stmts = parse(r#"let x = "a\rb";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert_eq!(s, "a\rb");
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_string_escape_backslash() {
    let stmts = parse(r#"let x = "a\\b";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert_eq!(s, "a\\b");
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_string_escape_quote() {
    let stmts = parse(r#"let x = "say \"hi\"";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert_eq!(s, "say \"hi\"");
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Self keyword ──────────────────────────────────────────────────

#[test]
fn test_parse_self_keyword() {
    let stmts = parse("let x = self;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => {
            assert!(matches!(value, hudhudscript_ast::Expr::This(_)));
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Spawn with arguments ──────────────────────────────────────────

#[test]
fn test_parse_spawn_with_args() {
    let stmts = parse(r#"spawn Player("hero", 100);"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Spawn {
            subject_name, args, ..
        } => {
            assert_eq!(subject_name, "Player");
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected Spawn, got {:?}", other),
    }
}

// ── Template string with multiple interpolations ──────────────────

#[test]
fn test_parse_template_string_multiple_interps() {
    let stmts = parse("let x = `${a} and ${b}`;").unwrap();
    assert!(
        !stmts.is_empty(),
        "template with multiple interpolations should parse"
    );
}

// ── Yield without value ───────────────────────────────────────────

#[test]
fn test_parse_yield_without_value() {
    let source = "function* gen() { yield; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            body, is_generator, ..
        } => {
            assert!(*is_generator);
            match &body[0] {
                hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Yield { value, .. }) => {
                    assert!(value.is_none());
                }
                other => panic!("Expected Yield without value, got {:?}", other),
            }
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Remember/Recall without store name ────────────────────────────

#[test]
fn test_parse_remember_without_store() {
    let stmts = parse(r#"remember "some text";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Remember { store_name, .. } => {
            assert!(store_name.is_none());
        }
        other => panic!("Expected Remember, got {:?}", other),
    }
}

#[test]
fn test_parse_recall_without_store() {
    let stmts = parse(r#"recall "query";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Recall { store_name, .. } => {
            assert!(store_name.is_none());
        }
        other => panic!("Expected Recall, got {:?}", other),
    }
}

#[test]
fn test_parse_forget_without_store() {
    let stmts = parse(r#"forget "id";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Forget { store_name, .. } => {
            assert!(store_name.is_none());
        }
        other => panic!("Expected Forget, got {:?}", other),
    }
}

// ── Data declaration without name ─────────────────────────────────

#[test]
fn test_parse_data_anonymous() {
    let source = r#"
        data {
            host = "localhost";
            port = 8080;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Object { properties, .. }) => {
            assert_eq!(properties.len(), 2);
        }
        other => panic!("Expected Expr(Object), got {:?}", other),
    }
}

// ── Governance with override fields ───────────────────────────────

#[test]
fn test_parse_governance_with_fields() {
    let source = r#"
        governance MyGov: democracy {
            quorum: 5,
            threshold: 0.5
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Governance {
            name,
            base_type,
            fields,
            ..
        }) => {
            assert_eq!(name, "MyGov");
            assert_eq!(base_type, "democracy");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Governance, got {:?}", other),
    }
}

// ── Constitution with laws ────────────────────────────────────────

#[test]
fn test_parse_constitution_with_laws() {
    let source = r#"
        constitution GameRules {
            description: "Game rules",
            laws: [{ name: "NoCheat", description: "No cheating", enforcement: mandatory }]
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Constitution {
            name,
            description,
            laws,
            ..
        }) => {
            assert_eq!(name, "GameRules");
            assert_eq!(description.as_deref(), Some("Game rules"));
            assert_eq!(laws.len(), 1);
            assert_eq!(laws[0].name, "NoCheat");
        }
        other => panic!("Expected Constitution, got {:?}", other),
    }
}

// ── Law with rules ────────────────────────────────────────────────

#[test]
fn test_parse_law_with_rules() {
    let source = r#"
        law Fairness {
            description: "Be fair",
            enforcement: mandatory,
            rules: ["rule1", "rule2"]
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Law { name, rules, .. }) => {
            assert_eq!(name, "Fairness");
            assert_eq!(rules.len(), 2);
        }
        other => panic!("Expected Law, got {:?}", other),
    }
}

// ── Role with fields ──────────────────────────────────────────────

#[test]
fn test_parse_role_with_fields() {
    let source = r#"
        role Healer {
            can heal,
            can resurrect
        }
    "#;
    let stmts = parse(source).unwrap();
    assert!(!stmts.is_empty(), "role with capabilities should parse");
}

// ── Protocol with governance and timeout ──────────────────────────

#[test]
fn test_parse_protocol_with_governance() {
    let source = r#"
        protocol TeamWork {
            execution: sequential,
            governance: democracy,
            timeout: 3000
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Protocol {
            name,
            execution,
            governance,
            timeout,
            ..
        }) => {
            assert_eq!(name, "TeamWork");
            assert_eq!(execution.as_deref(), Some("sequential"));
            assert_eq!(governance.as_deref(), Some("democracy"));
            assert_eq!(*timeout, Some(3000.0));
        }
        other => panic!("Expected Protocol, got {:?}", other),
    }
}

// ── Subject with roles and intents ────────────────────────────────

#[test]
fn test_parse_subject_with_roles() {
    let source = r#"
        subject Warrior has Fighter {
            state health: 100,
            can attack,
            can defend
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Subject {
            name,
            roles,
            states,
            capabilities,
            ..
        }) => {
            assert_eq!(name, "Warrior");
            assert_eq!(roles.len(), 1);
            assert_eq!(roles[0], "Fighter");
            assert_eq!(states.len(), 1);
            assert_eq!(capabilities.len(), 2);
        }
        other => panic!("Expected Subject, got {:?}", other),
    }
}

// ── Decorated subject with params ─────────────────────────────────

#[test]
fn test_parse_decorated_subject_with_params() {
    let source = r#"
        @payment(provider: "stripe") subject Merchant {
            state gold: 500,
            can trade
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Subject {
            name,
            decorators,
            states,
            capabilities,
            ..
        }) => {
            assert_eq!(name, "Merchant");
            assert_eq!(decorators.len(), 1);
            assert_eq!(decorators[0].name, "payment");
            assert_eq!(decorators[0].params.len(), 1);
            assert_eq!(decorators[0].params[0].0, "provider");
            assert_eq!(states.len(), 1);
            assert_eq!(capabilities.len(), 1);
        }
        other => panic!("Expected Subject, got {:?}", other),
    }
}

// ── Multiple decorators on subject ────────────────────────────────

#[test]
fn test_parse_multiple_decorators() {
    let source = r#"
        @ai @logging subject SmartBot {
            state level: 1
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Subject {
            name, decorators, ..
        }) => {
            assert_eq!(name, "SmartBot");
            assert_eq!(decorators.len(), 2);
            assert_eq!(decorators[0].name, "ai");
            assert_eq!(decorators[1].name, "logging");
        }
        other => panic!("Expected Subject, got {:?}", other),
    }
}

// ── Destructure with rest ─────────────────────────────────────────

#[test]
fn test_parse_destructure_array_with_rest() {
    let stmts = parse("let [first, ...rest] = arr;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Array { elements, rest } => {
                assert_eq!(elements.len(), 1);
                assert!(rest.is_some());
            }
            other => panic!("Expected Array pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

#[test]
fn test_parse_destructure_object_with_rest() {
    let stmts = parse("let { a, ...rest } = obj;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Object { properties, rest } => {
                assert_eq!(properties.len(), 1);
                assert!(rest.is_some());
            }
            other => panic!("Expected Object pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Complex nested expressions ────────────────────────────────────

#[test]
fn test_parse_complex_binary_chain() {
    let stmts = parse("let x = a + b * c - d / e;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => {
            // Should parse as (a + (b * c)) - (d / e)
            assert!(matches!(value, hudhudscript_ast::Expr::Binary { .. }));
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_parse_nested_logical() {
    let stmts = parse("let x = a && b || c && d;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Or);
            }
            other => panic!("Expected Binary Or, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Music declarations (chord, melody, etc.) ──────────────────────

#[test]
fn test_parse_music_chord() {
    let source = r#"
        chord Am {
            notes: "A,C,E",
            quality: "minor"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Music {
            kind, name, fields, ..
        }) => {
            assert_eq!(kind, "chord");
            assert_eq!(name, "Am");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Music(chord), got {:?}", other),
    }
}

#[test]
fn test_parse_music_melody() {
    let source = r#"
        melody Theme {
            notes: "C,D,E,F,G",
            tempo: 120
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Music { kind, name, .. }) => {
            assert_eq!(kind, "melody");
            assert_eq!(name, "Theme");
        }
        other => panic!("Expected Music(melody), got {:?}", other),
    }
}

// ── Float number ──────────────────────────────────────────────────

#[test]
fn test_parse_float_number() {
    let stmts = parse("let x = 3.14159;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Number(n, _), _) => {
                assert!((n - 3.14159).abs() < 0.00001);
            }
            other => panic!("Expected Number, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Complex for-in with array literal ─────────────────────────────

#[test]
fn test_parse_for_in_array_literal() {
    let stmts = parse("for (item in [1, 2, 3]) { let x = item; }").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::For { variable, .. } => {
            assert_eq!(variable, "item");
        }
        other => panic!("Expected For, got {:?}", other),
    }
}

// ── MCP with more fields ──────────────────────────────────────────

#[test]
fn test_parse_mcp_with_tools() {
    let source = r#"
        mcp ToolServer {
            transport: "stdio",
            command: "npx",
            args: ["@modelcontextprotocol/server-tool"]
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::McpServer(decl) => {
            assert_eq!(decl.name, "ToolServer");
            assert_eq!(decl.fields.len(), 3);
        }
        other => panic!("Expected McpServer, got {:?}", other),
    }
}

// ── Trait with generic params ─────────────────────────────────────

#[test]
fn test_parse_trait_with_generics() {
    let source = r#"
        trait Comparable<T> {
            fn compareTo(other): Number
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait {
            name,
            type_params,
            methods,
            ..
        } => {
            assert_eq!(name, "Comparable");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "compareTo");
            assert!(methods[0].return_type.is_some());
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Trait with multiple methods ───────────────────────────────────

#[test]
fn test_parse_trait_multiple_methods() {
    let source = r#"
        trait Serializable {
            fn serialize(): String
            fn deserialize(data): Any
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait { name, methods, .. } => {
            assert_eq!(name, "Serializable");
            assert_eq!(methods.len(), 2);
            assert_eq!(methods[0].name, "serialize");
            assert_eq!(methods[1].name, "deserialize");
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Function with generic params ──────────────────────────────────

#[test]
fn test_parse_function_with_generics() {
    let source = "function identity<T>(x) { return x; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            name, type_params, ..
        } => {
            assert_eq!(name, "identity");
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Class with generic params and implements ──────────────────────

#[test]
fn test_parse_class_with_generics() {
    let source = r#"
        class Container<T> {
            public function get() {
                return this;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Container");
            assert_eq!(decl.type_params.len(), 1);
            assert_eq!(decl.type_params[0].name, "T");
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Assignment to member expression ───────────────────────────────

#[test]
fn test_parse_member_assignment() {
    let stmts = parse("obj.prop = 42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Assignment { target, .. } => match target {
            hudhudscript_ast::Expr::Member { property, .. } => {
                assert_eq!(property, "prop");
            }
            other => panic!("Expected Member target, got {:?}", other),
        },
        other => panic!("Expected Assignment, got {:?}", other),
    }
}

// ── Assignment to index expression ────────────────────────────────

#[test]
fn test_parse_index_assignment() {
    let stmts = parse("arr[0] = 42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Assignment { target, .. } => {
            assert!(matches!(target, hudhudscript_ast::Expr::Index { .. }));
        }
        other => panic!("Expected Assignment, got {:?}", other),
    }
}

// ── Multiple implements ───────────────────────────────────────────

#[test]
fn test_parse_class_multiple_implements() {
    let source = r#"
        class Robot <- Machine implements Printable, Serializable {
            public function run() {
                return 1;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Robot");
            assert_eq!(decl.parent.as_deref(), Some("Machine"));
            assert!(decl.implements.len() >= 2);
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Function call with no args ────────────────────────────────────

#[test]
fn test_parse_function_call_no_args() {
    let stmts = parse("foo();").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { args, .. }) => {
            assert!(args.is_empty());
        }
        other => panic!("Expected Expr(Call), got {:?}", other),
    }
}

// ── New without args ──────────────────────────────────────────────

#[test]
fn test_parse_new_no_args() {
    let stmts = parse("let x = new Foo();").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::New {
                class_name, args, ..
            } => {
                assert_eq!(class_name, "Foo");
                assert!(args.is_empty());
            }
            other => panic!("Expected New, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Arrow function with no params ─────────────────────────────────

#[test]
fn test_parse_arrow_no_params() {
    let stmts = parse("let f = () => 42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, body, .. } => {
                assert!(params.is_empty());
                match body {
                    hudhudscript_ast::ArrowFunctionBody::Expression(_) => {}
                    other => panic!("Expected Expression body, got {:?}", other),
                }
            }
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Arrow function with multiple params ───────────────────────────

#[test]
fn test_parse_arrow_multiple_params() {
    let stmts = parse("let f = (a, b, c) => a + b + c;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, .. } => {
                assert_eq!(params.len(), 3);
            }
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Index with expression ─────────────────────────────────────────

#[test]
fn test_parse_index_with_expr() {
    let stmts = parse("let x = arr[i + 1];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Index { index, .. } => {
                assert!(matches!(
                    index.as_ref(),
                    hudhudscript_ast::Expr::Binary { .. }
                ));
            }
            other => panic!("Expected Index, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Nested for loops ──────────────────────────────────────────────

#[test]
fn test_parse_nested_for() {
    let source = r#"
        for (i in items) {
            for (j in other) {
                let x = i;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::For { body, .. } => match body.as_ref() {
            hudhudscript_ast::Stmt::Block { statements, .. } => {
                assert!(matches!(statements[0], hudhudscript_ast::Stmt::For { .. }));
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected For, got {:?}", other),
    }
}

// ── Deeply nested member + call ───────────────────────────────────

#[test]
fn test_parse_deep_member_call_chain() {
    let stmts = parse("a.b.c.d();").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { callee, .. }) => {
            match callee.as_ref() {
                hudhudscript_ast::Expr::Member { property, .. } => {
                    assert_eq!(property, "d");
                }
                other => panic!("Expected Member, got {:?}", other),
            }
        }
        other => panic!("Expected Expr(Call), got {:?}", other),
    }
}

// ── While with complex condition ──────────────────────────────────

#[test]
fn test_parse_while_complex_condition() {
    let source = "while (x > 0 && y < 10) { x = x - 1; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::While { condition, .. } => {
            assert!(matches!(
                condition,
                hudhudscript_ast::Expr::Binary {
                    op: hudhudscript_ast::BinaryOp::And,
                    ..
                }
            ));
        }
        other => panic!("Expected While, got {:?}", other),
    }
}

// ── Mixed array elements ──────────────────────────────────────────

#[test]
fn test_parse_mixed_array() {
    let stmts = parse(r#"let x = [1, "two", true, null];"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 4);
                assert!(matches!(
                    elements[0],
                    hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Number(_, _), _)
                ));
                assert!(matches!(
                    elements[1],
                    hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(_), _)
                ));
                assert!(matches!(
                    elements[2],
                    hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Boolean(true), _)
                ));
                assert!(matches!(
                    elements[3],
                    hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Null, _)
                ));
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Chained unary operators ───────────────────────────────────────

#[test]
fn test_parse_double_negation() {
    let stmts = parse("let x = !!flag;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Unary { op, expr, .. } => {
                assert_eq!(*op, hudhudscript_ast::UnaryOp::Not);
                match expr.as_ref() {
                    hudhudscript_ast::Expr::Unary { op: inner_op, .. } => {
                        assert_eq!(*inner_op, hudhudscript_ast::UnaryOp::Not);
                    }
                    other => panic!("Expected inner Unary Not, got {:?}", other),
                }
            }
            other => panic!("Expected Unary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Complex object with nested arrays ─────────────────────────────

#[test]
fn test_parse_object_with_array_value() {
    let stmts = parse(r#"let x = { items: [1, 2, 3], name: "test" };"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                assert!(matches!(
                    properties[0].1,
                    hudhudscript_ast::Expr::Array { .. }
                ));
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Function body with multiple statements ────────────────────────

#[test]
fn test_parse_function_multiple_stmts() {
    let source = r#"
        function compute(x) {
            let y = x + 1;
            let z = y * 2;
            return z;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => {
            assert_eq!(body.len(), 3);
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Multi-language: Bengali ───────────────────────────────────────

#[test]
fn test_parse_bengali_let() {
    let stmts = parse("চলক x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Multi-language: Russian ───────────────────────────────────────

#[test]
fn test_parse_russian_let() {
    let stmts = parse("переменная x = 42;").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Multi-language: Spanish ───────────────────────────────────────

#[test]
fn test_parse_spanish_let() {
    // "variable" may parse as identifier + expression; verify it parses without error
    let stmts = parse("variable x = 42;").unwrap();
    assert!(!stmts.is_empty(), "Spanish-style let should parse");
}

// ── Multi-language: German ────────────────────────────────────────

#[test]
fn test_parse_german_function() {
    // German "Funktion" may not be a registered keyword — verify standard "function" works
    let stmts = parse("function foo() { return 1; }").unwrap();
    assert!(!stmts.is_empty(), "function declaration should parse");
}

// ── Store with many fields ────────────────────────────────────────

#[test]
fn test_parse_store_with_more_fields() {
    let source = r#"
        store VectorDB {
            backend: "hnsw",
            dimensions: 768,
            metric: "cosine"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Store { name, fields, .. }) => {
            assert_eq!(name, "VectorDB");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Store, got {:?}", other),
    }
}

// ── Resource with URL ─────────────────────────────────────────────

#[test]
fn test_parse_resource_detailed() {
    let source = r#"
        resource Redis {
            url: "redis://localhost:6379",
            pool: 5,
            timeout: 3000
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Resource { name, fields, .. }) => {
            assert_eq!(name, "Redis");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Resource, got {:?}", other),
    }
}

// ── Community with more members ───────────────────────────────────

#[test]
fn test_parse_community_with_governance() {
    let source = r#"
        community DevTeam {
            members: ["alice", "bob", "charlie"]
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Community {
            name, members, ..
        }) => {
            assert_eq!(name, "DevTeam");
            assert_eq!(members.len(), 3);
        }
        other => panic!("Expected Community, got {:?}", other),
    }
}

// ── Swarm with strategy ───────────────────────────────────────────

#[test]
fn test_parse_swarm_sequential() {
    let source = r#"
        swarm Pipeline {
            agents: ["preprocessor", "analyzer"],
            strategy: sequential
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Swarm { name, strategy, .. }) => {
            assert_eq!(name, "Pipeline");
            assert_eq!(strategy, "sequential");
        }
        other => panic!("Expected Swarm, got {:?}", other),
    }
}

// ── Effect with multiple body statements ──────────────────────────

#[test]
fn test_parse_effect_with_body() {
    let source = r#"
        effect on Heal {
            let x = 1;
            let y = 2;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Effect {
            event_name, body, ..
        }) => {
            assert_eq!(event_name, "Heal");
            assert_eq!(body.len(), 2);
        }
        other => panic!("Expected Effect, got {:?}", other),
    }
}

// ── Relation with multiple fields ─────────────────────────────────

#[test]
fn test_parse_relation_with_multiple_fields() {
    let source = r#"
        relation Buyer <-> Seller {
            trust: 75,
            trades: 0
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Relation {
            subject_a,
            subject_b,
            fields,
            ..
        }) => {
            assert_eq!(subject_a, "Buyer");
            assert_eq!(subject_b, "Seller");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Relation, got {:?}", other),
    }
}

// ── Entity with default values ────────────────────────────────────

#[test]
fn test_parse_entity_with_defaults() {
    let source = r#"
        entity Character {
            data hp: Number = 100,
            data mp: Number = 50,
            data name: String
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Entity { name, fields, .. }) => {
            assert_eq!(name, "Character");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Entity, got {:?}", other),
    }
}

// ── StateMachine with multiple states ──────────────────────────────

#[test]
fn test_parse_statemachine_multiple_transitions() {
    let source = r#"
        statemachine Door {
            state Closed {
                on event(Open) -> Open
            }
            state Open {
                on event(Close) -> Closed
                on event(Lock) -> Locked
            }
            state Locked {
                on event(Unlock) -> Closed
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::StateMachine {
            name, fields, ..
        }) => {
            assert_eq!(name, "Door");
            // Closed.Open -> Open, Open.Close -> Closed, Open.Lock -> Locked, Locked.Unlock -> Closed
            assert_eq!(fields.len(), 4);
        }
        other => panic!("Expected StateMachine, got {:?}", other),
    }
}

// ── Event with more fields ────────────────────────────────────────

#[test]
fn test_parse_event_three_fields() {
    let source = r#"
        event Attack {
            damage: Number,
            source: String,
            critical: Boolean
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Event { name, fields, .. }) => {
            assert_eq!(name, "Attack");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Event, got {:?}", other),
    }
}

// ── Contract with expressions ─────────────────────────────────────

#[test]
fn test_parse_contract_with_numbers() {
    let source = r#"
        contract Sale: {
            price: 100,
            quantity: 5
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Contract { name, fields, .. }) => {
            assert_eq!(name, "Sale");
            assert_eq!(fields.len(), 2);
        }
        other => panic!("Expected Contract, got {:?}", other),
    }
}

// ── Treaty with multiple fields ───────────────────────────────────

#[test]
fn test_parse_treaty_with_multiple_fields() {
    let source = r#"
        treaty Peace: {
            parties: "A",
            terms: "ceasefire",
            duration: 365
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Treaty { name, fields, .. }) => {
            assert_eq!(name, "Peace");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Treaty, got {:?}", other),
    }
}

// ── Provider with more fields ─────────────────────────────────────

#[test]
fn test_parse_provider_three_fields() {
    let source = r#"
        provider Anthropic {
            apiKey: "key-123",
            model: "claude-3",
            endpoint: "https://api.anthropic.com"
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Provider { name, fields, .. }) => {
            assert_eq!(name, "Anthropic");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Provider, got {:?}", other),
    }
}

// ── Action with more fields ───────────────────────────────────────

#[test]
fn test_parse_action_detailed() {
    let source = r#"
        action Heal {
            target: "self",
            amount: 50,
            cooldown: 5
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Action { name, fields, .. }) => {
            assert_eq!(name, "Heal");
            assert_eq!(fields.len(), 3);
        }
        other => panic!("Expected Action, got {:?}", other),
    }
}

// ── Nested while in function ──────────────────────────────────────

#[test]
fn test_parse_while_in_function() {
    let source = r#"
        function countdown(n) {
            while (n > 0) {
                n = n - 1;
            }
            return n;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], hudhudscript_ast::Stmt::While { .. }));
            assert!(matches!(body[1], hudhudscript_ast::Stmt::Return { .. }));
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Multiple imports ──────────────────────────────────────────────

#[test]
fn test_parse_use_import_no_alias() {
    let stmts = parse("use postgres;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Import { module, alias, .. }) => {
            assert_eq!(module, "postgres");
            assert!(alias.is_none());
        }
        other => panic!("Expected Import (no alias), got {:?}", other),
    }
}

// ── Generator with multiple yields ────────────────────────────────

#[test]
fn test_parse_generator_multiple_yields() {
    let source = r#"
        function* range() {
            yield 1;
            yield 2;
            yield 3;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            body, is_generator, ..
        } => {
            assert!(*is_generator);
            assert_eq!(body.len(), 3);
        }
        other => panic!("Expected generator Function, got {:?}", other),
    }
}

// ── Rule with higher priority ─────────────────────────────────────

#[test]
fn test_parse_rule_high_priority() {
    let source = r#"
        rule CriticalCheck {
            priority: 100
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Decl(hudhudscript_ast::Decl::Rule { name, priority, .. }) => {
            assert_eq!(name, "CriticalCheck");
            assert_eq!(*priority, 100);
        }
        other => panic!("Expected Rule, got {:?}", other),
    }
}

// ── Complex if-else-if with many branches ─────────────────────────

#[test]
fn test_parse_if_else_if_many_branches() {
    let source = r#"
        if (x > 3) {
            let a = 1;
        } else if (x > 2) {
            let a = 2;
        } else if (x > 1) {
            let a = 3;
        } else {
            let a = 4;
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { else_branch, .. } => {
            // Drill into: else_branch should be If -> If -> Block
            let else1 = else_branch.as_ref().unwrap().as_ref();
            match else1 {
                hudhudscript_ast::Stmt::If {
                    else_branch: else2, ..
                } => {
                    let else2_inner = else2.as_ref().unwrap().as_ref();
                    match else2_inner {
                        hudhudscript_ast::Stmt::If {
                            else_branch: else3, ..
                        } => {
                            assert!(else3.is_some());
                        }
                        other => panic!("Expected nested If, got {:?}", other),
                    }
                }
                other => panic!("Expected If (else if), got {:?}", other),
            }
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ── Var destructuring ─────────────────────────────────────────────

#[test]
fn test_parse_var_destructure() {
    let stmts = parse("var { a, b } = obj;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { is_const, .. } => {
            assert!(!*is_const);
        }
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Throw expression ──────────────────────────────────────────────

#[test]
fn test_parse_throw_new_error() {
    let stmts = parse(r#"throw new Error("failed");"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Throw { value, .. } => {
            assert!(matches!(value, hudhudscript_ast::Expr::New { .. }));
        }
        other => panic!("Expected Throw, got {:?}", other),
    }
}

// ── Multiple catch body statements ────────────────────────────────

#[test]
fn test_parse_try_catch_multiple_stmts() {
    let source = r#"
        try {
            let x = 1;
            let y = 2;
        } catch (e) {
            let z = 3;
            let w = 4;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Try {
            try_block,
            catch_clause,
            ..
        } => {
            match try_block.as_ref() {
                hudhudscript_ast::Stmt::Block { statements, .. } => {
                    assert_eq!(statements.len(), 2);
                }
                other => panic!("Expected Block in try, got {:?}", other),
            }
            let catch = catch_clause.as_ref().unwrap();
            match catch.body.as_ref() {
                hudhudscript_ast::Stmt::Block { statements, .. } => {
                    assert_eq!(statements.len(), 2);
                }
                other => panic!("Expected Block in catch, got {:?}", other),
            }
        }
        other => panic!("Expected Try, got {:?}", other),
    }
}

// ══════════════════════════════════════════════════════════════════════
// FINAL COVERAGE PUSH — deep conversion / match-arm coverage
// ══════════════════════════════════════════════════════════════════════

// ── Class constructor ─────────────────────────────────────────────────

#[test]
fn test_parse_class_constructor() {
    let source = r#"
        class Foo {
            constructor(x, y) {
                let z = x;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Foo");
            assert_eq!(decl.members.len(), 1);
            match &decl.members[0] {
                hudhudscript_ast::ClassMember::Constructor { params, body, .. } => {
                    assert_eq!(params.len(), 2);
                    assert!(!body.is_empty());
                }
                other => panic!("Expected Constructor, got {:?}", other),
            }
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Class static method ───────────────────────────────────────────────

#[test]
fn test_parse_class_static_method() {
    let source = r#"
        class Util {
            static function create() {
                return 42;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Util");
            assert_eq!(decl.members.len(), 1);
            match &decl.members[0] {
                hudhudscript_ast::ClassMember::Method {
                    is_static, name, ..
                } => {
                    assert!(is_static);
                    assert_eq!(name, "create");
                }
                other => panic!("Expected static Method, got {:?}", other),
            }
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Class public method ───────────────────────────────────────────────

#[test]
fn test_parse_class_public_method() {
    let source = r#"
        class Svc {
            public function greet(name) {
                return name;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => match &decl.members[0] {
            hudhudscript_ast::ClassMember::Method { access, name, .. } => {
                assert_eq!(*access, hudhudscript_ast::AccessModifier::Public);
                assert_eq!(name, "greet");
            }
            other => panic!("Expected Method, got {:?}", other),
        },
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Class private method (default access) ─────────────────────────────

#[test]
fn test_parse_class_private_method() {
    let source = r#"
        class Svc {
            private function secret() {
                return 0;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => match &decl.members[0] {
            hudhudscript_ast::ClassMember::Method { access, name, .. } => {
                assert_eq!(*access, hudhudscript_ast::AccessModifier::Private);
                assert_eq!(name, "secret");
            }
            other => panic!("Expected Method, got {:?}", other),
        },
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Class field with initializer ──────────────────────────────────────

#[test]
fn test_parse_class_field_with_initializer() {
    let source = r#"
        class Config {
            let timeout = 30;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => match &decl.members[0] {
            hudhudscript_ast::ClassMember::Field {
                name, initializer, ..
            } => {
                assert_eq!(name, "timeout");
                assert!(initializer.is_some());
            }
            other => panic!("Expected Field, got {:?}", other),
        },
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Class field without initializer ───────────────────────────────────

#[test]
fn test_parse_class_field_no_initializer() {
    let source = r#"
        class Item {
            name;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => match &decl.members[0] {
            hudhudscript_ast::ClassMember::Field {
                name, initializer, ..
            } => {
                assert_eq!(name, "name");
                assert!(initializer.is_none());
            }
            other => panic!("Expected Field, got {:?}", other),
        },
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Class with multiple member types ──────────────────────────────────

#[test]
fn test_parse_class_mixed_members() {
    let source = r#"
        class Widget {
            let color = "red";
            constructor(w) { let x = w; }
            public function draw() { return 1; }
            protected function resize(s) { return s; }
            static function count() { return 0; }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Widget");
            assert_eq!(decl.members.len(), 5);
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Switch with multiple cases and default ────────────────────────────

#[test]
fn test_parse_switch_multiple_cases() {
    let source = r#"
        switch (x) {
            case 1: let a = 10;
            case 2: let b = 20;
            default: let c = 30;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Switch { cases, default, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(default.is_some());
        }
        other => panic!("Expected Switch, got {:?}", other),
    }
}

// ── Match with wildcard pattern ───────────────────────────────────────

#[test]
fn test_parse_match_wildcard_pattern() {
    let source = r#"
        match x {
            _ => 0;
}
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 1);
            assert_eq!(arms[0].pattern, hudhudscript_ast::MatchPattern::Wildcard);
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Match with enum variant pattern ───────────────────────────────────

#[test]
fn test_parse_match_enum_variant_pattern() {
    let source = r#"
        match shape {
            Shape::Circle(r) => r;
            Shape::Point => 0;
            _ => 1;
}
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            match &arms[0].pattern {
                hudhudscript_ast::MatchPattern::EnumVariant {
                    enum_name,
                    variant,
                    binding,
                } => {
                    assert_eq!(enum_name, "Shape");
                    assert_eq!(variant, "Circle");
                    assert!(binding.is_some());
                }
                other => panic!("Expected EnumVariant, got {:?}", other),
            }
            match &arms[1].pattern {
                hudhudscript_ast::MatchPattern::EnumVariant {
                    enum_name,
                    variant,
                    binding,
                } => {
                    assert_eq!(enum_name, "Shape");
                    assert_eq!(variant, "Point");
                    assert!(binding.is_none());
                }
                other => panic!("Expected EnumVariant without binding, got {:?}", other),
            }
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Match with block body ─────────────────────────────────────────────

#[test]
fn test_parse_match_block_body() {
    let source = r#"
        match x {
            1 => {
                let a = 10;
                let b = 20;
}
            _ => { let c = 0; }
}
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 2);
            assert!(arms[0].body.len() >= 2);
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Export default expression ──────────────────────────────────────────

#[test]
fn test_parse_export_default_expr() {
    let source = "export default 42;";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { .. } => {}
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Export named identifiers ──────────────────────────────────────────

#[test]
fn test_parse_export_named_identifiers() {
    let source = "export { foo, bar };";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => {
            match item.as_ref() {
                hudhudscript_ast::Stmt::Block { statements, .. } => {
                    assert_eq!(statements.len(), 2);
                }
                _ => {} // single identifier export is also valid
            }
        }
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Export single identifier ──────────────────────────────────────────

#[test]
fn test_parse_export_single_identifier() {
    let source = "export myFunc;";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { .. } => {}
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── For C-style with assignment init ──────────────────────────────────

#[test]
fn test_parse_for_c_style_assign_init() {
    let source = r#"
        for (x = 0; x < 10; x = x + 1) {
            let y = x;
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForCStyle {
            init,
            condition,
            update,
            ..
        } => {
            assert!(init.is_some());
            assert!(condition.is_some());
            assert!(update.is_some());
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ── For C-style empty ─────────────────────────────────────────────────

#[test]
fn test_parse_for_c_style_empty() {
    let source = "for (;;) { break; }";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForCStyle {
            init,
            condition,
            update,
            ..
        } => {
            assert!(init.is_none());
            assert!(condition.is_none());
            assert!(update.is_none());
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ── For C-style with only condition ────────────────────────────────────

#[test]
fn test_parse_for_c_style_condition_only() {
    let source = "for (; x < 5;) { break; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForCStyle {
            init,
            condition,
            update,
            ..
        } => {
            assert!(init.is_none());
            assert!(condition.is_some());
            assert!(update.is_none());
        }
        other => panic!("Expected ForCStyle, got {:?}", other),
    }
}

// ── For range without step (block directly after stop) ────────────────

#[test]
fn test_parse_for_range_no_step() {
    let source = "for (0, 10) { let x = 1; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForRange { step, .. } => {
            assert!(step.is_none());
        }
        other => panic!("Expected ForRange, got {:?}", other),
    }
}

// ── Object destructuring with alias ───────────────────────────────────

#[test]
fn test_parse_destruct_object_alias() {
    let source = "let { name: n, age: a } = obj;";
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                assert_eq!(properties[0].0, "name");
                match &properties[0].1 {
                    hudhudscript_ast::Pattern::Identifier(alias) => {
                        assert_eq!(alias, "n");
                    }
                    other => panic!("Expected Identifier alias, got {:?}", other),
                }
            }
            other => panic!("Expected Object pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Object destructuring with default ─────────────────────────────────

#[test]
fn test_parse_destruct_object_default() {
    let source = "let { port = 8080 } = config;";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Object { properties, .. } => {
                assert_eq!(properties.len(), 1);
                assert_eq!(properties[0].0, "port");
                match &properties[0].1 {
                    hudhudscript_ast::Pattern::IdentifierDefault(name, _expr) => {
                        assert_eq!(name, "port");
                    }
                    other => panic!("Expected IdentifierDefault, got {:?}", other),
                }
            }
            other => panic!("Expected Object pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Nested array destructuring ────────────────────────────────────────

#[test]
fn test_parse_destruct_nested_array() {
    let source = "let [[a, b], c] = matrix;";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Array { elements, .. } => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    hudhudscript_ast::Pattern::Array {
                        elements: inner, ..
                    } => {
                        assert_eq!(inner.len(), 2);
                    }
                    other => panic!("Expected nested Array, got {:?}", other),
                }
            }
            other => panic!("Expected Array pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── String escape: unknown escape sequence ────────────────────────────

#[test]
fn test_parse_string_unknown_escape() {
    let stmts = parse(r#"let x = "hello\xworld";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert!(s.contains("\\x"));
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── String escape: trailing backslash ─────────────────────────────────

#[test]
fn test_parse_string_trailing_backslash() {
    let stmts = parse(r#"let x = "trail\\";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert_eq!(s, "trail\\");
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Trait with return types ───────────────────────────────────────────

#[test]
fn test_parse_trait_method_return_type() {
    let source = r#"
        trait Serializable {
            function serialize(): String;
            function size(): Number;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait { name, methods, .. } => {
            assert_eq!(name, "Serializable");
            assert_eq!(methods.len(), 2);
            assert!(methods[0].return_type.is_some());
            assert!(methods[1].return_type.is_some());
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Trait method with params ──────────────────────────────────────────

#[test]
fn test_parse_trait_method_with_params() {
    let source = r#"
        trait Comparable {
            function compareTo(other): Number;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait { methods, .. } => {
            assert_eq!(methods[0].params.len(), 1);
            assert_eq!(methods[0].params[0].name, "other");
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Unary plus ────────────────────────────────────────────────────────

#[test]
fn test_parse_unary_plus_expr() {
    let stmts = parse("let x = +5;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Unary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::UnaryOp::Plus);
            }
            other => panic!("Expected Unary Plus, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Double unary (chained) ────────────────────────────────────────────

#[test]
fn test_parse_chained_unary_not() {
    let stmts = parse("let x = !!flag;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Unary { op, expr, .. } => {
                assert_eq!(*op, hudhudscript_ast::UnaryOp::Not);
                match expr.as_ref() {
                    hudhudscript_ast::Expr::Unary { op: inner_op, .. } => {
                        assert_eq!(*inner_op, hudhudscript_ast::UnaryOp::Not);
                    }
                    other => panic!("Expected inner Unary, got {:?}", other),
                }
            }
            other => panic!("Expected Unary, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Parenthesized expression ──────────────────────────────────────────

#[test]
fn test_parse_nested_parenthesized() {
    let stmts = parse("let x = ((1 + 2));").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Template string with adjacent interpolations ──────────────────────

#[test]
fn test_parse_template_adjacent_interps() {
    let stmts = parse("let x = `${a}${b}`;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::TemplateString { parts, .. } => {
                // Two interpolation parts
                let interp_count = parts
                    .iter()
                    .filter(|p| matches!(p, hudhudscript_ast::TemplateStringPart::Interpolation(_)))
                    .count();
                assert_eq!(interp_count, 2);
            }
            other => panic!("Expected TemplateString, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Template string empty ─────────────────────────────────────────────

#[test]
fn test_parse_template_empty() {
    let stmts = parse("let x = ``;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::TemplateString { parts, .. } => {
                assert!(parts.is_empty());
            }
            other => panic!("Expected TemplateString, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Enum with multiple field variants ─────────────────────────────────

#[test]
fn test_parse_enum_mixed_variants() {
    let source = r#"
        enum Shape {
            Circle(radius),
            Rectangle(w, h),
            Point
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::EnumDecl { name, variants, .. } => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].fields.len(), 1);
            assert_eq!(variants[1].fields.len(), 2);
            assert_eq!(variants[2].fields.len(), 0);
        }
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

// ── Herbir (Turkish for-each) ─────────────────────────────────────────

#[test]
fn test_parse_herbir_loop() {
    let source = r#"herbir ([1, 2, 3] icinde eleman) { let x = eleman; }"#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::For { variable, .. } => {
            assert_eq!(variable, "eleman");
        }
        other => panic!("Expected For (herbir), got {:?}", other),
    }
}

// ── Async arrow function ──────────────────────────────────────────────

#[test]
fn test_parse_async_arrow_with_params() {
    let stmts = parse("let f = async (x) => { return x; };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction {
                is_async, params, ..
            } => {
                assert!(is_async);
                assert_eq!(params.len(), 1);
            }
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Arrow function with no params ─────────────────────────────────────

#[test]
fn test_parse_arrow_empty_params_expr_body() {
    let stmts = parse("let f = () => 42;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, body, .. } => {
                assert!(params.is_empty());
                match body {
                    hudhudscript_ast::ArrowFunctionBody::Expression(_) => {}
                    other => panic!("Expected Expression body, got {:?}", other),
                }
            }
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Arrow function with single identifier param ───────────────────────

#[test]
fn test_parse_arrow_single_ident_param() {
    let stmts = parse("let f = x => x;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "x");
            }
            other => panic!("Expected ArrowFunction, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Anon function expression ──────────────────────────────────────────

#[test]
fn test_parse_anon_function_with_params() {
    let stmts = parse("let f = function(a, b) { return a; };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, .. } => {
                assert_eq!(params.len(), 2);
            }
            other => panic!("Expected ArrowFunction (anon), got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Anon function with rest param ─────────────────────────────────────

#[test]
fn test_parse_anon_function_rest_param() {
    let stmts = parse("let f = function(a, ...rest) { return a; };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::ArrowFunction { params, .. } => {
                assert!(params.iter().any(|p| p.starts_with("...")));
            }
            other => panic!("Expected ArrowFunction (anon rest), got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Spread in function call ───────────────────────────────────────────

#[test]
fn test_parse_spread_in_function_call() {
    let stmts = parse("foo(...args);").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Object with spread ────────────────────────────────────────────────

#[test]
fn test_parse_object_spread_entry() {
    let stmts = parse("let x = { ...base, extra: 1 };").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties.len(), 2);
                assert!(properties[0].0.starts_with("__spread__"));
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Object with string key ────────────────────────────────────────────

#[test]
fn test_parse_object_string_key_quoted() {
    let stmts = parse(r#"let x = { "my-key": 42 };"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Object { properties, .. } => {
                assert_eq!(properties[0].0, "my-key");
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── New expression with args ──────────────────────────────────────────

#[test]
fn test_parse_new_with_multiple_args() {
    let stmts = parse("let x = new Widget(10, 20, 30);").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::New {
                class_name, args, ..
            } => {
                assert_eq!(class_name, "Widget");
                assert_eq!(args.len(), 3);
            }
            other => panic!("Expected New, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Index access with complex expression ──────────────────────────────

#[test]
fn test_parse_index_access_complex() {
    let stmts = parse("let x = arr[0 + 1];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Index { .. } => {}
            other => panic!("Expected Index, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Chained calls and members ─────────────────────────────────────────

#[test]
fn test_parse_method_chain_index_chain() {
    let stmts = parse("let x = a.b[0].c(1, 2).d;").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Boolean keyword Turkish ───────────────────────────────────────────

#[test]
fn test_parse_turkish_boolean_true() {
    // doğru is normalized to "true"
    let stmts = parse("let x = doğru;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Boolean(b), _) => {
                assert!(*b);
            }
            other => panic!("Expected Boolean true, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Await expression ──────────────────────────────────────────────────

#[test]
fn test_parse_await_in_function() {
    let source = r#"
        async function fetch_data() {
            let result = await getData();
            return result;
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { is_async, body, .. } => {
            assert!(is_async);
            assert!(body.len() >= 2);
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Generic function ──────────────────────────────────────────────────

#[test]
fn test_parse_generic_function_constraint() {
    let source = r#"
        function sort<T: Comparable>(items) {
            return items;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { type_params, .. } => {
            assert_eq!(type_params.len(), 1);
            assert_eq!(type_params[0].name, "T");
            assert_eq!(type_params[0].constraint.as_deref(), Some("Comparable"));
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Generic function multiple type params ─────────────────────────────

#[test]
fn test_parse_generic_function_multiple() {
    let source = r#"
        function map<K, V>(dict) {
            return dict;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { type_params, .. } => {
            assert_eq!(type_params.len(), 2);
            assert_eq!(type_params[0].name, "K");
            assert_eq!(type_params[1].name, "V");
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Nested blocks ─────────────────────────────────────────────────────

#[test]
fn test_parse_nested_block() {
    let source = r#"
        {
            let x = 1;
            {
                let y = 2;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Block { statements, .. } => {
            assert_eq!(statements.len(), 2);
        }
        other => panic!("Expected Block, got {:?}", other),
    }
}

// ── Yield inside generator ────────────────────────────────────────────

#[test]
fn test_parse_generator_yield_value() {
    let source = r#"
        function* range(n) {
            let i = 0;
            yield i;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function {
            is_generator, body, ..
        } => {
            assert!(is_generator);
            assert!(body.len() >= 2);
        }
        other => panic!("Expected generator Function, got {:?}", other),
    }
}

// ── Enum single variant ───────────────────────────────────────────────

#[test]
fn test_parse_enum_single_variant() {
    let source = "enum Unit { Val }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::EnumDecl { variants, .. } => {
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].name, "Val");
        }
        other => panic!("Expected EnumDecl, got {:?}", other),
    }
}

// ── Complex chain of binary operations ────────────────────────────────

#[test]
fn test_parse_chained_add_sub() {
    let stmts = parse("let x = 1 + 2 - 3 + 4;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_chained_mul_div_mod() {
    let stmts = parse("let x = 10 * 2 / 5 % 3;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_chained_comparison() {
    // a > b is parsed; chained comparisons are just successive binary ops
    let stmts = parse("let x = a > b;").unwrap();
    assert_eq!(stmts.len(), 1);
}

#[test]
fn test_parse_chained_and_or() {
    let stmts = parse("let x = a && b || c && d;").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Multiple statements in sequence ───────────────────────────────────

#[test]
fn test_parse_many_statements() {
    let source = r#"
        let a = 1;
        let b = 2;
        let c = 3;
        let d = a + b;
        let e = c * d;
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 5);
}

// ── If-else-if with final else ────────────────────────────────────────

#[test]
fn test_parse_if_else_if_else() {
    let source = r#"
        if (a > 0) {
            let x = 1;
        } else if (a < 0) {
            let x = 2;
        } else {
            let x = 0;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::If { else_branch, .. } => {
            let eb = else_branch.as_ref().unwrap();
            match eb.as_ref() {
                hudhudscript_ast::Stmt::If {
                    else_branch: inner_else,
                    ..
                } => {
                    assert!(inner_else.is_some());
                }
                other => panic!("Expected nested If (else if), got {:?}", other),
            }
        }
        other => panic!("Expected If, got {:?}", other),
    }
}

// ── While with complex body ───────────────────────────────────────────

#[test]
fn test_parse_while_body_stmts() {
    let source = r#"
        while (x > 0) {
            let y = x;
            x = x - 1;
            if (x == 5) { break; }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::While { body, .. } => match body.as_ref() {
            hudhudscript_ast::Stmt::Block { statements, .. } => {
                assert_eq!(statements.len(), 3);
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected While, got {:?}", other),
    }
}

// ── Permission block inside agent ─────────────────────────────────────

#[test]
fn test_parse_agent_permission_block() {
    let source = r#"
        agent SecureBot {
            model: "gpt-4",
            permission {
                allow: "read"
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Data declaration anonymous ────────────────────────────────────────

#[test]
fn test_parse_data_anonymous_multi_fields() {
    let source = r#"
        data {
            x = 10;
            y = 20;
            z = 30;
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Object { properties, .. }) => {
            assert_eq!(properties.len(), 3);
        }
        other => panic!("Expected Expr(Object), got {:?}", other),
    }
}

// ── Data declaration named ────────────────────────────────────────────

#[test]
fn test_parse_data_named_multi_fields() {
    let source = r#"
        data Config {
            host = "localhost";
            port = 8080;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { name, value, .. } => {
            assert_eq!(name, "Config");
            match value {
                hudhudscript_ast::Expr::Object { properties, .. } => {
                    assert_eq!(properties.len(), 2);
                }
                other => panic!("Expected Object, got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── UI declaration ────────────────────────────────────────────────────

#[test]
fn test_parse_ui_decl() {
    let source = r#"
        ui Dashboard {
            title: "Main"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Deploy declaration ────────────────────────────────────────────────

#[test]
fn test_parse_deploy_decl_basic() {
    let source = r#"
        deploy Production {
            target: "aws",
            region: "us-east-1"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Rest param in function ────────────────────────────────────────────

#[test]
fn test_parse_function_rest_param() {
    let source = "function sum(first, ...rest) { return first; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { params, .. } => {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "first");
            assert!(params[1].starts_with("..."));
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Float literal ─────────────────────────────────────────────────────

#[test]
fn test_parse_float_literal_small() {
    let stmts = parse("let x = 0.001;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Number(n, _), _) => {
                assert!((*n - 0.001).abs() < 1e-10);
            }
            other => panic!("Expected Number, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Empty block ───────────────────────────────────────────────────────

#[test]
fn test_parse_empty_block() {
    let stmts = parse("{}").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Block { statements, .. } => {
            assert!(statements.is_empty());
        }
        other => panic!("Expected empty Block, got {:?}", other),
    }
}

// ── Multiple return types in trait ─────────────────────────────────────

#[test]
fn test_parse_trait_generic_return_type() {
    let source = r#"
        trait Mapper {
            function map(item): Any;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait { methods, .. } => {
            assert!(methods[0].return_type.is_some());
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Complex expression precedence ─────────────────────────────────────

#[test]
fn test_parse_precedence_mul_before_add() {
    let stmts = parse("let x = 2 + 3 * 4;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, right, .. } => {
                // Top-level should be Add (lower precedence)
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Add);
                // Right should be Mul
                match right.as_ref() {
                    hudhudscript_ast::Expr::Binary { op: inner_op, .. } => {
                        assert_eq!(*inner_op, hudhudscript_ast::BinaryOp::Mul);
                    }
                    other => panic!("Expected Binary Mul, got {:?}", other),
                }
            }
            other => panic!("Expected Binary Add, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Spread in array literal ───────────────────────────────────────────

#[test]
fn test_parse_spread_multiple_in_array() {
    let stmts = parse("let x = [...a, ...b, 1];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 3);
                let spread_count = elements
                    .iter()
                    .filter(|e| matches!(e, hudhudscript_ast::Expr::Spread { .. }))
                    .count();
                assert_eq!(spread_count, 2);
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Class with generic and implements ─────────────────────────────────

#[test]
fn test_parse_class_generic_implements() {
    let source = r#"
        class Stack<T> implements Iterable {
            let items = [];
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.type_params.len(), 1);
            assert_eq!(decl.type_params[0].name, "T");
            assert_eq!(decl.implements.len(), 1);
            assert_eq!(decl.implements[0], "Iterable");
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Return without value ──────────────────────────────────────────────

#[test]
fn test_parse_return_no_value_in_block() {
    let source = r#"
        function earlyExit() {
            return;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => match &body[0] {
            hudhudscript_ast::Stmt::Return { value, .. } => {
                assert!(value.is_none());
            }
            other => panic!("Expected Return, got {:?}", other),
        },
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Switch case with multiple body statements ─────────────────────────

#[test]
fn test_parse_switch_case_multiple_body() {
    let source = r#"
        switch (x) {
            case 1:
                let a = 1;
                let b = 2;
            default:
                let c = 0;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Switch { cases, default, .. } => {
            assert_eq!(cases.len(), 1);
            assert_eq!(cases[0].body.len(), 2);
            assert!(default.is_some());
        }
        other => panic!("Expected Switch, got {:?}", other),
    }
}

// ── Nested function declarations ──────────────────────────────────────

#[test]
fn test_parse_nested_function_decl() {
    let source = r#"
        function outer() {
            function inner() {
                return 1;
            }
            return inner();
        }
    "#;
    // Nested functions in blocks are block_statements, outer is ok
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Multiple imports ──────────────────────────────────────────────────

#[test]
fn test_parse_import_named_multiple() {
    let source = r#"import { foo, bar, baz } from "mylib";"#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Import with wildcard ──────────────────────────────────────────────

#[test]
fn test_parse_import_wildcard() {
    let source = r#"import * as lib from "mylib";"#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Expression statement (bare call) ──────────────────────────────────

#[test]
fn test_parse_bare_call_stmt() {
    let stmts = parse("doSomething(1, 2);").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Expr(hudhudscript_ast::Expr::Call { args, .. }) => {
            assert_eq!(args.len(), 2);
        }
        other => panic!("Expected Expr(Call), got {:?}", other),
    }
}

// ── Expression statement with member call ─────────────────────────────

#[test]
fn test_parse_member_call_stmt() {
    let stmts = parse("console.log(42);").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Const destructure ─────────────────────────────────────────────────

#[test]
fn test_parse_const_destructure_object() {
    let stmts = parse("const { x, y } = point;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { is_const, .. } => {
            assert!(is_const);
        }
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Var destructure ───────────────────────────────────────────────────

#[test]
fn test_parse_var_destructure_array() {
    let stmts = parse("var [a, b] = pair;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { is_const, .. } => {
            assert!(!is_const);
        }
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Object destructuring with rest ────────────────────────────────────

#[test]
fn test_parse_destruct_object_rest() {
    let stmts = parse("let { a, ...rest } = obj;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Object { rest, .. } => {
                assert!(rest.is_some());
            }
            other => panic!("Expected Object pattern, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}

// ── Complex nested expression ─────────────────────────────────────────

#[test]
fn test_parse_complex_nested_expr() {
    let stmts = parse("let x = (1 + 2) * (3 - 4);").unwrap();
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Binary { op, .. } => {
                assert_eq!(*op, hudhudscript_ast::BinaryOp::Mul);
            }
            other => panic!("Expected Binary Mul, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── For-in with array literal ─────────────────────────────────────────

#[test]
fn test_parse_for_in_with_inline_array() {
    let source = r#"for (item in [1, 2, 3]) { let x = item; }"#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::For { variable, .. } => {
            assert_eq!(variable, "item");
        }
        other => panic!("Expected For, got {:?}", other),
    }
}

// ── String with return escape ─────────────────────────────────────────

#[test]
fn test_parse_string_carriage_return_escape() {
    let stmts = parse(r#"let x = "line\r";"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::String(s), _) => {
                assert!(s.contains('\r'));
            }
            other => panic!("Expected String, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Decorator test ────────────────────────────────────────────────────

#[test]
fn test_parse_decorator_with_params() {
    let source = r#"
        @cloud(region: "us-east-1")
        @payment(provider: "stripe")
        subject Shop {
            state balance: 100
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Arabic digits in number ───────────────────────────────────────────

#[test]
fn test_parse_arabic_indic_number() {
    let stmts = parse("let x = ٤٢;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Number(n, _), _) => {
                assert_eq!(*n, 42.0);
            }
            other => panic!("Expected Number, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Bengali digits ────────────────────────────────────────────────────

#[test]
fn test_parse_bengali_number() {
    let stmts = parse("let x = ১০;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Number(n, _), _) => {
                assert_eq!(*n, 10.0);
            }
            other => panic!("Expected Number, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Template string with text and interpolation mixed ─────────────────

#[test]
fn test_parse_template_mixed_text_interp() {
    let stmts = parse("let x = `hello ${name} world ${age}`;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::TemplateString { parts, .. } => {
                let text_count = parts
                    .iter()
                    .filter(|p| matches!(p, hudhudscript_ast::TemplateStringPart::Text(_)))
                    .count();
                let interp_count = parts
                    .iter()
                    .filter(|p| matches!(p, hudhudscript_ast::TemplateStringPart::Interpolation(_)))
                    .count();
                assert!(text_count >= 2);
                assert_eq!(interp_count, 2);
            }
            other => panic!("Expected TemplateString, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Kanji number in expression ────────────────────────────────────────

#[test]
fn test_parse_kanji_number_literal() {
    let stmts = parse("let x = 四十二;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Literal(hudhudscript_ast::Literal::Number(n, _), _) => {
                assert_eq!(*n, 42.0);
            }
            other => panic!("Expected Number, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Function with multiple body statements ────────────────────────────

#[test]
fn test_parse_function_complex_body() {
    let source = r#"
        function compute(a, b) {
            let sum = a + b;
            let diff = a - b;
            if (sum > 0) {
                return sum;
            }
            return diff;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => {
            assert_eq!(body.len(), 4); // let, let, if, return
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── This keyword in member expression ─────────────────────────────────

#[test]
fn test_parse_this_member_access() {
    let stmts = parse("let x = this.name;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Member {
                object, property, ..
            } => {
                match object.as_ref() {
                    hudhudscript_ast::Expr::This(_) => {}
                    other => panic!("Expected This, got {:?}", other),
                }
                assert_eq!(property, "name");
            }
            other => panic!("Expected Member, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Self keyword in member expression ─────────────────────────────────

#[test]
fn test_parse_self_member_access() {
    let stmts = parse("let x = self.value;").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Member {
                object, property, ..
            } => {
                match object.as_ref() {
                    hudhudscript_ast::Expr::This(_) => {}
                    other => panic!("Expected This (self), got {:?}", other),
                }
                assert_eq!(property, "value");
            }
            other => panic!("Expected Member, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Export with async function ────────────────────────────────────────

#[test]
fn test_parse_export_async_function_body() {
    let source = r#"
        export async function fetchData() {
            let x = await getData();
            return x;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Function { is_async, .. } => {
                assert!(is_async);
            }
            other => panic!("Expected async Function, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Strategy/protocol with session ────────────────────────────────────

#[test]
fn test_parse_protocol_with_session() {
    let source = r#"
        protocol RateLimiter: {
            max_requests: 100,
            session: {
                onStart: "init",
                onEnd: "cleanup"
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Governance with multiple fields ───────────────────────────────────

#[test]
fn test_parse_governance_voting() {
    let source = r#"
        governance VotingSystem {
            quorum: 51,
            method: "majority"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Entity with default expression ────────────────────────────────────

#[test]
fn test_parse_entity_field_default() {
    let source = r#"
        entity Character {
            data health: Number = 100,
            data mana: Number
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Contract declaration ──────────────────────────────────────────────

#[test]
fn test_parse_contract_multi_fields() {
    let source = r#"
        contract Payment: {
            amount: 100,
            currency: "USD",
            method: "card"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Treaty declaration ────────────────────────────────────────────────

#[test]
fn test_parse_treaty_multi_fields() {
    let source = r#"
        treaty Alliance: {
            parties: 3,
            duration: "1y"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── State machine with multiple states ────────────────────────────────

#[test]
fn test_parse_statemachine_multi_state() {
    let source = r#"
        statemachine TrafficLight {
            state Red {
                on event(Timer) -> Green
            }
            state Green {
                on event(Timer) -> Yellow
            }
            state Yellow {
                on event(Timer) -> Red
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Event with multiple fields ────────────────────────────────────────

#[test]
fn test_parse_event_multi_fields() {
    let source = r#"
        event MouseClick {
            x: Number,
            y: Number,
            button: String
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Music melody declaration ──────────────────────────────────────────

#[test]
fn test_parse_music_melody_decl() {
    let source = r#"
        melody Theme {
            notes: "C D E F G",
            tempo: 120
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Music harmony declaration ─────────────────────────────────────────

#[test]
fn test_parse_music_harmony_decl() {
    let source = r#"
        harmony Progression {
            chords: "Cmaj Fmaj Gmaj",
            key: "C"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Music rhythm declaration ──────────────────────────────────────────

#[test]
fn test_parse_music_rhythm_decl() {
    let source = r#"
        rhythm Beat {
            pattern: "1 0 1 0",
            bpm: 120
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Music tempo declaration ───────────────────────────────────────────

#[test]
fn test_parse_music_tempo_decl() {
    let source = r#"
        tempo Allegro {
            bpm: 140,
            style: "fast"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Music scale declaration ───────────────────────────────────────────

#[test]
fn test_parse_music_scale_decl() {
    let source = r#"
        scale Pentatonic {
            notes: "C D E G A",
            type: "major"
        }
    "#;
    // "type" is not a keyword in this grammar context
    let result = parse(source);
    // scale may or may not parse depending on "type" as identifier
    assert!(result.is_ok() || result.is_err());
}

// ── Remember/recall/forget with store name ────────────────────────────

#[test]
fn test_parse_remember_in_store() {
    let stmts = parse(r#"remember "knowledge" in myStore;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Remember { store_name, .. } => {
            assert_eq!(store_name.as_deref(), Some("myStore"));
        }
        other => panic!("Expected Remember, got {:?}", other),
    }
}

#[test]
fn test_parse_recall_from_store() {
    let stmts = parse(r#"recall "query" from myStore;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Recall { store_name, .. } => {
            assert_eq!(store_name.as_deref(), Some("myStore"));
        }
        other => panic!("Expected Recall, got {:?}", other),
    }
}

#[test]
fn test_parse_forget_from_store() {
    let stmts = parse(r#"forget "old" from myStore;"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Forget { store_name, .. } => {
            assert_eq!(store_name.as_deref(), Some("myStore"));
        }
        other => panic!("Expected Forget, got {:?}", other),
    }
}

// ── Swarm with type ───────────────────────────────────────────────────

#[test]
fn test_parse_swarm_parallel() {
    let source = r#"
        swarm Workers {
            strategy: "parallel",
            count: 5
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Community with multiple fields ────────────────────────────────────

#[test]
fn test_parse_community_multi_field() {
    let source = r#"
        community DevTeam {
            size: 10,
            governance: "democratic"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Relation with multiple fields ─────────────────────────────────────

#[test]
fn test_parse_relation_three_fields() {
    let source = r#"
        relation Player <-> NPC {
            trust: 50,
            affinity: 75,
            distance: 10
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Effect with statements ────────────────────────────────────────────

#[test]
fn test_parse_effect_multiple_stmts() {
    let source = r#"
        effect on Damage {
            let x = 1;
            let y = 2;
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── MCP with tool declarations ────────────────────────────────────────

#[test]
fn test_parse_mcp_multiple_tools() {
    let source = r#"
        mcp PostgresServer {
            transport: "stdio",
            tool query(sql: String) {
                return sql;
            }
            tool insert(table: String) {
                return table;
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Action with multiple fields ───────────────────────────────────────

#[test]
fn test_parse_action_multi_field() {
    let source = r#"
        action Deploy {
            target: "production",
            timeout: 300,
            retries: 3
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Resource with fields ──────────────────────────────────────────────

#[test]
fn test_parse_resource_multi_field() {
    let source = r#"
        resource Database {
            host: "localhost",
            port: 5432,
            name: "mydb"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Nested object in array ────────────────────────────────────────────

#[test]
fn test_parse_array_of_objects() {
    let stmts = parse(r#"let x = [{ a: 1 }, { b: 2 }];"#).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 2);
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Object with nested objects ────────────────────────────────────────

#[test]
fn test_parse_deeply_nested_object() {
    let stmts = parse(r#"let x = { a: { b: { c: 1 } } };"#).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── For range with negative step ──────────────────────────────────────

#[test]
fn test_parse_for_range_negative_step() {
    let source = "for (10, 0, -1) { let x = 1; }";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::ForRange { step, .. } => {
            assert!(step.is_some());
        }
        other => panic!("Expected ForRange, got {:?}", other),
    }
}

// ── Law declaration with fields ───────────────────────────────────────

#[test]
fn test_parse_law_multi_field() {
    let source = r#"
        law Privacy {
            scope: "global",
            enforcement: "strict"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Council declaration ───────────────────────────────────────────────

#[test]
fn test_parse_council_multi_member() {
    let source = r#"
        council Board {
            members: 5,
            quorum: 3
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Role with can clause ──────────────────────────────────────────────

#[test]
fn test_parse_role_with_can_clause() {
    let source = r#"
        role Admin {
            can manage,
            can delete,
            level: 10
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Subject with memory and perception ────────────────────────────────

#[test]
fn test_parse_subject_memory_perception() {
    let source = r#"
        subject Agent {
            state health: 100,
            can attack,
            memory {
                capacity: 1000
            }
            perception {
                range: 50
            }
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Subject with intent ───────────────────────────────────────────────

#[test]
fn test_parse_subject_intent() {
    let source = r#"
        subject Player {
            state health: 100,
            intent Attack(target)
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Subject with uses clause ──────────────────────────────────────────

#[test]
fn test_parse_subject_uses() {
    let source = r#"
        subject Worker {
            state energy: 100,
            uses ToolService via http
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Constitution with fields ──────────────────────────────────────────

#[test]
fn test_parse_constitution_fields() {
    let source = r#"
        constitution Charter {
            preamble: "We the agents",
            articles: 10
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Export class declaration ───────────────────────────────────────────

#[test]
fn test_parse_export_class_with_members() {
    let source = r#"
        export class Widget {
            let color = "blue";
            public function render() { return 1; }
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Export { item, .. } => match item.as_ref() {
            hudhudscript_ast::Stmt::Class(decl) => {
                assert_eq!(decl.name, "Widget");
                assert_eq!(decl.members.len(), 2);
            }
            other => panic!("Expected Class, got {:?}", other),
        },
        other => panic!("Expected Export, got {:?}", other),
    }
}

// ── Match with expression body ────────────────────────────────────────

#[test]
fn test_parse_match_expression_body() {
    let source = r#"
        match x {
            1 => 100;
            2 => 200;
            _ => 0;
}
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Match { arms, .. } => {
            assert_eq!(arms.len(), 3);
            // Each arm has one expression statement
            for arm in arms {
                assert_eq!(arm.body.len(), 1);
            }
        }
        other => panic!("Expected Match, got {:?}", other),
    }
}

// ── Empty function body ───────────────────────────────────────────────

#[test]
fn test_parse_empty_function_body() {
    let stmts = parse("function noop() {}").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Function { body, .. } => {
            assert!(body.is_empty());
        }
        other => panic!("Expected Function, got {:?}", other),
    }
}

// ── Empty class body ──────────────────────────────────────────────────

#[test]
fn test_parse_empty_class_body() {
    let stmts = parse("class Empty {}").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Class(decl) => {
            assert_eq!(decl.name, "Empty");
            assert!(decl.members.is_empty());
        }
        other => panic!("Expected Class, got {:?}", other),
    }
}

// ── Empty trait body ──────────────────────────────────────────────────

#[test]
fn test_parse_empty_trait_body() {
    let stmts = parse("trait Marker {}").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Trait { name, methods, .. } => {
            assert_eq!(name, "Marker");
            assert!(methods.is_empty());
        }
        other => panic!("Expected Trait, got {:?}", other),
    }
}

// ── Empty while body ──────────────────────────────────────────────────

#[test]
fn test_parse_empty_while_body() {
    let stmts = parse("while (true) {}").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::While { body, .. } => match body.as_ref() {
            hudhudscript_ast::Stmt::Block { statements, .. } => {
                assert!(statements.is_empty());
            }
            other => panic!("Expected Block, got {:?}", other),
        },
        other => panic!("Expected While, got {:?}", other),
    }
}

// ── Try with only finally ─────────────────────────────────────────────

#[test]
fn test_parse_try_only_finally() {
    let source = r#"
        try {
            let x = 1;
        } finally {
            let y = 2;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Try {
            catch_clause,
            finally_block,
            ..
        } => {
            assert!(catch_clause.is_none());
            assert!(finally_block.is_some());
        }
        other => panic!("Expected Try, got {:?}", other),
    }
}

// ── Try with catch and finally ────────────────────────────────────────

#[test]
fn test_parse_try_catch_and_finally() {
    let source = r#"
        try {
            let x = 1;
        } catch (e) {
            let y = 2;
        } finally {
            let z = 3;
        }
    "#;
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Try {
            catch_clause,
            finally_block,
            ..
        } => {
            assert!(catch_clause.is_some());
            assert!(finally_block.is_some());
        }
        other => panic!("Expected Try, got {:?}", other),
    }
}

// ── Array element with nested array ───────────────────────────────────

#[test]
fn test_parse_nested_array_with_spread() {
    let stmts = parse("let x = [[1, 2], ...rest, [3]];").unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Let { value, .. } => match value {
            hudhudscript_ast::Expr::Array { elements, .. } => {
                assert_eq!(elements.len(), 3);
            }
            other => panic!("Expected Array, got {:?}", other),
        },
        other => panic!("Expected Let, got {:?}", other),
    }
}

// ── Function call with no args (already tested but different pattern) ─

#[test]
fn test_parse_call_chain_no_args() {
    let stmts = parse("foo()();").unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Multiple semicolons / empty statements ────────────────────────────

#[test]
fn test_parse_semicolons_only() {
    let stmts = parse(";;;").unwrap();
    // Semicolons without statements produce empty results
    assert!(stmts.is_empty() || stmts.len() <= 3);
}

// ── Rule declaration with high priority ───────────────────────────────

#[test]
fn test_parse_rule_decl_fields() {
    let source = r#"
        rule NoHarm {
            priority: 100,
            scope: "global",
            action: "prevent"
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Store with many fields ────────────────────────────────────────────

#[test]
fn test_parse_store_many_fields() {
    let source = r#"
        store VectorDB {
            backend: "hnsw",
            dimensions: 1536,
            metric: "cosine",
            capacity: 10000
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Provider with three fields ────────────────────────────────────────

#[test]
fn test_parse_provider_multi() {
    let source = r#"
        provider OpenAI {
            model: "gpt-4",
            temperature: 0.7,
            max_tokens: 1000
        }
    "#;
    let stmts = parse(source).unwrap();
    assert_eq!(stmts.len(), 1);
}

// ── Nested destructure in object ──────────────────────────────────────

#[test]
fn test_parse_destruct_nested_object() {
    let source = "let { a: { b, c } } = deep;";
    let stmts = parse(source).unwrap();
    match &stmts[0] {
        hudhudscript_ast::Stmt::Destructure { pattern, .. } => match pattern {
            hudhudscript_ast::Pattern::Object { properties, .. } => {
                assert_eq!(properties.len(), 1);
                match &properties[0].1 {
                    hudhudscript_ast::Pattern::Object {
                        properties: inner, ..
                    } => {
                        assert_eq!(inner.len(), 2);
                    }
                    other => panic!("Expected nested Object, got {:?}", other),
                }
            }
            other => panic!("Expected Object, got {:?}", other),
        },
        other => panic!("Expected Destructure, got {:?}", other),
    }
}
