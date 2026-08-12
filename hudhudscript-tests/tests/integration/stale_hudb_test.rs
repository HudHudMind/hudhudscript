use hudhud_script_tests::vm_interpreter::Interpreter;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_files(dir: &PathBuf) {
    // providers.hudhud
    let providers_src = r#"
provider MockAI {
    type: "mock"
}
"#;
    fs::write(dir.join("providers.hudhud"), providers_src).unwrap();

    // providers.hudb (stale/corrupt)
    let corrupt_bytecode = vec![0xBA, 0xDF, 0x00, 0xD5, 0x11, 0x22];
    fs::write(dir.join("providers.hudb"), corrupt_bytecode).unwrap();

    let agents_src = r#"
use "providers.hudhud";

agent DeepSeekv4Pro {
    provider: MockAI
    model: "deepseek-pro"

    action translate(text) {
        return "translated pro: " + text;
    }
}

agent DeepSeekv4Flash {
    provider: MockAI
    model: "deepseek-flash"

    action translate(text) {
        return "translated flash: " + text;
    }
}

agent MetinYazari {
    provider: MockAI
    model: "mock"

    action slogan_yaz(urun_adi) {
        print(urun_adi + " icin slogan yaziliyor");
        return urun_adi + " icin slogan";
    }
}

agent MantikAnalisti {
    provider: MockAI
    model: "mock"

    action veriyi_onayla(veri_metni) {
        print("Veri analiz ediliyor");
        return "onaylandi: " + veri_metni;
    }
}
"#;
    fs::write(dir.join("agents.hudhud"), agents_src).unwrap();
}

#[tokio::test]
async fn test_source_import_ignores_stale_hudb() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);

    // Main script reading providers.hudhud explicitly
    let main_src = r#"
use "providers.hudhud" as providers;

let ok = 123;
"#;
    let main_path = dir.join("main.hudhud");
    fs::write(&main_path, main_src).unwrap();

    let mut interpreter = Interpreter::new();
    let ast = hudhudscript_parser::parse(main_src).unwrap();

    // Use the compiler directly to set base dir so the VM knows where to load modules
    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();

    let ok_val = interpreter.vm.get_variable("ok").unwrap();
    assert_eq!(ok_val.as_int().unwrap(), 123);
}

#[tokio::test]
async fn test_extensionless_import_ignores_stale_hudb() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);

    // Main script reading providers without extension
    let main_src = r#"
use providers;

let ok = 123;
"#;
    let main_path = dir.join("main2.hudhud");
    fs::write(&main_path, main_src).unwrap();

    let mut interpreter = Interpreter::new();
    let ast = hudhudscript_parser::parse(main_src).unwrap();

    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();

    let ok_val = interpreter.vm.get_variable("ok").unwrap();
    assert_eq!(ok_val.as_int().unwrap(), 123);
}

#[tokio::test]
async fn test_registry_merge_only() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);

    let main_src = r#"
use "agents.hudhud" as agents;

let writer_name = agents.MetinYazari.name;
let checker_name = agents.MantikAnalisti.name;
"#;
    let main_path = dir.join("author_merge.hudhud");
    fs::write(&main_path, main_src).unwrap();

    let mut interpreter = Interpreter::new();
    let ast = hudhudscript_parser::parse(main_src).unwrap();

    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();

    let writer_name = interpreter.vm.get_variable("writer_name").unwrap();
    assert_eq!(writer_name.as_str().unwrap(), "MetinYazari");

    let checker_name = interpreter.vm.get_variable("checker_name").unwrap();
    assert_eq!(checker_name.as_str().unwrap(), "MantikAnalisti");

    // REGRESSION 1: agents object MUST contain MantikAnalisti
    let agents_obj = interpreter.vm.get_variable("agents").unwrap();
    let agents_map = agents_obj.as_object().unwrap();
    assert!(agents_map.contains_key("MantikAnalisti"));

    // REGRESSION 2: action_registry MUST contain all qualified names
    let action_registry = bc.action_registry.borrow();
    assert!(action_registry.contains_key("DeepSeekv4Pro.translate"));
    assert!(action_registry.contains_key("DeepSeekv4Flash.translate"));
    assert!(action_registry.contains_key("MetinYazari.slogan_yaz"));
    assert!(action_registry.contains_key("MantikAnalisti.veriyi_onayla"));

    // Test the absence of globals in `agents`
    assert!(!agents_map.contains_key("this"));
    assert!(!agents_map.contains_key("env"));
    assert!(!agents_map.contains_key("tcp"));
    assert!(!agents_map.contains_key("__hudhud_env"));
}

#[tokio::test]
async fn test_action_dispatch() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);

    let main_src = r#"
use "agents.hudhud" as agents;

let slogan = agents.MetinYazari.slogan_yaz("Robot yapimi");
let kontrol = agents.MantikAnalisti.veriyi_onayla(slogan);
"#;
    let main_path = dir.join("author_dispatch.hudhud");
    fs::write(&main_path, main_src).unwrap();

    let mut interpreter = Interpreter::new();
    let ast = hudhudscript_parser::parse(main_src).unwrap();

    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();

    let slogan_val = interpreter.vm.get_variable("slogan").unwrap();
    assert_eq!(slogan_val.as_str().unwrap(), "Robot yapimi icin slogan");

    let kontrol_val = interpreter.vm.get_variable("kontrol").unwrap();
    assert_eq!(
        kontrol_val.as_str().unwrap(),
        "onaylandi: Robot yapimi icin slogan"
    );
}

#[tokio::test]
async fn test_manual_user_script() {
    let temp_dir = TempDir::new().unwrap();
    let dir = temp_dir.path().to_path_buf();
    setup_test_files(&dir);

    let main_src = r#"
use "agents.hudhud" as agents;

print(agents);

let slogan = agents.MetinYazari.slogan_yaz("Robot yapimi");
print(slogan);
let kontrol = agents.MantikAnalisti.veriyi_onayla(slogan);
print(kontrol);
"#;
    let main_path = dir.join("manual_script.hudhud");
    fs::write(&main_path, main_src).unwrap();

    let mut interpreter = Interpreter::new();
    let ast = hudhudscript_parser::parse(main_src).unwrap();

    let mut compiler = hudhudscript_compiler::Compiler::new();
    compiler.set_module_base_dir(dir.clone());
    let bc = compiler.compile(&ast).unwrap();
    interpreter.vm.execute(&bc).unwrap();

    let slogan_val = interpreter.vm.get_variable("slogan").unwrap();
    assert_eq!(slogan_val.as_str().unwrap(), "Robot yapimi icin slogan");

    let kontrol_val = interpreter.vm.get_variable("kontrol").unwrap();
    assert_eq!(
        kontrol_val.as_str().unwrap(),
        "onaylandi: Robot yapimi icin slogan"
    );
}
