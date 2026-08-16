//! GATE-2: Telemetry JSON writer for `hudhud run --telemetry-json`.
//! Only compiled when `telemetry` feature is enabled.

use crate::common::CliError;
use hudhudscript_vm::packed_ops::dense_name;
use hudhudscript_vm::vm::telemetry::TelemetrySnapshot;
use hudhudscript_vm::VM;
use std::fs;
use std::path::Path;

/// Write telemetry JSON to the given path directly.
/// Creates parent directories if needed.
/// If execution failed, `execution_status` is set to "error".
pub fn write_telemetry_json(vm: &VM, path: &Path, ok: bool) -> Result<(), CliError> {
    let snap = vm.telemetry_snapshot();
    let json = build_telemetry_json(&snap, ok);
    let json_str = serde_json::to_string_pretty(&json)
        .map_err(|e| CliError::Runtime(format!("telemetry serialize: {}", e)))?;

    // Create parent dirs if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                CliError::Io(format!("telemetry mkdir {}: {}", parent.display(), e))
            })?;
        }
    }

    fs::write(path, &json_str)
        .map_err(|e| CliError::Io(format!("telemetry write {}: {}", path.display(), e)))?;

    eprintln!("telemetry: written to {}", path.display());
    Ok(())
}

/// Build the full telemetry JSON from a snapshot, following the schema
/// from hudhud-script-interactive/TELEMETRY.md §6.3.
fn build_telemetry_json(snap: &TelemetrySnapshot, ok: bool) -> serde_json::Value {
    let availability = build_availability(snap);
    let counters = build_counters(snap);

    // P0: opcode counts — top 32 sorted
    let mut opcode_list: Vec<serde_json::Value> = snap.opcode_counts.iter().enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| serde_json::json!({"op": format!("{}", dense_name(i as u8)), "op_code": i, "count": c}))
        .collect();
    opcode_list.sort_by(|a, b| b["count"].as_u64().cmp(&a["count"].as_u64()));
    opcode_list.truncate(32);

    // P0: bigrams — top 32
    let mut bigrams: Vec<&((u16, u16), u64)> = snap.opcode_bigrams.iter().collect();
    bigrams.sort_by(|a, b| b.1.cmp(&a.1));
    bigrams.truncate(32);
    let bigram_list: Vec<serde_json::Value> = bigrams.iter()
        .map(|((a, b), c)| serde_json::json!({"prev": format!("{}", dense_name(*a as u8)), "curr": format!("{}", dense_name(*b as u8)), "count": c}))
        .collect();

    // P0: fallthrough list — all non-zero
    let ft_list: Vec<serde_json::Value> = snap
        .fallthrough_by_opcode
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| serde_json::json!({"op": format!("{}", dense_name(i as u8)), "count": c}))
        .collect();

    // P0: unpacked opcode names — top
    let mut unpacked_list: Vec<(&str, u64)> = snap
        .unpacked_opcode_counts
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();
    unpacked_list.sort_by(|a, b| b.1.cmp(&a.1));
    let unpacked_json: serde_json::Value = unpacked_list
        .into_iter()
        .map(|(k, v)| (k.to_string(), serde_json::Value::from(v)))
        .collect();

    let kind_names = [
        "String",
        "StringAscii",
        "Array",
        "Object",
        "Function",
        "Promise",
        "Class",
        "Instance",
        "Data",
        "Set",
        "Map",
        "Generator",
        "Tool",
        "Resource",
        "Option",
        "Result",
        "BigInt",
    ];
    let mut logical_by_kind = serde_json::Map::new();
    for (i, &count) in snap.alloc_count_by_kind.iter().enumerate() {
        if i < kind_names.len() && count > 0 {
            logical_by_kind.insert(kind_names[i].to_string(), serde_json::Value::from(count));
        }
    }

    let mut fusion_emitted = serde_json::Map::new();
    for (k, v) in &snap.fusion_emitted_by_opcode {
        fusion_emitted.insert(k.clone(), serde_json::Value::from(*v));
    }
    let mut fusion_executed = serde_json::Map::new();
    for (k, v) in &snap.fusion_executed_by_opcode {
        fusion_executed.insert(k.clone(), serde_json::Value::from(*v));
    }
    let mut fusion_rejected = serde_json::Map::new();
    for (k, v) in &snap.fusion_rejected_by_reason {
        fusion_rejected.insert(k.clone(), serde_json::Value::from(*v));
    }

    serde_json::json!({
        "schema_version": 1,
        "telemetry_enabled": true,
        "execution_status": if ok { "ok" } else { "error" },
        "counter_availability": availability,
        "counters": counters,
        "opcodes": opcode_list,
        "opcode_bigrams": bigram_list,
        "fallthrough_by_opcode": ft_list,
        "unpacked_opcode_counts": unpacked_json,
        "sites": {
            "call": snap.site_call_count,
            "property": snap.site_property_count,
            "index": snap.site_index_count
        },
        "fusion": {
            "emitted_by_opcode": serde_json::Value::Object(fusion_emitted),
            "executed_by_opcode": serde_json::Value::Object(fusion_executed),
            "rejected_by_reason": serde_json::Value::Object(fusion_rejected)
        },
        "allocations": {
            "logical_by_kind": serde_json::Value::Object(logical_by_kind),
            "bytes_by_kind": {},
            "copy_bytes_by_kind": {}
        }
    })
}

fn build_availability(_snap: &TelemetrySnapshot) -> serde_json::Value {
    serde_json::json!({
        "total_instructions": "available",
        "bigint_promotion": "available",
        "bigint_alloc": "available",
        "unpacked_dispatch_count": "available",
        "packed_dispatch_count": "available",
        "packed_fallthrough_count": "available",
        "call_cache_hit": "available",
        "call_cache_miss": "available",
        "chunk_cache_hit": "available",
        "chunk_cache_miss": "available",
        "string_index_clone_count": "available",
        "string_index_clone_bytes": "available",
        "loop_begin_end_count": "available",
        "property_lookup_count": "available",
        "scope_cell_lookup_count": "available",
        "opcode_counts": "available",
        "opcode_bigrams": "available",
        "fusion_emitted": "available",
        "fusion_executed": "available",
        "property_cache_hit": "available",
        "property_cache_miss": "available",
        "alloc_count": "available",
        "alloc_bytes": "unavailable",
        "register_high_water": "unavailable",
        "opcode_trigrams": "unavailable",
        "site_types": "available",
        "fallback_reasons": "unavailable",
        "allocations": "available",
        "gc_cycle_count": "available",
        "gc_mark_count": "available",
        "gc_sweep_count": "available",
        "gc_pause_ns_total": "available",
        "gc_pause_ns_max": "available",
        "gc_heap_bytes_after_sweep": "available",
        "int_add_slow_count": "available",
    })
}

fn build_counters(snap: &TelemetrySnapshot) -> serde_json::Value {
    serde_json::json!({
        "total_instructions": snap.total_instructions,
        "call_cache_hit": snap.call_cache_hit,
        "call_cache_miss": snap.call_cache_miss,
        "chunk_cache_hit": snap.chunk_cache_hit,
        "chunk_cache_miss": snap.chunk_cache_miss,
        "bigint_promotion": snap.bigint_promotion,
        "bigint_alloc": snap.bigint_alloc,
        "packed_dispatch_count": snap.packed_dispatch_count,
        "packed_fallthrough_count": snap.packed_fallthrough_count,
        "unpacked_dispatch_count": snap.unpacked_dispatch_count,
        "string_index_clone_count": snap.string_index_clone_count,
        "string_index_clone_bytes": snap.string_index_clone_bytes,
        "loop_begin_end_count": snap.loop_begin_end_count,
        "property_lookup_count": snap.property_lookup_count,
        "scope_cell_lookup_count": snap.scope_cell_lookup_count,
        "gc_cycle_count": snap.gc_cycle_count,
        "gc_mark_count": snap.gc_mark_count,
        "gc_sweep_count": snap.gc_sweep_count,
        "gc_pause_ns_total": snap.gc_pause_ns_total,
        "gc_pause_ns_max": snap.gc_pause_ns_max,
        "gc_heap_bytes_after_sweep": snap.gc_heap_bytes_after_sweep,
        "int_add_slow_count": snap.int_add_slow_count
    })
}
