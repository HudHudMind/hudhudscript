use hudhudscript_runtime::agent::{AgentState, StateValue};
use hudhudscript_runtime::perspective::{FieldAccess, Perspective, PerspectiveHolder};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn make_holder(perspectives: Vec<Arc<Perspective>>) -> PerspectiveHolder {
    let mut state = AgentState::new("agent-test".to_string());
    state.set("balance".to_string(), StateValue::Number(1000.0));
    state.set(
        "audit_log".to_string(),
        StateValue::String("entry1".to_string()),
    );
    state.set(
        "secret_key".to_string(),
        StateValue::String("s3cr3t".to_string()),
    );

    let shared = Arc::new(RwLock::new(state));
    let mut holder = PerspectiveHolder::new("agent-test", shared);
    for p in perspectives {
        holder.add_perspective(p);
    }
    holder
}

fn trader_perspective() -> Arc<Perspective> {
    Arc::new(
        Perspective::new("trader")
            .with_description("Trader can read/write balance")
            .writable("balance"),
    )
}

fn auditor_perspective() -> Arc<Perspective> {
    Arc::new(
        Perspective::new("auditor")
            .with_description("Auditor can only read audit_log")
            .readable("audit_log"),
    )
}

#[tokio::test]
async fn trader_can_read_and_write_balance() {
    let holder = make_holder(vec![trader_perspective()]).await;

    let val = holder.read("balance").await;
    assert!(val.is_some());
    assert!(matches!(val.unwrap(), StateValue::Number(_)));

    let result = holder.write("balance", StateValue::Number(999.0)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn trader_cannot_read_hidden_fields() {
    let holder = make_holder(vec![trader_perspective()]).await;
    assert!(holder.read("secret_key").await.is_none());
    assert!(holder.read("audit_log").await.is_none());
}

#[tokio::test]
async fn auditor_readonly_cannot_write() {
    let holder = make_holder(vec![auditor_perspective()]).await;

    // Read is allowed.
    assert!(holder.read("audit_log").await.is_some());

    // Write is denied.
    let result = holder
        .write("audit_log", StateValue::String("tamper".to_string()))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn union_of_perspectives() {
    // A holder with both trader and auditor perspectives.
    let holder = make_holder(vec![trader_perspective(), auditor_perspective()]).await;

    assert_eq!(holder.effective_access("balance"), FieldAccess::ReadWrite);
    assert_eq!(holder.effective_access("audit_log"), FieldAccess::ReadOnly);
    assert_eq!(holder.effective_access("secret_key"), FieldAccess::Hidden);
}

#[tokio::test]
async fn projection_applied_on_read() {
    // Mask the balance: return "REDACTED" regardless of value.
    let masked_perspective = Arc::new(
        Perspective::new("masked-trader")
            .readable("balance")
            .with_projection("balance", |_| StateValue::String("REDACTED".to_string())),
    );

    let holder = make_holder(vec![masked_perspective]).await;
    let val = holder.read("balance").await.unwrap();
    assert!(matches!(val, StateValue::String(s) if s == "REDACTED"));
}

#[tokio::test]
async fn snapshot_contains_only_visible_fields() {
    let holder = make_holder(vec![trader_perspective()]).await;
    let snap = holder.snapshot().await;

    assert!(snap.contains_key("balance"));
    assert!(!snap.contains_key("secret_key"));
    assert!(!snap.contains_key("audit_log"));
}
