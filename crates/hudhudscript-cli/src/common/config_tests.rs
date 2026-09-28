use crate::common::config_build::{apply_profile, validate_config};
use crate::common::config_include::load_toml_with_includes;
use crate::common::HudHudConfig;

#[test]
fn test_provider_timeout_toml_merge() {
    let toml_str = r#"
[runtime]
provider_timeout_secs = 180
"#;
    let config: HudHudConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.runtime.provider_timeout_secs, 180);
}

#[test]
fn test_runtime_engine_from_toml() {
    let toml = r#"
[runtime]
engine = "jit"
backend = "cranelift"
"#;
    let cfg: HudHudConfig = toml::from_str(toml).expect("valid toml");
    assert_eq!(cfg.runtime.engine.as_deref(), Some("jit"));
    assert_eq!(cfg.runtime.backend.as_deref(), Some("cranelift"));
}

#[test]
fn test_runtime_jit_section_from_toml() {
    let toml = r#"
[runtime]
engine = "jit"

[runtime.jit]
backend = "llvm"
fallback = "vm"
"#;
    let cfg: HudHudConfig = toml::from_str(toml).expect("valid toml");
    assert_eq!(cfg.runtime.engine.as_deref(), Some("jit"));
    let jit = cfg.runtime.jit.expect("jit section");
    assert_eq!(jit.backend.as_deref(), Some("llvm"));
    assert_eq!(jit.fallback.as_deref(), Some("vm"));
}

#[test]
fn test_runtime_jit_full_spec_from_toml() {
    let toml = r#"
[runtime]
engine = "jit"

[runtime.vm]
max_call_depth = 2500

[runtime.jit]
backend = "cranelift"
policy = "hot"
hot_threshold = 1000
loop_threshold = 10000
fallback = "vm"
stats = true
verify_with_vm = false
cache = true
code_cache_mb = 128
opt_level = "speed"
opt_goal = "speed"
mir_opt_rounds = 6
"#;
    let cfg: HudHudConfig = toml::from_str(toml).expect("valid toml");
    let vm = cfg.runtime.vm.expect("vm section");
    assert_eq!(vm.max_call_depth, Some(2500));

    let jit = cfg.runtime.jit.expect("jit section");
    assert_eq!(jit.backend.as_deref(), Some("cranelift"));
    assert_eq!(jit.policy.as_deref(), Some("hot"));
    assert_eq!(jit.hot_threshold, Some(1000));
    assert_eq!(jit.loop_threshold, Some(10000));
    assert_eq!(jit.fallback.as_deref(), Some("vm"));
    assert_eq!(jit.stats, Some(true));
    assert_eq!(jit.cache, Some(true));
    assert_eq!(jit.code_cache_mb, Some(128));
    assert_eq!(jit.opt_level.as_deref(), Some("speed"));
    assert_eq!(jit.mir_opt_rounds, Some(6));
}

#[test]
fn test_build_config_and_profiles_from_toml() {
    let toml = r#"
[build]
mode = "aot"

[build.aot]
backend = "cranelift"
format = "executable"

[target]
triple = "native"
cpu = "native"
features = ["+avx2"]

[optimization]
level = 2
goal = "speed"

[link]
runtime = "static"

[profile.release]
build = "aot"
backend = "llvm"
optimization = 3
"#;
    let cfg: HudHudConfig = toml::from_str(toml).expect("valid toml");
    let b = cfg.build.as_ref().expect("build section");
    assert_eq!(b.mode.as_deref(), Some("aot"));
    let aot = b.aot.as_ref().expect("aot section");
    assert_eq!(aot.backend.as_deref(), Some("cranelift"));
    assert_eq!(aot.format.as_deref(), Some("executable"));

    let opt = cfg.optimization.as_ref().expect("optimization section");
    assert_eq!(opt.level, Some(2));
    assert_eq!(opt.goal.as_deref(), Some("speed"));

    // Apply profile "release"
    let applied = apply_profile(cfg, "release");
    let applied_aot = applied.build.unwrap().aot.unwrap();
    assert_eq!(applied_aot.backend.as_deref(), Some("llvm"));
    assert_eq!(applied.optimization.unwrap().level, Some(3));
}

#[test]
fn test_strict_validation_aot_and_jit_conflict() {
    let toml = r#"
[runtime]
engine = "jit"

[build]
mode = "aot"
"#;
    let cfg: HudHudConfig = toml::from_str(toml).unwrap();
    let err = validate_config(&cfg);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("has no meaning when build.mode=\"aot\""));
}

#[test]
fn test_include_resolution_and_deep_merge() {
    let temp_dir = std::env::temp_dir().join(format!("hudhud_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let base_file = temp_dir.join("hudhud.build.toml");
    let main_file = temp_dir.join("hudhud.toml");

    std::fs::write(&base_file, r#"
[runtime]
engine = "vm"

[optimization]
level = 1
goal = "size"
"#).unwrap();

    std::fs::write(&main_file, r#"
include = ["hudhud.build.toml"]

[runtime]
engine = "jit"

[optimization]
level = 3
"#).unwrap();

    let merged_val = load_toml_with_includes(&main_file).expect("load with includes");
    let cfg: HudHudConfig = merged_val.try_into().expect("deserialize merged config");

    // Main file overrides included file
    assert_eq!(cfg.runtime.engine.as_deref(), Some("jit"));
    let opt = cfg.optimization.expect("optimization merged");
    assert_eq!(opt.level, Some(3));
    // Inherited from base file
    assert_eq!(opt.goal.as_deref(), Some("size"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_include_cycle_detection() {
    let temp_dir = std::env::temp_dir().join(format!("hudhud_test_cycle_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    let a_file = temp_dir.join("a.toml");
    let b_file = temp_dir.join("b.toml");

    std::fs::write(&a_file, r#"include = ["b.toml"]"#).unwrap();
    std::fs::write(&b_file, r#"include = ["a.toml"]"#).unwrap();

    let res = load_toml_with_includes(&a_file);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("cyclical include detected"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_gc_growth_factor_normalization() {
    use super::try_load_config;
    let temp_dir = std::env::temp_dir().join(format!("hh_gc_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let cfg_file = temp_dir.join("hudhud.toml");

    std::fs::write(&cfg_file, "[gc]\nmin_objects = 500\ngrowth_factor = 200\n").unwrap();
    let loaded = try_load_config(&cfg_file, false).expect("loaded");
    assert_eq!(loaded.gc.min_objects, 500);
    assert_eq!(loaded.gc.growth_factor, 2, "200% should be normalized to 2x");

    std::fs::write(&cfg_file, "[gc]\nmin_objects = 800\ngrowth_factor = 3\n").unwrap();
    let loaded2 = try_load_config(&cfg_file, false).expect("loaded2");
    assert_eq!(loaded2.gc.min_objects, 800);
    assert_eq!(loaded2.gc.growth_factor, 3, "factor 3 should remain 3");

    let _ = std::fs::remove_dir_all(&temp_dir);
}
