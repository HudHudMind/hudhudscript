use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum EventExceptionCode {
    /// E0078 — Event bus channel is closed
    EventBusChannelClosed = 78,
    /// E0079 — No subscribers attached to event bus topic
    EventBusNoSubscribers = 79,
}
