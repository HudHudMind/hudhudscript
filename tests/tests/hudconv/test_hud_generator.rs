use hudconv::header_parser::{CppClass, CppEnum, CppField, CppFunction, CppHeader};
use hudconv::hud_generator::{
    cpp_type_to_hud, default_value_for_type, generate_agent, generate_enum, generate_hud,
    generate_subject, generate_tool, strip_member_prefix, to_snake_case,
};

#[test]
fn test_cpp_type_to_hud() {
    assert_eq!(cpp_type_to_hud("int"), "number");
    assert_eq!(cpp_type_to_hud("float"), "number");
    assert_eq!(cpp_type_to_hud("double"), "number");
    assert_eq!(cpp_type_to_hud("bool"), "boolean");
    assert_eq!(cpp_type_to_hud("std::string"), "text");
    assert_eq!(cpp_type_to_hud("const std::string&"), "text");
    assert_eq!(cpp_type_to_hud("void"), "void");
    assert_eq!(cpp_type_to_hud("MyClass"), "MyClass");
}

#[test]
fn test_strip_member_prefix() {
    assert_eq!(strip_member_prefix("m_value"), "value");
    assert_eq!(strip_member_prefix("_internal"), "internal");
    assert_eq!(strip_member_prefix("plain"), "plain");
}

#[test]
fn test_to_snake_case() {
    assert_eq!(to_snake_case("doSomething"), "do_something");
    assert_eq!(to_snake_case("getValue"), "get_value");
    assert_eq!(to_snake_case("simple"), "simple");
    assert_eq!(to_snake_case("XMLParser"), "x_m_l_parser");
}

#[test]
fn test_generate_enum() {
    let e = CppEnum {
        name: "Color".to_string(),
        variants: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        is_class: true,
    };
    let output = generate_enum(&e);
    assert!(output.contains("enum Color {"));
    assert!(output.contains("    Red,"));
    assert!(output.contains("    Green,"));
    assert!(output.contains("    Blue"));
}

#[test]
fn test_generate_subject() {
    let cls = CppClass {
        name: "Renderer".to_string(),
        parent: Some("Component".to_string()),
        is_struct: false,
        methods: vec![
            CppFunction {
                name: "render".to_string(),
                return_type: "void".to_string(),
                params: vec![("float".to_string(), "dt".to_string())],
                is_const: false,
                is_virtual: true,
                is_static: false,
            },
            CppFunction {
                name: "getInstance".to_string(),
                return_type: "Renderer*".to_string(),
                params: vec![],
                is_const: false,
                is_virtual: false,
                is_static: true,
            },
        ],
        fields: vec![CppField {
            name: "m_mode".to_string(),
            field_type: "int".to_string(),
            is_static: false,
            is_const: false,
        }],
    };

    let output = generate_subject(&cls, "renderer_lib");
    assert!(output.contains("@custom(native: \"renderer_lib\")"));
    assert!(output.contains("subject Renderer extends Component {"));
    assert!(output.contains("state mode: 0"));
    assert!(output.contains("can render(dt: number)"));
    assert!(output.contains("tool get_instance {"));
}

#[test]
fn test_generate_agent_from_struct() {
    let cls = CppClass {
        name: "Point".to_string(),
        parent: None,
        is_struct: true,
        methods: vec![],
        fields: vec![
            CppField {
                name: "x".to_string(),
                field_type: "double".to_string(),
                is_static: false,
                is_const: false,
            },
            CppField {
                name: "y".to_string(),
                field_type: "double".to_string(),
                is_static: false,
                is_const: false,
            },
        ],
    };

    let output = generate_agent(&cls);
    assert!(output.contains("agent Point {"));
    assert!(output.contains("state x: 0"));
    assert!(output.contains("state y: 0"));
}

#[test]
fn test_generate_tool() {
    let func = CppFunction {
        name: "initEngine".to_string(),
        return_type: "int".to_string(),
        params: vec![("const std::string&".to_string(), "config".to_string())],
        is_const: false,
        is_virtual: false,
        is_static: false,
    };

    let output = generate_tool(&func);
    assert!(output.contains("tool init_engine {"));
    assert!(output.contains("input config: text"));
    assert!(output.contains("output result: number"));
}

#[test]
fn test_default_value_for_type() {
    assert_eq!(default_value_for_type("number"), "0");
    assert_eq!(default_value_for_type("boolean"), "false");
    assert_eq!(default_value_for_type("text"), "\"\"");
    assert_eq!(default_value_for_type("CustomType"), "null");
}

#[test]
fn test_generate_tool_void_return() {
    let func = CppFunction {
        name: "shutdown".to_string(),
        return_type: "void".to_string(),
        params: vec![],
        is_const: false,
        is_virtual: false,
        is_static: false,
    };
    let output = generate_tool(&func);
    assert!(output.contains("tool shutdown {"));
    // Void return should not generate output line
    assert!(!output.contains("output result"));
}

#[test]
fn test_generate_tool_with_empty_param_name() {
    let func = CppFunction {
        name: "doIt".to_string(),
        return_type: "int".to_string(),
        params: vec![("int".to_string(), "".to_string())],
        is_const: false,
        is_virtual: false,
        is_static: false,
    };
    let output = generate_tool(&func);
    // Empty param name should be skipped
    assert!(!output.contains("input :"));
}

#[test]
fn test_generate_subject_no_parent() {
    let cls = CppClass {
        name: "Standalone".to_string(),
        parent: None,
        is_struct: false,
        methods: vec![CppFunction {
            name: "doWork".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_const: false,
            is_virtual: false,
            is_static: false,
        }],
        fields: vec![],
    };
    let output = generate_subject(&cls, "mylib");
    assert!(output.contains("subject Standalone {"));
    assert!(!output.contains("extends"));
    assert!(output.contains("can do_work"));
}

#[test]
fn test_generate_subject_method_no_params() {
    let cls = CppClass {
        name: "Foo".to_string(),
        parent: None,
        is_struct: false,
        methods: vec![CppFunction {
            name: "reset".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_const: false,
            is_virtual: false,
            is_static: false,
        }],
        fields: vec![],
    };
    let output = generate_subject(&cls, "mylib");
    assert!(output.contains("can reset\n"));
    // No parentheses for empty params
    assert!(!output.contains("can reset("));
}

#[test]
fn test_generate_hud_empty_header() {
    let header = CppHeader::default();
    let output = generate_hud(&header, "empty_lib");
    assert!(output.contains("// Auto-generated by hudconv"));
    assert!(output.contains("// Source library: empty_lib"));
}

#[test]
fn test_generate_agent_no_fields() {
    let cls = CppClass {
        name: "Empty".to_string(),
        parent: None,
        is_struct: true,
        methods: vec![],
        fields: vec![],
    };
    let output = generate_agent(&cls);
    assert!(output.contains("agent Empty {"));
    assert!(output.contains("}"));
}

#[test]
fn test_cpp_type_to_hud_more_types() {
    assert_eq!(cpp_type_to_hud("long"), "number");
    assert_eq!(cpp_type_to_hud("short"), "number");
    assert_eq!(cpp_type_to_hud("int32_t"), "number");
    assert_eq!(cpp_type_to_hud("int64_t"), "number");
    assert_eq!(cpp_type_to_hud("size_t"), "number");
    assert_eq!(cpp_type_to_hud("unsigned"), "number");
    assert_eq!(cpp_type_to_hud("string"), "text");
    assert_eq!(cpp_type_to_hud("char"), "text");
    assert_eq!(cpp_type_to_hud("int*"), "number");
}

#[test]
fn test_generate_hud_with_struct_and_class_and_free_fn() {
    let header = CppHeader {
        includes: vec![],
        namespaces: vec![],
        classes: vec![
            CppClass {
                name: "Renderer".to_string(),
                parent: None,
                is_struct: false,
                methods: vec![],
                fields: vec![],
            },
            CppClass {
                name: "Vertex".to_string(),
                parent: None,
                is_struct: true,
                methods: vec![],
                fields: vec![CppField {
                    name: "x".to_string(),
                    field_type: "float".to_string(),
                    is_static: false,
                    is_const: false,
                }],
            },
        ],
        functions: vec![CppFunction {
            name: "initSystem".to_string(),
            return_type: "bool".to_string(),
            params: vec![],
            is_const: false,
            is_virtual: false,
            is_static: false,
        }],
        enums: vec![],
    };
    let output = generate_hud(&header, "mylib");
    // Class produces subject, struct produces agent, free fn produces tool
    assert!(output.contains("subject Renderer"));
    assert!(output.contains("agent Vertex"));
    assert!(output.contains("tool init_system"));
    assert!(output.contains("output result: boolean"));
}

#[test]
fn test_generate_subject_method_with_empty_param_name_filtered() {
    let cls = CppClass {
        name: "Foo".to_string(),
        parent: None,
        is_struct: false,
        methods: vec![CppFunction {
            name: "process".to_string(),
            return_type: "void".to_string(),
            params: vec![
                ("int".to_string(), "".to_string()),
                ("float".to_string(), "dt".to_string()),
            ],
            is_const: false,
            is_virtual: false,
            is_static: false,
        }],
        fields: vec![],
    };
    let output = generate_subject(&cls, "mylib");
    // Empty param name should be filtered out from can declaration
    assert!(output.contains("can process(dt: number)"));
}

#[test]
fn test_generate_enum_single_variant() {
    let e = CppEnum {
        name: "Single".to_string(),
        variants: vec!["Only".to_string()],
        is_class: false,
    };
    let output = generate_enum(&e);
    assert!(output.contains("enum Single {"));
    // Single variant should not have trailing comma
    assert!(output.contains("    Only\n"));
    assert!(!output.contains("Only,"));
}

#[test]
fn test_generate_subject_separator_between_fields_and_methods() {
    let cls = CppClass {
        name: "Widget".to_string(),
        parent: None,
        is_struct: false,
        methods: vec![CppFunction {
            name: "draw".to_string(),
            return_type: "void".to_string(),
            params: vec![],
            is_const: false,
            is_virtual: false,
            is_static: false,
        }],
        fields: vec![CppField {
            name: "m_visible".to_string(),
            field_type: "bool".to_string(),
            is_static: false,
            is_const: false,
        }],
    };
    let output = generate_subject(&cls, "ui_lib");
    // There should be a blank line between state and can
    assert!(output.contains("state visible: false\n\n    can draw"));
}

#[test]
fn test_strip_member_prefix_underscore_single_char() {
    // Single underscore followed by one char
    assert_eq!(strip_member_prefix("_x"), "x");
}

#[test]
fn test_full_generation() {
    let header = CppHeader {
        includes: vec!["string".to_string()],
        namespaces: vec!["engine".to_string()],
        classes: vec![CppClass {
            name: "Engine".to_string(),
            parent: None,
            is_struct: false,
            methods: vec![CppFunction {
                name: "start".to_string(),
                return_type: "void".to_string(),
                params: vec![],
                is_const: false,
                is_virtual: false,
                is_static: false,
            }],
            fields: vec![CppField {
                name: "m_running".to_string(),
                field_type: "bool".to_string(),
                is_static: false,
                is_const: false,
            }],
        }],
        functions: vec![],
        enums: vec![],
    };

    let output = generate_hud(&header, "engine_lib");
    assert!(output.contains("// Auto-generated by hudconv"));
    assert!(output.contains("subject Engine {"));
    assert!(output.contains("state running: false"));
    assert!(output.contains("can start"));
}

#[test]
fn test_generate_enum_empty_variants() {
    let e = CppEnum {
        name: "Empty".to_string(),
        variants: vec![],
        is_class: false,
    };
    let output = generate_enum(&e);
    assert!(output.contains("enum Empty {"));
    assert!(output.contains("}"));
}

#[test]
fn test_generate_subject_multiple_static_methods() {
    let cls = CppClass {
        name: "Factory".to_string(),
        parent: None,
        is_struct: false,
        methods: vec![
            CppFunction {
                name: "createA".to_string(),
                return_type: "A*".to_string(),
                params: vec![("int".to_string(), "id".to_string())],
                is_const: false,
                is_virtual: false,
                is_static: true,
            },
            CppFunction {
                name: "createB".to_string(),
                return_type: "B*".to_string(),
                params: vec![],
                is_const: false,
                is_virtual: false,
                is_static: true,
            },
        ],
        fields: vec![],
    };
    let output = generate_subject(&cls, "factory_lib");
    assert!(output.contains("tool create_a {"));
    assert!(output.contains("input id: number"));
    assert!(output.contains("tool create_b {"));
}

#[test]
fn test_generate_subject_method_multiple_params() {
    let cls = CppClass {
        name: "Math".to_string(),
        parent: None,
        is_struct: false,
        methods: vec![CppFunction {
            name: "add".to_string(),
            return_type: "int".to_string(),
            params: vec![
                ("int".to_string(), "a".to_string()),
                ("int".to_string(), "b".to_string()),
            ],
            is_const: true,
            is_virtual: false,
            is_static: false,
        }],
        fields: vec![],
    };
    let output = generate_subject(&cls, "math_lib");
    assert!(output.contains("can add(a: number, b: number)"));
}

#[test]
fn test_generate_tool_multiple_params() {
    let func = CppFunction {
        name: "computeArea".to_string(),
        return_type: "double".to_string(),
        params: vec![
            ("double".to_string(), "width".to_string()),
            ("double".to_string(), "height".to_string()),
        ],
        is_const: false,
        is_virtual: false,
        is_static: false,
    };
    let output = generate_tool(&func);
    assert!(output.contains("tool compute_area {"));
    assert!(output.contains("input width: number"));
    assert!(output.contains("input height: number"));
    assert!(output.contains("output result: number"));
}

#[test]
fn test_strip_member_prefix_no_prefix() {
    assert_eq!(strip_member_prefix("value"), "value");
    assert_eq!(strip_member_prefix("count"), "count");
}

#[test]
fn test_strip_member_prefix_single_underscore() {
    // A bare underscore should not be stripped (len == 1)
    assert_eq!(strip_member_prefix("_"), "_");
}

#[test]
fn test_to_snake_case_all_lowercase() {
    assert_eq!(to_snake_case("lowercase"), "lowercase");
}

#[test]
fn test_to_snake_case_starts_with_uppercase() {
    assert_eq!(to_snake_case("StartHere"), "start_here");
}

#[test]
fn test_cpp_type_to_hud_const_pointer() {
    assert_eq!(cpp_type_to_hud("const int*"), "number");
    assert_eq!(cpp_type_to_hud("const char*"), "text");
}

#[test]
fn test_generate_agent_with_multiple_fields() {
    let cls = CppClass {
        name: "Config".to_string(),
        parent: None,
        is_struct: true,
        methods: vec![],
        fields: vec![
            CppField {
                name: "m_timeout".to_string(),
                field_type: "int".to_string(),
                is_static: false,
                is_const: false,
            },
            CppField {
                name: "m_verbose".to_string(),
                field_type: "bool".to_string(),
                is_static: false,
                is_const: false,
            },
            CppField {
                name: "m_name".to_string(),
                field_type: "std::string".to_string(),
                is_static: false,
                is_const: false,
            },
        ],
    };
    let output = generate_agent(&cls);
    assert!(output.contains("agent Config {"));
    assert!(output.contains("state timeout: 0"));
    assert!(output.contains("state verbose: false"));
    assert!(output.contains("state name: \"\""));
}

#[test]
fn test_generate_hud_with_enums() {
    let header = CppHeader {
        includes: vec![],
        namespaces: vec![],
        classes: vec![],
        functions: vec![],
        enums: vec![
            CppEnum {
                name: "Color".to_string(),
                variants: vec!["Red".to_string(), "Blue".to_string()],
                is_class: true,
            },
            CppEnum {
                name: "Size".to_string(),
                variants: vec!["Small".to_string(), "Large".to_string()],
                is_class: false,
            },
        ],
    };
    let output = generate_hud(&header, "mylib");
    assert!(output.contains("enum Color {"));
    assert!(output.contains("enum Size {"));
}

#[test]
fn test_default_value_for_unknown_type() {
    assert_eq!(default_value_for_type("SomeCustomType"), "null");
    assert_eq!(default_value_for_type("std::vector"), "null");
}
