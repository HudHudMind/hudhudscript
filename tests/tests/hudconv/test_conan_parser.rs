use hudconv::conan_parser::{extract_quoted_strings, parse_conan_content, ConanProject};

#[test]
fn test_parse_name_and_version() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.2.3"
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.name, "mypkg");
    assert_eq!(proj.version, "1.2.3");
}

#[test]
fn test_parse_requires_tuple() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    requires = ("boost/1.84.0", "fmt/10.2.0")
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.requires, vec!["boost/1.84.0", "fmt/10.2.0"]);
}

#[test]
fn test_parse_requires_single() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    requires = "boost/1.84.0"
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.requires, vec!["boost/1.84.0"]);
}

#[test]
fn test_parse_tool_requires() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    tool_requires = ("cmake/3.28", "ninja/1.11")
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.tool_requires, vec!["cmake/3.28", "ninja/1.11"]);
}

#[test]
fn test_parse_generators() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    generators = "CMakeDeps", "CMakeToolchain"
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.generators, vec!["CMakeDeps", "CMakeToolchain"]);
}

#[test]
fn test_parse_options() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    options = {"shared": [True, False], "fPIC": [True, False]}
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.options.len(), 2);
    assert_eq!(proj.options[0].0, "shared");
    assert_eq!(proj.options[0].1, vec!["True", "False"]);
}

#[test]
fn test_parse_self_requires() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"

    def requirements(self):
        self.requires("boost/1.84.0")
        self.requires("fmt/10.2.0")
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.requires, vec!["boost/1.84.0", "fmt/10.2.0"]);
}

#[test]
fn test_parse_self_tool_requires() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"

    def build_requirements(self):
        self.tool_requires("cmake/3.28")
        self.tool_requires("ninja/1.11")
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.tool_requires, vec!["cmake/3.28", "ninja/1.11"]);
}

#[test]
fn test_parse_requires_list_syntax() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    requires = ["boost/1.84.0", "fmt/10.2.0"]
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.requires, vec!["boost/1.84.0", "fmt/10.2.0"]);
}

#[test]
fn test_parse_empty_conanfile() {
    let content = r#"
class MyPkg(ConanFile):
    pass
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.name, "");
    assert_eq!(proj.version, "");
    assert!(proj.requires.is_empty());
}

#[test]
fn test_parse_name_with_single_quotes() {
    let content = r#"
class MyPkg(ConanFile):
    name = 'mypkg'
    version = '2.0'
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.name, "mypkg");
    assert_eq!(proj.version, "2.0");
}

#[test]
fn test_extract_quoted_strings_mixed_quotes() {
    let result = extract_quoted_strings(r#"("foo", 'bar', "baz")"#);
    assert_eq!(result, vec!["foo", "bar", "baz"]);
}

#[test]
fn test_parse_options_empty() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
"#;
    let proj = parse_conan_content(content).unwrap();
    assert!(proj.options.is_empty());
}

#[test]
fn test_parse_generators_empty() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
"#;
    let proj = parse_conan_content(content).unwrap();
    assert!(proj.generators.is_empty());
}

#[test]
fn test_parse_no_duplicate_self_tool_requires() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    tool_requires = ("cmake/3.28",)

    def build_requirements(self):
        self.tool_requires("cmake/3.28")
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(
        proj.tool_requires
            .iter()
            .filter(|r| *r == "cmake/3.28")
            .count(),
        1
    );
}

#[test]
fn test_parse_no_duplicate_self_requires() {
    let content = r#"
class MyPkg(ConanFile):
    name = "mypkg"
    version = "1.0"
    requires = ("boost/1.84.0",)

    def requirements(self):
        self.requires("boost/1.84.0")
"#;
    let proj = parse_conan_content(content).unwrap();
    // Should not duplicate
    assert_eq!(
        proj.requires
            .iter()
            .filter(|r| *r == "boost/1.84.0")
            .count(),
        1
    );
}

#[test]
fn test_full_conanfile() {
    let content = r#"
from conan import ConanFile
from conan.tools.cmake import CMake, CMakeDeps, CMakeToolchain

class MyProjectConan(ConanFile):
    name = "myproject"
    version = "2.0.0"
    requires = ("boost/1.84.0", "openssl/3.2.0", "fmt/10.2.0")
    tool_requires = ("cmake/3.28",)
    generators = "CMakeDeps", "CMakeToolchain"
    options = {"shared": [True, False], "fPIC": [True, False]}
    default_options = {"shared": True, "fPIC": True}

    def build(self):
        cmake = CMake(self)
        cmake.configure()
        cmake.build()
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.name, "myproject");
    assert_eq!(proj.version, "2.0.0");
    assert_eq!(proj.requires.len(), 3);
    assert_eq!(proj.tool_requires, vec!["cmake/3.28"]);
    assert_eq!(proj.generators, vec!["CMakeDeps", "CMakeToolchain"]);
    assert_eq!(proj.options.len(), 2);
}

#[test]
fn test_parse_conan_content_empty_string() {
    let proj = parse_conan_content("").unwrap();
    assert_eq!(proj.name, "");
    assert_eq!(proj.version, "");
    assert!(proj.requires.is_empty());
    assert!(proj.tool_requires.is_empty());
    assert!(proj.generators.is_empty());
    assert!(proj.options.is_empty());
}

#[test]
fn test_conan_project_default() {
    let proj = ConanProject::default();
    assert_eq!(proj.name, "");
    assert_eq!(proj.version, "");
    assert!(proj.requires.is_empty());
    assert!(proj.tool_requires.is_empty());
    assert!(proj.generators.is_empty());
    assert!(proj.options.is_empty());
}

#[test]
fn test_extract_quoted_strings_empty() {
    let result = extract_quoted_strings("no quotes here");
    assert!(result.is_empty());
}

#[test]
fn test_extract_quoted_strings_single() {
    let result = extract_quoted_strings(r#""only_one""#);
    assert_eq!(result, vec!["only_one"]);
}

#[test]
fn test_parse_options_single_option() {
    let content = r#"
class Pkg(ConanFile):
    name = "pkg"
    version = "1.0"
    options = {"shared": [True, False]}
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.options.len(), 1);
    assert_eq!(proj.options[0].0, "shared");
    assert_eq!(proj.options[0].1, vec!["True", "False"]);
}

#[test]
fn test_parse_requires_mixed_assign_and_method() {
    let content = r#"
class Pkg(ConanFile):
    name = "pkg"
    version = "1.0"
    requires = ("boost/1.84.0",)

    def requirements(self):
        self.requires("fmt/10.2.0")
"#;
    let proj = parse_conan_content(content).unwrap();
    assert!(proj.requires.contains(&"boost/1.84.0".to_string()));
    assert!(proj.requires.contains(&"fmt/10.2.0".to_string()));
}

#[test]
fn test_conan_project_clone() {
    let proj = ConanProject {
        name: "test".to_string(),
        version: "1.0".to_string(),
        requires: vec!["dep/1.0".to_string()],
        tool_requires: vec!["tool/2.0".to_string()],
        generators: vec!["CMakeDeps".to_string()],
        options: vec![("shared".to_string(), vec!["True".to_string()])],
    };
    let cloned = proj.clone();
    assert_eq!(cloned.name, "test");
    assert_eq!(cloned.requires.len(), 1);
    assert_eq!(cloned.tool_requires.len(), 1);
    assert_eq!(cloned.generators.len(), 1);
    assert_eq!(cloned.options.len(), 1);
}

#[test]
fn test_parse_generators_single() {
    let content = r#"
class Pkg(ConanFile):
    name = "pkg"
    version = "1.0"
    generators = "CMakeToolchain"
"#;
    let proj = parse_conan_content(content).unwrap();
    assert_eq!(proj.generators, vec!["CMakeToolchain"]);
}
