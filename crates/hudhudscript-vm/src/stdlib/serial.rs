//! Serialization and encoding module registrations.

use crate::vm::VM;

pub fn register(vm: &mut VM) {
    vm.register_module(
        "toml",
        Box::new(|method, args| match method {
            "parse" => hudhud_serial::toml_ops::parse(&args),
            "stringify" => hudhud_serial::toml_ops::stringify(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown toml method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "yaml",
        Box::new(|method, args| match method {
            "parse" => hudhud_serial::yaml_ops::parse(&args),
            "stringify" => hudhud_serial::yaml_ops::stringify(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown yaml method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "csv",
        Box::new(|method, args| match method {
            "parse" => hudhud_serial::csv_ops::parse(&args),
            "stringify" => hudhud_serial::csv_ops::stringify(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown csv method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "ini",
        Box::new(|method, args| match method {
            "parse" => hudhud_serial::ini_ops::parse(&args),
            "stringify" => hudhud_serial::ini_ops::stringify(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown ini method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "Base64",
        Box::new(|method, args| match method {
            "encode" => hudhud_encoding::base64_encode_args(&args),
            "decode" => hudhud_encoding::base64_decode_args(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown Base64 method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "Hex",
        Box::new(|method, args| match method {
            "encode" => hudhud_encoding::hex_encode_args(&args),
            "decode" => hudhud_encoding::hex_decode_args(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown Hex method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "URL",
        Box::new(|method, args| match method {
            "encode" => hudhud_encoding::url_encode_args(&args),
            "decode" => hudhud_encoding::url_decode_args(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown URL method: {}", method),
            )),
        }),
    );
    vm.register_module(
        "uuid",
        Box::new(|method, args| match method {
            "v4" => hudhud_encoding::uuid_v4_args(&args),
            "v7" => hudhud_encoding::uuid_v7_args(&args),
            "nil" => hudhud_encoding::uuid_nil_args(&args),
            "parse" => hudhud_encoding::uuid_parse_args(&args),
            _ => Err(hudhudscript_bytecode::error::compile_codes::runtime_error(
                format!("Unknown uuid method: {}", method),
            )),
        }),
    );
}
