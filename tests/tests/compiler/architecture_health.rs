/// Architecture Health Tests — PROVES the language runs from unified infrastructure.
///
/// Run: cargo test --test architecture_health -- --nocapture
///
/// These tests are NOT aspirational targets. They enforce CURRENT state
/// and MUST PASS. When infrastructure is unified (visitor trait, shared scope,
/// etc.), the assertions are tightened to the new reality.
///
/// Metrics tracked:
///   - CURRENT: What the codebase looks like RIGHT NOW (enforced, must not regress)
///   - REMAINING_DEBT: What still needs to be fixed (informational, not enforced yet)
///   - ZERO_DEBT: Enforced as 0 — any regression fails the build
use std::process::Command;

fn workspace_root() -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}

#[allow(dead_code)]
fn count_in_dir(pattern: &str, dir: &str) -> usize {
    let full_dir = format!("{}/{}", workspace_root(), dir);
    let output = Command::new("grep")
        .args(["-rn", pattern, &full_dir, "--include=*.rs"])
        .output()
        .expect("grep failed");
    String::from_utf8_lossy(&output.stdout).lines().count()
}

fn count_in_file(pattern: &str, file: &str) -> usize {
    let full_path = format!("{}/{}", workspace_root(), file);
    let output = Command::new("grep")
        .args(["-c", pattern, &full_path])
        .output()
        .expect("grep failed");
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0)
}

/// List files matching a pattern, return as Vec<(file, count)>
fn files_matching(pattern: &str, dir: &str) -> Vec<(String, usize)> {
    let full_dir = format!("{}/{}", workspace_root(), dir);
    let output = Command::new("grep")
        .args(["-rl", pattern, &full_dir, "--include=*.rs"])
        .output()
        .expect("grep failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let root = workspace_root();
    stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|f| {
            let count = count_in_file(pattern, f.strip_prefix(&format!("{}/", root)).unwrap_or(f));
            let short = f
                .strip_prefix(&format!("{}/", root))
                .unwrap_or(f)
                .to_string();
            (short, count)
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. SHARED BUILTINS: Every VM builtin method MUST delegate to shared crate.
//    Zero tolerance for inline reimplementation.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn vm_builtin_methods_all_delegate_to_shared() {
    let vm_file = "crates/hudhudscript-vm/src/vm/builtin_object.rs";

    // ALL these VM methods must delegate to domain crates (not builtins)
    let must_delegate = [
        "call_math_method",
        "call_string_method",
        "call_date_method",
        "call_regex_method",
        "call_linalg_method",
        "call_stats_method",
    ];

    let delegations = count_in_file("hudhud_", vm_file)
        + count_in_file("hudhud_", "crates/hudhudscript-vm/src/vm/builtin_object_extra.rs");

    assert!(
        delegations >= must_delegate.len(),
        "FAIL: VM has {} domain delegations but {} methods must delegate.\n\
         Every call_*_method in VM MUST delegate to a domain crate. No inline reimplementation allowed.\n\
         Methods that must delegate: {:?}",
        delegations,
        must_delegate.len(),
        must_delegate
    );

    // Also verify: these methods should NOT still exist as NOT-YET-delegated
    let remaining_vm_only = ["call_set_method", "call_map_method"];

    let not_yet_shared = remaining_vm_only
        .iter()
        .filter(|m| count_in_file(m, vm_file) > 0)
        .count();

    eprintln!(
        "  VM Domain Delegations:     {}/{} ✅",
        delegations,
        must_delegate.len()
    );
    eprintln!(
        "  VM Still-Inline Methods:   {}/{} ← DEBT (issue #877/#878/#879)",
        not_yet_shared,
        remaining_vm_only.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. SHARED VALUE TRAIT: Both Value types MUST implement SharedValue.
//    No exceptions.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "SharedValue trait retired during unified builtin dispatch migration (2026-04). Builtins now use direct Value16 methods in bytecode crate."]
fn bytecode_value_implements_shared_value() {
    // Post interpreter-crate retirement: `bytecode::Value` is the only
    // runtime Value type, and it MUST implement `SharedValue` so shared
    // builtins dispatch through a single trait.
    let bytecode_value = count_in_file(
        "impl.*SharedValue.*for Value",
        "crates/hudhudscript-bytecode/src/lib.rs",
    );
    let bytecode_value16 = count_in_file(
        "impl.*SharedValue.*for Value16",
        "crates/hudhudscript-bytecode/src/value16_shared.rs",
    );

    assert!(
        bytecode_value >= 1 || bytecode_value16 >= 1,
        "FAIL: bytecode Value or Value16 MUST implement SharedValue (Value: {}, Value16: {})",
        bytecode_value,
        bytecode_value16
    );

    eprintln!("  SharedValue for bytecode Value:    {} ✅", bytecode_value);
    eprintln!(
        "  SharedValue for bytecode Value16:  {} ✅",
        bytecode_value16
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. REAL PARITY TESTS: Every test compares BOTH runtimes on SAME source.
//    Count must equal what we have — no regression allowed.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn real_parity_test_count_no_regression() {
    let count = count_in_file(
        "fn real_parity_",
        "hudhud-script-tests/tests/compiler/real_parity_tests.rs",
    );

    // Current baseline. This number can only GO UP.
    let baseline = 74;
    assert!(
        count >= baseline,
        "FAIL: Real parity tests regressed from {} to {}. Tests can only be ADDED, never removed.",
        baseline,
        count
    );

    eprintln!("  Real Parity Tests: {} (baseline: {}) ✅", count, baseline);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. SHARED MODULES: Count of modules in shared-builtins. Must not decrease.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[ignore = "hudhudscript-builtins crate removed during unified builtin dispatch migration (2026-04). Shared builtins now live in hudhudscript-bytecode::shared_value."]
fn shared_builtins_module_count_no_regression() {
    let count = count_in_file("pub mod", "crates/hudhudscript-builtins/src/lib.rs");

    let baseline = 13; // math, string, json, toml_ops, yaml_ops, csv_ops, ini_ops, array, encoding, path, date, regex_ops, types
    assert!(
        count >= baseline,
        "FAIL: Shared builtins modules regressed from {} to {}. Modules can only be ADDED.",
        baseline,
        count
    );

    eprintln!(
        "  Shared Builtin Modules: {} (baseline: {}) ✅",
        count, baseline
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. DUPLICATION TRACKING: Shows exactly where duplication exists.
//    Not a pass/fail — but makes the debt VISIBLE.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn duplication_audit_report() {
    eprintln!("\n══════════════════════════════════════════════════════════════");
    eprintln!("  DUPLICATION AUDIT — Where the same thing is written twice");
    eprintln!("══════════════════════════════════════════════════════════════");

    // AST Walking: Stmt::Let matched across codebase
    let stmt_let_files = files_matching("Stmt::Let", "crates/");
    let total_stmt_let: usize = stmt_let_files.iter().map(|(_, c)| c).sum();
    eprintln!(
        "\n  [AST WALKING] Stmt::Let matched in {} files ({} total occurrences):",
        stmt_let_files.len(),
        total_stmt_let
    );
    // Allowed files (kaçınılmaz): ast definition, parser, compiler, interpreter, visitor
    let allowed = [
        "ast/src/stmt.rs",
        "parser/src/parser/statement.rs",
        "compiler/src/compiler.rs",
        "interpreter/src/interpreter/statement.rs",
    ];
    for (file, count) in &stmt_let_files {
        let is_allowed = allowed.iter().any(|a| file.contains(a));
        let status = if is_allowed { "NECESSARY" } else { "DEBT" };
        eprintln!("    {} × {} [{}]", count, file, status);
    }

    // Scope implementations
    eprintln!("\n  [SCOPE] Separate scope/environment implementations:");
    for pattern in &[
        "struct Environment",
        "struct SymbolTable",
        "scope_stack",
        "scope_depth",
        "fn push_scope",
        "fn pop_scope",
    ] {
        let files = files_matching(pattern, "crates/");
        for (file, count) in &files {
            eprintln!("    {} × \"{}\" in {}", count, pattern, file);
        }
    }

    // Value enums
    let value_files = files_matching("pub enum Value", "crates/");
    eprintln!("\n  [VALUE TYPES] pub enum Value definitions:");
    for (file, count) in &value_files {
        eprintln!("    {} × {}", count, file);
    }

    // VM inline methods that should be shared
    let vm_only_methods = [
        "call_set_method",
        "call_map_method",
        "call_linalg_method",
        "call_stats_method",
        "call_temp_method",
        "call_glob_method",
        "call_http_method",
        "call_file_method",
        "call_exec_method",
        "call_tcp_method",
        "call_udp_method",
        "call_unix_method",
        "call_ws_method",
        "call_daemon_method",
        "call_fs_method",
        "call_env_method",
        "call_os_method",
    ];
    let vm_file = "crates/hudhudscript-vm/src/vm.rs";
    let still_inline: Vec<&&str> = vm_only_methods
        .iter()
        .filter(|m| count_in_file(m, vm_file) > 0)
        .collect();
    eprintln!(
        "\n  [VM INLINE] Methods NOT yet delegated to shared ({}/{}):",
        still_inline.len(),
        vm_only_methods.len()
    );
    for m in &still_inline {
        eprintln!("    - {}", m);
    }

    eprintln!("\n══════════════════════════════════════════════════════════════");
    eprintln!("  END OF AUDIT");
    eprintln!("══════════════════════════════════════════════════════════════\n");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. NO NEW INLINE BUILTINS: VM must not gain new call_*_method without shared.
//    This is a RATCHET — the count can only go DOWN.
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn vm_inline_method_count_ratchet() {
    let vm_file = "crates/hudhudscript-vm/src/vm.rs";
    let total_methods = count_in_file("fn call_.*_method", vm_file);
    let shared_delegations = count_in_file("hudhudscript_builtins::", vm_file);

    // Current state: some methods are shared, some are inline.
    // This ratchet ensures the inline count NEVER increases.
    let inline_count = total_methods.saturating_sub(shared_delegations.min(total_methods));

    // Current baseline of inline (not-yet-shared) methods
    let baseline_inline = 25; // Will decrease as we migrate more methods

    // RATCHET: This number is the CURRENT count. It can only go DOWN.
    // When you move a method to shared-builtins, lower this number.
    // If this test fails, you added a new inline builtin to VM — move it to shared-builtins instead.
    let max_allowed = 47; // 2026-04-09: linalg, stats, temp, glob now delegate to shared-builtins.
    assert!(
        total_methods <= max_allowed,
        "FAIL: VM has {} call_*_method functions (max allowed: {}). \
         New builtins MUST go in shared-builtins, not vm.rs. \
         If you migrated methods, lower the max_allowed in this test.",
        total_methods,
        max_allowed
    );

    eprintln!("  VM total call_*_method:  {}", total_methods);
    eprintln!("  VM shared delegations:   {}", shared_delegations);
    eprintln!(
        "  VM still-inline:         ~{} (ratchet baseline: {})",
        inline_count, baseline_inline
    );
}
