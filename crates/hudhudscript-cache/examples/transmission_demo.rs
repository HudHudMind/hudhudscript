//! Cache Transmission Demo
//!
//! Demonstrates the cache transmission protocol with retry logic

use hudhudscript_cache::{CacheTransmitter, CommandCache, TransmissionConfig, TransmissionResult};
use std::time::Duration;

fn main() {
    println!("=== Cache Transmission Protocol Demo ===\n");

    // Create a cache with some data
    let mut cache = CommandCache::new();
    println!("1. Created empty cache");

    // In a real scenario, you would populate the cache with governance structures
    // For this demo, we'll use an empty cache
    println!(
        "   Cache size: {} items\n",
        cache.constitutions.len() + cache.laws.len() + cache.rules.len()
    );

    // Create transmitter with custom configuration
    let config = TransmissionConfig {
        max_retries: 3,
        initial_retry_delay: Duration::from_secs(1),
        max_retry_delay: Duration::from_secs(60),
        backoff_multiplier: 2.0,
        validate_checksum: true,
    };
    let transmitter = CacheTransmitter::with_config(config);
    println!("2. Created transmitter with configuration:");
    println!("   - Max retries: 3");
    println!("   - Initial delay: 1s");
    println!("   - Max delay: 60s");
    println!("   - Backoff multiplier: 2.0");
    println!("   - Checksum validation: enabled\n");

    // Prepare full transmission
    let transmission = transmitter
        .prepare_full_transmission(
            &cache,
            Some("agent1".to_string()),
            Some("agent2".to_string()),
        )
        .expect("Failed to prepare transmission");

    println!("3. Prepared full cache transmission:");
    println!(
        "   - Transmission ID: {}",
        transmission.metadata.transmission_id
    );
    println!("   - Source: {:?}", transmission.metadata.source_id);
    println!(
        "   - Destination: {:?}",
        transmission.metadata.destination_id
    );
    println!("   - Item count: {}", transmission.metadata.item_count);
    println!("   - Checksum: {:?}\n", transmission.checksum);

    // Simulate transmission with retry logic
    println!("4. Simulating transmission with retry logic...");

    let mut attempt = 0;
    let result = transmitter.transmit_with_retry(&transmission, |_tx| {
        attempt += 1;
        println!("   Attempt {}: Sending...", attempt);

        // Simulate network failure for first 2 attempts
        if attempt < 3 {
            println!("   Attempt {}: Failed (simulated network error)", attempt);
            Err("Network timeout".to_string())
        } else {
            println!("   Attempt {}: Success!", attempt);
            Ok(())
        }
    });

    println!();
    match result {
        TransmissionResult::Success {
            items_transmitted, ..
        } => {
            println!("5. Transmission successful!");
            println!("   - Items transmitted: {}", items_transmitted);
            println!("   - Total attempts: {}", attempt);
        }
        TransmissionResult::Failed { error, retry_count } => {
            println!("5. Transmission failed!");
            println!("   - Error: {}", error);
            println!("   - Retry count: {}", retry_count);
        }
        _ => {}
    }

    // Receive transmission
    println!("\n6. Receiving transmission on target agent...");
    let mut target_cache = CommandCache::new();

    match transmitter.receive_transmission(&transmission, &mut target_cache) {
        Ok(TransmissionResult::Success {
            items_transmitted,
            duration,
        }) => {
            println!("   Received successfully!");
            println!("   - Items received: {}", items_transmitted);
            println!("   - Duration: {:?}", duration);
            println!("   - Checksum validated: ✓");
        }
        Err(e) => {
            println!("   Failed to receive: {}", e);
        }
        _ => {}
    }

    // Demonstrate retry delay calculation
    println!("\n7. Retry delay calculation (exponential backoff):");
    for i in 0..5 {
        let delay = transmitter.calculate_retry_delay(i);
        println!("   Retry {}: {:?}", i, delay);
    }

    println!("\n=== Demo Complete ===");
    println!("\nKey Features Demonstrated:");
    println!("✓ Full cache transmission preparation");
    println!("✓ Metadata tracking (ID, source, destination)");
    println!("✓ Checksum validation");
    println!("✓ Automatic retry with exponential backoff");
    println!("✓ Configurable retry parameters");
    println!("✓ Transmission success/failure handling");
}
