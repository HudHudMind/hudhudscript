//! GATE-2: Telemetry integration tests.
//! Run with: cargo test -p hudhudscript-cli --features telemetry -- telemetry

#[cfg(feature = "telemetry")]
mod telemetry_tests {
    use hudhudscript_bytecode::Value16;
    use hudhudscript_vm::VM;
    use std::fs;

    fn run_script(src: &str) -> (VM, Value16) {
        let mut vm = VM::new();
        let ast = hudhudscript_parser::parse(src).unwrap();
        let mut compiler = hudhudscript_compiler::Compiler::new();
        let bc = compiler.compile(&ast).unwrap();
        vm.execute(&bc).unwrap();
        let result = vm.last_return_value();
        (vm, result)
    }

    fn run_script_ret(src: &str) -> (VM, Value16) {
        let wrapped = format!("fn __test() {{ {} }} __test()", src);
        run_script(&wrapped)
    }

    // ── 1. total_instructions > 0 ──────────────────────────────────

    #[test]
    fn total_instructions_nonzero() {
        let (vm, _) = run_script("let x = 1 + 2");
        let snap = vm.telemetry_snapshot();
        assert!(
            snap.total_instructions > 0,
            "total_instructions should be > 0"
        );
    }

    // ── 2. bigint_promotion on mul overflow (Int * Int → BigInt) ──

    #[test]
    fn bigint_promotion_on_mul_overflow() {
        let (vm, result) = run_script_ret("let a = 3037000500; let b = 3037000500; return a * b");
        assert!(result.is_bigint(), "overflow mul must produce BigInt");
        let snap = vm.telemetry_snapshot();
        assert!(
            snap.bigint_promotion > 0,
            "bigint_promotion must be > 0, got {}",
            snap.bigint_promotion
        );
    }

    // ── 3. No promotion without overflow ──────────────────────────

    #[test]
    fn no_bigint_promotion_without_overflow() {
        let (vm, _) = run_script("let x = 1 + 2");
        let snap = vm.telemetry_snapshot();
        assert_eq!(snap.bigint_promotion, 0, "no overflow => no promotion");
    }

    // ── 4. Two execs on same VM — counters reset, no accumulation ─

    #[test]
    fn counters_reset_per_execution() {
        let bytecode = {
            let mut compiler = hudhudscript_compiler::Compiler::new();
            let ast = hudhudscript_parser::parse("let x = 1 + 2").unwrap();
            compiler.compile(&ast).unwrap()
        };

        // First execution on a reused VM
        let mut vm = VM::new();
        vm.execute(&bytecode).unwrap();
        let snap_a = vm.telemetry_snapshot();

        // Second execution on the SAME VM — auto-reset in execute()
        vm.execute(&bytecode).unwrap();
        let snap_b = vm.telemetry_snapshot();

        // Compare with fresh VM
        let mut vm2 = VM::new();
        vm2.execute(&bytecode).unwrap();
        let snap_fresh = vm2.telemetry_snapshot();

        assert_eq!(
            snap_a.total_instructions, snap_b.total_instructions,
            "same bytecode, same VM: {} vs {} (no accumulation)",
            snap_a.total_instructions, snap_b.total_instructions
        );
        assert_eq!(
            snap_a.total_instructions, snap_fresh.total_instructions,
            "same bytecode, different VM: {} vs {} (reset is correct)",
            snap_a.total_instructions, snap_fresh.total_instructions
        );
    }

    // ── 5. execution_status: "error" ───────────────────────────────

    #[test]
    fn execution_status_error_on_runtime_failure() {
        let mut vm = VM::new();
        let src = "let x = 5 / 0";
        let ast = hudhudscript_parser::parse(src).unwrap();
        let bc = hudhudscript_compiler::Compiler::new()
            .compile(&ast)
            .unwrap();
        let result = vm.execute(&bc);
        assert!(result.is_err(), "division by zero should fail");

        use hudhudscript_cli::common::telemetry_writer::write_telemetry_json;
        let path = std::path::Path::new("target/telemetry_test_error.json");
        write_telemetry_json(&vm, path, false).unwrap();
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("\"execution_status\": \"error\""));
        let _ = fs::remove_file(path);
    }

    // ── 6. JSON schema required fields ────────────────────────────

    #[test]
    fn json_schema_has_required_fields() {
        let (vm, _) = run_script("let x = 1");
        use hudhudscript_cli::common::telemetry_writer::write_telemetry_json;
        let path = std::path::Path::new("target/telemetry_test_schema.json");
        write_telemetry_json(&vm, path, true).unwrap();

        let content = fs::read_to_string(path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["telemetry_enabled"], true);
        assert_eq!(json["execution_status"], "ok");
        assert!(
            json["counter_availability"]["total_instructions"]
                .as_str()
                .unwrap()
                == "available"
        );
        assert!(json["counters"]["total_instructions"].as_u64().unwrap() > 0);

        let _ = fs::remove_file(path);
    }

    // ── 7. Invalid path → error ───────────────────────────────────

    #[test]
    fn json_write_to_invalid_path_fails() {
        let (vm, _) = run_script("1");
        use hudhudscript_cli::common::telemetry_writer::write_telemetry_json;
        let result = write_telemetry_json(
            &vm,
            std::path::Path::new("/dev/null/nonexistent.json"),
            true,
        );
        assert!(result.is_err());
    }

    // ── 8. Fused branch overflow promotion ────────────────────────

    #[test]
    fn packed_dispatch_overflow_counts_promotion() {
        // Mul overflow inside a function — the values are in registers
        // so packed dispatch (D_INT_MUL_RR) handles the operation.
        // Overflow triggers Int→BigInt promotion counted in telemetry.
        let (vm, result) = run_script_ret("let a = 3037000500; let b = 3037000500; return a * b");
        assert!(
            result.is_bigint(),
            "overflow mul must produce BigInt, got {}",
            result.type_name_str()
        );
        let snap = vm.telemetry_snapshot();
        assert!(snap.total_instructions > 0);
        assert!(
            snap.bigint_promotion > 0,
            "overflow promotion must be > 0, got {}",
            snap.bigint_promotion
        );
    }
}
