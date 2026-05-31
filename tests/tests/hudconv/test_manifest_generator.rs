use hudconv::cmake_parser::CMakeProject;
use hudconv::cmake_parser::{CMakeTarget, TargetType};
use hudconv::conan_parser::ConanProject;
use hudconv::manifest_generator::{generate_build_script, generate_manifest, to_kebab_case};

#[test]
fn test_to_kebab_case() {
    assert_eq!(to_kebab_case("MyProject"), "my-project");
    assert_eq!(to_kebab_case("my_lib"), "my-lib");
    assert_eq!(to_kebab_case("simple"), "simple");
}

#[test]
fn test_generate_manifest_cmake_only() {
    let cmake = CMakeProject {
        name: "MyLib".to_string(),
        version: Some("1.2.3".to_string()),
        libraries: vec![CMakeTarget {
            name: "mylib".to_string(),
            target_type: TargetType::SharedLib,
            sources: vec![],
            link_libraries: vec![],
        }],
        executables: vec![],
        dependencies: vec![],
        include_dirs: vec![],
    };

    let manifest = generate_manifest(Some(&cmake), None);
    assert!(manifest.contains("name = \"my-lib\""));
    assert!(manifest.contains("version = \"1.2.3\""));
    assert!(manifest.contains("type = \"library\""));
    assert!(manifest.contains("[native-dependencies]"));
    assert!(manifest.contains("mylib = { path = \"./\", type = \"cmake\" }"));
}

#[test]
fn test_generate_manifest_conan_only() {
    let conan = ConanProject {
        name: "myproject".to_string(),
        version: "2.0.0".to_string(),
        requires: vec!["boost/1.84.0".to_string(), "fmt/10.2.0".to_string()],
        tool_requires: vec![],
        generators: vec![],
        options: vec![],
    };

    let manifest = generate_manifest(None, Some(&conan));
    assert!(manifest.contains("name = \"myproject\""));
    assert!(manifest.contains("version = \"2.0.0\""));
    assert!(manifest.contains("boost = { conan = \"boost/1.84.0\" }"));
    assert!(manifest.contains("fmt = { conan = \"fmt/10.2.0\" }"));
}

#[test]
fn test_generate_manifest_combined() {
    let cmake = CMakeProject {
        name: "MyApp".to_string(),
        version: Some("1.0.0".to_string()),
        libraries: vec![CMakeTarget {
            name: "mycore".to_string(),
            target_type: TargetType::SharedLib,
            sources: vec![],
            link_libraries: vec![],
        }],
        executables: vec![CMakeTarget {
            name: "myapp".to_string(),
            target_type: TargetType::Executable,
            sources: vec![],
            link_libraries: vec![],
        }],
        dependencies: vec![],
        include_dirs: vec![],
    };

    let conan = ConanProject {
        name: "myapp".to_string(),
        version: "1.0.0".to_string(),
        requires: vec!["openssl/3.2.0".to_string()],
        tool_requires: vec![],
        generators: vec![],
        options: vec![],
    };

    let manifest = generate_manifest(Some(&cmake), Some(&conan));
    assert!(manifest.contains("type = \"mixed\""));
    assert!(manifest.contains("openssl = { conan = \"openssl/3.2.0\" }"));
}

#[test]
fn test_generate_build_script_full() {
    let cmake = CMakeProject::default();
    let conan = ConanProject::default();

    let script = generate_build_script(Some(&cmake), Some(&conan));
    assert!(script.contains("#!/bin/bash"));
    assert!(script.contains("conan install . --output-folder=build --build=missing"));
    assert!(script.contains("cmake -B build -DCMAKE_TOOLCHAIN_FILE=build/conan_toolchain.cmake"));
    assert!(script.contains("cmake --build build"));
}

#[test]
fn test_generate_build_script_cmake_only() {
    let cmake = CMakeProject::default();

    let script = generate_build_script(Some(&cmake), None);
    assert!(script.contains("cmake -B build\n"));
    assert!(!script.contains("conan"));
}

#[test]
fn test_generate_build_script_conan_only() {
    let conan = ConanProject::default();

    let script = generate_build_script(None, Some(&conan));
    assert!(script.contains("conan install"));
    assert!(!script.contains("cmake"));
}

#[test]
fn test_generate_manifest_no_cmake_no_conan() {
    let manifest = generate_manifest(None, None);
    assert!(manifest.contains("name = \"converted-package\""));
    assert!(manifest.contains("version = \"0.1.0\""));
    assert!(manifest.contains("type = \"library\""));
}

#[test]
fn test_generate_manifest_executable_only() {
    let cmake = CMakeProject {
        name: "MyApp".to_string(),
        version: Some("1.0.0".to_string()),
        libraries: vec![],
        executables: vec![CMakeTarget {
            name: "myapp".to_string(),
            target_type: TargetType::Executable,
            sources: vec!["main.cpp".to_string()],
            link_libraries: vec![],
        }],
        dependencies: vec![],
        include_dirs: vec![],
    };
    let manifest = generate_manifest(Some(&cmake), None);
    assert!(manifest.contains("type = \"executable\""));
}

#[test]
fn test_generate_build_script_neither() {
    let script = generate_build_script(None, None);
    assert!(script.contains("#!/bin/bash"));
    assert!(!script.contains("conan"));
    assert!(!script.contains("cmake"));
}

#[test]
fn test_to_kebab_case_all_lowercase() {
    assert_eq!(to_kebab_case("alllowercase"), "alllowercase");
}

#[test]
fn test_to_kebab_case_consecutive_uppercase() {
    assert_eq!(to_kebab_case("XMLParser"), "x-m-l-parser");
}

#[test]
fn test_generate_manifest_cmake_no_version() {
    let cmake = CMakeProject {
        name: "NoVer".to_string(),
        version: None,
        libraries: vec![],
        executables: vec![],
        dependencies: vec![],
        include_dirs: vec![],
    };
    let manifest = generate_manifest(Some(&cmake), None);
    assert!(manifest.contains("version = \"0.1.0\""));
}

#[test]
fn test_generate_manifest_empty_conan_name() {
    let conan = ConanProject {
        name: "".to_string(),
        version: "".to_string(),
        requires: vec![],
        tool_requires: vec![],
        generators: vec![],
        options: vec![],
    };
    // When cmake is None and conan has empty name, falls back to conan name (empty)
    let manifest = generate_manifest(None, Some(&conan));
    // empty name from conan
    assert!(manifest.contains("name = \"\""));
}

#[test]
fn test_generate_build_script_empty() {
    let script = generate_build_script(None, None);
    assert!(script.starts_with("#!/bin/bash"));
    assert!(script.contains("set -euo pipefail"));
}

#[test]
fn test_to_kebab_case_underscores_and_uppercase() {
    assert_eq!(to_kebab_case("My_Project_Name"), "my--project--name");
}

#[test]
fn test_generate_manifest_conan_dep_without_version() {
    let conan = ConanProject {
        name: "test".to_string(),
        version: "1.0".to_string(),
        requires: vec!["singleword".to_string()],
        tool_requires: vec![],
        generators: vec![],
        options: vec![],
    };
    let manifest = generate_manifest(None, Some(&conan));
    // "singleword" without "/" should not produce a native-dependency entry
    assert!(!manifest.contains("singleword = {"));
}

#[test]
fn test_generate_manifest_cmake_library_only_is_library() {
    let cmake = CMakeProject {
        name: "LibOnly".to_string(),
        version: Some("2.0.0".to_string()),
        libraries: vec![CMakeTarget {
            name: "mylib".to_string(),
            target_type: TargetType::StaticLib,
            sources: vec!["src/lib.cpp".to_string()],
            link_libraries: vec![],
        }],
        executables: vec![],
        dependencies: vec![],
        include_dirs: vec![],
    };
    let manifest = generate_manifest(Some(&cmake), None);
    assert!(manifest.contains("type = \"library\""));
    assert!(manifest.contains("version = \"2.0.0\""));
}

#[test]
fn test_generate_manifest_cmake_empty_no_libs_no_exes() {
    let cmake = CMakeProject {
        name: "Empty".to_string(),
        version: None,
        libraries: vec![],
        executables: vec![],
        dependencies: vec![],
        include_dirs: vec![],
    };
    let manifest = generate_manifest(Some(&cmake), None);
    // No libraries, no executables -> "mixed" type (neither condition matches)
    assert!(manifest.contains("type = \"mixed\""));
    assert!(manifest.contains("name = \"empty\""));
}

#[test]
fn test_generate_manifest_conan_multiple_deps() {
    let conan = ConanProject {
        name: "multidep".to_string(),
        version: "1.0.0".to_string(),
        requires: vec![
            "boost/1.84.0".to_string(),
            "fmt/10.2.0".to_string(),
            "zlib/1.3.0".to_string(),
        ],
        tool_requires: vec![],
        generators: vec![],
        options: vec![],
    };
    let manifest = generate_manifest(None, Some(&conan));
    assert!(manifest.contains("boost = { conan = \"boost/1.84.0\" }"));
    assert!(manifest.contains("fmt = { conan = \"fmt/10.2.0\" }"));
    assert!(manifest.contains("zlib = { conan = \"zlib/1.3.0\" }"));
}

#[test]
fn test_generate_build_script_conan_then_cmake() {
    let cmake = CMakeProject::default();
    let conan = ConanProject::default();

    let script = generate_build_script(Some(&cmake), Some(&conan));
    // Conan install should come before cmake configure
    let conan_pos = script.find("conan install").unwrap();
    let cmake_pos = script.find("cmake -B build").unwrap();
    assert!(conan_pos < cmake_pos);
}

#[test]
fn test_to_kebab_case_empty_string() {
    assert_eq!(to_kebab_case(""), "");
}

#[test]
fn test_to_kebab_case_single_char() {
    assert_eq!(to_kebab_case("A"), "a");
    assert_eq!(to_kebab_case("a"), "a");
}

#[test]
fn test_generate_manifest_has_build_section() {
    let manifest = generate_manifest(None, None);
    assert!(manifest.contains("[build]"));
    assert!(manifest.contains("script = \"build.sh\""));
}

#[test]
fn test_generate_manifest_cmake_takes_priority_for_name() {
    let cmake = CMakeProject {
        name: "CmakeName".to_string(),
        version: Some("1.0.0".to_string()),
        libraries: vec![],
        executables: vec![],
        dependencies: vec![],
        include_dirs: vec![],
    };
    let conan = ConanProject {
        name: "conan_name".to_string(),
        version: "2.0.0".to_string(),
        requires: vec![],
        tool_requires: vec![],
        generators: vec![],
        options: vec![],
    };
    let manifest = generate_manifest(Some(&cmake), Some(&conan));
    // cmake name takes priority
    assert!(manifest.contains("name = \"cmake-name\""));
    // cmake version takes priority
    assert!(manifest.contains("version = \"1.0.0\""));
}

#[test]
fn test_generate_manifest_auto_generated_comment() {
    let manifest = generate_manifest(None, None);
    assert!(manifest.starts_with("# Auto-generated by hudconv"));
}

#[test]
fn test_generate_manifest_description_present() {
    let manifest = generate_manifest(None, None);
    assert!(manifest.contains("description = \"Converted from C++ project\""));
}
