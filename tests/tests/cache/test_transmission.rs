use hudhudscript_cache::{
    CacheTransmission, CacheTransmitter, CommandCache, TransmissionConfig, TransmissionResult,
    TransmissionType,
};
use std::time::{Duration, SystemTime};

#[test]
fn test_transmission_metadata_creation() {
    use hudhudscript_cache::TransmissionMetadata;
    let metadata = TransmissionMetadata {
        version: "1.0".to_string(),
        timestamp: SystemTime::now(),
        item_count: 10,
        transmission_id: "tx-123".to_string(),
        source_id: Some("agent1".to_string()),
        destination_id: Some("agent2".to_string()),
        transmission_type: TransmissionType::Full,
    };

    assert_eq!(metadata.version, "1.0");
    assert_eq!(metadata.item_count, 10);
    assert_eq!(metadata.transmission_type, TransmissionType::Full);
}

#[test]
fn test_transmitter_creation() {
    let transmitter = CacheTransmitter::new();
    assert_eq!(transmitter.config.max_retries, 3);
    assert!(transmitter.config.validate_checksum);
}

#[test]
fn test_transmitter_with_config() {
    let config = TransmissionConfig {
        max_retries: 5,
        initial_retry_delay: Duration::from_secs(2),
        max_retry_delay: Duration::from_secs(120),
        backoff_multiplier: 3.0,
        validate_checksum: false,
    };

    let transmitter = CacheTransmitter::with_config(config.clone());
    assert_eq!(transmitter.config.max_retries, 5);
    assert!(!transmitter.config.validate_checksum);
}

#[test]
fn test_prepare_full_transmission() {
    let cache = CommandCache::new();
    let transmitter = CacheTransmitter::new();

    let result = transmitter.prepare_full_transmission(
        &cache,
        Some("agent1".to_string()),
        Some("agent2".to_string()),
    );

    assert!(result.is_ok());
    let transmission = result.unwrap();
    assert_eq!(
        transmission.metadata.transmission_type,
        TransmissionType::Full
    );
    assert!(transmission.checksum.is_some());
}

#[test]
fn test_prepare_incremental_transmission() {
    let cache = CommandCache::new();
    let transmitter = CacheTransmitter::new();
    let changed_ids = vec!["cons.1".to_string(), "cons.2".to_string()];

    let result = transmitter.prepare_incremental_transmission(
        &cache,
        changed_ids.clone(),
        Some("agent1".to_string()),
        None,
    );

    assert!(result.is_ok());
    let transmission = result.unwrap();
    assert_eq!(
        transmission.metadata.transmission_type,
        TransmissionType::Incremental
    );
    assert_eq!(transmission.metadata.item_count, 2);
}

#[test]
fn test_receive_transmission() {
    let cache = CommandCache::new();
    let transmitter = CacheTransmitter::new();

    let transmission = transmitter
        .prepare_full_transmission(
            &cache,
            Some("agent1".to_string()),
            Some("agent2".to_string()),
        )
        .unwrap();

    let mut target_cache = CommandCache::new();
    let result = transmitter.receive_transmission(&transmission, &mut target_cache);

    assert!(result.is_ok());
    match result.unwrap() {
        TransmissionResult::Success {
            items_transmitted, ..
        } => {
            assert_eq!(items_transmitted, 0);
        }
        _ => panic!("Expected success result"),
    }
}

#[test]
fn test_checksum_validation() {
    let cache = CommandCache::new();
    let transmitter = CacheTransmitter::new();

    let mut transmission = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    transmission.checksum = Some("invalid".to_string());

    let mut target_cache = CommandCache::new();
    let result = transmitter.receive_transmission(&transmission, &mut target_cache);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Checksum validation failed"));
}

#[test]
fn test_calculate_retry_delay() {
    let transmitter = CacheTransmitter::new();

    let delay0 = transmitter.calculate_retry_delay(0);
    let delay1 = transmitter.calculate_retry_delay(1);
    let delay2 = transmitter.calculate_retry_delay(2);

    assert_eq!(delay0, Duration::from_secs(1));
    assert_eq!(delay1, Duration::from_secs(2));
    assert_eq!(delay2, Duration::from_secs(4));
}

#[test]
fn test_retry_delay_max_cap() {
    let transmitter = CacheTransmitter::new();

    let delay = transmitter.calculate_retry_delay(10);
    assert!(delay <= Duration::from_secs(60));
}

#[test]
fn test_transmission_id_generation() {
    let transmitter = CacheTransmitter::new();

    let cache = CommandCache::new();
    let tx1 = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();
    let tx2 = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    assert_ne!(tx1.metadata.transmission_id, tx2.metadata.transmission_id);
}

#[test]
fn test_transmit_with_retry_success() {
    let cache = CommandCache::new();
    let transmitter = CacheTransmitter::new();
    let transmission = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    let result = transmitter.transmit_with_retry(&transmission, |_| Ok(()));

    match result {
        TransmissionResult::Success { .. } => {}
        _ => panic!("Expected success"),
    }
}

#[test]
fn test_transmit_with_retry_failure() {
    let cache = CommandCache::new();
    let mut config = TransmissionConfig::default();
    config.max_retries = 2;
    config.initial_retry_delay = Duration::from_millis(10);

    let transmitter = CacheTransmitter::with_config(config);
    let transmission = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    let result =
        transmitter.transmit_with_retry(&transmission, |_| Err("Network error".to_string()));

    match result {
        TransmissionResult::Failed { retry_count, .. } => {
            assert_eq!(retry_count, 2);
        }
        _ => panic!("Expected failure"),
    }
}

#[test]
fn test_checksum_calculation() {
    let transmitter = CacheTransmitter::new();

    let checksum1 = transmitter.calculate_checksum("test data");
    let checksum2 = transmitter.calculate_checksum("test data");
    let checksum3 = transmitter.calculate_checksum("different data");

    assert_eq!(checksum1, checksum2);
    assert_ne!(checksum1, checksum3);
}

#[test]
fn test_transmission_serialization() {
    let cache = CommandCache::new();
    let transmitter = CacheTransmitter::new();
    let transmission = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    let json = serde_json::to_string(&transmission).unwrap();
    let deserialized: CacheTransmission = serde_json::from_str(&json).unwrap();

    assert_eq!(
        transmission.metadata.transmission_id,
        deserialized.metadata.transmission_id
    );
    assert_eq!(
        transmission.metadata.item_count,
        deserialized.metadata.item_count
    );
}

#[test]
fn test_transmitter_default_trait() {
    let transmitter = CacheTransmitter::default();
    assert_eq!(transmitter.config.max_retries, 3);
}

#[test]
fn test_full_transmission_without_checksum() {
    let config = TransmissionConfig {
        max_retries: 3,
        initial_retry_delay: Duration::from_secs(1),
        max_retry_delay: Duration::from_secs(60),
        backoff_multiplier: 2.0,
        validate_checksum: false,
    };
    let transmitter = CacheTransmitter::with_config(config);
    let cache = CommandCache::new();

    let tx = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();
    assert!(tx.checksum.is_none());
}

#[test]
fn test_incremental_transmission_without_checksum() {
    let config = TransmissionConfig {
        max_retries: 3,
        initial_retry_delay: Duration::from_secs(1),
        max_retry_delay: Duration::from_secs(60),
        backoff_multiplier: 2.0,
        validate_checksum: false,
    };
    let transmitter = CacheTransmitter::with_config(config);
    let cache = CommandCache::new();

    let tx = transmitter
        .prepare_incremental_transmission(
            &cache,
            vec!["a".to_string()],
            Some("src".to_string()),
            Some("dst".to_string()),
        )
        .unwrap();
    assert!(tx.checksum.is_none());
    assert_eq!(tx.metadata.transmission_type, TransmissionType::Incremental);
    assert_eq!(tx.metadata.source_id.as_deref(), Some("src"));
    assert_eq!(tx.metadata.destination_id.as_deref(), Some("dst"));
}

#[test]
fn test_receive_transmission_without_checksum_validation() {
    let config = TransmissionConfig {
        max_retries: 3,
        initial_retry_delay: Duration::from_secs(1),
        max_retry_delay: Duration::from_secs(60),
        backoff_multiplier: 2.0,
        validate_checksum: false,
    };
    let transmitter = CacheTransmitter::with_config(config);
    let cache = CommandCache::new();

    let tx = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    let mut target = CommandCache::new();
    let result = transmitter.receive_transmission(&tx, &mut target);
    assert!(result.is_ok());
}

#[test]
fn test_receive_transmission_no_checksum_in_payload() {
    let transmitter = CacheTransmitter::new();
    let cache = CommandCache::new();

    let mut tx = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();
    tx.checksum = None;

    let mut target = CommandCache::new();
    let result = transmitter.receive_transmission(&tx, &mut target);
    assert!(result.is_ok());
}

#[test]
fn test_transmit_with_retry_partial_success() {
    let cache = CommandCache::new();
    let mut config = TransmissionConfig::default();
    config.max_retries = 5;
    config.initial_retry_delay = Duration::from_millis(1);

    let transmitter = CacheTransmitter::with_config(config);
    let transmission = transmitter
        .prepare_full_transmission(&cache, None, None)
        .unwrap();

    let mut attempts = 0;
    let result = transmitter.transmit_with_retry(&transmission, |_| {
        attempts += 1;
        if attempts < 3 {
            Err("transient error".to_string())
        } else {
            Ok(())
        }
    });

    match result {
        TransmissionResult::Success {
            items_transmitted, ..
        } => {
            assert_eq!(items_transmitted, 0);
        }
        _ => panic!("Expected success after retries"),
    }
}
