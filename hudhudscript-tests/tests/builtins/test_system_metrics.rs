use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::system_metrics_ops::cpu::sys_cpu_count;
use hudhudscript_shared_builtins::system_metrics_ops::disk::sys_disk_usage;
use hudhudscript_shared_builtins::system_metrics_ops::memory::sys_memory;
use hudhudscript_shared_builtins::system_metrics_ops::network::sys_network_interfaces;
use hudhudscript_shared_builtins::system_metrics_ops::process::sys_processes;
use hudhudscript_shared_builtins::system_metrics_ops::system::{
    sys_hostname, sys_load_average, sys_uptime,
};

#[test]
fn test_cpu_count() {
    let result = sys_cpu_count(&[]).unwrap();
    if let Some(n) = result.as_number() {
        assert!(n >= 1.0);
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_memory() {
    let result = sys_memory(&[]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("total"));
        assert!(obj.contains_key("used"));
        assert!(obj.contains_key("free"));
        assert!(obj.contains_key("available"));
        // On Linux, total should be > 0
        if let Some(total) = obj.get("total").and_then(|v| v.as_number()) {
            assert!(total > 0.0, "total memory should be > 0");
        }
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_disk_usage_root() {
    let result = sys_disk_usage(&[Value16::string("/".to_string())]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("total"));
        assert!(obj.contains_key("used"));
        assert!(obj.contains_key("free"));
        assert!(obj.contains_key("percent"));
        if let Some(total) = obj.get("total").and_then(|v| v.as_number()) {
            assert!(total > 0.0, "disk total should be > 0");
        }
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_disk_usage_default_path() {
    let result = sys_disk_usage(&[]).unwrap();
    if let Some(obj) = result.as_object() {
        assert!(obj.contains_key("total"));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_load_average() {
    let result = sys_load_average(&[]).unwrap();
    if let Some(arr) = result.as_array() {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_uptime() {
    let result = sys_uptime(&[]).unwrap();
    if let Some(n) = result.as_number() {
        assert!(n > 0.0, "uptime should be > 0");
    } else {
        panic!("Expected number");
    }
}

#[test]
fn test_hostname() {
    let result = sys_hostname(&[]).unwrap();
    if let Some(h) = result.as_str() {
        assert!(!h.is_empty());
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_network_interfaces() {
    let result = sys_network_interfaces(&[]).unwrap();
    if let Some(arr) = result.as_array() {
        // Should have at least lo (loopback)
        assert!(!arr.is_empty(), "should have at least one interface");
        if let Some(iface) = &arr[0].as_object() {
            assert!(iface.contains_key("name"));
            assert!(iface.contains_key("rx_bytes"));
            assert!(iface.contains_key("tx_bytes"));
        }
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_processes() {
    let result = sys_processes(&[]).unwrap();
    if let Some(arr) = result.as_array() {
        assert!(!arr.is_empty(), "should list at least one process");
        if let Some(proc_info) = &arr[0].as_object() {
            assert!(proc_info.contains_key("pid"));
            assert!(proc_info.contains_key("name"));
            assert!(proc_info.contains_key("cpu_percent"));
            assert!(proc_info.contains_key("memory_kb"));
        }
    } else {
        panic!("Expected array");
    }
}

// NOTE: `test_create_system_module` was deleted with the rest of
// `hudhudscript-builtins`. It probed the interpreter-era
// `Value::NativeFunction`-based module layout produced by
// `create_system_module()`; the VM + shared path exposes those functions
// via `call_system_metrics_method` / the direct `sys_*` helpers above,
// so the structural assertion has no semantic counterpart.
