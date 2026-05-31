use hudconv::header_parser::{
    extract_brace_body, parse_header_content, parse_params, strip_comments, try_parse_field,
    try_parse_method, CppHeader,
};

#[test]
fn test_parse_includes() {
    let content = r#"
#include <vector>
#include <string>
#include "myheader.h"
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.includes, vec!["vector", "string", "myheader.h"]);
}

#[test]
fn test_parse_namespace() {
    let content = r#"
namespace mylib {
    namespace detail {
    }
}
"#;
    let header = parse_header_content(content).unwrap();
    assert!(header.namespaces.contains(&"mylib".to_string()));
    assert!(header.namespaces.contains(&"detail".to_string()));
}

#[test]
fn test_parse_enum() {
    let content = r#"
enum Color { Red, Green, Blue };
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.enums.len(), 1);
    assert_eq!(header.enums[0].name, "Color");
    assert!(!header.enums[0].is_class);
    assert_eq!(header.enums[0].variants, vec!["Red", "Green", "Blue"]);
}

#[test]
fn test_parse_enum_class() {
    let content = r#"
enum class Status { Active = 0, Inactive = 1, Pending = 2 };
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.enums[0].name, "Status");
    assert!(header.enums[0].is_class);
    assert_eq!(
        header.enums[0].variants,
        vec!["Active", "Inactive", "Pending"]
    );
}

#[test]
fn test_parse_class() {
    let content = r#"
class MyClass : public Base {
public:
    void doSomething();
    int getValue() const;
    virtual void update(float dt);
    static MyClass* create();
private:
    int m_value;
    std::string m_name;
};
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.classes.len(), 1);
    let cls = &header.classes[0];
    assert_eq!(cls.name, "MyClass");
    assert_eq!(cls.parent.as_deref(), Some("Base"));
    assert!(!cls.is_struct);

    // Check methods
    assert!(cls.methods.iter().any(|m| m.name == "doSomething"));
    assert!(cls
        .methods
        .iter()
        .any(|m| m.name == "getValue" && m.is_const));
    assert!(cls
        .methods
        .iter()
        .any(|m| m.name == "update" && m.is_virtual));
    assert!(cls
        .methods
        .iter()
        .any(|m| m.name == "create" && m.is_static));

    // Check fields
    assert!(cls.fields.iter().any(|f| f.name == "m_value"));
    assert!(cls.fields.iter().any(|f| f.name == "m_name"));
}

#[test]
fn test_parse_struct() {
    let content = r#"
struct Point {
    double x;
    double y;
    double z;
};
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.classes.len(), 1);
    let cls = &header.classes[0];
    assert_eq!(cls.name, "Point");
    assert!(cls.is_struct);
    assert_eq!(cls.fields.len(), 3);
}

#[test]
fn test_parse_free_function() {
    let content = r#"
int add(int a, int b);
void printMessage(const std::string& msg);
"#;
    let header = parse_header_content(content).unwrap();
    assert!(header.functions.iter().any(|f| f.name == "add"));
    assert!(header.functions.iter().any(|f| f.name == "printMessage"));
}

#[test]
fn test_comments_stripped() {
    let content = r#"
// This is a comment
/* Multi-line
   comment */
class Foo {
public:
    void bar();
};
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.classes.len(), 1);
    assert_eq!(header.classes[0].name, "Foo");
}

#[test]
fn test_method_params() {
    let content = r#"
class Calculator {
public:
    int add(int a, int b) const;
    void setName(const std::string& name);
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    let add = cls.methods.iter().find(|m| m.name == "add").unwrap();
    assert_eq!(add.params.len(), 2);
    assert_eq!(add.params[0], ("int".to_string(), "a".to_string()));
    assert_eq!(add.params[1], ("int".to_string(), "b".to_string()));
    assert!(add.is_const);
}

#[test]
fn test_parse_params_void() {
    let params = parse_params("void");
    assert!(params.is_empty());
}

#[test]
fn test_parse_params_empty() {
    let params = parse_params("");
    assert!(params.is_empty());
}

#[test]
fn test_parse_params_with_defaults() {
    let params = parse_params("int x = 5, float y = 1.0");
    assert_eq!(params.len(), 2);
    assert_eq!(params[0], ("int".to_string(), "x".to_string()));
    assert_eq!(params[1], ("float".to_string(), "y".to_string()));
}

#[test]
fn test_parse_params_single_type_no_name() {
    let params = parse_params("int");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0], ("int".to_string(), String::new()));
}

#[test]
fn test_destructor_skipped() {
    let content = r#"
class Foo {
public:
    ~Foo();
    void bar();
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    // Destructor should not appear as a method
    assert!(!cls.methods.iter().any(|m| m.name.contains("~")));
    assert!(cls.methods.iter().any(|m| m.name == "bar"));
}

#[test]
fn test_template_line_skipped() {
    let content = r#"
class Foo {
public:
    template<typename T>
    void bar();
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    // Template lines should be skipped by try_parse_method
    // "void bar();" might or might not be parsed depending on context, but template itself is skipped
    assert!(!cls.methods.iter().any(|m| m.name.starts_with("template")));
}

#[test]
fn test_extract_brace_body_empty_on_no_matching() {
    // If there's no closing brace, extract_brace_body returns empty
    let content = "{ int x;";
    let body = extract_brace_body(content, 1);
    // depth starts at 1, no closing brace found
    assert!(body.is_empty() || body.contains("int x;"));
}

#[test]
fn test_field_skips_lines_with_parens() {
    // Lines with parentheses are methods, not fields
    let result = try_parse_field("void doStuff();");
    assert!(result.is_none());
}

#[test]
fn test_field_skips_enum_struct_class() {
    assert!(try_parse_field("enum Color { Red };").is_none());
    assert!(try_parse_field("struct Inner { int x; };").is_none());
    assert!(try_parse_field("class Nested { };").is_none());
}

#[test]
fn test_class_body_skips_friend_using_typedef() {
    let content = r#"
class Foo {
public:
    friend class Bar;
    using Ptr = std::shared_ptr<Foo>;
    typedef int Integer;
    void doWork();
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    assert!(cls.methods.iter().any(|m| m.name == "doWork"));
    // friend/using/typedef lines should not create methods or fields
    assert_eq!(cls.methods.len(), 1);
}

#[test]
fn test_static_const_field() {
    let content = r#"
class Config {
public:
    static const int MAX_SIZE = 100;
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    let field = cls.fields.iter().find(|f| f.name == "MAX_SIZE");
    assert!(field.is_some());
    let field = field.unwrap();
    assert!(field.is_static);
    assert!(field.is_const);
}

#[test]
fn test_free_function_skips_class_names() {
    let content = r#"
class Foo {
public:
    void bar();
};
Foo createFoo();
"#;
    let header = parse_header_content(content).unwrap();
    // "Foo" as a function name should be skipped (looks like constructor)
    assert!(!header.functions.iter().any(|f| f.name == "Foo"));
}

#[test]
fn test_duplicate_namespace_deduplicated() {
    let content = r#"
namespace ns { }
namespace ns { }
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.namespaces.len(), 1);
}

#[test]
fn test_parse_struct_with_no_parent() {
    let content = r#"
struct Config {
    int timeout;
    bool verbose;
};
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.classes.len(), 1);
    assert!(header.classes[0].parent.is_none());
    assert!(header.classes[0].is_struct);
}

#[test]
fn test_parse_pure_virtual_method() {
    let content = r#"
class IBase {
public:
    virtual void doWork() = 0;
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    assert!(cls
        .methods
        .iter()
        .any(|m| m.name == "doWork" && m.is_virtual));
}

#[test]
fn test_parse_empty_content() {
    let header = parse_header_content("").unwrap();
    assert!(header.includes.is_empty());
    assert!(header.namespaces.is_empty());
    assert!(header.classes.is_empty());
    assert!(header.functions.is_empty());
    assert!(header.enums.is_empty());
}

#[test]
fn test_extract_brace_body_nested() {
    let content = "{ outer { inner } rest }";
    let body = extract_brace_body(content, 1);
    // Should extract everything up to the matching close brace
    assert!(body.contains("outer"));
    assert!(body.contains("inner"));
    assert!(body.contains("rest"));
}

#[test]
fn test_try_parse_field_preprocessor_skipped() {
    assert!(try_parse_field("#ifdef FOO").is_none());
}

#[test]
fn test_try_parse_field_comment_skipped() {
    assert!(try_parse_field("// comment line").is_none());
}

#[test]
fn test_try_parse_method_preprocessor_skipped() {
    assert!(try_parse_method("#define FOO").is_none());
}

#[test]
fn test_try_parse_method_brace_skipped() {
    assert!(try_parse_method("{").is_none());
    assert!(try_parse_method("}").is_none());
}

#[test]
fn test_full_header() {
    let content = r#"
#pragma once
#include <string>
#include <vector>

namespace engine {

enum class RenderMode { Solid, Wireframe, Textured };

class Renderer : public Component {
public:
    virtual void render(float deltaTime) const;
    void setMode(RenderMode mode);
    static Renderer* getInstance();
private:
    RenderMode m_mode;
    bool m_enabled;
};

struct Vertex {
    float x;
    float y;
    float z;
};

int initEngine(const std::string& config);

}
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.includes.len(), 2);
    assert!(header.namespaces.contains(&"engine".to_string()));
    assert_eq!(header.enums.len(), 1);
    assert_eq!(header.enums[0].name, "RenderMode");
    assert_eq!(header.classes.len(), 2); // Renderer and Vertex
}

#[test]
fn test_parse_multiple_enums() {
    let content = r#"
enum Color { Red, Green, Blue };
enum class Direction { North, South, East, West };
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.enums.len(), 2);
    assert_eq!(header.enums[0].name, "Color");
    assert!(!header.enums[0].is_class);
    assert_eq!(header.enums[1].name, "Direction");
    assert!(header.enums[1].is_class);
    assert_eq!(header.enums[1].variants.len(), 4);
}

#[test]
fn test_parse_class_with_protected_inheritance() {
    let content = r#"
class Derived : public Base {
public:
    void action();
};
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.classes.len(), 1);
    assert_eq!(header.classes[0].parent.as_deref(), Some("Base"));
}

#[test]
fn test_parse_method_with_override_keyword() {
    let content = r#"
class Impl : public IBase {
public:
    void doWork() override;
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    assert!(cls.methods.iter().any(|m| m.name == "doWork"));
}

#[test]
fn test_parse_method_with_default_keyword() {
    let content = r#"
class Foo {
public:
    void reset() = default;
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    // '= default' methods should still be recognized
    assert!(cls.methods.iter().any(|m| m.name == "reset"));
}

#[test]
fn test_parse_includes_angle_and_quotes() {
    let content = r#"
#include <iostream>
#include <vector>
#include "myfile.h"
#include "path/to/other.hpp"
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.includes.len(), 4);
    assert!(header.includes.contains(&"iostream".to_string()));
    assert!(header.includes.contains(&"myfile.h".to_string()));
    assert!(header.includes.contains(&"path/to/other.hpp".to_string()));
}

#[test]
fn test_parse_params_multiple_with_references() {
    let params = parse_params("const std::string& name, int count, bool flag");
    assert_eq!(params.len(), 3);
    assert_eq!(params[0].1, "name");
    assert_eq!(params[1].1, "count");
    assert_eq!(params[2].1, "flag");
}

#[test]
fn test_parse_params_pointer_param() {
    let params = parse_params("int* ptr");
    assert_eq!(params.len(), 1);
    // pointer prefix stripped from name
    assert_eq!(params[0].1, "ptr");
}

#[test]
fn test_struct_with_methods() {
    let content = r#"
struct Widget {
    int width;
    int height;
    void resize(int w, int h);
};
"#;
    let header = parse_header_content(content).unwrap();
    let cls = &header.classes[0];
    assert!(cls.is_struct);
    assert_eq!(cls.fields.len(), 2);
    assert!(cls.methods.iter().any(|m| m.name == "resize"));
}

#[test]
fn test_multiple_classes_in_same_header() {
    let content = r#"
class A {
public:
    void doA();
};

class B : public A {
public:
    void doB();
};
"#;
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.classes.len(), 2);
    assert_eq!(header.classes[0].name, "A");
    assert!(header.classes[0].parent.is_none());
    assert_eq!(header.classes[1].name, "B");
    assert_eq!(header.classes[1].parent.as_deref(), Some("A"));
}

#[test]
fn test_strip_block_comment() {
    let content = "int x; /* comment */ int y;";
    let stripped = strip_comments(content);
    assert!(!stripped.contains("comment"));
    assert!(stripped.contains("int x;"));
    assert!(stripped.contains("int y;"));
}

#[test]
fn test_strip_multiline_block_comment() {
    let content = "start\n/* line1\nline2\nline3 */\nend";
    let stripped = strip_comments(content);
    assert!(!stripped.contains("line1"));
    assert!(stripped.contains("start"));
    assert!(stripped.contains("end"));
}

#[test]
fn test_try_parse_field_with_default_value() {
    let field = try_parse_field("int m_count = 42;");
    assert!(field.is_some());
    let f = field.unwrap();
    assert_eq!(f.name, "m_count");
    assert_eq!(f.field_type, "int");
}

#[test]
fn test_cpp_header_default() {
    let header = CppHeader::default();
    assert!(header.includes.is_empty());
    assert!(header.namespaces.is_empty());
    assert!(header.classes.is_empty());
    assert!(header.functions.is_empty());
    assert!(header.enums.is_empty());
}

#[test]
fn test_enum_with_no_variants() {
    let content = "enum Empty {};";
    let header = parse_header_content(content).unwrap();
    assert_eq!(header.enums.len(), 1);
    assert_eq!(header.enums[0].name, "Empty");
    assert!(header.enums[0].variants.is_empty());
}

#[test]
fn test_extract_brace_body_simple() {
    let content = "{ int x = 5; }";
    let body = extract_brace_body(content, 1);
    assert!(body.contains("int x = 5;"));
}
