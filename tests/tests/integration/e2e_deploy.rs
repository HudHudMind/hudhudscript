//! E2E Deploy pipeline tests (#560)
//!
//! Tests the full flow: build DeployPlan → adapter create → generate → verify artifacts

use hudhudscript_deploy_core::adapters::*;
use hudhudscript_deploy_core::*;
use std::collections::HashMap;

fn web_plan() -> DeployPlan {
    DeployPlan {
        app_name: "test-web".into(),
        targets: vec![Target {
            platform: TargetPlatform::Web,
            framework: "nextjs".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    }
}

fn fullstack_plan() -> DeployPlan {
    DeployPlan {
        app_name: "fullstack".into(),
        targets: vec![
            Target {
                platform: TargetPlatform::Web,
                framework: "nextjs".into(),
                config: HashMap::new(),
            },
            Target {
                platform: TargetPlatform::Desktop,
                framework: "tauri".into(),
                config: HashMap::new(),
            },
            Target {
                platform: TargetPlatform::Mobile,
                framework: "flutter".into(),
                config: HashMap::new(),
            },
            Target {
                platform: TargetPlatform::Wasm,
                framework: "wasm-bindgen".into(),
                config: HashMap::new(),
            },
        ],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "hudhud/fullstack".into(),
            registry: "ghcr.io".into(),
            dockerfile: None,
        }),
        kubernetes: Some(KubernetesConfig {
            namespace: "production".into(),
            replicas: 3,
            resources: {
                let mut r = HashMap::new();
                r.insert("cpu".into(), "500m".into());
                r.insert("memory".into(), "512Mi".into());
                r
            },
        }),
    }
}

// ── GitHub Actions ──────────────────────────────────────────────────

#[test]
fn test_github_web_workflow() {
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&web_plan()).unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].filename, ".github/workflows/deploy.yml");
    assert!(artifacts[0].content.contains("npm run build"));
    assert!(artifacts[0].content.contains("test-web"));
}

#[test]
fn test_github_multi_target() {
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&fullstack_plan()).unwrap();
    let content = &artifacts[0].content;
    assert!(content.contains("npm run build"));
    assert!(content.contains("cargo tauri build"));
    assert!(content.contains("flutter build apk"));
    assert!(content.contains("wasm-pack"));
}

// ── Docker ──────────────────────────────────────────────────────────

#[test]
fn test_docker_generates_dockerfile_and_compose() {
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    let artifacts = adapter.generate(&fullstack_plan()).unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[0].filename, "Dockerfile");
    assert_eq!(artifacts[1].filename, "docker-compose.yml");
    assert!(artifacts[0].content.contains("FROM"));
    assert!(artifacts[0].content.contains("EXPOSE 3000"));
    assert!(artifacts[1].content.contains("hudhud/fullstack"));
}

// ── Vercel ──────────────────────────────────────────────────────────

#[test]
fn test_vercel_nextjs_config() {
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    let artifacts = adapter.generate(&web_plan()).unwrap();
    assert_eq!(artifacts[0].filename, "vercel.json");
    assert!(artifacts[0].content.contains("nextjs"));
}

#[test]
fn test_vercel_wasm_config() {
    let plan = DeployPlan {
        app_name: "wasm-app".into(),
        targets: vec![Target {
            platform: TargetPlatform::Wasm,
            framework: "wasm-bindgen".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("wasm-pack"));
}

// ── Kubernetes ──────────────────────────────────────────────────────

#[test]
fn test_k8s_manifests() {
    let adapter = create_adapter(&Adapter::Kubernetes).unwrap();
    let artifacts = adapter.generate(&fullstack_plan()).unwrap();
    assert_eq!(artifacts.len(), 2);
    assert!(artifacts[0].filename.contains("deployment"));
    assert!(artifacts[1].filename.contains("service"));
    assert!(artifacts[0].content.contains("replicas: 3"));
    assert!(artifacts[0].content.contains("production"));
    assert!(artifacts[0].content.contains("ghcr.io/hudhud/fullstack"));
    assert!(artifacts[0].content.contains("500m"));
}

// ── Deploy result ───────────────────────────────────────────────────

#[test]
fn test_vercel_deploy_returns_url() {
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    let result = adapter.deploy(&web_plan()).unwrap();
    assert!(result.success);
    assert!(result.url.unwrap().contains("test-web.vercel.app"));
}

#[test]
fn test_github_deploy_requires_project_dir() {
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    // Real deploy needs project_dir for git operations
    let result = adapter.deploy(&web_plan());
    assert!(result.is_err());
}
