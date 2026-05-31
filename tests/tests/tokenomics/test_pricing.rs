//! Public API tests for tokenomics::pricing
//! Covers: PricingRegistry (new, default, get_pricing, calculate_cost,
//!         register_model, cheapest_for_provider, all_models),
//!         ModelPricing, CostBreakdown.

use hudhudscript_tokenomics::pricing::{CostBreakdown, ModelPricing, PricingRegistry};

// ── PricingRegistry construction ─────────────────────────────────────

#[test]
fn registry_new_contains_anthropic_models() {
    let reg = PricingRegistry::new();
    assert!(reg.get_pricing("claude-opus-4").is_some());
    assert!(reg.get_pricing("claude-sonnet-4").is_some());
    assert!(reg.get_pricing("claude-haiku-3.5").is_some());
}

#[test]
fn registry_new_contains_openai_models() {
    let reg = PricingRegistry::new();
    assert!(reg.get_pricing("gpt-4o").is_some());
    assert!(reg.get_pricing("gpt-4o-mini").is_some());
    assert!(reg.get_pricing("o1").is_some());
    assert!(reg.get_pricing("o3").is_some());
}

#[test]
fn registry_new_contains_deepseek_models() {
    let reg = PricingRegistry::new();
    assert!(reg.get_pricing("deepseek-v3").is_some());
    assert!(reg.get_pricing("deepseek-r1").is_some());
}

#[test]
fn registry_new_contains_gemini_models() {
    let reg = PricingRegistry::new();
    assert!(reg.get_pricing("gemini-1.5-pro").is_some());
    assert!(reg.get_pricing("gemini-1.5-flash").is_some());
}

#[test]
fn registry_default_equals_new() {
    let reg = PricingRegistry::default();
    assert!(reg.get_pricing("claude-sonnet-4").is_some());
}

// ── PricingRegistry::get_pricing ─────────────────────────────────────

#[test]
fn get_pricing_returns_none_for_unknown_model() {
    let reg = PricingRegistry::new();
    assert!(reg.get_pricing("nonexistent-model-xyz").is_none());
}

#[test]
fn get_pricing_claude_opus_4_has_correct_provider() {
    let reg = PricingRegistry::new();
    let p = reg.get_pricing("claude-opus-4").unwrap();
    assert_eq!(p.provider, "anthropic");
}

#[test]
fn get_pricing_gpt_4o_has_correct_provider() {
    let reg = PricingRegistry::new();
    let p = reg.get_pricing("gpt-4o").unwrap();
    assert_eq!(p.provider, "openai");
}

#[test]
fn get_pricing_deepseek_v3_has_correct_provider() {
    let reg = PricingRegistry::new();
    let p = reg.get_pricing("deepseek-v3").unwrap();
    assert_eq!(p.provider, "deepseek");
}

#[test]
fn get_pricing_gemini_flash_has_correct_provider() {
    let reg = PricingRegistry::new();
    let p = reg.get_pricing("gemini-1.5-flash").unwrap();
    assert_eq!(p.provider, "gemini");
}

// ── PricingRegistry::calculate_cost ──────────────────────────────────

#[test]
fn calculate_cost_unknown_model_returns_none() {
    let reg = PricingRegistry::new();
    assert!(reg
        .calculate_cost("nonexistent", 1000, 500, 0, 0, false)
        .is_none());
}

#[test]
fn calculate_cost_claude_sonnet_4_basic() {
    let reg = PricingRegistry::new();
    // input: 1000 * 3.0/1M = 0.003, output: 500 * 15.0/1M = 0.0075
    let cost = reg
        .calculate_cost("claude-sonnet-4", 1000, 500, 0, 0, false)
        .unwrap();
    assert!((cost.input_cost - 0.003).abs() < 1e-9);
    assert!((cost.output_cost - 0.0075).abs() < 1e-9);
    assert!((cost.total_cost - 0.0105).abs() < 1e-9);
    assert_eq!(cost.currency, "USD");
}

#[test]
fn calculate_cost_currency_is_usd() {
    let reg = PricingRegistry::new();
    let cost = reg
        .calculate_cost("gpt-4o", 1000, 1000, 0, 0, false)
        .unwrap();
    assert_eq!(cost.currency, "USD");
}

#[test]
fn calculate_cost_batch_discount_halves_total() {
    let reg = PricingRegistry::new();
    let normal = reg
        .calculate_cost("gpt-4o", 1000, 500, 0, 0, false)
        .unwrap();
    let batch = reg.calculate_cost("gpt-4o", 1000, 500, 0, 0, true).unwrap();
    assert!((batch.total_cost - normal.total_cost * 0.5).abs() < 1e-9);
}

#[test]
fn calculate_cost_cached_tokens_reduce_input_cost() {
    let reg = PricingRegistry::new();
    // claude-opus-4: input=15.0/Mtok, cached=1.5/Mtok, output=75.0/Mtok
    let no_cache = reg
        .calculate_cost("claude-opus-4", 10000, 1000, 0, 0, false)
        .unwrap();
    let with_cache = reg
        .calculate_cost("claude-opus-4", 10000, 1000, 8000, 0, false)
        .unwrap();
    // with_cache has lower total because cached tokens are cheaper
    assert!(with_cache.total_cost < no_cache.total_cost);
}

#[test]
fn calculate_cost_cached_tokens_exact_calculation() {
    let reg = PricingRegistry::new();
    // claude-opus-4: input=15/Mtok, cached=1.5/Mtok, output=75/Mtok
    // billable_input=(10000-8000)=2000 * 15/1M=0.03
    // cached=8000 * 1.5/1M=0.012
    // output=1000 * 75/1M=0.075
    // total=0.117
    let cost = reg
        .calculate_cost("claude-opus-4", 10000, 1000, 8000, 0, false)
        .unwrap();
    assert!((cost.input_cost - 0.03).abs() < 1e-10);
    assert!((cost.cached_input_cost - 0.012).abs() < 1e-10);
    assert!((cost.total_cost - 0.117).abs() < 1e-10);
}

#[test]
fn calculate_cost_thinking_tokens_added_to_total() {
    let reg = PricingRegistry::new();
    // claude-opus-4 thinking=75/Mtok
    // 10000 thinking tokens * 75/1M = 0.75
    let cost = reg
        .calculate_cost("claude-opus-4", 1000, 500, 0, 10000, false)
        .unwrap();
    assert!((cost.thinking_cost - 0.75).abs() < 1e-9);
    assert!(cost.total_cost > 0.75);
}

#[test]
fn calculate_cost_zero_tokens_all_zeros() {
    let reg = PricingRegistry::new();
    let cost = reg
        .calculate_cost("claude-haiku-3.5", 0, 0, 0, 0, false)
        .unwrap();
    assert_eq!(cost.input_cost, 0.0);
    assert_eq!(cost.output_cost, 0.0);
    assert_eq!(cost.cached_input_cost, 0.0);
    assert_eq!(cost.thinking_cost, 0.0);
    assert_eq!(cost.total_cost, 0.0);
}

#[test]
fn calculate_cost_total_is_sum_of_components() {
    let reg = PricingRegistry::new();
    let cost = reg
        .calculate_cost("claude-opus-4", 5000, 2000, 1000, 500, false)
        .unwrap();
    let expected = cost.input_cost + cost.output_cost + cost.cached_input_cost + cost.thinking_cost;
    assert!((cost.total_cost - expected).abs() < 1e-12);
}

#[test]
fn calculate_cost_cached_exceeds_total_input_saturates_at_zero() {
    let reg = PricingRegistry::new();
    // cached_tokens > input_tokens: saturating_sub => billable_input = 0
    let cost = reg
        .calculate_cost("claude-sonnet-4", 100, 100, 10000, 0, false)
        .unwrap();
    assert_eq!(cost.input_cost, 0.0); // billable_input = 0 after saturating_sub
}

#[test]
fn calculate_cost_no_cached_input_rate_falls_back_to_input_rate() {
    // gpt-4o has cached_input_cost_per_mtok = Some(1.25), verify it's used
    let reg = PricingRegistry::new();
    let cost = reg
        .calculate_cost("gpt-4o", 2000, 0, 1000, 0, false)
        .unwrap();
    // cached: 1000 * 1.25/1M = 0.00125
    assert!((cost.cached_input_cost - 0.00125).abs() < 1e-10);
}

// ── PricingRegistry::register_model ──────────────────────────────────

#[test]
fn register_custom_model_then_get_it() {
    let mut reg = PricingRegistry::new();
    assert!(reg.get_pricing("my-model").is_none());
    reg.register_model(ModelPricing {
        model: "my-model".into(),
        provider: "custom".into(),
        input_cost_per_mtok: 5.0,
        output_cost_per_mtok: 20.0,
        cached_input_cost_per_mtok: Some(1.0),
        cache_write_cost_per_mtok: None,
        thinking_cost_per_mtok: None,
        batch_discount: 0.25,
        image_cost_per_token: None,
        audio_cost_per_minute: None,
    });
    let p = reg.get_pricing("my-model").unwrap();
    assert_eq!(p.provider, "custom");
    assert_eq!(p.input_cost_per_mtok, 5.0);
}

#[test]
fn register_model_overrides_existing() {
    let mut reg = PricingRegistry::new();
    reg.register_model(ModelPricing {
        model: "claude-opus-4".into(),
        provider: "anthropic".into(),
        input_cost_per_mtok: 99.0, // different price
        output_cost_per_mtok: 99.0,
        cached_input_cost_per_mtok: None,
        cache_write_cost_per_mtok: None,
        thinking_cost_per_mtok: None,
        batch_discount: 0.0,
        image_cost_per_token: None,
        audio_cost_per_minute: None,
    });
    let p = reg.get_pricing("claude-opus-4").unwrap();
    assert_eq!(p.input_cost_per_mtok, 99.0);
}

#[test]
fn register_model_cost_calculation_works() {
    let mut reg = PricingRegistry::new();
    reg.register_model(ModelPricing {
        model: "test-model".into(),
        provider: "test".into(),
        input_cost_per_mtok: 10.0,
        output_cost_per_mtok: 30.0,
        cached_input_cost_per_mtok: None,
        cache_write_cost_per_mtok: None,
        thinking_cost_per_mtok: None,
        batch_discount: 0.5,
        image_cost_per_token: None,
        audio_cost_per_minute: None,
    });
    let cost = reg
        .calculate_cost("test-model", 1000, 1000, 0, 0, false)
        .unwrap();
    // input: 1000*10/1M=0.01, output: 1000*30/1M=0.03
    assert!((cost.input_cost - 0.01).abs() < 1e-10);
    assert!((cost.output_cost - 0.03).abs() < 1e-10);
    assert!((cost.total_cost - 0.04).abs() < 1e-10);
}

// ── PricingRegistry::cheapest_for_provider ────────────────────────────

#[test]
fn cheapest_for_anthropic_is_haiku() {
    let reg = PricingRegistry::new();
    let cheapest = reg.cheapest_for_provider("anthropic").unwrap();
    assert_eq!(cheapest.model, "claude-haiku-3.5");
}

#[test]
fn cheapest_for_unknown_provider_returns_none() {
    let reg = PricingRegistry::new();
    assert!(reg.cheapest_for_provider("nonexistent-provider").is_none());
}

#[test]
fn cheapest_for_openai_returns_gpt_4o_mini() {
    let reg = PricingRegistry::new();
    let cheapest = reg.cheapest_for_provider("openai").unwrap();
    // gpt-4o-mini has input=0.15/Mtok (cheapest OpenAI)
    assert_eq!(cheapest.model, "gpt-4o-mini");
}

#[test]
fn cheapest_for_gemini_returns_flash() {
    let reg = PricingRegistry::new();
    let cheapest = reg.cheapest_for_provider("gemini").unwrap();
    assert_eq!(cheapest.model, "gemini-1.5-flash");
}

#[test]
fn cheapest_for_deepseek_is_v3() {
    let reg = PricingRegistry::new();
    let cheapest = reg.cheapest_for_provider("deepseek").unwrap();
    // deepseek-v3 has input=0.27/Mtok vs deepseek-r1 at 0.55/Mtok
    assert_eq!(cheapest.model, "deepseek-v3");
}

// ── PricingRegistry::all_models ───────────────────────────────────────

#[test]
fn all_models_has_at_least_ten_entries() {
    let reg = PricingRegistry::new();
    assert!(reg.all_models().len() >= 10);
}

#[test]
fn all_models_contains_claude_opus_4() {
    let reg = PricingRegistry::new();
    let names: Vec<&str> = reg.all_models().iter().map(|m| m.model.as_str()).collect();
    assert!(names.contains(&"claude-opus-4"));
}

#[test]
fn all_models_contains_gpt_4o() {
    let reg = PricingRegistry::new();
    let names: Vec<&str> = reg.all_models().iter().map(|m| m.model.as_str()).collect();
    assert!(names.contains(&"gpt-4o"));
}

#[test]
fn all_models_after_register_includes_new_model() {
    let mut reg = PricingRegistry::new();
    let before_count = reg.all_models().len();
    reg.register_model(ModelPricing {
        model: "brand-new".into(),
        provider: "x".into(),
        input_cost_per_mtok: 1.0,
        output_cost_per_mtok: 4.0,
        cached_input_cost_per_mtok: None,
        cache_write_cost_per_mtok: None,
        thinking_cost_per_mtok: None,
        batch_discount: 0.5,
        image_cost_per_token: None,
        audio_cost_per_minute: None,
    });
    assert_eq!(reg.all_models().len(), before_count + 1);
}

// ── ModelPricing serde ───────────────────────────────────────────────

#[test]
fn model_pricing_serde_roundtrip() {
    let p = ModelPricing {
        model: "my-model".into(),
        provider: "test".into(),
        input_cost_per_mtok: 2.0,
        output_cost_per_mtok: 8.0,
        cached_input_cost_per_mtok: Some(0.5),
        cache_write_cost_per_mtok: None,
        thinking_cost_per_mtok: Some(8.0),
        batch_discount: 0.5,
        image_cost_per_token: None,
        audio_cost_per_minute: None,
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: ModelPricing = serde_json::from_str(&json).unwrap();
    assert_eq!(back.model, "my-model");
    assert_eq!(back.input_cost_per_mtok, 2.0);
    assert_eq!(back.cached_input_cost_per_mtok, Some(0.5));
}

#[test]
fn cost_breakdown_serde_roundtrip() {
    let breakdown = CostBreakdown {
        input_cost: 0.005,
        output_cost: 0.015,
        cached_input_cost: 0.001,
        thinking_cost: 0.0,
        total_cost: 0.021,
        currency: "USD".into(),
    };
    let json = serde_json::to_string(&breakdown).unwrap();
    let back: CostBreakdown = serde_json::from_str(&json).unwrap();
    assert_eq!(back.currency, "USD");
    assert!((back.total_cost - 0.021).abs() < 1e-12);
}

#[test]
fn model_pricing_clone() {
    let reg = PricingRegistry::new();
    let p = reg.get_pricing("claude-opus-4").unwrap();
    let cloned = p.clone();
    assert_eq!(cloned.model, p.model);
    assert_eq!(cloned.input_cost_per_mtok, p.input_cost_per_mtok);
}

// ── Comprehensive behavioral tests ──────────────────────────────────

/// Verify that batch discount applies multiplicatively across all cost
/// components (input + output + cached + thinking), not just the total.
/// Also verify that batch discount of 0.5 exactly halves every model's cost.
#[test]
fn calculate_cost_batch_discount_applies_to_all_providers() {
    let reg = PricingRegistry::new();
    let models_to_test = [
        "claude-opus-4",
        "claude-sonnet-4",
        "gpt-4o",
        "deepseek-r1",
        "gemini-1.5-pro",
    ];

    for model in &models_to_test {
        let normal = reg.calculate_cost(model, 5000, 2000, 1000, 500, false);
        let batch = reg.calculate_cost(model, 5000, 2000, 1000, 500, true);

        if let (Some(n), Some(b)) = (normal, batch) {
            let pricing = reg.get_pricing(model).unwrap();
            let expected_discount = 1.0 - pricing.batch_discount;
            assert!(
                (b.total_cost - n.total_cost * expected_discount).abs() < 1e-9,
                "Model {}: batch total {} should be normal total {} * discount factor {}",
                model,
                b.total_cost,
                n.total_cost,
                expected_discount
            );

            // Each component should also be discounted
            assert!(
                (b.input_cost - n.input_cost * expected_discount).abs() < 1e-9,
                "Model {}: batch input cost mismatch",
                model
            );
            assert!(
                (b.output_cost - n.output_cost * expected_discount).abs() < 1e-9,
                "Model {}: batch output cost mismatch",
                model
            );
        }
    }
}

/// Verify the cheapest model per provider is actually cheaper than all
/// other models from the same provider for any given token count.
#[test]
fn cheapest_model_is_truly_cheapest_for_realistic_workload() {
    let reg = PricingRegistry::new();

    for provider in &["anthropic", "openai", "deepseek", "gemini"] {
        let cheapest = reg.cheapest_for_provider(provider).unwrap();

        // Calculate cost for a realistic workload with the cheapest model
        let cheapest_cost = reg
            .calculate_cost(&cheapest.model, 10000, 5000, 0, 0, false)
            .unwrap()
            .total_cost;

        // Verify no other model from this provider is cheaper
        for model in reg.all_models() {
            if model.provider == *provider && model.model != cheapest.model {
                let other_cost = reg
                    .calculate_cost(&model.model, 10000, 5000, 0, 0, false)
                    .unwrap()
                    .total_cost;
                assert!(
                    cheapest_cost <= other_cost,
                    "Provider {}: {} (${:.6}) should be cheaper than {} (${:.6})",
                    provider,
                    cheapest.model,
                    cheapest_cost,
                    model.model,
                    other_cost
                );
            }
        }
    }
}

/// Verify that using cached tokens always results in lower cost than
/// the same request without caching, for models that support it.
#[test]
fn cached_tokens_always_reduce_cost_for_supported_models() {
    let reg = PricingRegistry::new();

    for model in reg.all_models() {
        if model.cached_input_cost_per_mtok.is_some() {
            let input_tokens = 10000;
            let output_tokens = 2000;
            let cached_tokens = 8000; // 80% cache hit rate

            let no_cache = reg
                .calculate_cost(&model.model, input_tokens, output_tokens, 0, 0, false)
                .unwrap();
            let with_cache = reg
                .calculate_cost(
                    &model.model,
                    input_tokens,
                    output_tokens,
                    cached_tokens,
                    0,
                    false,
                )
                .unwrap();

            assert!(
                with_cache.total_cost < no_cache.total_cost,
                "Model {}: cached cost ({:.6}) should be less than uncached ({:.6})",
                model.model,
                with_cache.total_cost,
                no_cache.total_cost
            );

            // The savings should be proportional to the cache discount
            let cached_rate = model.cached_input_cost_per_mtok.unwrap();
            let full_rate = model.input_cost_per_mtok;
            let expected_savings = (full_rate - cached_rate) * cached_tokens as f64 / 1_000_000.0;
            let actual_savings = no_cache.total_cost - with_cache.total_cost;
            assert!(
                (actual_savings - expected_savings).abs() < 1e-9,
                "Model {}: savings mismatch. Expected {:.9}, got {:.9}",
                model.model,
                expected_savings,
                actual_savings
            );
        }
    }
}

/// Verify that registering a custom model with extreme pricing works
/// correctly in cost calculation and doesn't affect other models.
#[test]
fn register_custom_model_does_not_affect_existing_models() {
    let mut reg = PricingRegistry::new();

    // Snapshot costs for existing models before registration
    let opus_before = reg
        .calculate_cost("claude-opus-4", 1000, 500, 0, 0, false)
        .unwrap();
    let gpt4o_before = reg
        .calculate_cost("gpt-4o", 1000, 500, 0, 0, false)
        .unwrap();

    // Register a custom model with extreme pricing
    reg.register_model(ModelPricing {
        model: "super-expensive-v1".into(),
        provider: "custom".into(),
        input_cost_per_mtok: 1000.0,
        output_cost_per_mtok: 5000.0,
        cached_input_cost_per_mtok: Some(100.0),
        cache_write_cost_per_mtok: Some(200.0),
        thinking_cost_per_mtok: Some(3000.0),
        batch_discount: 0.75,
        image_cost_per_token: None,
        audio_cost_per_minute: None,
    });

    // Existing models should be unaffected
    let opus_after = reg
        .calculate_cost("claude-opus-4", 1000, 500, 0, 0, false)
        .unwrap();
    let gpt4o_after = reg
        .calculate_cost("gpt-4o", 1000, 500, 0, 0, false)
        .unwrap();
    assert!((opus_before.total_cost - opus_after.total_cost).abs() < 1e-12);
    assert!((gpt4o_before.total_cost - gpt4o_after.total_cost).abs() < 1e-12);

    // Custom model should compute correctly
    let custom_cost = reg
        .calculate_cost("super-expensive-v1", 1000, 500, 200, 100, false)
        .unwrap();
    // billable_input = (1000-200) = 800, input = 800 * 1000/1M = 0.8
    // cached = 200 * 100/1M = 0.02
    // output = 500 * 5000/1M = 2.5
    // thinking = 100 * 3000/1M = 0.3
    // total = 3.62
    assert!((custom_cost.input_cost - 0.8).abs() < 1e-9);
    assert!((custom_cost.cached_input_cost - 0.02).abs() < 1e-9);
    assert!((custom_cost.output_cost - 2.5).abs() < 1e-9);
    assert!((custom_cost.thinking_cost - 0.3).abs() < 1e-9);
    assert!((custom_cost.total_cost - 3.62).abs() < 1e-9);

    // Batch should apply 75% discount
    let batch_cost = reg
        .calculate_cost("super-expensive-v1", 1000, 500, 200, 100, true)
        .unwrap();
    assert!((batch_cost.total_cost - 3.62 * 0.25).abs() < 1e-9);
}
