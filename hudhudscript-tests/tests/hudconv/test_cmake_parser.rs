use hudconv::cmake_parser::{
    extract_command_args, parse_cmake_content, split_cmake_args, strip_comments, CMakeProject,
    CMakeTarget, TargetType,
};

#[test]
fn test_parse_project_name_and_version() {
    let content = r#"
cmake_minimum_required(VERSION 3.20)
project(MyLib VERSION 2.1.0)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.name, "MyLib");
    assert_eq!(proj.version.as_deref(), Some("2.1.0"));
}

#[test]
fn test_parse_project_no_version() {
    let content = "project(Foo)\n";
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.name, "Foo");
    assert!(proj.version.is_none());
}

#[test]
fn test_parse_shared_library() {
    let content = r#"
project(Test)
add_library(mylib SHARED src/lib.cpp src/util.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
    assert_eq!(proj.libraries[0].name, "mylib");
    assert_eq!(proj.libraries[0].target_type, TargetType::SharedLib);
    assert_eq!(
        proj.libraries[0].sources,
        vec!["src/lib.cpp", "src/util.cpp"]
    );
}

#[test]
fn test_parse_static_library() {
    let content = r#"
project(Test)
add_library(mylib STATIC src/lib.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries[0].target_type, TargetType::StaticLib);
}

#[test]
fn test_parse_executable() {
    let content = r#"
project(Test)
add_executable(myapp main.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.executables.len(), 1);
    assert_eq!(proj.executables[0].name, "myapp");
    assert_eq!(proj.executables[0].sources, vec!["main.cpp"]);
}

#[test]
fn test_parse_link_libraries() {
    let content = r#"
project(Test)
add_executable(myapp main.cpp)
target_link_libraries(myapp PUBLIC Boost::boost fmt::fmt)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(
        proj.executables[0].link_libraries,
        vec!["Boost::boost", "fmt::fmt"]
    );
}

#[test]
fn test_parse_find_package() {
    let content = r#"
project(Test)
find_package(Boost REQUIRED)
find_package(fmt REQUIRED)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.dependencies.contains(&"Boost".to_string()));
    assert!(proj.dependencies.contains(&"fmt".to_string()));
}

#[test]
fn test_parse_include_directories() {
    let content = r#"
project(Test)
add_library(mylib STATIC src/lib.cpp)
target_include_directories(mylib PUBLIC include src)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.include_dirs.contains(&"include".to_string()));
    assert!(proj.include_dirs.contains(&"src".to_string()));
}

#[test]
fn test_parse_multiline_command() {
    let content = r#"
project(Test)
add_library(mylib SHARED
    src/a.cpp
    src/b.cpp
    src/c.cpp
)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries[0].sources.len(), 3);
}

#[test]
fn test_comments_stripped() {
    let content = r#"
# This is a comment
project(Test VERSION 1.0)
# add_library(fake SHARED nothing.cpp)
add_library(real STATIC src/real.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
    assert_eq!(proj.libraries[0].name, "real");
}

#[test]
fn test_comments_in_quoted_string_preserved() {
    let content = r#"
project(Test)
set(MY_VAR "path/with#hash/in/it")
add_library(mylib STATIC src/lib.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    // The library should be parsed despite the # in a quoted string
    assert_eq!(proj.libraries.len(), 1);
}

#[test]
fn test_default_library_type_is_static() {
    // No SHARED/STATIC keyword → defaults to StaticLib
    let content = r#"
project(Test)
add_library(mylib src/lib.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries[0].target_type, TargetType::StaticLib);
    assert_eq!(proj.libraries[0].sources, vec!["src/lib.cpp"]);
}

#[test]
fn test_interface_library() {
    let content = r#"
project(Test)
add_library(header_only INTERFACE)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
    assert_eq!(proj.libraries[0].name, "header_only");
    assert!(proj.libraries[0].sources.is_empty());
}

#[test]
fn test_link_libraries_unknown_target() {
    // When target_link_libraries references a target not yet defined,
    // the libs go to project-level dependencies
    let content = r#"
project(Test)
target_link_libraries(unknown_target PUBLIC libfoo libbar)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.dependencies.contains(&"libfoo".to_string()));
    assert!(proj.dependencies.contains(&"libbar".to_string()));
}

#[test]
fn test_plain_include_directories() {
    let content = r#"
project(Test)
include_directories(include third_party/headers)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.include_dirs.contains(&"include".to_string()));
    assert!(proj
        .include_dirs
        .contains(&"third_party/headers".to_string()));
}

#[test]
fn test_split_cmake_args_with_quotes() {
    let args = split_cmake_args(r#"mylib "path with spaces/lib.cpp" src/main.cpp"#);
    assert_eq!(args.len(), 3);
    assert_eq!(args[0], "mylib");
    assert_eq!(args[1], "path with spaces/lib.cpp");
    assert_eq!(args[2], "src/main.cpp");
}

#[test]
fn test_extract_command_args_nested_parens() {
    // Nested parentheses: e.g. if(...) inside cmake
    let content = "set(VAR \"a(b)c\")";
    let args = extract_command_args(content, "set");
    assert_eq!(args.len(), 1);
    assert!(args[0].contains("VAR"));
}

#[test]
fn test_extract_command_args_no_match() {
    let content = "project(Test)";
    let args = extract_command_args(content, "nonexistent_command");
    assert!(args.is_empty());
}

#[test]
fn test_extract_command_args_unclosed_paren() {
    // Unclosed paren should return no results (depth never reaches 0)
    let content = "add_library(mylib SHARED src/lib.cpp";
    let args = extract_command_args(content, "add_library");
    assert!(args.is_empty());
}

#[test]
fn test_split_cmake_args_empty_string() {
    let args = split_cmake_args("");
    assert!(args.is_empty());
}

#[test]
fn test_split_cmake_args_whitespace_only() {
    let args = split_cmake_args("   \n\t  ");
    assert!(args.is_empty());
}

#[test]
fn test_parse_library_object_type() {
    let content = r#"
project(Test)
add_library(mylib OBJECT src/lib.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
    // OBJECT type defaults to StaticLib internally
    assert_eq!(proj.libraries[0].target_type, TargetType::StaticLib);
    assert_eq!(proj.libraries[0].sources, vec!["src/lib.cpp"]);
}

#[test]
fn test_parse_library_module_type() {
    let content = r#"
project(Test)
add_library(myplugin MODULE src/plugin.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
    assert_eq!(proj.libraries[0].name, "myplugin");
    assert_eq!(proj.libraries[0].sources, vec!["src/plugin.cpp"]);
}

#[test]
fn test_parse_empty_add_library() {
    // add_library with no args after extraction should be skipped
    let content = r#"
project(Test)
add_library()
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.libraries.is_empty());
}

#[test]
fn test_parse_empty_add_executable() {
    let content = r#"
project(Test)
add_executable()
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.executables.is_empty());
}

#[test]
fn test_link_libraries_to_library_target() {
    // target_link_libraries should attach to library targets
    let content = r#"
project(Test)
add_library(mylib STATIC src/lib.cpp)
target_link_libraries(mylib PRIVATE zlib pthread)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries[0].link_libraries, vec!["zlib", "pthread"]);
}

#[test]
fn test_link_libraries_single_arg_skipped() {
    // target_link_libraries with only a target name (no libs) should be skipped
    let content = r#"
project(Test)
target_link_libraries(mylib)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.dependencies.is_empty());
}

#[test]
fn test_find_package_deduplication() {
    let content = r#"
project(Test)
find_package(Boost REQUIRED)
find_package(Boost REQUIRED)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(
        proj.dependencies.iter().filter(|d| *d == "Boost").count(),
        1
    );
}

#[test]
fn test_include_directories_system_keyword_skipped() {
    let content = r#"
project(Test)
include_directories(SYSTEM /usr/include)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.include_dirs.contains(&"/usr/include".to_string()));
    assert!(!proj.include_dirs.contains(&"SYSTEM".to_string()));
}

#[test]
fn test_parse_cmake_content_empty() {
    let proj = parse_cmake_content("").unwrap();
    assert_eq!(proj.name, "");
    assert!(proj.version.is_none());
    assert!(proj.libraries.is_empty());
    assert!(proj.executables.is_empty());
}

#[test]
fn test_case_insensitive_project() {
    let content = "PROJECT(MyLib VERSION 1.0)\n";
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.name, "MyLib");
    assert_eq!(proj.version.as_deref(), Some("1.0"));
}

#[test]
fn test_duplicate_include_dirs_deduplicated() {
    let content = r#"
project(Test)
add_library(mylib STATIC src/lib.cpp)
target_include_directories(mylib PUBLIC include)
include_directories(include)
"#;
    let proj = parse_cmake_content(content).unwrap();
    // "include" should appear only once
    assert_eq!(
        proj.include_dirs.iter().filter(|d| *d == "include").count(),
        1
    );
}

#[test]
fn test_full_cmake_project() {
    let content = r#"
cmake_minimum_required(VERSION 3.20)
project(MyProject VERSION 1.2.3)

find_package(Boost REQUIRED COMPONENTS system filesystem)
find_package(OpenSSL REQUIRED)

add_library(mycore SHARED
    src/core.cpp
    src/utils.cpp
)

add_executable(myapp
    src/main.cpp
)

target_link_libraries(mycore PUBLIC Boost::system Boost::filesystem)
target_link_libraries(myapp PRIVATE mycore OpenSSL::SSL)

target_include_directories(mycore PUBLIC include)
target_include_directories(myapp PRIVATE src)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.name, "MyProject");
    assert_eq!(proj.version.as_deref(), Some("1.2.3"));
    assert_eq!(proj.libraries.len(), 1);
    assert_eq!(proj.executables.len(), 1);
    assert!(proj.dependencies.contains(&"Boost".to_string()));
    assert!(proj.dependencies.contains(&"OpenSSL".to_string()));
    assert_eq!(
        proj.libraries[0].link_libraries,
        vec!["Boost::system", "Boost::filesystem"]
    );
    assert_eq!(
        proj.executables[0].link_libraries,
        vec!["mycore", "OpenSSL::SSL"]
    );
}

#[test]
fn test_multiple_libraries() {
    let content = r#"
project(Multi)
add_library(liba SHARED src/a.cpp)
add_library(libb STATIC src/b.cpp)
add_library(libc src/c.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 3);
    assert_eq!(proj.libraries[0].name, "liba");
    assert_eq!(proj.libraries[0].target_type, TargetType::SharedLib);
    assert_eq!(proj.libraries[1].name, "libb");
    assert_eq!(proj.libraries[1].target_type, TargetType::StaticLib);
    assert_eq!(proj.libraries[2].name, "libc");
    assert_eq!(proj.libraries[2].target_type, TargetType::StaticLib);
}

#[test]
fn test_multiple_executables() {
    let content = r#"
project(Test)
add_executable(app1 main1.cpp)
add_executable(app2 main2.cpp util.cpp)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.executables.len(), 2);
    assert_eq!(proj.executables[0].sources, vec!["main1.cpp"]);
    assert_eq!(proj.executables[1].sources, vec!["main2.cpp", "util.cpp"]);
}

#[test]
fn test_imported_library() {
    let content = r#"
project(Test)
add_library(ext IMPORTED)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
    assert_eq!(proj.libraries[0].name, "ext");
    assert!(proj.libraries[0].sources.is_empty());
}

#[test]
fn test_alias_library() {
    let content = r#"
project(Test)
add_library(MyLib::mylib ALIAS mylib_impl)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries.len(), 1);
}

#[test]
fn test_target_include_dirs_skips_keywords() {
    let content = r#"
project(Test)
add_library(mylib STATIC src/lib.cpp)
target_include_directories(mylib PUBLIC SYSTEM BEFORE include third_party)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj.include_dirs.contains(&"include".to_string()));
    assert!(proj.include_dirs.contains(&"third_party".to_string()));
    assert!(!proj.include_dirs.contains(&"SYSTEM".to_string()));
    assert!(!proj.include_dirs.contains(&"BEFORE".to_string()));
    assert!(!proj.include_dirs.contains(&"PUBLIC".to_string()));
}

#[test]
fn test_strip_comments_preserves_hash_in_quotes() {
    let content = r#"set(VAR "path/with#hash")"#;
    let stripped = strip_comments(content);
    assert!(stripped.contains("#hash"));
}

#[test]
fn test_strip_comments_removes_line_comments() {
    let content = "# full line comment\nreal_code()";
    let stripped = strip_comments(content);
    assert!(!stripped.contains("full line comment"));
    assert!(stripped.contains("real_code()"));
}

#[test]
fn test_extract_command_args_multiple_calls() {
    let content = r#"
find_package(Boost REQUIRED)
find_package(fmt CONFIG)
find_package(OpenSSL)
"#;
    let args = extract_command_args(content, "find_package");
    assert_eq!(args.len(), 3);
}

#[test]
fn test_split_cmake_args_tab_separated() {
    let args = split_cmake_args("a\tb\tc");
    assert_eq!(args, vec!["a", "b", "c"]);
}

#[test]
fn test_split_cmake_args_newline_separated() {
    let args = split_cmake_args("arg1\narg2\narg3");
    assert_eq!(args, vec!["arg1", "arg2", "arg3"]);
}

#[test]
fn test_cmake_project_default() {
    let proj = CMakeProject::default();
    assert_eq!(proj.name, "");
    assert!(proj.version.is_none());
    assert!(proj.libraries.is_empty());
    assert!(proj.executables.is_empty());
    assert!(proj.dependencies.is_empty());
    assert!(proj.include_dirs.is_empty());
}

#[test]
fn test_cmake_target_clone() {
    let target = CMakeTarget {
        name: "mylib".to_string(),
        target_type: TargetType::SharedLib,
        sources: vec!["src/a.cpp".to_string()],
        link_libraries: vec!["boost".to_string()],
    };
    let cloned = target.clone();
    assert_eq!(cloned.name, "mylib");
    assert_eq!(cloned.target_type, TargetType::SharedLib);
    assert_eq!(cloned.sources.len(), 1);
    assert_eq!(cloned.link_libraries.len(), 1);
}

#[test]
fn test_link_libraries_private_keyword_filtered() {
    let content = r#"
project(Test)
add_library(mylib STATIC src/lib.cpp)
target_link_libraries(mylib PRIVATE INTERFACE PUBLIC zlib)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert_eq!(proj.libraries[0].link_libraries, vec!["zlib"]);
}

#[test]
fn test_include_directories_after_keyword_skipped() {
    let content = r#"
project(Test)
include_directories(AFTER /usr/local/include)
"#;
    let proj = parse_cmake_content(content).unwrap();
    assert!(proj
        .include_dirs
        .contains(&"/usr/local/include".to_string()));
    assert!(!proj.include_dirs.contains(&"AFTER".to_string()));
}
