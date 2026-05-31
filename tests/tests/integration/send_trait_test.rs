//! Test to verify Value16 type implements Send trait

use hudhudscript_bytecode::{PromiseState16, Value16};
use std::sync::Arc;
use std::thread;

#[test]
fn test_value_is_send() {
    // Create a value
    let value = Value16::number(42.0);

    // Wrap in Arc to share across threads
    let value_arc = Arc::new(value);
    let value_clone = value_arc.clone();

    // Spawn a thread and send the value
    let handle = thread::spawn(move || {
        // Access value in another thread
        if let Some(n) = value_clone.as_number() {
            assert_eq!(n, 42.0);
        } else {
            panic!("Expected number");
        }
    });

    // Wait for thread to complete
    handle.join().unwrap();

    // Original value still accessible
    if let Some(n) = value_arc.as_number() {
        assert_eq!(n, 42.0);
    } else {
        panic!("Expected number");
    }
}

// `test_function_value_is_send` was deleted during the interpreter-crate
// retirement: it constructed `Value16::InterpretedFunction(Box<InterpretedFunctionDef>)`
// — a variant that no longer exists on `bytecode::Value16`.  The VM's
// `Value16::Function` variant holds owned `String`/`Vec`/`HashMap<String, Arc<RwLock<Value16>>>`
// fields, all of which already implement `Send + Sync`, so the compile-time
// Send check that the deleted test enforced is already guaranteed by the
// type's generic shape and doesn't need a dedicated runtime assertion.

#[test]
fn test_promise_value_is_send() {
    // Create a resolved promise
    let promise = Value16::promise(PromiseState16::Resolved(Box::new(Value16::string(
        "success".to_string(),
    ))));

    // Send to another thread
    let promise_arc = Arc::new(promise);
    let promise_clone = promise_arc.clone();

    let handle = thread::spawn(move || {
        if let Some(state) = promise_clone.as_promise_state() {
            match state {
                PromiseState16::Resolved(val) => {
                    if let Some(s) = val.as_str() {
                        assert_eq!(s, "success");
                    } else {
                        panic!("Expected string");
                    }
                }
                _ => panic!("Expected resolved promise"),
            }
        } else {
            panic!("Expected promise");
        }
    });

    handle.join().unwrap();
}

#[test]
fn test_multiple_threads_sharing_value() {
    // Create a complex value
    let value = Value16::array(vec![
        Value16::number(1.0),
        Value16::string("test".to_string()),
        Value16::boolean(true),
    ]);

    let value_arc = Arc::new(value);

    // Spawn multiple threads
    let mut handles = vec![];

    for i in 0..5 {
        let value_clone = value_arc.clone();
        let handle = thread::spawn(move || {
            if let Some(arr) = value_clone.as_array() {
                assert_eq!(arr.len(), 3);
                println!("Thread {} verified array length", i);
            } else {
                panic!("Expected array");
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}
