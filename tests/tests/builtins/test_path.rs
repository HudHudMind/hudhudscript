use hudhudscript_bytecode::Value16;

fn path_join(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::path::dispatch("join", args)
}
fn path_parent(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::path::dispatch("parent", args)
}
fn path_filename(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::path::dispatch("filename", args)
}
fn path_extension(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::path::dispatch("extension", args)
}
fn path_is_absolute(args: &[Value16]) -> hudhudscript_errors::HudHudResult<Value16> {
    hudhudscript_shared_builtins::path::dispatch("is_absolute", args)
}

#[test]
fn test_path_join() {
    let result = path_join(&[
        Value16::string("/home".to_string()),
        Value16::string("user".to_string()),
        Value16::string("file.txt".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value16::string("/home/user/file.txt".to_string()));
}

#[test]
fn test_path_parent() {
    let result = path_parent(&[Value16::string("/home/user/file.txt".to_string())]).unwrap();
    assert_eq!(result, Value16::string("/home/user".to_string()));
}

#[test]
fn test_path_filename() {
    let result = path_filename(&[Value16::string("/home/user/file.txt".to_string())]).unwrap();
    assert_eq!(result, Value16::string("file.txt".to_string()));
}

#[test]
fn test_path_extension() {
    let result = path_extension(&[Value16::string("/home/user/file.txt".to_string())]).unwrap();
    assert_eq!(result, Value16::string("txt".to_string()));
}

#[test]
fn test_path_extension_none() {
    let result = path_extension(&[Value16::string("/home/user/Makefile".to_string())]).unwrap();
    assert_eq!(result, Value16::null());
}

#[test]
fn test_path_is_absolute() {
    let result = path_is_absolute(&[Value16::string("/home/user".to_string())]).unwrap();
    assert_eq!(result, Value16::boolean(true));

    let result = path_is_absolute(&[Value16::string("relative/path".to_string())]).unwrap();
    assert_eq!(result, Value16::boolean(false));
}
