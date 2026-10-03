//! VM Promise combinator tests across detached and registered transports.

use hudhudscript_vm::vm::VM;
use hudhudscript_bytecode::{PromiseState16, Value16};
use std::sync::mpsc;

fn async_pending(id: String) -> Value16 {
    Value16::promise(PromiseState16::AsyncPending(id))
}

#[test]
fn all_combines_detached_and_registered_receivers() {
    let mut vm = VM::new();
    let detached = vm.spawn_async_task(|| Ok(Value16::string("detached".to_string())));
    let (sender, receiver) = mpsc::channel();
    sender
        .send(Ok(Value16::string("registered".to_string())))
        .expect("registered receiver must remain open");
    let registered = async_pending(vm.register_promise_owned(receiver));

    let values = vm
        .resolve_promise_all(vec![registered, detached])
        .expect("both resolver transports must settle");

    assert_eq!(values[0].as_string(), Some("registered".to_string()));
    assert_eq!(values[1].as_string(), Some("detached".to_string()));
}

#[test]
fn race_combines_detached_and_registered_receivers() {
    let mut vm = VM::new();
    let detached = vm.spawn_async_task(|| {
        std::thread::sleep(std::time::Duration::from_millis(80));
        Ok(Value16::string("detached".to_string()))
    });
    let (sender, receiver) = mpsc::channel();
    sender
        .send(Ok(Value16::string("registered".to_string())))
        .expect("registered receiver must remain open");
    let registered = async_pending(vm.register_promise_owned(receiver));

    let winner = vm
        .resolve_promise_race(vec![detached, registered])
        .expect("the ready registered receiver must win");

    assert_eq!(winner.as_string(), Some("registered".to_string()));
}

#[test]
fn all_reports_an_unknown_resolver_id() {
    let mut vm = VM::new();
    let missing = async_pending("promise-missing".to_string());

    let error = vm
        .resolve_promise_all(vec![missing])
        .expect_err("an unknown resolver id must fail");

    assert_eq!(
        error,
        "Promise promise-missing has no registered resolver".to_string()
    );
}
