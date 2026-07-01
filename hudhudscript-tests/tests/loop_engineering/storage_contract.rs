use hudhudscript_compiler::compiler::Compiler;
use hudhudscript_parser::parse;
use hudhudscript_vm::VM;
use std::path::{Path, PathBuf};

fn storage_guard_path() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent = manifest_dir
        .parent()
        .expect("test crate must have a parent directory");
    let relative = Path::new("samples/09-loop-engineering/05_shared_storage_guard.hud");
    let candidates = [
        parent.join("hudhud-script").join(relative),
        parent.join(relative),
    ];

    candidates
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| {
            panic!(
                "storage guard script not found; checked paths rooted at {}",
                parent.display()
            )
        })
}

fn execute_storage_guard() -> VM {
    let path = storage_guard_path();
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let statements = parse(&source).expect("storage guard must parse");
    let bytecode = Compiler::default()
        .compile(&statements)
        .expect("storage guard must compile");
    let mut vm = VM::new();
    vm.execute(&bytecode).expect("storage guard must execute");
    vm
}

#[test]
fn shared_loop_state_has_one_canonical_storage_home() {
    let vm = execute_storage_guard();

    assert_eq!(
        vm.get_variable("loop_storage_marker")
            .and_then(|value| value.as_bool()),
        Some(true),
        "canonical shared marker must be observable through get_variable",
    );
    assert_eq!(
        vm.get_variable("loop_storage_counter")
            .and_then(|value| value.as_int()),
        Some(1),
        "canonical shared counter must be observable through get_variable",
    );

    assert_eq!(
        vm.get_global("loop_storage_marker"),
        None,
        "shared marker must not be mirrored into the globals HashMap",
    );
    assert_eq!(
        vm.get_global("loop_storage_counter"),
        None,
        "shared counter must not be mirrored into the globals HashMap",
    );

    let ret = vm.last_return_value();
    let result = ret.as_object().expect("loop must return a result object");
    assert_eq!(
        result.get("success").and_then(|value| value.as_bool()),
        Some(true),
    );
    assert_eq!(
        result.get("status").and_then(|value| value.as_str()),
        Some("done"),
    );
}
