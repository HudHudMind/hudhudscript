//! Event Bus — tokio broadcast ile agent-arası iletişim (Issue #13)
//!
//! AgentEvent enum, global broadcast channel, emit/subscribe API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Kapasite: aynı anda kaç event buffer'da tutulabilir
const EVENT_BUS_CAPACITY: usize = 1024;

/// Agent event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentEvent {
    /// Bir task başarıyla tamamlandı
    TaskCompleted {
        agent_id: String,
        task_id: String,
        output: serde_json::Value,
    },
    /// Bir task başarısız oldu
    TaskFailed {
        agent_id: String,
        task_id: String,
        error: String,
    },
    /// Agent state değişti
    StateChanged {
        agent_id: String,
        key: String,
        value: serde_json::Value,
    },
    /// Council oylama sonucu
    VoteCompleted {
        council_id: String,
        decision: String,
        votes_for: usize,
        votes_against: usize,
    },
    /// Swarm koordinasyon eventi
    SwarmConsensus {
        swarm_id: String,
        result: serde_json::Value,
    },
    /// Darbe tetiklendi (Issue #16)
    CoupTriggered {
        agent_id: String,
        target_agent_id: String,
        reason: String,
    },
    /// Permission ihlali
    PermissionDenied {
        agent_id: String,
        resource: String,
        action: String,
    },
    /// Workflow kaydedildi
    WorkflowRegistered {
        workflow_id: String,
    },
    /// Workflow tamamlandı
    WorkflowCompleted {
        workflow_id: String,
    },
    /// Kullanıcı tanımlı özel event
    Custom {
        agent_id: String,
        event_type: String,
        payload: serde_json::Value,
    },
}

impl AgentEvent {
    /// Event'i yayınlayan agent ID'sini döndür
    pub fn agent_id(&self) -> &str {
        match self {
            AgentEvent::TaskCompleted { agent_id, .. } => agent_id,
            AgentEvent::TaskFailed { agent_id, .. } => agent_id,
            AgentEvent::StateChanged { agent_id, .. } => agent_id,
            AgentEvent::VoteCompleted { council_id, .. } => council_id,
            AgentEvent::SwarmConsensus { swarm_id, .. } => swarm_id,
            AgentEvent::WorkflowRegistered { workflow_id, .. } => workflow_id,
            AgentEvent::WorkflowCompleted { workflow_id, .. } => workflow_id,
            AgentEvent::CoupTriggered { agent_id, .. } => agent_id,
            AgentEvent::PermissionDenied { agent_id, .. } => agent_id,
            AgentEvent::Custom { agent_id, .. } => agent_id,
        }
    }

    /// Event tipini string olarak döndür
    pub fn event_type(&self) -> &str {
        match self {
            AgentEvent::TaskCompleted { .. } => "task_completed",
            AgentEvent::TaskFailed { .. } => "task_failed",
            AgentEvent::StateChanged { .. } => "state_changed",
            AgentEvent::VoteCompleted { .. } => "vote_completed",
            AgentEvent::SwarmConsensus { .. } => "swarm_consensus",
            AgentEvent::WorkflowRegistered { .. } => "workflow_registered",
            AgentEvent::WorkflowCompleted { .. } => "workflow_completed",
            AgentEvent::CoupTriggered { .. } => "coup_triggered",
            AgentEvent::PermissionDenied { .. } => "permission_denied",
            AgentEvent::Custom { event_type, .. } => event_type,
        }
    }
}

/// Global event bus — tokio broadcast channel üzerine kurulu
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
    /// Event istatistikleri
    stats: Arc<RwLock<EventStats>>,
}

/// Event istatistikleri
#[derive(Debug, Default, Clone)]
pub struct EventStats {
    pub total_emitted: u64,
    pub by_type: HashMap<String, u64>,
}

impl EventBus {
    /// Yeni event bus oluştur
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self {
            sender,
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// Event yayınla (HudHudScript: `olay_yayinla`)
    pub async fn emit(&self, event: AgentEvent) -> Result<usize, EventBusError> {
        // İstatistik güncelle
        {
            let mut stats = self.stats.write().await;
            stats.total_emitted += 1;
            *stats
                .by_type
                .entry(event.event_type().to_string())
                .or_insert(0) += 1;
        }

        self.sender
            .send(event)
            .map_err(|_| EventBusError::NoSubscribers)
    }

    /// Event dinleyici oluştur (HudHudScript: `dinle`)
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }

    /// Aktif subscriber sayısı
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// İstatistikleri al
    pub async fn stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Event bus hataları
#[derive(Debug)]
pub enum EventBusError {
    NoSubscribers,
    ChannelClosed,
}

impl std::fmt::Display for EventBusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry = self.code().entry();
        write!(f, "[{}] {} — ", entry.short_code, entry.title)?;
        match self {
            EventBusError::NoSubscribers => write!(f, "No active subscribers"),
            EventBusError::ChannelClosed => write!(f, "Channel closed"),
        }
    }
}

impl std::error::Error for EventBusError {}

// ---------------------------------------------------------------------------
// Auto-generated bridge to the unified error catalog (v0.4.48)
// ---------------------------------------------------------------------------
impl EventBusError {
    /// Stable catalog code for this error variant.
    pub fn code(&self) -> hudhudscript_errors::ErrorCode {
        match self {
            EventBusError::ChannelClosed => hudhudscript_errors::ErrorCode::EventBusChannelClosed,
            EventBusError::NoSubscribers => hudhudscript_errors::ErrorCode::EventBusNoSubscribers,
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

impl From<EventBusError> for hudhudscript_errors::Error {
    fn from(e: EventBusError) -> hudhudscript_errors::Error {
        let code = e.code();
        hudhudscript_errors::Error::new(code, e.to_string())
    }
}
