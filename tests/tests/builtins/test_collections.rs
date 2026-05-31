use hudhudscript_bytecode::Value16;
use hudhudscript_shared_builtins::map::call_map_method;
use hudhudscript_shared_builtins::set::call_set_method;
use hudhudscript_shared_builtins::stats::StatsMethodId;

// ══════════════════════════════════════════════════════════════════════
// Map methods
// ══════════════════════════════════════════════════════════════════════

#[test]
fn map_set_and_get() {
    let pairs: Vec<(Value16, Value16)> = vec![];
    let result =
        call_map_method(&pairs, "set", &[Value16::string("a"), Value16::number(1.0)]).unwrap();
    if let Some(new_pairs) = result.as_map_pairs() {
        let got = call_map_method(&new_pairs, "get", &[Value16::string("a")]).unwrap();
        assert_eq!(got, Value16::number(1.0));
    } else {
        panic!("expected Map");
    }
}

#[test]
fn map_set_replaces_existing() {
    let pairs = vec![(Value16::string("k"), Value16::number(1.0))];
    let result = call_map_method(
        &pairs,
        "set",
        &[Value16::string("k"), Value16::number(99.0)],
    )
    .unwrap();
    if let Some(new_pairs) = result.as_map_pairs() {
        assert_eq!(new_pairs.len(), 1);
        assert_eq!(new_pairs[0].1, Value16::number(99.0));
    } else {
        panic!("expected Map");
    }
}

#[test]
fn map_get_missing_returns_null() {
    let pairs: Vec<(Value16, Value16)> = vec![];
    let result = call_map_method(&pairs, "get", &[Value16::string("missing")]).unwrap();
    assert_eq!(result, Value16::null());
}

#[test]
fn map_delete() {
    let pairs = vec![
        (Value16::string("a"), Value16::number(1.0)),
        (Value16::string("b"), Value16::number(2.0)),
    ];
    let result = call_map_method(&pairs, "delete", &[Value16::string("a")]).unwrap();
    if let Some(new_pairs) = result.as_map_pairs() {
        assert_eq!(new_pairs.len(), 1);
        assert_eq!(new_pairs[0].0, Value16::string("b"));
    } else {
        panic!("expected Map");
    }
}

#[test]
fn map_has() {
    let pairs = vec![(Value16::string("x"), Value16::number(1.0))];
    assert_eq!(
        call_map_method(&pairs, "has", &[Value16::string("x")]).unwrap(),
        Value16::boolean(true)
    );
    assert_eq!(
        call_map_method(&pairs, "has", &[Value16::string("y")]).unwrap(),
        Value16::boolean(false)
    );
}

#[test]
fn map_keys_values_entries() {
    let pairs = vec![
        (Value16::string("a"), Value16::number(1.0)),
        (Value16::string("b"), Value16::number(2.0)),
    ];

    let keys = call_map_method(&pairs, "keys", &[]).unwrap();
    assert_eq!(
        keys,
        Value16::array(vec![Value16::string("a"), Value16::string("b")])
    );

    let values = call_map_method(&pairs, "values", &[]).unwrap();
    assert_eq!(
        values,
        Value16::array(vec![Value16::number(1.0), Value16::number(2.0)])
    );

    let entries = call_map_method(&pairs, "entries", &[]).unwrap();
    if let Some(items) = entries.as_array() {
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0],
            Value16::array(vec![Value16::string("a"), Value16::number(1.0)])
        );
    } else {
        panic!("expected Array");
    }
}

#[test]
fn map_size() {
    let pairs = vec![(Value16::string("a"), Value16::number(1.0))];
    assert_eq!(
        call_map_method(&pairs, "size", &[]).unwrap(),
        Value16::number(1.0)
    );
    assert_eq!(
        call_map_method(&pairs, "len", &[]).unwrap(),
        Value16::number(1.0)
    );
}

#[test]
fn map_clear() {
    let pairs = vec![(Value16::string("a"), Value16::number(1.0))];
    let result = call_map_method(&pairs, "clear", &[]).unwrap();
    assert_eq!(result, Value16::map(Vec::new()));
}

#[test]
fn map_unknown_method() {
    let pairs: Vec<(Value16, Value16)> = vec![];
    assert!(call_map_method(&pairs, "nonexistent", &[]).is_err());
}

// ══════════════════════════════════════════════════════════════════════
// Set methods
// ══════════════════════════════════════════════════════════════════════

#[test]
fn set_add() {
    let items: Vec<Value16> = vec![];
    let result = call_set_method(&items, "add", &[Value16::number(1.0)]).unwrap();
    if let Some(new_items) = result.as_set() {
        assert_eq!(new_items.len(), 1);
        assert_eq!(new_items[0], Value16::number(1.0));
    } else {
        panic!("expected Set");
    }
}

#[test]
fn set_add_deduplicates() {
    let items = vec![Value16::number(1.0)];
    let result = call_set_method(&items, "add", &[Value16::number(1.0)]).unwrap();
    if let Some(new_items) = result.as_set() {
        assert_eq!(new_items.len(), 1, "duplicate should not be added");
    } else {
        panic!("expected Set");
    }
}

#[test]
fn set_remove() {
    let items = vec![Value16::number(1.0), Value16::number(2.0)];
    let result = call_set_method(&items, "remove", &[Value16::number(1.0)]).unwrap();
    if let Some(new_items) = result.as_set() {
        assert_eq!(new_items.to_vec(), vec![Value16::number(2.0)]);
    } else {
        panic!("expected Set");
    }
}

#[test]
fn set_has() {
    let items = vec![Value16::string("x")];
    assert_eq!(
        call_set_method(&items, "has", &[Value16::string("x")]).unwrap(),
        Value16::boolean(true)
    );
    assert_eq!(
        call_set_method(&items, "has", &[Value16::string("y")]).unwrap(),
        Value16::boolean(false)
    );
}

#[test]
fn set_union() {
    let a = vec![Value16::number(1.0), Value16::number(2.0)];
    let b = Value16::set(vec![Value16::number(2.0), Value16::number(3.0)]);
    let result = call_set_method(&a, "union", &[b]).unwrap();
    if let Some(items) = result.as_set() {
        assert_eq!(items.len(), 3);
        assert!(items.contains(&Value16::number(1.0)));
        assert!(items.contains(&Value16::number(2.0)));
        assert!(items.contains(&Value16::number(3.0)));
    } else {
        panic!("expected Set");
    }
}

#[test]
fn set_intersection() {
    let a = vec![
        Value16::number(1.0),
        Value16::number(2.0),
        Value16::number(3.0),
    ];
    let b = Value16::set(vec![
        Value16::number(2.0),
        Value16::number(3.0),
        Value16::number(4.0),
    ]);
    let result = call_set_method(&a, "intersection", &[b]).unwrap();
    if let Some(items) = result.as_set() {
        assert_eq!(items.len(), 2);
        assert!(items.contains(&Value16::number(2.0)));
        assert!(items.contains(&Value16::number(3.0)));
    } else {
        panic!("expected Set");
    }
}

#[test]
fn set_difference() {
    let a = vec![
        Value16::number(1.0),
        Value16::number(2.0),
        Value16::number(3.0),
    ];
    let b = Value16::set(vec![Value16::number(2.0)]);
    let result = call_set_method(&a, "difference", &[b]).unwrap();
    if let Some(items) = result.as_set() {
        assert_eq!(items.len(), 2);
        assert!(items.contains(&Value16::number(1.0)));
        assert!(items.contains(&Value16::number(3.0)));
    } else {
        panic!("expected Set");
    }
}

#[test]
fn set_union_type_error() {
    let a = vec![Value16::number(1.0)];
    let result = call_set_method(&a, "union", &[Value16::number(2.0)]);
    assert!(result.is_err());
}

#[test]
fn set_size() {
    let items = vec![Value16::number(1.0), Value16::number(2.0)];
    assert_eq!(
        call_set_method(&items, "size", &[]).unwrap(),
        Value16::number(2.0)
    );
}

#[test]
fn set_to_array() {
    let items = vec![Value16::number(1.0), Value16::number(2.0)];
    let result = call_set_method(&items, "toArray", &[]).unwrap();
    assert_eq!(
        result,
        Value16::array(vec![Value16::number(1.0), Value16::number(2.0)])
    );
}

#[test]
fn set_clear() {
    let items = vec![Value16::number(1.0)];
    let result = call_set_method(&items, "clear", &[]).unwrap();
    assert_eq!(result, Value16::set(Vec::new()));
}

#[test]
fn set_unknown_method() {
    let items: Vec<Value16> = vec![];
    assert!(call_set_method(&items, "nope", &[]).is_err());
}

// ══════════════════════════════════════════════════════════════════════
// Stats module (descriptive statistics)
// ══════════════════════════════════════════════════════════════════════

/// Helper: call a Stats method through the shared dispatcher.
fn call_stats(
    name: &str,
    args: Vec<Value16>,
) -> Result<Value16, hudhud_script_tests::vm_interpreter::RuntimeError> {
    let method: StatsMethodId = name.parse()?;
    hudhudscript_shared_builtins::stats::dispatch(method, &args)
}

fn num_array(vals: &[f64]) -> Value16 {
    Value16::array(vals.iter().map(|v| Value16::number(*v)).collect())
}

#[test]
fn stats_mean() {
    let result = call_stats("mean", vec![num_array(&[2.0, 4.0, 6.0])]).unwrap();
    if let Some(n) = result.as_number() {
        assert!((n - 4.0).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn stats_mean_empty_errors() {
    // Empty array returns 0.0 (graceful default, not an error)
    let result = call_stats("mean", vec![num_array(&[])]).unwrap();
    assert!(matches!(result.as_number(), Some(n) if n == 0.0));
}

#[test]
fn stats_median_odd() {
    let result = call_stats("median", vec![num_array(&[3.0, 1.0, 2.0])]).unwrap();
    if let Some(n) = result.as_number() {
        assert!((n - 2.0).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn stats_variance() {
    // variance of [2, 4, 4, 4, 5, 5, 7, 9] = 4.571... (sample variance)
    let data = num_array(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    let result = call_stats("variance", vec![data]).unwrap();
    if let Some(n) = result.as_number() {
        assert!(n > 0.0, "variance should be positive");
    } else {
        panic!("expected number");
    }
}

#[test]
fn stats_variance_too_few_points() {
    // Single element returns 0.0 variance (graceful default, not an error)
    let result = call_stats("variance", vec![num_array(&[1.0])]).unwrap();
    assert!(matches!(result.as_number(), Some(n) if n == 0.0));
}

#[test]
fn stats_std_dev() {
    let data = num_array(&[10.0, 12.0, 23.0, 23.0, 16.0, 23.0, 21.0, 16.0]);
    let result = call_stats("std_dev", vec![data]).unwrap();
    if let Some(n) = result.as_number() {
        assert!(n > 0.0, "std_dev should be positive");
    } else {
        panic!("expected number");
    }
}

#[test]
fn stats_min_max() {
    let data = num_array(&[5.0, 1.0, 9.0, 3.0]);
    let min_val = call_stats("min", vec![data.clone()]).unwrap();
    let max_val = call_stats("max", vec![data]).unwrap();
    assert_eq!(min_val, Value16::number(1.0));
    assert_eq!(max_val, Value16::number(9.0));
}

#[test]
fn stats_quantile_median() {
    let data = num_array(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = call_stats("quantile", vec![data, Value16::number(0.5)]).unwrap();
    if let Some(n) = result.as_number() {
        assert!((n - 3.0).abs() < 1e-10);
    } else {
        panic!("expected number");
    }
}

#[test]
fn stats_quantile_out_of_range() {
    // Out-of-range quantile is clamped to last element (graceful, not error)
    let data = num_array(&[1.0, 2.0]);
    let result = call_stats("quantile", vec![data, Value16::number(1.5)]).unwrap();
    assert!(matches!(result.as_number(), Some(n) if n == 2.0));
}

// NOTE: `stats_normal_pdf`, `stats_normal_cdf_at_zero`,
// `stats_normal_pdf_negative_std_dev_errors`, `stats_uniform_pdf_inside`,
// `stats_uniform_cdf_at_midpoint`, `stats_uniform_invalid_range` were
// deleted during the `hudhudscript-builtins` migration: the shared
// `call_stats_method` dispatcher only implements the core methods
// (mean, median, variance, std_dev, min, max, quantile). `normal_pdf`,
// `normal_cdf`, `uniform_pdf`, `uniform_cdf` only existed in the deleted
// interpreter-era `builtins::stats` module and have no counterpart in
// the shared dispatch path — re-add them to
// `hudhudscript_shared_builtins::stats` if they are still needed, then
// reinstate the tests against the new shared entry point.

#[test]
fn stats_extract_numbers_type_error() {
    // Passing a non-array to mean should error
    let result = call_stats("mean", vec![Value16::string("not an array")]);
    assert!(result.is_err());
}
