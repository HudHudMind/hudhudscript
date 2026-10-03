//! Coverage tests for hudhudscript-cli config handling beyond the cases in
//! test_cli_config.rs: defaults, section parsing, unknown-key leniency, numeric
//! boundaries, include-engine errors, GC clamping, deep-merge, validation.

use hudhudscript_cli::common::config_build::{apply_profile, validate_config};
use hudhudscript_cli::common::config_include::{deep_merge_toml, load_toml_with_includes};
use hudhudscript_cli::common::{try_load_config, BuildAotConfig, BuildConfig, HudHudConfig, OptimizationConfig, ProfileConfig, RedeclarePolicy, RuntimeJitConfig};
use std::path::Path;

fn parse(toml_str: &str) -> HudHudConfig {
    toml::from_str(toml_str).expect("valid hudhud.toml")
}

fn write_file(path: &Path, content: &str) { std::fs::write(path, content).unwrap(); }

/// Writes to <dir>/hudhud.toml and loads via try_load_config (GC normalization applied).
fn load_from(dir: &Path, content: &str) -> HudHudConfig {
    let file = dir.join("hudhud.toml");
    write_file(&file, content);
    try_load_config(&file, false).expect("config loads")
}

fn invalid(cfg: &HudHudConfig) -> String { validate_config(cfg).unwrap_err() }

fn profiled_with_optimization(opt: toml::Value) -> HudHudConfig {
    let mut c = HudHudConfig::default();
    c.profile.insert("p".into(), ProfileConfig { optimization: Some(opt), ..Default::default() });
    c
}

// ── defaults and section parsing ──────────────────────────────────

#[test]
fn empty_toml_yields_documented_defaults() {
    let c = parse("");
    let r = &c.runtime;
    assert_eq!((r.max_recursion, r.stack_limit, r.fuel_limit), (2000, 1_000_000, 0));
    assert_eq!((r.thread_stack_mb, r.register_arena_kb), (64, 64));
    assert_eq!((r.mailbox_capacity, r.max_mcp_servers, r.execution_timeout_ms), (128, 128, 0));
    assert_eq!((r.builtin_max_iter, r.max_call_depth_hard_ceiling), (10_000, 4000));
    assert_eq!((r.default_stack_bytes, r.provider_timeout_secs), (8 * 1024 * 1024, 120));
    assert!(!r.allow_network && !r.allow_process && !r.allow_insecure_http && !r.allow_privileged);
    assert!(r.engine.is_none() && r.backend.is_none() && r.vm.is_none() && r.jit.is_none());
    assert!(c.include.is_empty() && c.profile.is_empty() && c.providers.is_empty());
    assert!(c.build.is_none() && c.target.is_none() && c.optimization.is_none() && c.link.is_none() && c.host_access.is_none());
    assert!(matches!(c.lint.redeclare, RedeclarePolicy::Warn));
    assert_eq!((c.gc.min_objects, c.gc.growth_factor), (1024, 2));
    assert_eq!((c._stream._chunk_size, c._stream._timeout, c._stream._max_tokens, c._stream._buffer_size), (1024, 30_000, 4096, 8192));
    assert!(!c._security._sandbox);
    // [stream] overrides are per-key; untouched keys keep their defaults.
    let s = parse("[stream]\nchunk_size = 512\n");
    assert_eq!(s._stream._chunk_size, 512);
    assert_eq!((s._stream._timeout, s._stream._max_tokens, s._stream._buffer_size), (30_000, 4096, 8192));
}

#[test]
fn runtime_section_parses_every_scalar_knob_and_boundary_values() {
    let c = parse(concat!(
        "[runtime]\nmax_recursion = 300\nstack_limit = 500000\nfuel_limit = 12345\n",
        "thread_stack_mb = 128\nregister_arena_kb = 256\nmailbox_capacity = 64\nmax_mcp_servers = 4\n",
        "execution_timeout_ms = 9000\nbuiltin_max_iter = 200000\nmax_call_depth_hard_ceiling = 8000\n",
        "default_stack_bytes = 16777216\nprovider_timeout_secs = 15\nallow_network = true\n",
        "allow_process = true\nallow_insecure_http = true\nallow_privileged = true\n",
    ));
    let r = c.runtime;
    assert_eq!((r.max_recursion, r.stack_limit, r.fuel_limit, r.thread_stack_mb, r.register_arena_kb), (300, 500000, 12345, 128, 256));
    assert_eq!((r.mailbox_capacity, r.max_mcp_servers, r.execution_timeout_ms, r.builtin_max_iter), (64, 4, 9000, 200000));
    assert_eq!((r.max_call_depth_hard_ceiling, r.default_stack_bytes, r.provider_timeout_secs), (8000, 16777216, 15));
    assert!(r.allow_network && r.allow_process && r.allow_insecure_http && r.allow_privileged);

    // u32 boundary parses; fuel_limit is u64 but toml 0.8 parses literals as i64,
    // so i64::MAX is the largest file-expressible value and u64::MAX is rejected.
    let maxed = parse("[runtime]\nthread_stack_mb = 4294967295\nfuel_limit = 9223372036854775807\n");
    assert_eq!((maxed.runtime.thread_stack_mb, maxed.runtime.fuel_limit), (u32::MAX, i64::MAX as u64));
    assert!(toml::from_str::<HudHudConfig>("[runtime]\nthread_stack_mb = -1\n").is_err());
    assert!(toml::from_str::<HudHudConfig>("[runtime]\nthread_stack_mb = 4294967296\n").is_err());
    assert!(toml::from_str::<HudHudConfig>("[runtime]\nfuel_limit = 18446744073709551615\n").is_err());
    let vm = parse("[runtime.vm]\nmax_call_depth = 777\nstack_limit = 123456\nfuel_limit = 99\n").runtime.vm.expect("vm section");
    assert_eq!((vm.max_call_depth, vm.stack_limit, vm.fuel_limit), (Some(777), Some(123456), Some(99)));
}

#[test]
fn security_mcp_and_providers_sections_parse() {
    let c = parse(concat!(
        "[security]\nsandbox = true\n\n[security.commands]\nsafe = [\"ls\", \"cat\"]\nask = [\"rm\"]\n",
        "dangerous = [\"mkfs\"]\nblocked = [\"shutdown\"]\n[mcp.servers.fetch]\ncommand = \"npx\"\n",
        "args = [\"-y\", \"server-fetch\"]\n\n[mcp.servers.fetch.env]\nAPI_KEY = \"k1\"\n",
        "[providers.openai]\napi_key = \"sk-x\"\nmodel = \"gpt\"\n",
    ));
    assert!(c._security._sandbox);
    let cmds = &c._security._commands;
    assert_eq!(cmds._safe, vec!["ls", "cat"]);
    assert_eq!(cmds._ask, vec!["rm"]);
    assert_eq!(cmds._dangerous, vec!["mkfs"]);
    assert_eq!(cmds._blocked, vec!["shutdown"]);

    assert_eq!(c.mcp.servers.len(), 1);
    let fetch = c.mcp.servers.get("fetch").expect("fetch server");
    assert_eq!(fetch.command, "npx");
    assert_eq!(fetch.args, vec!["-y", "server-fetch"]);
    assert_eq!(fetch.env.get("API_KEY").map(String::as_str), Some("k1"));

    let openai = c.providers.get("openai").expect("openai provider");
    assert_eq!(openai.get("api_key").map(String::as_str), Some("sk-x"));
    assert_eq!(openai.get("model").map(String::as_str), Some("gpt"));
}

#[test]
fn lint_policies_parse_and_unknown_keys_are_tolerated() {
    assert!(matches!(parse("[lint]\nredeclare = \"error\"\n").lint.redeclare, RedeclarePolicy::Error));
    assert!(matches!(parse("[lint]\nredeclare = \"allow\"\n").lint.redeclare, RedeclarePolicy::Allow));
    assert!(matches!(parse("[lint]\nredeclare = \"warn\"\n").lint.redeclare, RedeclarePolicy::Warn));
    let err = toml::from_str::<HudHudConfig>("[lint]\nredeclare = \"strict\"\n").unwrap_err();
    assert!(err.to_string().contains("unknown variant `strict`"), "{err}");
    // No deny_unknown_fields: unknown keys are ignored, known keys still bind.
    let c = parse("totally_unknown = 42\n\n[runtime]\nmystery_option = \"x\"\nengine = \"jit\"\n");
    assert_eq!(c.runtime.engine.as_deref(), Some("jit"));
}

// ── try_load_config failure paths and GC normalization edges ──────

#[test]
fn try_load_config_returns_none_for_missing_invalid_or_mistyped_files() {
    let dir = tempfile::tempdir().unwrap();
    assert!(try_load_config(&dir.path().join("absent.toml"), false).is_none());
    let bad_syntax = dir.path().join("bad_syntax.toml");
    write_file(&bad_syntax, "[[runtime\n");
    assert!(try_load_config(&bad_syntax, false).is_none());
    let bad_type = dir.path().join("bad_type.toml");
    write_file(&bad_type, "[runtime]\nmax_recursion = \"lots\"\n");
    assert!(try_load_config(&bad_type, false).is_none());
}

#[test]
fn gc_growth_factor_below_threshold_is_left_untouched() {
    let dir = tempfile::tempdir().unwrap();
    for (raw, expected) in [(0usize, 0usize), (1, 1), (7, 7), (49, 49)] {
        let c = load_from(dir.path(), &format!("[gc]\ngrowth_factor = {raw}\n"));
        assert_eq!(c.gc.growth_factor, expected, "raw growth_factor {raw}");
        assert_eq!(c.gc.min_objects, 1024, "min_objects keeps its default");
    }
}

#[test]
fn gc_growth_factor_percent_values_are_divided_by_100() {
    let dir = tempfile::tempdir().unwrap();
    for (raw, expected) in [(50usize, 1usize), (99, 1), (100, 1), (149, 1), (250, 2), (350, 3), (1200, 12)] {
        let c = load_from(dir.path(), &format!("[gc]\nmin_objects = 250\ngrowth_factor = {raw}\n"));
        assert_eq!(c.gc.min_objects, 250);
        assert_eq!(c.gc.growth_factor, expected, "{raw}% must normalize to {expected}x");
    }
}

// ── include engine: error reasons, depth limit, shapes, merging ───

#[test]
fn include_errors_report_exact_reasons() {
    let dir = tempfile::tempdir().unwrap();

    let missing = load_toml_with_includes(&dir.path().join("missing.toml")).unwrap_err();
    assert!(missing.starts_with("failed to read config file "), "{missing}");
    assert!(missing.contains("missing.toml"));
    let broken = dir.path().join("broken.toml");
    write_file(&broken, "[runtime\nengine = \"vm\"\n");
    let parse_err = load_toml_with_includes(&broken).unwrap_err();
    assert!(parse_err.starts_with("parse error in "), "{parse_err}");
    assert!(parse_err.contains("broken.toml"));
    let non_string = dir.path().join("non_string.toml");
    write_file(&non_string, "include = [42]\n");
    let err = load_toml_with_includes(&non_string).unwrap_err();
    assert!(err.contains("invalid non-string item in `include` array of"), "{err}");
    let wrong_shape = dir.path().join("wrong_shape.toml");
    write_file(&wrong_shape, "include = 5\n");
    let err = load_toml_with_includes(&wrong_shape).unwrap_err();
    assert!(err.contains("`include` key must be an array of strings or string in"), "{err}");
}

#[test]
fn include_chain_beyond_depth_limit_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    for i in 0..9 {
        write_file(&dir.path().join(format!("c{i}.toml")), &format!("include = [\"c{}.toml\"]\n", i + 1));
    }
    write_file(&dir.path().join("c9.toml"), "[gc]\nmin_objects = 1\n");
    let err = load_toml_with_includes(&dir.path().join("c0.toml")).unwrap_err();
    assert!(err.starts_with("maximum include depth of 8 exceeded"), "{err}");
}

#[test]
fn include_accepts_single_string_form() {
    // The string include IS resolved+merged (main's engine wins, base's
    // provider_timeout_secs is inherited), but the loader leaves the resolved
    // `include` key as a string, so HudHudConfig deserialization fails —
    // pinned behavior, suspected source bug.
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir.path().join("base.toml"), "[runtime]\nengine = \"vm\"\nprovider_timeout_secs = 90\n");
    let main = dir.path().join("main.toml");
    write_file(&main, "include = \"base.toml\"\n\n[runtime]\nengine = \"jit\"\n");
    let merged = load_toml_with_includes(&main).expect("string include resolves");
    let rt = merged.get("runtime").expect("runtime table merged");
    assert_eq!(rt.get("engine").and_then(toml::Value::as_str), Some("jit"), "main overrides included file");
    assert_eq!(rt.get("provider_timeout_secs").and_then(toml::Value::as_integer), Some(90), "inherited from included file");
    let bad: Result<HudHudConfig, _> = merged.try_into();
    let err = bad.expect_err("leftover string include key is not a sequence");
    assert!(err.to_string().contains("invalid type: string \"base.toml\", expected a sequence"), "Display, not Debug: {err}");
}

#[test]
fn diamond_includes_merge_without_false_cycle_errors() {
    let dir = tempfile::tempdir().unwrap();
    write_file(&dir.path().join("d.toml"), "[optimization]\nlevel = 1\ngoal = \"size\"\n");
    write_file(&dir.path().join("b.toml"), "include = [\"d.toml\"]\n\n[runtime]\nfuel_limit = 100\n");
    write_file(&dir.path().join("c.toml"), "include = [\"d.toml\"]\n\n[runtime]\nbuiltin_max_iter = 55\n");
    let main = dir.path().join("main.toml");
    write_file(&main, "include = [\"b.toml\", \"c.toml\"]\n\n[optimization]\nlevel = 3\n");
    let cfg: HudHudConfig = load_toml_with_includes(&main).expect("diamond include is not a cycle").try_into().expect("deserialize");
    let opt = cfg.optimization.expect("optimization merged");
    assert_eq!(opt.level, Some(3), "main file wins");
    assert_eq!(opt.goal.as_deref(), Some("size"), "inherited through two branches");
    assert_eq!(cfg.runtime.fuel_limit, 100, "b.toml applied");
    assert_eq!(cfg.runtime.builtin_max_iter, 55, "c.toml applied");
}

#[test]
fn deep_merge_toml_merges_tables_and_replaces_scalars_and_arrays() {
    let mut base: toml::Value = toml::from_str(
        "top = \"a\"\nlist = [1, 2]\n[runtime]\nengine = \"vm\"\n[runtime.vm]\nmax_call_depth = 100\nstack_limit = 50\n",
    ).unwrap();
    let overlay: toml::Value = toml::from_str(
        "top = \"b\"\nlist = [9]\n[runtime]\nengine = \"jit\"\n[runtime.vm]\nstack_limit = 77\n",
    ).unwrap();
    deep_merge_toml(&mut base, overlay);
    assert_eq!(base.get("top").and_then(toml::Value::as_str), Some("b"));
    let list = base.get("list").and_then(toml::Value::as_array).unwrap();
    assert_eq!(list.len(), 1, "arrays are replaced, not concatenated");
    assert_eq!(list[0].as_integer(), Some(9));
    let runtime = base.get("runtime").unwrap();
    assert_eq!(runtime.get("engine").and_then(toml::Value::as_str), Some("jit"));
    let vm = runtime.get("vm").unwrap();
    assert_eq!(vm.get("max_call_depth").and_then(toml::Value::as_integer), Some(100), "sibling key survives");
    assert_eq!(vm.get("stack_limit").and_then(toml::Value::as_integer), Some(77), "overlay wins");
}

// ── strict validation rule set ────────────────────────────────────

#[test]
fn validate_accepts_known_engines_and_non_conflicting_combos() {
    assert_eq!(validate_config(&HudHudConfig::default()), Ok(()));
    let mut vm = HudHudConfig::default();
    vm.runtime.engine = Some("vm".into());
    assert!(validate_config(&vm).is_ok());
    let mut jit = HudHudConfig::default();
    jit.runtime.engine = Some("jit".into());
    assert!(validate_config(&jit).is_ok());
    // build.mode="aot" only conflicts with runtime.engine="jit".
    let mut aot_vm = HudHudConfig::default();
    aot_vm.runtime.engine = Some("vm".into());
    aot_vm.build = Some(BuildConfig { mode: Some("aot".into()), aot: None });
    assert!(validate_config(&aot_vm).is_ok());
    let mut bytecode_jit = HudHudConfig::default();
    bytecode_jit.runtime.engine = Some("jit".into());
    bytecode_jit.build = Some(BuildConfig { mode: Some("bytecode".into()), aot: None });
    assert!(validate_config(&bytecode_jit).is_ok());
    // Unknown engine produces the exact documented message.
    let mut bad = HudHudConfig::default();
    bad.runtime.engine = Some("quantum".into());
    assert_eq!(invalid(&bad), "unknown runtime.engine `quantum` — available: vm, jit");
}

#[test]
fn validate_checks_jit_fields_with_exact_messages() {
    for backend in ["auto", "cranelift", "llvm", "gccjit"] {
        let mut c = HudHudConfig::default();
        c.runtime.jit = Some(RuntimeJitConfig { backend: Some(backend.into()), ..Default::default() });
        assert!(validate_config(&c).is_ok(), "jit backend {backend}");
    }
    for (policy, fallback) in [("hot", "none"), ("lazy", "vm"), ("eager", "none")] {
        let mut c = HudHudConfig::default();
        let jit = RuntimeJitConfig { policy: Some(policy.into()), fallback: Some(fallback.into()), ..Default::default() };
        c.runtime.jit = Some(jit);
        assert!(validate_config(&c).is_ok(), "policy {policy} / fallback {fallback}");
    }
    let mut c = HudHudConfig::default();
    c.runtime.jit = Some(RuntimeJitConfig { backend: Some("msvc".into()), ..Default::default() });
    assert_eq!(invalid(&c), "unknown runtime.jit.backend `msvc` — available: auto, cranelift, llvm, gccjit");
    let mut c = HudHudConfig::default();
    c.runtime.jit = Some(RuntimeJitConfig { policy: Some("always".into()), ..Default::default() });
    assert_eq!(invalid(&c), "unknown runtime.jit.policy `always` — available: hot, lazy, eager");
    let mut c = HudHudConfig::default();
    c.runtime.jit = Some(RuntimeJitConfig { fallback: Some("cranelift".into()), ..Default::default() });
    assert_eq!(invalid(&c), "unknown runtime.jit.fallback `cranelift` — available: none, vm");
}

#[test]
fn validate_checks_build_fields_with_exact_messages() {
    let mut c = HudHudConfig::default();
    c.build = Some(BuildConfig { mode: Some("wasm".into()), aot: None });
    assert_eq!(invalid(&c), "unknown build.mode `wasm` — available: bytecode, aot");

    let mut c = HudHudConfig::default();
    let aot = BuildAotConfig { backend: Some("qbe".into()), format: None };
    c.build = Some(BuildConfig { mode: Some("aot".into()), aot: Some(aot) });
    assert_eq!(invalid(&c), "unknown build.aot.backend `qbe` — available: auto, cranelift, llvm, gccjit");

    let mut c = HudHudConfig::default();
    let aot = BuildAotConfig { backend: Some("llvm".into()), format: Some("object".into()) };
    c.build = Some(BuildConfig { mode: Some("aot".into()), aot: Some(aot) });
    assert_eq!(invalid(&c), "unknown build.aot.format `object` — available: executable, shared");

    // Both valid formats pass.
    for format in ["executable", "shared"] {
        let mut c = HudHudConfig::default();
        let aot = BuildAotConfig { backend: Some("gccjit".into()), format: Some(format.into()) };
        c.build = Some(BuildConfig { mode: Some("aot".into()), aot: Some(aot) });
        assert!(validate_config(&c).is_ok(), "format {format}");
    }
}

// ── apply_profile branch coverage ─────────────────────────────────

#[test]
fn apply_profile_unknown_name_is_a_noop() {
    let mut c = HudHudConfig::default();
    c.runtime.engine = Some("vm".into());
    let out = apply_profile(c, "does-not-exist");
    assert_eq!(out.runtime.engine.as_deref(), Some("vm"));
    assert!(out.build.is_none() && out.target.is_none() && out.optimization.is_none());
}

#[test]
fn apply_profile_sets_runtime_and_target_without_build() {
    let mut c = HudHudConfig::default();
    let prof = ProfileConfig {
        runtime: Some("jit".into()),
        target: Some("wasm32-unknown-unknown".into()),
        ..Default::default()
    };
    c.profile.insert("embed".into(), prof);
    let out = apply_profile(c, "embed");
    assert_eq!(out.runtime.engine.as_deref(), Some("jit"));
    let t = out.target.expect("target section created");
    assert_eq!(t.triple.as_deref(), Some("wasm32-unknown-unknown"));
    assert!(t.cpu.is_none() && t.features.is_empty());
    assert!(out.build.is_none(), "profile without build/backend must not create [build]");
}

#[test]
fn apply_profile_backend_creates_sections_and_mode_overrides() {
    let mut c = HudHudConfig::default();
    c.profile.insert("native".into(), ProfileConfig { backend: Some("llvm".into()), ..Default::default() });
    let out = apply_profile(c, "native");
    let b = out.build.expect("build section created");
    assert_eq!(b.mode, None);
    let aot = b.aot.expect("aot section created");
    assert_eq!(aot.backend.as_deref(), Some("llvm"));
    assert_eq!(aot.format, None);

    // profile build mode overrides an existing mode while preserving aot fields.
    let mut c = HudHudConfig::default();
    let aot = BuildAotConfig { backend: Some("cranelift".into()), format: Some("shared".into()) };
    c.build = Some(BuildConfig { mode: Some("bytecode".into()), aot: Some(aot) });
    c.profile.insert("rel".into(), ProfileConfig { build: Some("aot".into()), ..Default::default() });
    let b = apply_profile(c, "rel").build.unwrap();
    assert_eq!(b.mode.as_deref(), Some("aot"));
    let aot = b.aot.unwrap();
    assert_eq!(aot.backend.as_deref(), Some("cranelift"));
    assert_eq!(aot.format.as_deref(), Some("shared"));
}

#[test]
fn apply_profile_optimization_value_kinds() {
    // In-range integer (lower boundary) sets the level.
    let leveled = apply_profile(profiled_with_optimization(toml::Value::Integer(0)), "p");
    assert_eq!(leveled.optimization.expect("section created").level, Some(0));
    // apply_profile inserts the default [optimization] section BEFORE matching
    // the value (config_build.rs), so a dropped value leaves an untouched
    // default section — pinned behavior, suspected source wart.
    let dropped_values = [toml::Value::Integer(4), toml::Value::Integer(-1), toml::Value::Float(2.5), toml::Value::Boolean(true)];
    for dropped in dropped_values {
        let out = apply_profile(profiled_with_optimization(dropped.clone()), "p");
        let opt = out.optimization.expect("default section is created but left untouched");
        assert_eq!(opt.level, None, "value {dropped:?} must not set level");
        assert_eq!(opt.goal, None, "value {dropped:?} must not set goal");
    }
    // A string goes to goal, never to level.
    let out = apply_profile(profiled_with_optimization(toml::Value::String("size".into())), "p");
    let opt = out.optimization.expect("section created");
    assert_eq!(opt.goal.as_deref(), Some("size"));
    assert_eq!(opt.level, None);
    // Level override on an existing [optimization] keeps the previous goal.
    let mut c = HudHudConfig::default();
    c.optimization = Some(OptimizationConfig { level: Some(1), goal: Some("speed".into()) });
    c.profile.insert(
        "p".into(),
        ProfileConfig { optimization: Some(toml::Value::Integer(3)), ..Default::default() },
    );
    let opt = apply_profile(c, "p").optimization.unwrap();
    assert_eq!(opt.level, Some(3));
    assert_eq!(opt.goal.as_deref(), Some("speed"), "goal must survive the level override");
}
