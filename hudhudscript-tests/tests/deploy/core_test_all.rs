//! External tests for hudhudscript-deploy-core crate.
//!
//! Covers: config construction, deploy plan, error types, adapters,
//! bundle, deb packaging, systemd generation.

use hudhudscript_deploy_core::adapters::{self, Adapter};
use hudhudscript_deploy_core::bundle::Bundle;
use hudhudscript_deploy_core::deb::DebPackage;
use hudhudscript_deploy_core::systemd::{RestartPolicy, ServiceConfig};
use hudhudscript_deploy_core::*;
use std::collections::HashMap;

// ── Helpers ────────────────────────────────────────────────────────

fn minimal_plan(name: &str) -> DeployPlan {
    DeployPlan {
        app_name: name.to_string(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    }
}

fn web_target() -> Target {
    Target {
        platform: TargetPlatform::Web,
        framework: "nextjs".to_string(),
        config: HashMap::new(),
    }
}

fn wasm_target() -> Target {
    Target {
        platform: TargetPlatform::Wasm,
        framework: "wasm-pack".to_string(),
        config: HashMap::new(),
    }
}

fn plan_with_targets(name: &str, targets: Vec<Target>) -> DeployPlan {
    DeployPlan {
        app_name: name.to_string(),
        targets,
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    }
}

fn plan_with_docker_and_k8s(name: &str) -> DeployPlan {
    let mut resources = HashMap::new();
    resources.insert("cpu".to_string(), "500m".to_string());
    resources.insert("memory".to_string(), "512Mi".to_string());
    DeployPlan {
        app_name: name.to_string(),
        targets: vec![web_target()],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: format!("{}-img", name),
            registry: "ghcr.io/test".to_string(),
            dockerfile: Some("node:18-slim".to_string()),
        }),
        kubernetes: Some(KubernetesConfig {
            namespace: "production".to_string(),
            replicas: 3,
            resources,
        }),
    }
}

// ── 1. DeployError Display ─────────────────────────────────────────

#[test]
fn deploy_error_display_variants() {
    let cases: Vec<(DeployError, &str)> = vec![
        (DeployError::ConfigError("bad".into()), "Config error: bad"),
        (
            DeployError::BuildFailed("oops".into()),
            "Build failed: oops",
        ),
        (
            DeployError::DeployFailed("timeout".into()),
            "Deploy failed: timeout",
        ),
        (
            DeployError::RollbackFailed("no prev".into()),
            "Rollback failed: no prev",
        ),
        (
            DeployError::AdapterError("missing".into()),
            "Adapter error: missing",
        ),
    ];
    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected);
    }
}

#[test]
fn deploy_error_is_std_error() {
    let err: Box<dyn std::error::Error> = Box::new(DeployError::ConfigError("test".into()));
    assert!(err.to_string().contains("Config error"));
}

// ── 2. Adapter parsing ────────────────────────────────────────────

#[test]
fn adapter_parse_known_providers() {
    let cases = vec![
        ("github", "GitHub"),
        ("GITHUB", "GitHub"),
        ("gitlab", "GitLab"),
        ("jenkins", "Jenkins"),
        ("docker", "Docker"),
        ("kubernetes", "Kubernetes"),
        ("k8s", "Kubernetes"),
        ("vercel", "Vercel"),
    ];
    for (input, expected_debug) in cases {
        let a = Adapter::parse(input).expect("parse should succeed");
        let dbg = format!("{:?}", a);
        assert_eq!(dbg, expected_debug, "failed for input: {}", input);
    }
}

#[test]
fn adapter_parse_unknown_becomes_custom() {
    let a = Adapter::parse("my-ci").expect("parse should succeed");
    match a {
        Adapter::Custom(name) => assert_eq!(name, "my-ci"),
        other => panic!("expected Custom, got {:?}", other),
    }
}

// ── 3. create_adapter + DeployAdapter trait ───────────────────────

#[test]
fn create_adapter_returns_correct_names() {
    let providers = vec![
        (Adapter::GitHub, "github"),
        (Adapter::Docker, "docker"),
        (Adapter::Vercel, "vercel"),
        (Adapter::Kubernetes, "kubernetes"),
    ];
    for (adapter_enum, expected_name) in providers {
        let adapter = adapters::create_adapter(&adapter_enum).expect("create_adapter should work");
        assert_eq!(adapter.name(), expected_name);
    }
}

#[test]
fn stub_adapter_for_unsupported_providers() {
    let adapter =
        adapters::create_adapter(&Adapter::GitLab).expect("create_adapter should work for gitlab");
    assert!(
        adapter.name().starts_with("unsupported("),
        "gitlab should use unsupported adapter, got: {}",
        adapter.name()
    );
}

// ── 4. GitHub adapter generate ────────────────────────────────────

#[test]
fn github_adapter_generates_workflow_yaml() {
    let adapter = adapters::create_adapter(&Adapter::GitHub).unwrap();
    let plan = plan_with_targets("myapp", vec![web_target()]);
    let artifacts = adapter.generate(&plan).unwrap();

    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].filename, ".github/workflows/deploy.yml");
    let content = &artifacts[0].content;
    assert!(
        content.contains("name: Deploy"),
        "should have workflow name"
    );
    assert!(
        content.contains("myapp"),
        "should reference app name in comment"
    );
    assert!(
        content.contains("npm run build"),
        "web target should produce npm build step"
    );
}

// ── 5. Docker adapter generate ────────────────────────────────────

#[test]
fn docker_adapter_generates_dockerfile_and_compose() {
    let adapter = adapters::create_adapter(&Adapter::Docker).unwrap();
    let plan = plan_with_docker_and_k8s("webapp");
    let artifacts = adapter.generate(&plan).unwrap();

    assert_eq!(artifacts.len(), 2);

    let dockerfile = artifacts.iter().find(|a| a.filename == "Dockerfile");
    assert!(dockerfile.is_some(), "should generate Dockerfile");
    let df_content = &dockerfile.unwrap().content;
    assert!(
        df_content.contains("node:18-slim"),
        "should use custom base image from DockerConfig"
    );

    let compose = artifacts
        .iter()
        .find(|a| a.filename == "docker-compose.yml");
    assert!(compose.is_some(), "should generate docker-compose.yml");
    let c_content = &compose.unwrap().content;
    assert!(
        c_content.contains("webapp-img"),
        "compose should reference the docker image name"
    );
}

// ── 6. K8s adapter generate ───────────────────────────────────────

#[test]
fn k8s_adapter_generates_deployment_and_service() {
    let adapter = adapters::create_adapter(&Adapter::Kubernetes).unwrap();
    let plan = plan_with_docker_and_k8s("api-server");
    let artifacts = adapter.generate(&plan).unwrap();

    assert_eq!(artifacts.len(), 2);
    let deployment = artifacts
        .iter()
        .find(|a| a.filename == "k8s/deployment.yaml")
        .expect("should have deployment.yaml");
    assert!(deployment.content.contains("replicas: 3"));
    assert!(deployment.content.contains("namespace: production"));
    assert!(deployment.content.contains("cpu: 500m"));
    assert!(deployment.content.contains("memory: 512Mi"));
    assert!(deployment.content.contains("ghcr.io/test/api-server-img"));

    let service = artifacts
        .iter()
        .find(|a| a.filename == "k8s/service.yaml")
        .expect("should have service.yaml");
    assert!(service.content.contains("kind: Service"));
    assert!(service.content.contains("namespace: production"));
}

// ── 7. Vercel adapter: wasm vs non-wasm ───────────────────────────

#[test]
fn vercel_adapter_wasm_vs_standard() {
    let adapter = adapters::create_adapter(&Adapter::Vercel).unwrap();

    // Non-wasm plan
    let plan_std = plan_with_targets("site", vec![web_target()]);
    let arts_std = adapter.generate(&plan_std).unwrap();
    assert_eq!(arts_std[0].filename, "vercel.json");
    assert!(
        arts_std[0].content.contains("\"framework\": \"nextjs\""),
        "non-wasm should default to nextjs"
    );

    // Wasm plan
    let plan_wasm = plan_with_targets("wasmsite", vec![wasm_target()]);
    let arts_wasm = adapter.generate(&plan_wasm).unwrap();
    assert!(
        arts_wasm[0].content.contains("wasm-pack build"),
        "wasm target should use wasm-pack build command"
    );
    assert!(
        arts_wasm[0].content.contains("\"framework\": null"),
        "wasm target should set framework to null"
    );
}

// ── 8. Vercel deploy returns URL ──────────────────────────────────

#[test]
fn vercel_deploy_returns_url() {
    let adapter = adapters::create_adapter(&Adapter::Vercel).unwrap();
    let plan = minimal_plan("cool-app");
    let result = adapter.deploy(&plan).unwrap();
    assert!(result.success);
    assert_eq!(result.url.as_deref(), Some("https://cool-app.vercel.app"));
}

// ── 9. Bundle wrapper script ──────────────────────────────────────

#[test]
fn bundle_wrapper_script_content() {
    let bundle = Bundle::new("myservice", "1.2.3", "main.hud");
    let wrapper = bundle.generate_wrapper_script();

    assert!(wrapper.starts_with("#!/bin/sh"), "should be a shell script");
    assert!(
        wrapper.contains("HUDHUD_HOME=\"/usr/share/hudhud/myservice\""),
        "should set HUDHUD_HOME"
    );
    assert!(
        wrapper.contains("HUDHUD_CONFIG=\"/etc/hudhud/myservice\""),
        "should set HUDHUD_CONFIG"
    );
    assert!(
        wrapper.contains("scripts/main.hud"),
        "should exec the entry point"
    );
    assert!(
        wrapper.contains("v1.2.3"),
        "should contain the version in the comment"
    );
}

// ── 10. Bundle directory structure (tempdir) ──────────────────────

#[test]
fn bundle_create_directory_structure() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let base = tmp.path();

    // Create a fake script file to bundle
    let script_path = base.join("app.hud");
    std::fs::write(&script_path, "print(\"hello\")").unwrap();

    let mut bundle = Bundle::new("testapp", "0.1.0", "app.hud");
    bundle.add_script(&script_path);

    let out_dir = base.join("output");
    std::fs::create_dir_all(&out_dir).unwrap();
    bundle
        .create_directory_structure(&out_dir)
        .expect("create_directory_structure should succeed");

    // Verify directories exist
    assert!(out_dir.join("usr/share/hudhud/testapp/scripts").is_dir());
    assert!(out_dir.join("usr/share/hudhud/testapp/assets").is_dir());
    assert!(out_dir.join("usr/share/hudhud/plugins").is_dir());
    assert!(out_dir.join("etc/hudhud/testapp").is_dir());
    assert!(out_dir.join("usr/bin").is_dir());

    // Verify script was copied
    let copied_script = out_dir.join("usr/share/hudhud/testapp/scripts/app.hud");
    assert!(copied_script.is_file(), "script should be copied");
    let content = std::fs::read_to_string(&copied_script).unwrap();
    assert_eq!(content, "print(\"hello\")");

    // Verify wrapper script was written
    let wrapper = out_dir.join("usr/bin/testapp");
    assert!(wrapper.is_file(), "wrapper should be written");
    let wrapper_content = std::fs::read_to_string(&wrapper).unwrap();
    assert!(wrapper_content.contains("#!/bin/sh"));
}

// ── 11. DebPackage control generation ─────────────────────────────

#[test]
fn deb_package_generate_control() {
    let mut pkg = DebPackage::new(
        "hudhud-app",
        "1.0.0",
        "A HudHud application",
        "Dev <d@e.com>",
    );
    pkg.dependencies
        .push("hudhudscript-runtime (>= 0.1)".to_string());
    pkg.extra_fields
        .insert("Section".to_string(), "devel".to_string());

    let control = pkg.generate_control();
    assert!(control.contains("Package: hudhud-app"));
    assert!(control.contains("Version: 1.0.0"));
    assert!(control.contains("Architecture: amd64"));
    assert!(control.contains("Maintainer: Dev <d@e.com>"));
    assert!(control.contains("Description: A HudHud application"));
    assert!(control.contains("Depends: hudhudscript-runtime (>= 0.1)"));
    assert!(control.contains("Section: devel"));
}

// ── 12. DebPackage conffiles generation ───────────────────────────

#[test]
fn deb_package_conffiles_generation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg_src = tmp.path().join("app.conf");
    std::fs::write(&cfg_src, "key=value").unwrap();

    let mut pkg = DebPackage::new("testpkg", "0.1.0", "desc", "m <m@m.m>");
    pkg.add_config(&cfg_src, "/etc/testpkg/app.conf");

    let conffiles = pkg.generate_conffiles();
    assert_eq!(conffiles, "/etc/testpkg/app.conf");
}

// ── 13. DebPackage build writes control file ──────────────────────

#[test]
fn deb_package_build_creates_debian_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let mut pkg = DebPackage::new("mypkg", "2.0.0", "test package", "Test <t@t.t>");
    pkg.set_postinst("#!/bin/sh\necho post");
    pkg.set_prerm("#!/bin/sh\necho pre");

    let pkg_root = pkg.build(tmp.path()).expect("build should succeed");

    assert!(pkg_root.join("DEBIAN/control").is_file());
    assert!(pkg_root.join("DEBIAN/postinst").is_file());
    assert!(pkg_root.join("DEBIAN/prerm").is_file());

    let control = std::fs::read_to_string(pkg_root.join("DEBIAN/control")).unwrap();
    assert!(control.contains("Package: mypkg"));
    assert!(control.contains("Version: 2.0.0"));

    let postinst = std::fs::read_to_string(pkg_root.join("DEBIAN/postinst")).unwrap();
    assert!(postinst.contains("echo post"));
}

// ── 14. Systemd service unit generation ───────────────────────────

#[test]
fn systemd_service_unit_generation() {
    let mut svc = ServiceConfig::new("myapp", "My Application", "/usr/bin/myapp --serve");
    svc.user = Some("www-data".to_string());
    svc.group = Some("www-data".to_string());
    svc.working_dir = Some("/var/lib/myapp".to_string());
    svc.environment
        .insert("PORT".to_string(), "8080".to_string());

    let unit = svc.generate_unit();

    assert!(unit.contains("[Unit]"));
    assert!(unit.contains("Description=My Application"));
    assert!(unit.contains("After=network.target"));
    assert!(unit.contains("[Service]"));
    assert!(unit.contains("Type=simple"));
    assert!(unit.contains("ExecStart=/usr/bin/myapp --serve"));
    assert!(unit.contains("Restart=on-failure"));
    assert!(unit.contains("User=www-data"));
    assert!(unit.contains("Group=www-data"));
    assert!(unit.contains("WorkingDirectory=/var/lib/myapp"));
    assert!(unit.contains("Environment=\"PORT=8080\""));
    assert!(unit.contains("[Install]"));
    assert!(unit.contains("WantedBy=multi-user.target"));
}

// ── 15. Systemd timer generation ──────────────────────────────────

#[test]
fn systemd_timer_generation() {
    let svc = ServiceConfig::new("cleanup", "Cleanup job", "/usr/bin/cleanup");
    let timer = svc.generate_timer("*-*-* 03:00:00");

    assert!(timer.contains("[Unit]"));
    assert!(timer.contains("Timer for Cleanup job"));
    assert!(timer.contains("[Timer]"));
    assert!(timer.contains("OnCalendar=*-*-* 03:00:00"));
    assert!(timer.contains("Persistent=true"));
    assert!(timer.contains("Unit=cleanup.service"));
    assert!(timer.contains("[Install]"));
    assert!(timer.contains("WantedBy=timers.target"));
}

// ── 16. RestartPolicy as_str values ───────────────────────────────

#[test]
fn restart_policy_as_str_coverage() {
    assert_eq!(RestartPolicy::OnFailure.as_str(), "on-failure");
    assert_eq!(RestartPolicy::Always.as_str(), "always");
    assert_eq!(RestartPolicy::No.as_str(), "no");
    assert_eq!(RestartPolicy::OnAbnormal.as_str(), "on-abnormal");
    assert_eq!(RestartPolicy::OnAbort.as_str(), "on-abort");
    assert_eq!(RestartPolicy::OnWatchdog.as_str(), "on-watchdog");
}
