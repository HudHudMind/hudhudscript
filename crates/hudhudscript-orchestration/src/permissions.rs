//! Permission Sistemi — allow/deny kuralları, realm, RuntimeError::PermissionDenied (Issue #17)
//!
//! Agent başına izin kuralları, realm bazlı erişim kontrolü, event yayını.

use crate::events::{AgentEvent, EventBus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// İzin kuralı tipi
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleEffect {
    Allow,
    Deny,
}

/// Tek bir izin kuralı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// Kural etkisi: allow veya deny
    pub effect: RuleEffect,
    /// Kaynak (örn: "file", "network", "llm", "*")
    pub resource: String,
    /// Eylem (örn: "read", "write", "execute", "*")
    pub action: String,
    /// Realm kısıtlaması (None = tüm realm'lar)
    pub realm: Option<String>,
}

impl PermissionRule {
    pub fn allow(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            effect: RuleEffect::Allow,
            resource: resource.into(),
            action: action.into(),
            realm: None,
        }
    }

    pub fn deny(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            effect: RuleEffect::Deny,
            resource: resource.into(),
            action: action.into(),
            realm: None,
        }
    }

    pub fn with_realm(mut self, realm: impl Into<String>) -> Self {
        self.realm = Some(realm.into());
        self
    }

    /// Bu kural verilen istek için geçerli mi?
    fn matches(&self, resource: &str, action: &str, realm: Option<&str>) -> bool {
        let res_match = self.resource == "*" || self.resource == resource;
        let act_match = self.action == "*" || self.action == action;
        let realm_match = match (&self.realm, realm) {
            (Some(r), Some(req_r)) => r == req_r,
            (Some(_), None) => false,
            (None, _) => true,
        };
        res_match && act_match && realm_match
    }
}

/// Agent izin konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionConfig {
    pub rules: Vec<PermissionRule>,
    /// Varsayılan etki: kural eşleşmezse ne yapılsın
    pub default_effect: Option<RuleEffect>,
}

impl PermissionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(mut self, rule: PermissionRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn default_deny(mut self) -> Self {
        self.default_effect = Some(RuleEffect::Deny);
        self
    }

    pub fn default_allow(mut self) -> Self {
        self.default_effect = Some(RuleEffect::Allow);
        self
    }
}

/// Permission engine
pub struct PermissionEngine {
    event_bus: Arc<EventBus>,
    /// agent_id → PermissionConfig
    configs: Arc<RwLock<HashMap<String, PermissionConfig>>>,
}

impl PermissionEngine {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Agent için izin konfigürasyonu kaydet
    pub async fn register(&self, agent_id: String, config: PermissionConfig) {
        self.configs.write().await.insert(agent_id, config);
    }

    /// İzin kontrolü yap
    /// Deny kuralları allow'dan önce değerlendirilir (deny-first)
    pub async fn check(
        &self,
        agent_id: &str,
        resource: &str,
        action: &str,
        realm: Option<&str>,
    ) -> Result<(), PermissionError> {
        let configs = self.configs.read().await;
        let config = configs.get(agent_id);

        let allowed = match config {
            None => {
                // Kayıtlı kural yok → varsayılan olarak izin ver
                true
            }
            Some(cfg) => {
                // Deny kurallarını önce kontrol et
                let denied = cfg
                    .rules
                    .iter()
                    .filter(|r| r.effect == RuleEffect::Deny)
                    .any(|r| r.matches(resource, action, realm));

                if denied {
                    false
                } else {
                    // Allow kurallarını kontrol et
                    let explicitly_allowed = cfg
                        .rules
                        .iter()
                        .filter(|r| r.effect == RuleEffect::Allow)
                        .any(|r| r.matches(resource, action, realm));

                    if explicitly_allowed {
                        true
                    } else {
                        // Varsayılan etki
                        match &cfg.default_effect {
                            Some(RuleEffect::Allow) => true,
                            Some(RuleEffect::Deny) => false,
                            None => true, // kural yoksa izin ver
                        }
                    }
                }
            }
        };

        if !allowed {
            // PermissionDenied event yayınla
            let _ = self
                .event_bus
                .emit(AgentEvent::PermissionDenied {
                    agent_id: agent_id.to_string(),
                    resource: resource.to_string(),
                    action: action.to_string(),
                })
                .await;

            return Err(PermissionError::Denied {
                agent_id: agent_id.to_string(),
                resource: resource.to_string(),
                action: action.to_string(),
                realm: realm.map(|r| r.to_string()),
            });
        }

        Ok(())
    }

    /// Tüm izinleri listele
    pub async fn list_rules(&self, agent_id: &str) -> Vec<PermissionRule> {
        self.configs
            .read()
            .await
            .get(agent_id)
            .map(|c| c.rules.clone())
            .unwrap_or_default()
    }
}

/// Permission hataları
#[derive(Debug)]
pub enum PermissionError {
    Denied {
        agent_id: String,
        resource: String,
        action: String,
        realm: Option<String>,
    },
    AgentNotRegistered(String),
}

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            PermissionError::Denied {
                agent_id,
                resource,
                action,
                ..
            } => write!(
                f,
                "Permission denied: agent={}, resource={}, action={}",
                agent_id, resource, action
            ),
            PermissionError::AgentNotRegistered(s) => write!(f, "Agent not registered: {}", s),
        }
    }
}

impl std::error::Error for PermissionError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl PermissionError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            PermissionError::AgentNotRegistered(..) => {
                hudhudscript_errors::ErrorCode::PermissionAgentNotRegistered
            }
            PermissionError::Denied { .. } => hudhudscript_errors::ErrorCode::PermissionDenied,
        }
    }

    /// Catalog short code (e.g. `"E0120"`).
    pub fn short_code(&self) -> &'static str {
        self.code().short_code()
    }

    /// Catalog title.
    pub fn title(&self) -> &'static str {
        self.code().title()
    }

    /// Render with full catalog metadata: `[E0XXX] Title — message`.
    pub fn display_full(&self) -> String {
        let entry = self.code().entry();
        format!("[{}] {} — {}", entry.short_code, entry.title, self)
    }
}

impl From<PermissionError> for hudhudscript_errors::Error {
    fn from(e: PermissionError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
