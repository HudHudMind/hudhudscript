//! Basic tokenomics usage example

use hudhudscript_tokenomics::{OptimizationStrategy, TokenomicsEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 HudHudScript Tokenomics - Basic Usage\n");

    // Create engine with balanced strategy
    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);

    // Create budget for user
    println!("Creating budget for user...");
    let budget = engine.get_or_create_budget("alice", 100_000).await?;
    println!("✓ Budget created: {} tokens\n", budget.total);

    // Simulate some operations
    println!("Simulating operations...");

    engine
        .consume_tokens("alice", 1000, "code_execution")
        .await?;
    println!("✓ Consumed 1000 tokens for code_execution");

    engine.consume_tokens("alice", 500, "syntax_check").await?;
    println!("✓ Consumed 500 tokens for syntax_check");

    engine.consume_tokens("alice", 2000, "compilation").await?;
    println!("✓ Consumed 2000 tokens for compilation");

    // Check remaining budget
    let budget = engine.get_or_create_budget("alice", 100_000).await?;
    println!("\n📊 Current Status:");
    println!("   Total: {} tokens", budget.total);
    println!("   Used: {} tokens", budget.used);
    println!("   Remaining: {} tokens", budget.remaining);
    println!("   Usage: {:.1}%", budget.usage_percentage());

    // Get statistics
    let stats = engine.get_statistics("alice").await?;
    println!("\n📈 Statistics:");
    println!("   Total usage: {} tokens", stats.total_usage);
    println!(
        "   Average per operation: {:.2} tokens",
        stats.average_usage
    );
    println!("   Number of operations: {}", stats.operation_count);

    Ok(())
}
