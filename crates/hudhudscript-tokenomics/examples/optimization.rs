//! Optimization strategy comparison

use hudhudscript_tokenomics::{OptimizationStrategy, TokenomicsEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("⚡ HudHudScript Tokenomics - Strategy Comparison\n");

    // Test different strategies
    let strategies = vec![
        (OptimizationStrategy::Conservative, "Conservative"),
        (OptimizationStrategy::Balanced, "Balanced"),
        (OptimizationStrategy::Aggressive, "Aggressive"),
        (
            OptimizationStrategy::Custom {
                performance_weight: 0.7,
                cost_weight: 0.3,
            },
            "Custom (70/30)",
        ),
    ];

    for (strategy, name) in strategies {
        println!("📊 Testing {} Strategy", name);
        println!("─────────────────────────────────────");

        let engine = TokenomicsEngine::new(strategy);
        let user_id = format!("user_{}", name.to_lowercase().replace(" ", "_"));

        // Create budget
        engine.get_or_create_budget(&user_id, 100_000).await?;

        // Simulate some usage
        for i in 0..10 {
            engine
                .consume_tokens(&user_id, 1000 + (i * 100), "operation")
                .await?;
        }

        // Get optimized allocation
        match engine.optimize_allocation(&user_id).await {
            Ok(allocation) => {
                println!("   Optimized allocation: {} tokens", allocation);
            }
            Err(e) => {
                println!("   Allocation failed: {}", e);
            }
        }

        // Get statistics
        let stats = engine.get_statistics(&user_id).await?;
        println!("   Total usage: {} tokens", stats.total_usage);
        println!("   Average: {:.2} tokens", stats.average_usage);
        println!();
    }

    println!("💡 Strategy Recommendations:");
    println!("   • Conservative: Best for cost-sensitive applications");
    println!("   • Balanced: Good default for most use cases");
    println!("   • Aggressive: Best for performance-critical applications");
    println!("   • Custom: Fine-tune based on your specific needs");

    Ok(())
}
