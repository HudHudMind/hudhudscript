//! Public API tests for hudhudscript-deploy-core
//! Covers: lib.rs (DeployPlan, Target, Pipeline, Trigger, PipelineStep, StepAction,
//!          CIProvider, TargetPlatform, DockerConfig, KubernetesConfig,
//!          DeployArtifact, DeployResult, DeployError),
//!         systemd.rs (ServiceConfig, RestartPolicy),
//!         deb.rs (DebPackage),
//!         bundle.rs (Bundle).

use hudhudscript_deploy_core::{
    bundle::Bundle,
    deb::DebPackage,
    systemd::{RestartPolicy, ServiceConfig},
    CIProvider, DeployArtifact, DeployError, DeployPlan, DeployResult, DockerConfig,
    KubernetesConfig, Pipeline, PipelineStep, StepAction, Target, TargetPlatform, Trigger,
};
use std::collections::HashMap;

// ── TargetPlatform ──────────────────────────────────────────────────

#[test]
fn target_platform_web_roundtrip() {
    let t = Target {
        platform: TargetPlatform::Web,
        framework: "nextjs".into(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(back.framework, "nextjs");
}

#[test]
fn target_platform_desktop_json_contains_variant() {
    let t = Target {
        platform: TargetPlatform::Desktop,
        framework: "tauri".into(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("Desktop"));
}

#[test]
fn target_platform_mobile_roundtrip() {
    let t = Target {
        platform: TargetPlatform::Mobile,
        framework: "flutter".into(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(back.framework, "flutter");
}

#[test]
fn target_platform_wasm_roundtrip() {
    let t = Target {
        platform: TargetPlatform::Wasm,
        framework: "wasm-pack".into(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(back.framework, "wasm-pack");
}

#[test]
fn target_platform_custom_carries_inner_string() {
    let t = Target {
        platform: TargetPlatform::Custom("embedded".into()),
        framework: "bare-metal".into(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("embedded"));
    assert!(json.contains("bare-metal"));
}

#[test]
fn target_config_entries_preserved_through_serde() {
    let mut cfg = HashMap::new();
    cfg.insert("region".into(), "eu-west".into());
    let t = Target {
        platform: TargetPlatform::Web,
        framework: "react".into(),
        config: cfg,
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(
        back.config.get("region").map(String::as_str),
        Some("eu-west")
    );
}

#[test]
fn target_clone_is_independent() {
    let t = Target {
        platform: TargetPlatform::Web,
        framework: "svelte".into(),
        config: HashMap::new(),
    };
    let cloned = t.clone();
    assert_eq!(cloned.framework, "svelte");
}

// ── DeployPlan ──────────────────────────────────────────────────────

#[test]
fn deploy_plan_creation_minimal() {
    let plan = DeployPlan {
        app_name: "app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    assert_eq!(plan.app_name, "app");
    assert!(plan.targets.is_empty());
    assert!(plan.pipelines.is_empty());
    assert!(plan.docker.is_none());
    assert!(plan.kubernetes.is_none());
}

#[test]
fn deploy_plan_with_multiple_targets() {
    let plan = DeployPlan {
        app_name: "multi".into(),
        targets: vec![
            Target {
                platform: TargetPlatform::Web,
                framework: "react".into(),
                config: HashMap::new(),
            },
            Target {
                platform: TargetPlatform::Desktop,
                framework: "tauri".into(),
                config: HashMap::new(),
            },
        ],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    assert_eq!(plan.targets.len(), 2);
}

#[test]
fn deploy_plan_with_docker_config_fields() {
    let plan = DeployPlan {
        app_name: "dockerized".into(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "org/app".into(),
            registry: "ghcr.io".into(),
            dockerfile: Some("Dockerfile.prod".into()),
        }),
        kubernetes: None,
    };
    let d = plan.docker.as_ref().unwrap();
    assert_eq!(d.image, "org/app");
    assert_eq!(d.registry, "ghcr.io");
    assert_eq!(d.dockerfile.as_deref(), Some("Dockerfile.prod"));
}

#[test]
fn deploy_plan_docker_config_none_dockerfile() {
    let plan = DeployPlan {
        app_name: "app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "img".into(),
            registry: "reg".into(),
            dockerfile: None,
        }),
        kubernetes: None,
    };
    assert!(plan.docker.as_ref().unwrap().dockerfile.is_none());
}

#[test]
fn deploy_plan_with_kubernetes_config_fields() {
    let mut resources = HashMap::new();
    resources.insert("cpu".into(), "500m".into());
    let plan = DeployPlan {
        app_name: "k8s-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: Some(KubernetesConfig {
            namespace: "prod".into(),
            replicas: 3,
            resources,
        }),
    };
    let k = plan.kubernetes.as_ref().unwrap();
    assert_eq!(k.namespace, "prod");
    assert_eq!(k.replicas, 3);
    assert_eq!(k.resources.get("cpu").map(String::as_str), Some("500m"));
}

#[test]
fn deploy_plan_serialization_roundtrip() {
    let plan = DeployPlan {
        app_name: "serde-test".into(),
        targets: vec![Target {
            platform: TargetPlatform::Wasm,
            framework: "wasm-pack".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "img".into(),
            registry: "reg".into(),
            dockerfile: None,
        }),
        kubernetes: None,
    };
    let json = serde_json::to_string(&plan).unwrap();
    let back: DeployPlan = serde_json::from_str(&json).unwrap();
    assert_eq!(back.app_name, "serde-test");
    assert_eq!(back.targets.len(), 1);
}

#[test]
fn deploy_plan_clone_independent() {
    let plan = DeployPlan {
        app_name: "original".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let cloned = plan.clone();
    assert_eq!(cloned.app_name, "original");
}

#[test]
fn deploy_plan_with_both_docker_and_k8s() {
    let plan = DeployPlan {
        app_name: "full".into(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "img".into(),
            registry: "reg".into(),
            dockerfile: None,
        }),
        kubernetes: Some(KubernetesConfig {
            namespace: "staging".into(),
            replicas: 2,
            resources: HashMap::new(),
        }),
    };
    assert!(plan.docker.is_some());
    assert!(plan.kubernetes.is_some());
}

// ── CIProvider & Pipeline ───────────────────────────────────────────

#[test]
fn ci_provider_github_in_json() {
    let json = serde_json::to_string(&CIProvider::GitHub).unwrap();
    assert!(json.contains("GitHub"));
}

#[test]
fn ci_provider_gitlab_in_json() {
    let json = serde_json::to_string(&CIProvider::GitLab).unwrap();
    assert!(json.contains("GitLab"));
}

#[test]
fn ci_provider_jenkins_in_json() {
    let json = serde_json::to_string(&CIProvider::Jenkins).unwrap();
    assert!(json.contains("Jenkins"));
}

#[test]
fn ci_provider_custom_carries_name_in_json() {
    let json = serde_json::to_string(&CIProvider::Custom("CircleCI".into())).unwrap();
    assert!(json.contains("CircleCI"));
}

#[test]
fn trigger_push_with_branch() {
    let t = Trigger {
        event: "push".into(),
        branch: Some("main".into()),
        cron: None,
    };
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("push"));
    assert!(json.contains("main"));
}

#[test]
fn trigger_schedule_with_cron_expression() {
    let t = Trigger {
        event: "schedule".into(),
        branch: None,
        cron: Some("0 3 * * *".into()),
    };
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("0 3 * * *"));
}

#[test]
fn trigger_pull_request_event() {
    let t = Trigger {
        event: "pull_request".into(),
        branch: Some("develop".into()),
        cron: None,
    };
    let json = serde_json::to_string(&t).unwrap();
    assert!(json.contains("pull_request"));
}

#[test]
fn pipeline_full_serialization() {
    let pipeline = Pipeline {
        provider: CIProvider::GitHub,
        config: HashMap::new(),
        triggers: vec![Trigger {
            event: "push".into(),
            branch: Some("main".into()),
            cron: None,
        }],
        steps: vec![
            PipelineStep {
                name: "test".into(),
                action: StepAction::Test,
            },
            PipelineStep {
                name: "custom".into(),
                action: StepAction::Custom {
                    command: "echo ok".into(),
                },
            },
        ],
    };
    let json = serde_json::to_string(&pipeline).unwrap();
    assert!(json.contains("echo ok"));
    assert!(json.contains("push"));
}

#[test]
fn pipeline_clone_deep_copies_steps() {
    let pipeline = Pipeline {
        provider: CIProvider::GitLab,
        config: HashMap::new(),
        triggers: vec![],
        steps: vec![PipelineStep {
            name: "test".into(),
            action: StepAction::Test,
        }],
    };
    let cloned = pipeline.clone();
    assert_eq!(cloned.steps.len(), 1);
}

#[test]
fn step_action_build_contains_target_info() {
    let step = PipelineStep {
        name: "build".into(),
        action: StepAction::Build {
            target: Target {
                platform: TargetPlatform::Web,
                framework: "react".into(),
                config: HashMap::new(),
            },
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("Build"));
    assert!(json.contains("react"));
}

#[test]
fn step_action_deploy_contains_host() {
    let step = PipelineStep {
        name: "deploy".into(),
        action: StepAction::Deploy {
            target: Target {
                platform: TargetPlatform::Web,
                framework: "vue".into(),
                config: HashMap::new(),
            },
            host: "prod.example.com".into(),
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("prod.example.com"));
}

#[test]
fn step_action_docker_contains_registry() {
    let step = PipelineStep {
        name: "push".into(),
        action: StepAction::Docker {
            image: "myimg:latest".into(),
            registry: "docker.io".into(),
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("docker.io"));
}

#[test]
fn step_action_test_in_json() {
    let step = PipelineStep {
        name: "test".into(),
        action: StepAction::Test,
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("Test"));
}

#[test]
fn step_action_custom_contains_command() {
    let step = PipelineStep {
        name: "c".into(),
        action: StepAction::Custom {
            command: "make release".into(),
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("make release"));
}

// ── DeployArtifact & DeployResult ──────────────────────────────────

#[test]
fn deploy_artifact_fields_accessible() {
    let a = DeployArtifact {
        filename: "Dockerfile".into(),
        content: "FROM rust:1.75".into(),
    };
    assert_eq!(a.filename, "Dockerfile");
    assert!(a.content.contains("rust"));
}

#[test]
fn deploy_artifact_debug_format() {
    let a = DeployArtifact {
        filename: "k8s.yaml".into(),
        content: "apiVersion: v1".into(),
    };
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("k8s.yaml"));
}

#[test]
fn deploy_artifact_clone() {
    let a = DeployArtifact {
        filename: "f".into(),
        content: "c".into(),
    };
    let b = a.clone();
    assert_eq!(b.filename, "f");
    assert_eq!(b.content, "c");
}

#[test]
fn deploy_result_success_fields() {
    let r = DeployResult {
        success: true,
        url: Some("https://example.com".into()),
        message: "ok".into(),
    };
    assert!(r.success);
    assert_eq!(r.url.as_deref(), Some("https://example.com"));
    assert_eq!(r.message, "ok");
}

#[test]
fn deploy_result_failure_has_no_url() {
    let r = DeployResult {
        success: false,
        url: None,
        message: "timeout".into(),
    };
    assert!(!r.success);
    assert!(r.url.is_none());
}

#[test]
fn deploy_result_debug_format() {
    let r = DeployResult {
        success: true,
        url: None,
        message: "deployed".into(),
    };
    let dbg = format!("{:?}", r);
    assert!(dbg.contains("deployed"));
}

#[test]
fn deploy_result_clone() {
    let r = DeployResult {
        success: false,
        url: None,
        message: "err".into(),
    };
    let c = r.clone();
    assert!(!c.success);
}

// ── DeployError ─────────────────────────────────────────────────────

#[test]
fn deploy_error_config_display() {
    let e = DeployError::ConfigError("bad field".into());
    assert_eq!(e.to_string(), "Config error: bad field");
}

#[test]
fn deploy_error_build_failed_display() {
    let e = DeployError::BuildFailed("compile error".into());
    assert_eq!(e.to_string(), "Build failed: compile error");
}

#[test]
fn deploy_error_deploy_failed_display() {
    let e = DeployError::DeployFailed("timeout".into());
    assert_eq!(e.to_string(), "Deploy failed: timeout");
}

#[test]
fn deploy_error_rollback_failed_display() {
    let e = DeployError::RollbackFailed("no snapshot".into());
    assert_eq!(e.to_string(), "Rollback failed: no snapshot");
}

#[test]
fn deploy_error_adapter_display() {
    let e = DeployError::AdapterError("unsupported op".into());
    assert_eq!(e.to_string(), "Adapter error: unsupported op");
}

#[test]
fn deploy_error_implements_std_error() {
    let e = DeployError::ConfigError("x".into());
    let _: &dyn std::error::Error = &e;
}

#[test]
fn deploy_error_clone_preserves_message() {
    let e = DeployError::BuildFailed("orig".into());
    let c = e.clone();
    assert_eq!(format!("{}", c), "Build failed: orig");
}

#[test]
fn deploy_error_debug_format() {
    let e = DeployError::DeployFailed("x".into());
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("DeployFailed"));
}

// ── ServiceConfig & RestartPolicy ──────────────────────────────────

#[test]
fn service_config_unit_has_all_required_sections() {
    let svc = ServiceConfig::new("myapp", "My App Service", "/usr/bin/myapp");
    let unit = svc.generate_unit();
    assert!(unit.contains("[Unit]"));
    assert!(unit.contains("Description=My App Service"));
    assert!(unit.contains("After=network.target"));
    assert!(unit.contains("[Service]"));
    assert!(unit.contains("Type=simple"));
    assert!(unit.contains("ExecStart=/usr/bin/myapp"));
    assert!(unit.contains("Restart=on-failure"));
    assert!(unit.contains("[Install]"));
    assert!(unit.contains("WantedBy=multi-user.target"));
}

#[test]
fn service_config_user_group_workdir_appear_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.user = Some("appuser".into());
    svc.group = Some("appgroup".into());
    svc.working_dir = Some("/var/lib/app".into());
    let unit = svc.generate_unit();
    assert!(unit.contains("User=appuser"));
    assert!(unit.contains("Group=appgroup"));
    assert!(unit.contains("WorkingDirectory=/var/lib/app"));
}

#[test]
fn service_config_minimal_has_no_user_group_workdir_lines() {
    let svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    let unit = svc.generate_unit();
    assert!(!unit.contains("User="));
    assert!(!unit.contains("Group="));
    assert!(!unit.contains("WorkingDirectory="));
}

#[test]
fn service_config_environment_var_quoted() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.environment
        .insert("APP_ENV".into(), "production".into());
    let unit = svc.generate_unit();
    assert!(unit.contains("Environment=\"APP_ENV=production\""));
}

#[test]
fn service_config_extra_directive_appears_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.extra_service
        .insert("LimitNOFILE".into(), "65535".into());
    let unit = svc.generate_unit();
    assert!(unit.contains("LimitNOFILE=65535"));
}

#[test]
fn restart_policy_always_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::Always;
    assert!(svc.generate_unit().contains("Restart=always"));
}

#[test]
fn restart_policy_no_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::No;
    assert!(svc.generate_unit().contains("Restart=no"));
}

#[test]
fn restart_policy_on_abnormal_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::OnAbnormal;
    assert!(svc.generate_unit().contains("Restart=on-abnormal"));
}

#[test]
fn restart_policy_on_abort_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::OnAbort;
    assert!(svc.generate_unit().contains("Restart=on-abort"));
}

#[test]
fn restart_policy_on_watchdog_in_unit() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::OnWatchdog;
    assert!(svc.generate_unit().contains("Restart=on-watchdog"));
}

#[test]
fn restart_policy_default_is_on_failure() {
    let svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    assert!(svc.generate_unit().contains("Restart=on-failure"));
}

#[test]
fn service_config_timer_contains_timer_section() {
    let svc = ServiceConfig::new("cron-app", "Cron App", "/usr/bin/cron-app");
    let timer = svc.generate_timer("hourly");
    assert!(timer.contains("[Timer]"));
    assert!(timer.contains("OnCalendar=hourly"));
    assert!(timer.contains("Persistent=true"));
    assert!(timer.contains("Unit=cron-app.service"));
    assert!(timer.contains("[Install]"));
    assert!(timer.contains("WantedBy=timers.target"));
}

#[test]
fn service_config_timer_calendar_syntax_preserved() {
    let svc = ServiceConfig::new("backup", "Backup", "/usr/bin/backup");
    let timer = svc.generate_timer("*-*-* 03:00:00");
    assert!(timer.contains("OnCalendar=*-*-* 03:00:00"));
    assert!(timer.contains("Description=Timer for Backup"));
}

#[test]
fn service_config_timer_references_correct_service_name() {
    let svc = ServiceConfig::new("sync", "Sync Service", "/usr/bin/sync");
    let timer = svc.generate_timer("15min");
    assert!(timer.contains("Unit=sync.service"));
}

#[test]
fn service_config_debug_format_contains_struct_name() {
    let svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    let dbg = format!("{:?}", svc);
    assert!(dbg.contains("ServiceConfig"));
}

#[test]
fn service_config_clone_deep_copies() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.user = Some("root".into());
    let c = svc.clone();
    assert_eq!(c.user.as_deref(), Some("root"));
}

// ── DebPackage ──────────────────────────────────────────────────────

#[test]
fn deb_package_default_arch_is_amd64() {
    let pkg = DebPackage::new("app", "1.0.0", "desc", "maint <m@m.com>");
    assert_eq!(pkg.architecture, "amd64");
}

#[test]
fn deb_package_initial_collections_empty() {
    let pkg = DebPackage::new("myapp", "1.0.0", "desc", "m");
    assert!(pkg.dependencies.is_empty());
    assert!(pkg.files.is_empty());
    assert!(pkg.config_files.is_empty());
    assert!(pkg.postinst.is_none());
    assert!(pkg.prerm.is_none());
    assert!(pkg.extra_fields.is_empty());
}

#[test]
fn deb_control_contains_basic_fields() {
    let pkg = DebPackage::new("myapp", "2.0.1", "My Application", "Dev <dev@dev.com>");
    let ctrl = pkg.generate_control();
    assert!(ctrl.contains("Package: myapp"));
    assert!(ctrl.contains("Version: 2.0.1"));
    assert!(ctrl.contains("Architecture: amd64"));
    assert!(ctrl.contains("Maintainer: Dev <dev@dev.com>"));
    assert!(ctrl.contains("Description: My Application"));
}

#[test]
fn deb_control_no_depends_line_when_empty() {
    let pkg = DebPackage::new("app", "1.0.0", "d", "m");
    assert!(!pkg.generate_control().contains("Depends:"));
}

#[test]
fn deb_control_depends_multiple_comma_separated() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.dependencies.push("libc6 (>= 2.31)".into());
    pkg.dependencies.push("libssl3".into());
    let ctrl = pkg.generate_control();
    assert!(ctrl.contains("Depends: libc6 (>= 2.31), libssl3"));
}

#[test]
fn deb_control_single_dependency() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.dependencies.push("curl".into());
    assert!(pkg.generate_control().contains("Depends: curl"));
}

#[test]
fn deb_control_extra_fields_included() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.extra_fields.insert("Section".into(), "utils".into());
    pkg.extra_fields
        .insert("Priority".into(), "optional".into());
    let ctrl = pkg.generate_control();
    assert!(ctrl.contains("Section: utils"));
    assert!(ctrl.contains("Priority: optional"));
}

#[test]
fn deb_conffiles_empty_string_when_no_configs() {
    let pkg = DebPackage::new("app", "1.0.0", "d", "m");
    assert_eq!(pkg.generate_conffiles(), "");
}

#[test]
fn deb_conffiles_lists_dest_paths() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.add_config("/dev/null", "/etc/app/config.toml");
    pkg.add_config("/dev/null", "/etc/app/plugin.toml");
    let conffiles = pkg.generate_conffiles();
    assert!(conffiles.contains("/etc/app/config.toml"));
    assert!(conffiles.contains("/etc/app/plugin.toml"));
}

#[test]
fn deb_set_postinst_stores_script() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.set_postinst("#!/bin/sh\necho post");
    assert_eq!(pkg.postinst.as_deref(), Some("#!/bin/sh\necho post"));
}

#[test]
fn deb_set_prerm_stores_script() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.set_prerm("#!/bin/sh\necho pre");
    assert_eq!(pkg.prerm.as_deref(), Some("#!/bin/sh\necho pre"));
}

#[test]
fn deb_add_file_increments_files_count() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.add_file("/src/bin1", "/usr/bin/app1");
    pkg.add_file("/src/bin2", "/usr/bin/app2");
    assert_eq!(pkg.files.len(), 2);
}

#[test]
fn deb_add_config_increments_config_files_count() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.add_config("/src/conf", "/etc/app/config");
    assert_eq!(pkg.config_files.len(), 1);
}

#[test]
fn deb_file_dest_path_preserved_correctly() {
    let mut pkg = DebPackage::new("app", "1.0.0", "d", "m");
    pkg.add_file("/src/bin", "/usr/bin/app");
    assert_eq!(pkg.files[0].dest, std::path::PathBuf::from("/usr/bin/app"));
}

#[test]
fn deb_build_creates_control_postinst_prerm_and_files() {
    let tmp = std::env::temp_dir().join("hudhud_deb_build_ext_test");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let mut pkg = DebPackage::new("test-pkg", "1.0.0", "Test", "dev <d@d.com>");
    pkg.set_postinst("#!/bin/sh\necho post");
    pkg.set_prerm("#!/bin/sh\necho pre");

    let src_bin = tmp.join("bin");
    std::fs::write(&src_bin, "binary").unwrap();
    pkg.add_file(&src_bin, "/usr/bin/test-pkg");

    let src_cfg = tmp.join("cfg.toml");
    std::fs::write(&src_cfg, "key = true").unwrap();
    pkg.add_config(&src_cfg, "/etc/test-pkg/config.toml");

    let out = tmp.join("out");
    let root = pkg.build(&out).unwrap();
    assert!(root.join("DEBIAN/control").exists());
    assert!(root.join("DEBIAN/postinst").exists());
    assert!(root.join("DEBIAN/prerm").exists());
    assert!(root.join("DEBIAN/conffiles").exists());
    assert!(root.join("usr/bin/test-pkg").exists());
    assert!(root.join("etc/test-pkg/config.toml").exists());

    let ctrl = std::fs::read_to_string(root.join("DEBIAN/control")).unwrap();
    assert!(ctrl.contains("Package: test-pkg"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn deb_build_dir_name_is_name_version_arch() {
    let tmp = std::env::temp_dir().join("hudhud_deb_dirname_ext_test");
    let _ = std::fs::remove_dir_all(&tmp);
    let pkg = DebPackage::new("myapp", "1.2.3", "desc", "m");
    let root = pkg.build(&tmp).unwrap();
    let dir_name = root.file_name().unwrap().to_str().unwrap();
    assert_eq!(dir_name, "myapp_1.2.3_amd64");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn deb_build_no_conffiles_file_when_no_configs() {
    let tmp = std::env::temp_dir().join("hudhud_deb_no_conffiles_ext");
    let _ = std::fs::remove_dir_all(&tmp);
    let pkg = DebPackage::new("nocfg", "1.0.0", "d", "m");
    let root = pkg.build(&tmp).unwrap();
    assert!(root.join("DEBIAN/control").exists());
    assert!(!root.join("DEBIAN/conffiles").exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

// ── Bundle ──────────────────────────────────────────────────────────

#[test]
fn bundle_new_has_correct_initial_state() {
    let b = Bundle::new("myapp", "1.0.0", "main.hud");
    assert_eq!(b.name, "myapp");
    assert_eq!(b.version, "1.0.0");
    assert_eq!(b.entry_point, "main.hud");
    assert!(b.scripts.is_empty());
    assert!(b.assets.is_empty());
    assert!(b.config_files.is_empty());
}

#[test]
fn bundle_add_script_adds_to_scripts_vec() {
    let mut b = Bundle::new("app", "1.0.0", "main.hud");
    b.add_script("/scripts/main.hud");
    b.add_script("/scripts/helper.hud");
    assert_eq!(b.scripts.len(), 2);
}

#[test]
fn bundle_add_asset_stores_dest_path() {
    let mut b = Bundle::new("app", "1.0.0", "main.hud");
    b.add_asset("/src/logo.png", "logo.png");
    assert_eq!(b.assets.len(), 1);
    assert_eq!(b.assets[0].dest.to_str().unwrap(), "logo.png");
}

#[test]
fn bundle_add_config_template_adds_to_config_files() {
    let mut b = Bundle::new("app", "1.0.0", "main.hud");
    b.add_config_template("/src/conf.toml", "conf.toml");
    assert_eq!(b.config_files.len(), 1);
}

#[test]
fn bundle_wrapper_script_has_shebang_and_exec() {
    let b = Bundle::new("hudhud-app", "2.0.0", "start.hud");
    let wrapper = b.generate_wrapper_script();
    assert!(wrapper.contains("#!/bin/sh"));
    assert!(wrapper.contains("HUDHUD_HOME=\"/usr/share/hudhud/hudhud-app\""));
    assert!(wrapper.contains("HUDHUD_CONFIG=\"/etc/hudhud/hudhud-app\""));
    assert!(wrapper.contains("exec hudhudscript \"$HUDHUD_HOME/scripts/start.hud\""));
}

#[test]
fn bundle_wrapper_script_contains_name_and_version() {
    let b = Bundle::new("myapp", "3.1.4", "run.hud");
    let wrapper = b.generate_wrapper_script();
    assert!(wrapper.contains("myapp"));
    assert!(wrapper.contains("3.1.4"));
}

#[test]
fn bundle_create_directory_structure_creates_expected_dirs() {
    let tmp = std::env::temp_dir().join("hudhud_bundle_dir_ext");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let script_src = tmp.join("main.hud");
    std::fs::write(&script_src, "print(1)").unwrap();

    let mut b = Bundle::new("smoke-test", "0.1.0", "main.hud");
    b.add_script(&script_src);

    let base = tmp.join("base");
    b.create_directory_structure(&base).unwrap();

    assert!(base
        .join("usr/share/hudhud/smoke-test/scripts/main.hud")
        .exists());
    assert!(base.join("usr/share/hudhud/plugins").exists());
    assert!(base.join("etc/hudhud/smoke-test").exists());
    assert!(base.join("usr/bin/smoke-test").exists());

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn bundle_create_directory_structure_wrapper_contains_exec() {
    let tmp = std::env::temp_dir().join("hudhud_bundle_wrapper_check_ext");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let b = Bundle::new("wrapper-test", "1.0.0", "app.hud");
    let base = tmp.join("base");
    b.create_directory_structure(&base).unwrap();

    let content = std::fs::read_to_string(base.join("usr/bin/wrapper-test")).unwrap();
    assert!(content.contains("exec hudhudscript"));

    let _ = std::fs::remove_dir_all(&tmp);
}

// ============================================================================
// From lib.rs inline tests
// ============================================================================

#[test]
fn test_deploy_plan_creation() {
    let plan = DeployPlan {
        app_name: "my-app".to_string(),
        targets: vec![Target {
            platform: TargetPlatform::Web,
            framework: "html".to_string(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    assert_eq!(plan.app_name, "my-app");
    assert_eq!(plan.targets.len(), 1);
}

#[test]
fn test_target_serialization() {
    let target = Target {
        platform: TargetPlatform::Desktop,
        framework: "tauri".to_string(),
        config: {
            let mut c = HashMap::new();
            c.insert("os".to_string(), "linux".to_string());
            c
        },
    };
    let json = serde_json::to_string(&target).unwrap();
    assert!(json.contains("tauri"));
}

#[test]
fn test_deploy_error_display_config() {
    let err = DeployError::ConfigError("missing field".to_string());
    assert_eq!(err.to_string(), "Config error: missing field");
}

#[test]
fn test_deploy_error_display_build_failed() {
    let err = DeployError::BuildFailed("compile error".to_string());
    assert_eq!(err.to_string(), "Build failed: compile error");
}

#[test]
fn test_deploy_error_display_deploy_failed() {
    let err = DeployError::DeployFailed("timeout".to_string());
    assert_eq!(err.to_string(), "Deploy failed: timeout");
}

#[test]
fn test_deploy_error_display_rollback_failed() {
    let err = DeployError::RollbackFailed("no previous version".to_string());
    assert_eq!(err.to_string(), "Rollback failed: no previous version");
}

#[test]
fn test_deploy_error_display_adapter() {
    let err = DeployError::AdapterError("unsupported".to_string());
    assert_eq!(err.to_string(), "Adapter error: unsupported");
}

#[test]
fn test_deploy_error_is_std_error() {
    let err = DeployError::ConfigError("test".to_string());
    let _: &dyn std::error::Error = &err;
}

#[test]
fn test_target_platform_custom_serialization() {
    let target = Target {
        platform: TargetPlatform::Custom("embedded".to_string()),
        framework: "bare-metal".to_string(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&target).unwrap();
    assert!(json.contains("embedded"));
}

#[test]
fn test_target_platform_wasm_serialization() {
    let target = Target {
        platform: TargetPlatform::Wasm,
        framework: "wasm-pack".to_string(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&target).unwrap();
    let deserialized: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.framework, "wasm-pack");
}

#[test]
fn test_target_platform_mobile_serialization() {
    let target = Target {
        platform: TargetPlatform::Mobile,
        framework: "flutter".to_string(),
        config: HashMap::new(),
    };
    let json = serde_json::to_string(&target).unwrap();
    let deserialized: Target = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.framework, "flutter");
}

#[test]
fn test_deploy_plan_with_docker_and_k8s() {
    let plan = DeployPlan {
        app_name: "full-app".to_string(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "myorg/app".to_string(),
            registry: "ghcr.io".to_string(),
            dockerfile: Some("Dockerfile.prod".to_string()),
        }),
        kubernetes: Some(KubernetesConfig {
            namespace: "prod".to_string(),
            replicas: 3,
            resources: HashMap::new(),
        }),
    };
    let json = serde_json::to_string(&plan).unwrap();
    assert!(json.contains("full-app"));
    assert!(json.contains("ghcr.io"));
    assert!(json.contains("prod"));
}

#[test]
fn test_pipeline_serialization() {
    let pipeline = Pipeline {
        provider: CIProvider::GitHub,
        config: HashMap::new(),
        triggers: vec![Trigger {
            event: "push".to_string(),
            branch: Some("main".to_string()),
            cron: None,
        }],
        steps: vec![
            PipelineStep {
                name: "test".to_string(),
                action: StepAction::Test,
            },
            PipelineStep {
                name: "custom".to_string(),
                action: StepAction::Custom {
                    command: "echo hello".to_string(),
                },
            },
        ],
    };
    let json = serde_json::to_string(&pipeline).unwrap();
    assert!(json.contains("push"));
    assert!(json.contains("main"));
    assert!(json.contains("echo hello"));
}

#[test]
fn test_ci_provider_custom_serialization() {
    let provider = CIProvider::Custom("CircleCI".to_string());
    let json = serde_json::to_string(&provider).unwrap();
    assert!(json.contains("CircleCI"));
}

#[test]
fn test_step_action_docker_serialization() {
    let step = PipelineStep {
        name: "docker-push".to_string(),
        action: StepAction::Docker {
            image: "myimg".to_string(),
            registry: "docker.io".to_string(),
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("docker.io"));
}

#[test]
fn test_step_action_deploy_serialization() {
    let step = PipelineStep {
        name: "deploy-web".to_string(),
        action: StepAction::Deploy {
            target: Target {
                platform: TargetPlatform::Web,
                framework: "nextjs".to_string(),
                config: HashMap::new(),
            },
            host: "prod.example.com".to_string(),
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("prod.example.com"));
}

#[test]
fn test_trigger_with_cron() {
    let trigger = Trigger {
        event: "schedule".to_string(),
        branch: None,
        cron: Some("0 3 * * *".to_string()),
    };
    let json = serde_json::to_string(&trigger).unwrap();
    assert!(json.contains("0 3 * * *"));
}

#[test]
fn test_ci_provider_gitlab_serialization() {
    let provider = CIProvider::GitLab;
    let json = serde_json::to_string(&provider).unwrap();
    assert!(json.contains("GitLab"));
}

#[test]
fn test_ci_provider_jenkins_serialization() {
    let provider = CIProvider::Jenkins;
    let json = serde_json::to_string(&provider).unwrap();
    assert!(json.contains("Jenkins"));
}

#[test]
fn test_step_action_build_serialization() {
    let step = PipelineStep {
        name: "build".to_string(),
        action: StepAction::Build {
            target: Target {
                platform: TargetPlatform::Web,
                framework: "react".to_string(),
                config: HashMap::new(),
            },
        },
    };
    let json = serde_json::to_string(&step).unwrap();
    assert!(json.contains("Build"));
    assert!(json.contains("react"));
}

#[test]
fn test_deploy_artifact_debug() {
    let artifact = DeployArtifact {
        filename: "Dockerfile".to_string(),
        content: "FROM rust:1.70".to_string(),
    };
    let debug = format!("{:?}", artifact);
    assert!(debug.contains("Dockerfile"));
}

#[test]
fn test_deploy_result_debug() {
    let result = DeployResult {
        success: true,
        url: Some("https://app.example.com".to_string()),
        message: "deployed successfully".to_string(),
    };
    let debug = format!("{:?}", result);
    assert!(debug.contains("deployed successfully"));
}

// ============================================================================
// From systemd.rs inline tests
// ============================================================================

#[test]
fn test_generate_unit_basic() {
    let svc = ServiceConfig::new("hudhud-app", "HudHud Application", "/usr/bin/hudhud-app");
    let unit = svc.generate_unit();

    assert!(unit.contains("[Unit]"));
    assert!(unit.contains("Description=HudHud Application"));
    assert!(unit.contains("After=network.target"));
    assert!(unit.contains("[Service]"));
    assert!(unit.contains("ExecStart=/usr/bin/hudhud-app"));
    assert!(unit.contains("Restart=on-failure"));
    assert!(unit.contains("[Install]"));
    assert!(unit.contains("WantedBy=multi-user.target"));
}

#[test]
fn test_generate_unit_with_user_env() {
    let mut svc = ServiceConfig::new("hudhud-app", "HudHud Application", "/usr/bin/hudhud-app");
    svc.user = Some("hudhud".to_string());
    svc.group = Some("hudhud".to_string());
    svc.working_dir = Some("/var/lib/hudhud".to_string());
    svc.restart_policy = RestartPolicy::Always;
    svc.environment
        .insert("HUD_ENV".to_string(), "production".to_string());

    let unit = svc.generate_unit();
    assert!(unit.contains("User=hudhud"));
    assert!(unit.contains("Group=hudhud"));
    assert!(unit.contains("WorkingDirectory=/var/lib/hudhud"));
    assert!(unit.contains("Restart=always"));
    assert!(unit.contains("Environment=\"HUD_ENV=production\""));
}

#[test]
fn test_generate_timer() {
    let svc = ServiceConfig::new("hudhud-cron", "HudHud Cron", "/usr/bin/hudhud-cron");
    let timer = svc.generate_timer("hourly");

    assert!(timer.contains("[Timer]"));
    assert!(timer.contains("OnCalendar=hourly"));
    assert!(timer.contains("Persistent=true"));
    assert!(timer.contains("Unit=hudhud-cron.service"));
    assert!(timer.contains("WantedBy=timers.target"));
}

#[test]
fn test_restart_policy_no() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::No;
    let unit = svc.generate_unit();
    assert!(unit.contains("Restart=no"));
}

#[test]
fn test_restart_policy_on_abnormal() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::OnAbnormal;
    let unit = svc.generate_unit();
    assert!(unit.contains("Restart=on-abnormal"));
}

#[test]
fn test_restart_policy_on_abort() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::OnAbort;
    let unit = svc.generate_unit();
    assert!(unit.contains("Restart=on-abort"));
}

#[test]
fn test_restart_policy_on_watchdog() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.restart_policy = RestartPolicy::OnWatchdog;
    let unit = svc.generate_unit();
    assert!(unit.contains("Restart=on-watchdog"));
}

#[test]
fn test_restart_policy_default_is_on_failure() {
    let policy = RestartPolicy::default();
    assert_eq!(policy.as_str(), "on-failure");
}

#[test]
fn test_extra_service_directives() {
    let mut svc = ServiceConfig::new("app", "App", "/usr/bin/app");
    svc.extra_service
        .insert("LimitNOFILE".to_string(), "65535".to_string());
    let unit = svc.generate_unit();
    assert!(unit.contains("LimitNOFILE=65535"));
}

#[test]
fn test_generate_timer_calendar_syntax() {
    let svc = ServiceConfig::new("backup", "Backup", "/usr/bin/backup");
    let timer = svc.generate_timer("*-*-* 03:00:00");
    assert!(timer.contains("OnCalendar=*-*-* 03:00:00"));
    assert!(timer.contains("Description=Timer for Backup"));
}

// ============================================================================
// From deb.rs inline tests
// ============================================================================

#[test]
fn test_generate_control_basic() {
    let pkg = DebPackage::new("hudhud-app", "1.0.0", "A test app", "Test <test@test.com>");
    let control = pkg.generate_control();
    assert!(control.contains("Package: hudhud-app"));
    assert!(control.contains("Version: 1.0.0"));
    assert!(control.contains("Architecture: amd64"));
    assert!(control.contains("Maintainer: Test <test@test.com>"));
    assert!(control.contains("Description: A test app"));
    // No Depends line when empty
    assert!(!control.contains("Depends:"));
}

#[test]
fn test_generate_control_with_dependencies() {
    let mut pkg = DebPackage::new("hudhud-app", "1.0.0", "A test app", "Test <t@t.com>");
    pkg.dependencies.push("libc6 (>= 2.31)".to_string());
    pkg.dependencies.push("libssl3".to_string());
    let control = pkg.generate_control();
    assert!(control.contains("Depends: libc6 (>= 2.31), libssl3"));
}

#[test]
fn test_generate_conffiles() {
    let mut pkg = DebPackage::new("x", "1.0.0", "d", "m");
    pkg.add_config("/dev/null", "/etc/hudhud/config.toml");
    pkg.add_config("/dev/null", "/etc/hudhud/plugins.toml");
    let conffiles = pkg.generate_conffiles();
    assert!(conffiles.contains("/etc/hudhud/config.toml"));
    assert!(conffiles.contains("/etc/hudhud/plugins.toml"));
}

#[test]
fn test_build_creates_structure() {
    let tmp = std::env::temp_dir().join("deb_test_build");
    let _ = std::fs::remove_dir_all(&tmp);

    let mut pkg = DebPackage::new("hudhud-test", "0.1.0", "test", "m");
    pkg.set_postinst("#!/bin/sh\necho postinst");
    pkg.set_prerm("#!/bin/sh\necho prerm");

    // Create a temp source file to copy
    let src_file = tmp.join("src_bin");
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(&src_file, "binary-content").unwrap();
    pkg.add_file(&src_file, "/usr/bin/hudhud-test");

    let conf_file = tmp.join("src_conf");
    std::fs::write(&conf_file, "key = true").unwrap();
    pkg.add_config(&conf_file, "/etc/hudhud/config.toml");

    let out_dir = tmp.join("output");
    let pkg_root = pkg.build(&out_dir).unwrap();

    assert!(pkg_root.join("DEBIAN/control").exists());
    assert!(pkg_root.join("DEBIAN/postinst").exists());
    assert!(pkg_root.join("DEBIAN/prerm").exists());
    assert!(pkg_root.join("DEBIAN/conffiles").exists());
    assert!(pkg_root.join("usr/bin/hudhud-test").exists());
    assert!(pkg_root.join("etc/hudhud/config.toml").exists());

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_generate_control_with_extra_fields() {
    let mut pkg = DebPackage::new("app", "1.0.0", "desc", "maint");
    pkg.extra_fields
        .insert("Section".to_string(), "utils".to_string());
    pkg.extra_fields
        .insert("Priority".to_string(), "optional".to_string());
    let control = pkg.generate_control();
    assert!(control.contains("Section: utils"));
    assert!(control.contains("Priority: optional"));
}

#[test]
fn test_generate_conffiles_empty() {
    let pkg = DebPackage::new("app", "1.0.0", "desc", "maint");
    let conffiles = pkg.generate_conffiles();
    assert_eq!(conffiles, "");
}

#[test]
fn test_deb_package_default_architecture() {
    let pkg = DebPackage::new("app", "1.0.0", "desc", "maint");
    assert_eq!(pkg.architecture, "amd64");
}

#[test]
fn test_deb_package_set_scripts() {
    let mut pkg = DebPackage::new("app", "1.0.0", "desc", "maint");
    pkg.set_postinst("#!/bin/sh\necho post");
    pkg.set_prerm("#!/bin/sh\necho pre");
    assert_eq!(pkg.postinst, Some("#!/bin/sh\necho post".to_string()));
    assert_eq!(pkg.prerm, Some("#!/bin/sh\necho pre".to_string()));
}

#[test]
fn test_add_file_and_config() {
    let mut pkg = DebPackage::new("app", "1.0.0", "desc", "maint");
    pkg.add_file("/src/bin", "/usr/bin/app");
    pkg.add_config("/src/conf", "/etc/app/config");
    assert_eq!(pkg.files.len(), 1);
    assert_eq!(pkg.config_files.len(), 1);
    assert_eq!(pkg.files[0].dest, std::path::PathBuf::from("/usr/bin/app"));
    assert_eq!(
        pkg.config_files[0].dest,
        std::path::PathBuf::from("/etc/app/config")
    );
}

// ============================================================================
// From bundle.rs inline tests
// ============================================================================

#[test]
fn test_generate_wrapper_script() {
    let bundle = Bundle::new("my-app", "1.0.0", "main.hud");
    let wrapper = bundle.generate_wrapper_script();

    assert!(wrapper.contains("#!/bin/sh"));
    assert!(wrapper.contains("HUDHUD_HOME=\"/usr/share/hudhud/my-app\""));
    assert!(wrapper.contains("HUDHUD_CONFIG=\"/etc/hudhud/my-app\""));
    assert!(wrapper.contains("exec hudhudscript \"$HUDHUD_HOME/scripts/main.hud\""));
}

#[test]
fn test_create_directory_structure() {
    let tmp = std::env::temp_dir().join("bundle_test_structure");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Create a dummy script source file
    let script_src = tmp.join("main.hud");
    std::fs::write(&script_src, "yazdır(\"merhaba\")").unwrap();

    // Create a dummy asset
    let asset_src = tmp.join("logo.png");
    std::fs::write(&asset_src, "PNG-DATA").unwrap();

    // Create a dummy config
    let cfg_src = tmp.join("config.toml");
    std::fs::write(&cfg_src, "port = 8080").unwrap();

    let mut bundle = Bundle::new("test-app", "0.1.0", "main.hud");
    bundle.add_script(&script_src);
    bundle.add_asset(&asset_src, "logo.png");
    bundle.add_config_template(&cfg_src, "config.toml");

    let base = tmp.join("pkg");
    bundle.create_directory_structure(&base).unwrap();

    assert!(base
        .join("usr/share/hudhud/test-app/scripts/main.hud")
        .exists());
    assert!(base
        .join("usr/share/hudhud/test-app/assets/logo.png")
        .exists());
    assert!(base.join("usr/share/hudhud/plugins").exists());
    assert!(base.join("etc/hudhud/test-app/config.toml").exists());
    assert!(base.join("usr/bin/test-app").exists());

    // Verify wrapper content
    let wrapper = std::fs::read_to_string(base.join("usr/bin/test-app")).unwrap();
    assert!(wrapper.contains("exec hudhudscript"));

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_bundle_new_defaults() {
    let b = Bundle::new("app", "2.0.0", "entry.hud");
    assert_eq!(b.name, "app");
    assert_eq!(b.version, "2.0.0");
    assert_eq!(b.entry_point, "entry.hud");
    assert!(b.scripts.is_empty());
    assert!(b.assets.is_empty());
    assert!(b.config_files.is_empty());
}
