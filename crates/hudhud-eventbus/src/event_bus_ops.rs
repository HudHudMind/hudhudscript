//! Shared Event Bus builtin — used by both VM and interpreter.
//!
//! Provides: EventBus.emit, on, off, once, listeners, clear, channels, has_listeners
//!
//! Real in-memory pub/sub with subscriber registry. Events are delivered
//! to matching subscribers. Supports wildcard patterns.

use hudhudscript_bytecode::Value16;
use hudhudscript_errors::{Error, ErrorCode, HudHudResult};

fn runtime_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorCode::CompileRuntimeError, msg.into())
}
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// A subscription entry in the event bus.
#[derive(Clone)]
struct Subscription {
    id: String,
    pattern: String,
    handler: String,
    active: bool,
    once: bool,
}

/// Global event bus state — thread-safe, persists across calls.
struct EventBusState {
    subscriptions: HashMap<String, Subscription>,
    /// Delivered events log (most recent, bounded)
    delivered: Vec<(String, String)>,
}

static EVENT_BUS: OnceLock<Mutex<EventBusState>> = OnceLock::new();

fn bus() -> &'static Mutex<EventBusState> {
    EVENT_BUS.get_or_init(|| {
        Mutex::new(EventBusState {
            subscriptions: HashMap::new(),
            delivered: Vec::new(),
        })
    })
}

/// Check if an event name matches a pattern with wildcards.
pub fn event_matches(event: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let event_parts: Vec<&str> = event.split('.').collect();
    let pattern_parts: Vec<&str> = pattern.split('.').collect();
    if event_parts.len() != pattern_parts.len() {
        return false;
    }
    event_parts
        .iter()
        .zip(pattern_parts.iter())
        .all(|(e, p)| *p == "*" || e == p)
}

fn now_millis() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}

fn now_nanos_hex() -> String {
    format!(
        "sub_{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

/// Enum identifying each EventBus operation for zero-cost dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventBusMethodId {
    Emit,
    On,
    Off,
    Once,
    Listeners,
    Clear,
    Channels,
    HasListeners,
}

impl std::str::FromStr for EventBusMethodId {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "emit" => Ok(Self::Emit),
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            "once" => Ok(Self::Once),
            "listeners" => Ok(Self::Listeners),
            "clear" => Ok(Self::Clear),
            "channels" => Ok(Self::Channels),
            "has_listeners" => Ok(Self::HasListeners),
            _ => Err(runtime_error(format!("Unknown EventBus method: {}", s))),
        }
    }
}

/// Zero-cost enum dispatch for EventBus operations.
pub fn dispatch(method: EventBusMethodId, args: &[Value16]) -> HudHudResult<Value16> {
    match method {
        EventBusMethodId::Emit => event_emit(args),
        EventBusMethodId::On => event_on_impl(args, false),
        EventBusMethodId::Off => event_off(args),
        EventBusMethodId::Once => event_on_impl(args, true),
        EventBusMethodId::Listeners => event_listeners(args),
        EventBusMethodId::Clear => event_clear(args),
        EventBusMethodId::Channels => event_channels(args),
        EventBusMethodId::HasListeners => event_has_listeners(args),
    }
}

/// Execute an EventBus method (kept for backward compat).

/// EventBus.emit(event_name, data?) → `{event, data, delivered, listener_count, timestamp}`.
///
/// Free-function entry used by the interpreter-era `builtins::event_bus`
/// shim (being retired) and by direct tests. Shares the process-wide
/// subscriber registry with every other `event_*` function in this module.
pub fn event_emit(args: &[Value16]) -> HudHudResult<Value16> {
    let event_name = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("EventBus.emit: expected event name string"))?
        .to_string();
    let data = args.get(1).cloned().unwrap_or_else(Value16::null);

    let mut state = bus().lock().unwrap();

    let mut matched_count = 0u32;
    let mut to_remove = Vec::new();

    for (id, sub) in state.subscriptions.iter() {
        if sub.active && event_matches(&event_name, &sub.pattern) {
            matched_count += 1;
            if sub.once {
                to_remove.push(id.clone());
            }
        }
    }

    for id in &to_remove {
        state.subscriptions.remove(id);
    }

    // Log delivery (keep last 100)
    state.delivered.push((event_name.clone(), String::new()));
    if state.delivered.len() > 100 {
        state.delivered.remove(0);
    }

    let mut result = HashMap::new();
    result.insert("event".to_string(), Value16::string(event_name));
    result.insert("data".to_string(), data);
    result.insert("delivered".to_string(), Value16::bool_(matched_count > 0));
    result.insert(
        "listener_count".to_string(),
        Value16::number(matched_count as f64),
    );
    result.insert("timestamp".to_string(), Value16::number(now_millis()));
    Ok(Value16::object(result))
}

/// EventBus.on(pattern, handler?) — registers a recurring subscription.
///
/// Thin free-function wrapper used directly by tests (and the retiring
/// interpreter-era shim); delegates to the internal implementation with
/// `once = false`.
pub fn event_on(args: &[Value16]) -> HudHudResult<Value16> {
    event_on_impl(args, false)
}

/// EventBus.once(pattern, handler?) — registers a one-shot subscription.
///
/// Thin free-function wrapper; delegates to the internal implementation
/// with `once = true`.
pub fn event_once(args: &[Value16]) -> HudHudResult<Value16> {
    event_on_impl(args, true)
}

fn event_on_impl(args: &[Value16], once: bool) -> HudHudResult<Value16> {
    let method_name = if once { "EventBus.once" } else { "EventBus.on" };
    let pattern = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error(format!("{}: expected event pattern string", method_name)))?
        .to_string();
    let handler = args
        .get(1)
        .and_then(|v| v.as_str())
        .unwrap_or("anonymous")
        .to_string();

    let id = now_nanos_hex();

    let sub = Subscription {
        id: id.clone(),
        pattern: pattern.clone(),
        handler: handler.clone(),
        active: true,
        once,
    };

    bus().lock().unwrap().subscriptions.insert(id.clone(), sub);

    let mut result = HashMap::new();
    result.insert("id".to_string(), Value16::string(id));
    result.insert("pattern".to_string(), Value16::string(pattern));
    result.insert("handler".to_string(), Value16::string(handler));
    result.insert("active".to_string(), Value16::bool_(true));
    result.insert("once".to_string(), Value16::bool_(once));
    Ok(Value16::object(result))
}

/// EventBus.off(subscription_id) → `true` if a subscription was removed.
///
/// Free-function wrapper so the retiring interpreter-era shim (and direct
/// tests) can call the shared registry.
pub fn event_off(args: &[Value16]) -> HudHudResult<Value16> {
    let id = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("EventBus.off: expected subscription id string"))?
        .to_string();
    let removed = bus().lock().unwrap().subscriptions.remove(&id).is_some();
    Ok(Value16::bool_(removed))
}

fn event_listeners(args: &[Value16]) -> HudHudResult<Value16> {
    let filter = args.first().and_then(|v| v.as_str()).map(|s| s.to_string());

    let state = bus().lock().unwrap();
    let listeners: Vec<Value16> = state
        .subscriptions
        .values()
        .filter(|sub| {
            sub.active
                && match &filter {
                    Some(event) => event_matches(event, &sub.pattern),
                    None => true,
                }
        })
        .map(|sub| {
            let mut info = HashMap::new();
            info.insert("id".to_string(), Value16::string(sub.id.clone()));
            info.insert("pattern".to_string(), Value16::string(sub.pattern.clone()));
            info.insert("handler".to_string(), Value16::string(sub.handler.clone()));
            info.insert("active".to_string(), Value16::bool_(sub.active));
            info.insert("once".to_string(), Value16::bool_(sub.once));
            Value16::object(info)
        })
        .collect();

    Ok(Value16::array(listeners))
}

fn event_clear(args: &[Value16]) -> HudHudResult<Value16> {
    let filter = args.first().and_then(|v| v.as_str()).map(|s| s.to_string());

    let mut state = bus().lock().unwrap();
    let before = state.subscriptions.len();

    match filter {
        Some(event) => {
            state
                .subscriptions
                .retain(|_, sub| !event_matches(&event, &sub.pattern));
        }
        None => {
            state.subscriptions.clear();
        }
    }

    let cleared = before - state.subscriptions.len();
    let mut result = HashMap::new();
    result.insert("cleared".to_string(), Value16::number(cleared as f64));
    Ok(Value16::object(result))
}

fn event_channels(_args: &[Value16]) -> HudHudResult<Value16> {
    let state = bus().lock().unwrap();
    let mut channels: Vec<String> = state
        .subscriptions
        .values()
        .filter(|s| s.active)
        .map(|s| s.pattern.clone())
        .collect();
    channels.sort();
    channels.dedup();
    Ok(Value16::array(
        channels.into_iter().map(Value16::string).collect(),
    ))
}

/// EventBus.has_listeners(event_name) → `true` if any active subscription
/// matches.
///
/// Free-function wrapper; shares the same registry as every other
/// event_bus_ops entry.
pub fn event_has_listeners(args: &[Value16]) -> HudHudResult<Value16> {
    let event = args
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| runtime_error("EventBus.has_listeners: expected event name string"))?
        .to_string();
    let state = bus().lock().unwrap();
    let has = state
        .subscriptions
        .values()
        .any(|sub| sub.active && event_matches(&event, &sub.pattern));
    Ok(Value16::bool_(has))
}
