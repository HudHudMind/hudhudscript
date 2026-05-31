//! Darbe Mekanizması — Constitution'a coup_condition/coup_authority (Issue #16)
//!
//! Güven skoru takibi, CoupTriggered event, audit log.

use crate::events::{AgentEvent, EventBus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Darbe tetikleme koşulu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoupCondition {
    /// Güven skoru bu değerin altına düşerse darbe tetiklenir (0.0–1.0)
    pub trust_threshold: f64,
    /// Arka arkaya kaç başarısız görev darbe tetikler
    pub consecutive_failures: usize,
}

impl Default for CoupCondition {
    fn default() -> Self {
        Self {
            trust_threshold: 0.3,
            consecutive_failures: 3,
        }
    }
}

/// Darbe konfigürasyonu (Constitution'a eklenir)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoupConfig {
    /// Darbe yetkisi olan agent ID'si
    pub coup_authority: String,
    /// Darbe koşulları
    pub coup_condition: CoupCondition,
    /// Audit log tutulsun mu
    pub audit_log: bool,
}

impl Default for CoupConfig {
    fn default() -> Self {
        Self {
            coup_authority: "system".to_string(),
            coup_condition: CoupCondition::default(),
            audit_log: true,
        }
    }
}

/// Agent güven durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrustState {
    pub agent_id: String,
    pub trust_score: f64,
    pub consecutive_failures: usize,
    pub total_tasks: usize,
    pub successful_tasks: usize,
}

impl AgentTrustState {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            trust_score: 1.0,
            consecutive_failures: 0,
            total_tasks: 0,
            successful_tasks: 0,
        }
    }

    /// Başarılı görev kaydı
    pub fn record_success(&mut self) {
        self.total_tasks += 1;
        self.successful_tasks += 1;
        self.consecutive_failures = 0;
        self.trust_score = (self.successful_tasks as f64 / self.total_tasks as f64).min(1.0);
    }

    /// Başarısız görev kaydı
    pub fn record_failure(&mut self) {
        self.total_tasks += 1;
        self.consecutive_failures += 1;
        self.trust_score = (self.successful_tasks as f64 / self.total_tasks as f64).max(0.0);
    }
}

/// Audit log kaydı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub event: String,
    pub agent_id: String,
    pub details: serde_json::Value,
}

/// Darbe sonucu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoupResult {
    pub triggered: bool,
    pub target_agent_id: String,
    pub reason: String,
    pub trust_score: f64,
}

/// Darbe executor
pub struct CoupExecutor {
    event_bus: Arc<EventBus>,
    configs: Arc<RwLock<HashMap<String, CoupConfig>>>,
    trust_states: Arc<RwLock<HashMap<String, AgentTrustState>>>,
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

impl CoupExecutor {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            event_bus,
            configs: Arc::new(RwLock::new(HashMap::new())),
            trust_states: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Constitution'a darbe konfigürasyonu kaydet
    pub async fn register(&self, constitution_id: String, config: CoupConfig) {
        self.configs.write().await.insert(constitution_id, config);
    }

    /// Agent güven durumunu başlat
    pub async fn init_agent(&self, agent_id: impl Into<String>) {
        let id = agent_id.into();
        self.trust_states
            .write()
            .await
            .entry(id.clone())
            .or_insert_with(|| AgentTrustState::new(id));
    }

    /// Görev sonucunu kaydet ve darbe kontrolü yap
    pub async fn record_task_result(
        &self,
        constitution_id: &str,
        agent_id: &str,
        success: bool,
    ) -> Result<Option<CoupResult>, CoupError> {
        // Trust state güncelle
        {
            let mut states = self.trust_states.write().await;
            let state = states
                .entry(agent_id.to_string())
                .or_insert_with(|| AgentTrustState::new(agent_id));
            if success {
                state.record_success();
            } else {
                state.record_failure();
            }
        }

        // Darbe koşulunu kontrol et
        self.check_coup(constitution_id, agent_id).await
    }

    /// Darbe koşulunu kontrol et
    pub async fn check_coup(
        &self,
        constitution_id: &str,
        agent_id: &str,
    ) -> Result<Option<CoupResult>, CoupError> {
        let config = {
            let configs = self.configs.read().await;
            configs.get(constitution_id).cloned().unwrap_or_default()
        };

        let state = {
            let states = self.trust_states.read().await;
            states.get(agent_id).cloned()
        };

        let state = match state {
            Some(s) => s,
            None => return Ok(None),
        };

        let trust_violated = state.trust_score < config.coup_condition.trust_threshold;
        let failures_violated =
            state.consecutive_failures >= config.coup_condition.consecutive_failures;

        if trust_violated || failures_violated {
            let reason = if trust_violated {
                format!(
                    "trust_score_below_threshold ({:.2} < {:.2})",
                    state.trust_score, config.coup_condition.trust_threshold
                )
            } else {
                format!(
                    "consecutive_failures_exceeded ({} >= {})",
                    state.consecutive_failures, config.coup_condition.consecutive_failures
                )
            };

            // Audit log
            if config.audit_log {
                self.append_audit(AuditEntry {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    event: "coup_triggered".to_string(),
                    agent_id: agent_id.to_string(),
                    details: serde_json::json!({
                        "reason": reason,
                        "trust_score": state.trust_score,
                        "consecutive_failures": state.consecutive_failures,
                        "authority": config.coup_authority,
                    }),
                })
                .await;
            }

            // CoupTriggered event yayınla
            let _ = self
                .event_bus
                .emit(AgentEvent::CoupTriggered {
                    agent_id: config.coup_authority.clone(),
                    target_agent_id: agent_id.to_string(),
                    reason: reason.clone(),
                })
                .await;

            return Ok(Some(CoupResult {
                triggered: true,
                target_agent_id: agent_id.to_string(),
                reason,
                trust_score: state.trust_score,
            }));
        }

        Ok(None)
    }

    /// Agent güven skorunu al
    pub async fn trust_score(&self, agent_id: &str) -> f64 {
        self.trust_states
            .read()
            .await
            .get(agent_id)
            .map(|s| s.trust_score)
            .unwrap_or(1.0)
    }

    /// Audit log'u al
    pub async fn audit_log(&self) -> Vec<AuditEntry> {
        self.audit_log.read().await.clone()
    }

    async fn append_audit(&self, entry: AuditEntry) {
        self.audit_log.write().await.push(entry);
    }
}

/// Darbe hataları
#[derive(Debug)]
pub enum CoupError {
    ConstitutionNotFound(String),
    AgentNotFound(String),
}

impl std::fmt::Display for CoupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            CoupError::ConstitutionNotFound(s) => write!(f, "Constitution not found: {}", s),
            CoupError::AgentNotFound(s) => write!(f, "Agent not found: {}", s),
        }
    }
}

impl std::error::Error for CoupError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl CoupError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            CoupError::AgentNotFound(..) => hudhudscript_errors::ErrorCode::CoupAgentNotFound,
            CoupError::ConstitutionNotFound(..) => {
                hudhudscript_errors::ErrorCode::CoupConstitutionNotFound
            }
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

impl From<CoupError> for hudhudscript_errors::Error {
    fn from(e: CoupError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
