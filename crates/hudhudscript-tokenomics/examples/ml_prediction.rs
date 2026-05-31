//! ML-powered prediction example

use hudhudscript_tokenomics::{OptimizationStrategy, TokenomicsEngine};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🤖 HudHudScript Tokenomics - ML Prediction\n");

    let engine = TokenomicsEngine::new(OptimizationStrategy::Balanced);

    // Create user and simulate usage pattern
    println!("Creating user and simulating usage...");
    engine.get_or_create_budget("bob", 100_000).await?;

    // Simulate 20 operations with varying usage
    for i in 0..20 {
        let amount = 1000 + (i * 100);
        engine
            .consume_tokens("bob", amount, "code_execution")
            .await?;

        if i % 5 == 0 {
            println!("✓ Completed {} operations", i + 1);
        }

        sleep(Duration::from_millis(10)).await;
    }

    println!("\n🔮 Making predictions...\n");

    // Predict for different time horizons
    let horizons = vec![(3600, "1 hour"), (7200, "2 hours"), (86400, "24 hours")];

    for (seconds, label) in horizons {
        match engine.predict_usage("bob", seconds).await {
            Ok(prediction) => {
                println!("📊 Prediction for {}:", label);
                println!("   Expected usage: {} tokens", prediction.predicted_usage);
                println!("   Confidence: {:.1}%", prediction.confidence * 100.0);
                println!("   Model: {}", prediction.model_version);
                println!();
            }
            Err(e) => {
                println!("⚠️  Prediction failed for {}: {}", label, e);
                println!();
            }
        }
    }

    // Train model with accumulated data
    println!("🎓 Training ML model...");
    match engine.train_model().await {
        Ok(_) => {
            println!("✓ Model trained successfully");

            // Make prediction again with trained model
            if let Ok(prediction) = engine.predict_usage("bob", 3600).await {
                println!("\n📊 Improved prediction (after training):");
                println!("   Expected usage: {} tokens", prediction.predicted_usage);
                println!("   Confidence: {:.1}%", prediction.confidence * 100.0);
            }
        }
        Err(e) => {
            println!("⚠️  Training failed: {}", e);
        }
    }

    Ok(())
}
