// Extracted from hudhudscript-deploy-bridge inline #[cfg(test)] blocks
use hudhudscript_deploy_core::adapters::{create_adapter, Adapter};
use hudhudscript_deploy_core::{
    DeployAdapter, DeployPlan, DockerConfig, KubernetesConfig, Target, TargetPlatform,
};
use std::collections::HashMap;

// === from lib.rs ===

#[test]
fn test_adapter_parsing() {
    assert!(matches!(Adapter::parse("github"), Some(Adapter::GitHub)));
    assert!(matches!(Adapter::parse("docker"), Some(Adapter::Docker)));
    assert!(matches!(Adapter::parse("k8s"), Some(Adapter::Kubernetes)));
}

#[test]
fn test_create_github_adapter() {
    let adapter = create_adapter(&Adapter::GitHub);
    assert!(adapter.is_ok());
    assert_eq!(adapter.unwrap().name(), "github");
}

#[test]
fn test_adapter_parse_gitlab() {
    assert!(matches!(Adapter::parse("gitlab"), Some(Adapter::GitLab)));
}

#[test]
fn test_adapter_parse_jenkins() {
    assert!(matches!(Adapter::parse("jenkins"), Some(Adapter::Jenkins)));
}

#[test]
fn test_adapter_parse_vercel() {
    assert!(matches!(Adapter::parse("vercel"), Some(Adapter::Vercel)));
}

#[test]
fn test_adapter_parse_kubernetes_full() {
    assert!(matches!(
        Adapter::parse("kubernetes"),
        Some(Adapter::Kubernetes)
    ));
}

#[test]
fn test_adapter_parse_custom() {
    match Adapter::parse("circleci") {
        Some(Adapter::Custom(name)) => assert_eq!(name, "circleci"),
        other => panic!("Expected Custom(\"circleci\"), got {:?}", other),
    }
}

#[test]
fn test_adapter_parse_case_insensitive() {
    assert!(matches!(Adapter::parse("GitHub"), Some(Adapter::GitHub)));
    assert!(matches!(Adapter::parse("DOCKER"), Some(Adapter::Docker)));
    assert!(matches!(Adapter::parse("K8S"), Some(Adapter::Kubernetes)));
}

#[test]
fn test_create_docker_adapter() {
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    assert_eq!(adapter.name(), "docker");
}

#[test]
fn test_create_vercel_adapter() {
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    assert_eq!(adapter.name(), "vercel");
}

#[test]
fn test_create_kubernetes_adapter() {
    let adapter = create_adapter(&Adapter::Kubernetes).unwrap();
    assert_eq!(adapter.name(), "kubernetes");
}

#[test]
fn test_create_gitlab_adapter_uses_stub() {
    let adapter = create_adapter(&Adapter::GitLab).unwrap();
    assert_eq!(adapter.name(), "unsupported(GitLab)");
}

#[test]
fn test_create_jenkins_adapter_uses_stub() {
    let adapter = create_adapter(&Adapter::Jenkins).unwrap();
    assert_eq!(adapter.name(), "unsupported(Jenkins)");
}

#[test]
fn test_create_custom_adapter_uses_stub() {
    let adapter = create_adapter(&Adapter::Custom("Travis".to_string())).unwrap();
    assert_eq!(adapter.name(), "unsupported(Custom(\"Travis\"))");
}

// === from stub.rs ===

fn empty_plan() -> DeployPlan {
    DeployPlan {
        app_name: "test-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    }
}

#[test]
fn test_unsupported_generate_returns_error() {
    let adapter = create_adapter(&Adapter::GitLab).unwrap();
    let result = adapter.generate(&empty_plan());
    assert!(
        result.is_err(),
        "Unsupported adapter should return error, not fake artifacts"
    );
}

#[test]
fn test_unsupported_deploy_returns_error() {
    let adapter = create_adapter(&Adapter::Jenkins).unwrap();
    let result = adapter.deploy(&empty_plan());
    assert!(
        result.is_err(),
        "Unsupported adapter should return error, not fake success"
    );
}

#[test]
fn test_unsupported_rollback_returns_error() {
    let adapter = create_adapter(&Adapter::Custom("Custom".to_string())).unwrap();
    let result = adapter.rollback("my-app");
    assert!(
        result.is_err(),
        "Unsupported adapter should return error, not fake success"
    );
}

#[test]
fn test_stub_name() {
    let adapter = create_adapter(&Adapter::GitLab).unwrap();
    assert_eq!(adapter.name(), "unsupported(GitLab)");
}

// === from docker.rs ===

#[test]
fn test_docker_generate() {
    let plan = DeployPlan {
        app_name: "my-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "myorg/my-app".into(),
            registry: "ghcr.io".into(),
            dockerfile: None,
        }),
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert_eq!(artifacts.len(), 2);
    assert!(artifacts[0].content.contains("FROM"));
    assert!(artifacts[1].content.contains("myorg/my-app"));
}

#[test]
fn test_docker_generate_without_docker_config() {
    let plan = DeployPlan {
        app_name: "bare-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert_eq!(artifacts.len(), 2);
    // Uses default base image
    assert!(artifacts[0].content.contains("node:20-alpine"));
    // Uses app_name as image name
    assert!(artifacts[1].content.contains("bare-app"));
}

#[test]
fn test_docker_generate_with_custom_dockerfile() {
    let plan = DeployPlan {
        app_name: "custom-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "org/custom".into(),
            registry: "docker.io".into(),
            dockerfile: Some("rust:1.70-slim".into()),
        }),
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("rust:1.70-slim"));
}

#[test]
fn test_docker_deploy_requires_project_dir() {
    let plan = DeployPlan {
        app_name: "my-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    // Real deploy requires project_dir — without it, returns clear error
    let result = adapter.deploy(&plan);
    assert!(result.is_err(), "Deploy without project_dir should error");
}

#[test]
fn test_docker_rollback() {
    let adapter = create_adapter(&Adapter::Docker).unwrap();
    assert!(adapter.rollback("my-app").is_ok());
}

#[test]
fn test_docker_name() {
    assert_eq!(create_adapter(&Adapter::Docker).unwrap().name(), "docker");
}

// === from github.rs ===

#[test]
fn test_github_generate_empty() {
    let plan = DeployPlan {
        app_name: "test-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].filename, ".github/workflows/deploy.yml");
    assert!(artifacts[0].content.contains("test-app"));
}

#[test]
fn test_github_generate_web_target() {
    let plan = DeployPlan {
        app_name: "web-app".into(),
        targets: vec![Target {
            platform: TargetPlatform::Web,
            framework: "nextjs".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("npm run build"));
}

#[test]
fn test_github_generate_desktop_target() {
    let plan = DeployPlan {
        app_name: "desk-app".into(),
        targets: vec![Target {
            platform: TargetPlatform::Desktop,
            framework: "tauri".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("cargo tauri build"));
    assert!(artifacts[0].content.contains("Build desktop (tauri)"));
}

#[test]
fn test_github_generate_mobile_target() {
    let plan = DeployPlan {
        app_name: "mobile-app".into(),
        targets: vec![Target {
            platform: TargetPlatform::Mobile,
            framework: "flutter".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("flutter build apk"));
    assert!(artifacts[0].content.contains("Build mobile (flutter)"));
}

#[test]
fn test_github_generate_wasm_target() {
    let plan = DeployPlan {
        app_name: "wasm-app".into(),
        targets: vec![Target {
            platform: TargetPlatform::Wasm,
            framework: "wasm-pack".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0]
        .content
        .contains("wasm-pack build --target web"));
    assert!(artifacts[0].content.contains("Build WASM"));
}

#[test]
fn test_github_generate_custom_target() {
    let plan = DeployPlan {
        app_name: "custom-app".into(),
        targets: vec![Target {
            platform: TargetPlatform::Custom("embedded".into()),
            framework: "bare".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("Build embedded"));
    assert!(artifacts[0].content.contains("Custom build for embedded"));
}

#[test]
fn test_github_deploy() {
    let plan = DeployPlan {
        app_name: "my-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::GitHub).unwrap();
    // Real deploy requires project_dir — without it, returns clear error
    let result = adapter.deploy(&plan);
    assert!(result.is_err(), "Deploy without project_dir should error");
}

#[test]
fn test_github_rollback_requires_project_dir() {
    let result = create_adapter(&Adapter::GitHub).unwrap().rollback("my-app");
    // Rollback also requires project_dir for real git operations
    assert!(result.is_err(), "Rollback without project_dir should error");
}

#[test]
fn test_github_name() {
    assert_eq!(create_adapter(&Adapter::GitHub).unwrap().name(), "github");
}

// === from k8s.rs ===

#[test]
fn test_k8s_generate() {
    let plan = DeployPlan {
        app_name: "my-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: Some(DockerConfig {
            image: "my-app".into(),
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
    };
    let adapter = create_adapter(&Adapter::Kubernetes).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert_eq!(artifacts.len(), 2);
    assert!(artifacts[0].content.contains("replicas: 3"));
    assert!(artifacts[0].content.contains("production"));
    assert!(artifacts[1].content.contains("Service"));
}

#[test]
fn test_k8s_generate_defaults_without_config() {
    let plan = DeployPlan {
        app_name: "bare-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Kubernetes).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert_eq!(artifacts.len(), 2);
    // Default namespace
    assert!(artifacts[0].content.contains("namespace: default"));
    // Default replicas
    assert!(artifacts[0].content.contains("replicas: 1"));
    // Default resources
    assert!(artifacts[0].content.contains("cpu: 250m"));
    assert!(artifacts[0].content.contains("memory: 256Mi"));
    // Falls back to app_name for image
    assert!(artifacts[0].content.contains("image: bare-app"));
}

#[test]
fn test_k8s_generate_partial_resources() {
    let plan = DeployPlan {
        app_name: "partial-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: Some(KubernetesConfig {
            namespace: "staging".into(),
            replicas: 2,
            resources: {
                let mut r = HashMap::new();
                r.insert("cpu".into(), "1000m".into());
                // no memory key -> should default to 256Mi
                r
            },
        }),
    };
    let adapter = create_adapter(&Adapter::Kubernetes).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("cpu: 1000m"));
    assert!(artifacts[0].content.contains("memory: 256Mi"));
}

#[test]
fn test_k8s_deploy() {
    let plan = DeployPlan {
        app_name: "my-app".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Kubernetes).unwrap();
    let result = adapter.deploy(&plan).unwrap();
    assert!(result.success);
    assert!(result.url.is_none());
    assert_eq!(
        result.message,
        "Kubernetes manifests generated for 'my-app'"
    );
}

#[test]
fn test_k8s_rollback() {
    assert!(create_adapter(&Adapter::Kubernetes)
        .unwrap()
        .rollback("my-app")
        .is_ok());
}

#[test]
fn test_k8s_name() {
    assert_eq!(
        create_adapter(&Adapter::Kubernetes).unwrap().name(),
        "kubernetes"
    );
}

// === from vercel.rs ===

#[test]
fn test_vercel_generate_nextjs() {
    let plan = DeployPlan {
        app_name: "my-web".into(),
        targets: vec![Target {
            platform: TargetPlatform::Web,
            framework: "nextjs".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert!(artifacts[0].content.contains("nextjs"));
}

#[test]
fn test_vercel_deploy_url() {
    let plan = DeployPlan {
        app_name: "my-web".into(),
        targets: vec![],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    let result = adapter.deploy(&plan).unwrap();
    assert_eq!(result.url.as_deref(), Some("https://my-web.vercel.app"));
    assert!(result.success);
    assert_eq!(result.message, "Vercel config generated for 'my-web'");
}

#[test]
fn test_vercel_generate_wasm_target() {
    let plan = DeployPlan {
        app_name: "wasm-web".into(),
        targets: vec![Target {
            platform: TargetPlatform::Wasm,
            framework: "wasm-pack".into(),
            config: HashMap::new(),
        }],
        pipelines: vec![],
        docker: None,
        kubernetes: None,
    };
    let adapter = create_adapter(&Adapter::Vercel).unwrap();
    let artifacts = adapter.generate(&plan).unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].filename, "vercel.json");
    assert!(artifacts[0]
        .content
        .contains("wasm-pack build --target web"));
    assert!(artifacts[0]
        .content
        .contains("\"outputDirectory\": \"pkg\""));
    assert!(artifacts[0].content.contains("\"framework\": null"));
}

#[test]
fn test_vercel_rollback() {
    assert!(create_adapter(&Adapter::Vercel)
        .unwrap()
        .rollback("my-web")
        .is_ok());
}

#[test]
fn test_vercel_name() {
    assert_eq!(create_adapter(&Adapter::Vercel).unwrap().name(), "vercel");
}
