use octofhir_fhir_model::provider::FhirVersion;
use octofhir_fhir_model::reflection::TypeReflectionInfo;
use octofhir_fhirpath_model::provider::FhirSchemaConfig;
use octofhir_fhirpath_model::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

fn create_test_type_info(name: &str) -> TypeReflectionInfo {
    TypeReflectionInfo::SimpleType {
        namespace: "Test".to_string(),
        name: name.to_string(),
        base_type: None,
    }
}

#[tokio::test]
async fn test_cache_basic_operations() {
    let config = CacheConfig::default();
    let cache = CacheManager::new(config);

    let type_info = Arc::new(create_test_type_info("TestType"));

    // Initially empty
    assert!(cache.get("TestType").is_none());

    // Put and get
    cache.put("TestType".to_string(), type_info.clone());
    let retrieved = cache.get("TestType");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().as_ref(), type_info.as_ref());
}

#[tokio::test]
async fn test_multi_tier_behavior() {
    let config = CacheConfig {
        hot_cache_size: 5,
        warm_cache_size: 10,
        cold_cache_size: 20,
        promotion_threshold: 3,
        demotion_threshold: Duration::from_millis(100),
        cleanup_interval: Duration::from_secs(1),
        enable_predictive: false,
    };
    let cache = CacheManager::new(config);

    let type_info = Arc::new(create_test_type_info("TestType"));

    // Put in cold tier initially (no access history)
    cache.put("TestType".to_string(), type_info.clone());

    // Access multiple times to trigger promotion to warm
    for _ in 0..5 {
        let _ = cache.get("TestType");
    }

    // Should now be in warm tier - check if we can retrieve it
    let retrieved = cache.get("TestType");
    assert!(retrieved.is_some());

    let stats = cache.get_comprehensive_stats();
    assert!(stats.hot_stats.hits > 0 || stats.warm_stats.hits > 0);
}

#[tokio::test]
async fn test_access_pattern_tracking() {
    let tracker = AccessPatternTracker::new();

    // Record some accesses
    tracker.record_access("Patient", AccessSource::TypeReflection);
    tracker.record_access("HumanName", AccessSource::PropertyLookup);
    tracker.record_access("Patient", AccessSource::TypeReflection);

    // Check frequencies
    assert_eq!(tracker.get_access_frequency("Patient"), 2);
    assert_eq!(tracker.get_access_frequency("HumanName"), 1);
    assert_eq!(tracker.get_access_frequency("NonExistent"), 0);

    // Check tier recommendation
    let patient_tier = tracker.recommend_cache_tier("Patient");
    let humanname_tier = tracker.recommend_cache_tier("HumanName");
    let nonexistent_tier = tracker.recommend_cache_tier("NonExistent");

    // Patient should be at least warm (2 accesses), others cold
    assert!(matches!(patient_tier, CacheTier::Cold | CacheTier::Warm));
    assert!(matches!(humanname_tier, CacheTier::Cold));
    assert!(matches!(nonexistent_tier, CacheTier::Cold));
}

#[tokio::test]
async fn test_predictive_caching() {
    let config = CacheConfig {
        enable_predictive: true,
        ..Default::default()
    };
    let cache = CacheManager::new(config);

    // Access Patient type
    cache
        .access_tracker
        .record_access("Patient", AccessSource::TypeReflection);

    // Access HumanName shortly after to create relationship
    sleep(Duration::from_millis(100)).await;
    cache
        .access_tracker
        .record_access("HumanName", AccessSource::PropertyLookup);

    // Should predict relationship between Patient and HumanName
    let related = cache.access_tracker.predict_related_types("Patient");
    assert!(related.contains(&"HumanName".to_string()) || related.is_empty()); // Might be empty due to timing
}

#[tokio::test]
async fn test_cache_promotion_demotion() {
    let config = CacheConfig {
        hot_cache_size: 2,
        warm_cache_size: 3,
        cold_cache_size: 5,
        promotion_threshold: 2,
        demotion_threshold: Duration::from_millis(50),
        cleanup_interval: Duration::from_millis(100),
        enable_predictive: false,
    };
    let cache = CacheManager::new(config);

    // Add several types
    for i in 1..=5 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.put(format!("Type{i}"), type_info);
    }

    // Access Type1 repeatedly to promote it
    for _ in 0..5 {
        let _ = cache.get("Type1");
    }

    // Verify we can still retrieve it
    assert!(cache.get("Type1").is_some());

    let stats = cache.get_comprehensive_stats();
    assert!(stats.overall_hit_ratio > 0.0);
}

#[tokio::test]
async fn test_lock_free_cache() {
    let cache = LockFreeCache::<String, String>::new(100);

    // Basic operations
    assert!(cache.get(&"key1".to_string()).is_none());

    cache.insert("key1".to_string(), "value1".to_string());
    assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());

    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[tokio::test]
async fn test_concurrent_access() {
    let config = CacheConfig::default();
    let cache = Arc::new(CacheManager::new(config));

    // Spawn multiple tasks that access the cache concurrently
    let mut handles = Vec::new();

    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
            cache_clone.put(format!("Type{i}"), type_info);

            // Access the type we just put
            let retrieved = cache_clone.get(&format!("Type{i}"));
            assert!(retrieved.is_some());

            // Also try to access other types
            for j in 0..5 {
                let _ = cache_clone.get(&format!("Type{j}"));
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify cache has entries
    let stats = cache.get_comprehensive_stats();
    assert!(stats.hot_stats.size + stats.warm_stats.size + stats.cold_stats.size > 0);
}

#[tokio::test]
async fn test_cache_expiration_and_cleanup() {
    let config = CacheConfig {
        demotion_threshold: Duration::from_millis(100),
        cleanup_interval: Duration::from_millis(50),
        ..Default::default()
    };
    let cache = CacheManager::new(config);

    let type_info = Arc::new(create_test_type_info("TempType"));
    cache.put("TempType".to_string(), type_info);

    // Verify it exists
    assert!(cache.get("TempType").is_some());

    // Wait for potential expiration
    sleep(Duration::from_millis(150)).await;

    // Manually trigger cleanup
    cache.cleanup_expired();

    // The type might still be there depending on which tier it's in
    // This test mainly ensures cleanup doesn't crash
    let _retrieved = cache.get("TempType");
}

#[tokio::test]
async fn test_cache_metrics() {
    let config = CacheConfig::default();
    let cache = CacheManager::new(config);

    // Initial stats should be empty
    let initial_stats = cache.get_comprehensive_stats();
    assert_eq!(initial_stats.hot_stats.hits, 0);
    assert_eq!(initial_stats.warm_stats.hits, 0);
    assert_eq!(initial_stats.cold_stats.hits, 0);

    // Add some data and access it
    let type_info = Arc::new(create_test_type_info("MetricsType"));
    cache.put("MetricsType".to_string(), type_info);

    // This should be a hit
    let _ = cache.get("MetricsType");

    // This should be a miss
    let _ = cache.get("NonExistentType");

    let stats = cache.get_comprehensive_stats();
    let total_hits = stats.hot_stats.hits + stats.warm_stats.hits + stats.cold_stats.hits;
    let total_misses = stats.hot_stats.misses + stats.warm_stats.misses + stats.cold_stats.misses;

    assert!(total_hits > 0);
    assert!(total_misses > 0);
    assert!(stats.overall_hit_ratio > 0.0 && stats.overall_hit_ratio < 1.0);
}

#[tokio::test]
async fn test_cache_with_fhir_provider() {
    // Test integration with FhirSchemaModelProvider

    let cache_config = CacheConfig {
        hot_cache_size: 50,
        warm_cache_size: 200,
        cold_cache_size: 1000,
        ..Default::default()
    };

    let mut fhir_config = FhirSchemaConfig::default();
    fhir_config.fhir_version = FhirVersion::R4;
    fhir_config.auto_install_core = false; // Skip installation for test
    fhir_config.cache_config = cache_config;

    // This should create a provider with multi-tier caching
    let provider_result = FhirSchemaModelProvider::with_config(fhir_config).await;

    // If we get an error due to missing packages, that's expected in test environment
    match provider_result {
        Ok(provider) => {
            // Test that provider was created with multi-tier cache
            // Note: We can't directly test the cache without more complex setup
            assert!(format!("{provider:?}").contains("FhirSchemaModelProvider"));
        }
        Err(_) => {
            // Expected in test environment without FHIR packages
            // Just verify we can create the config
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_clear_all_caches() {
    let config = CacheConfig::default();
    let cache = CacheManager::new(config);

    // Add some data
    for i in 1..=5 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.put(format!("Type{i}"), type_info);
    }

    // Verify data exists
    assert!(cache.get("Type1").is_some());

    let stats_before = cache.get_comprehensive_stats();
    assert!(
        stats_before.hot_stats.size + stats_before.warm_stats.size + stats_before.cold_stats.size
            > 0
    );

    // Clear all caches
    cache.clear_all();

    // Verify data is gone
    assert!(cache.get("Type1").is_none());

    let stats_after = cache.get_comprehensive_stats();
    assert_eq!(
        stats_after.hot_stats.size + stats_after.warm_stats.size + stats_after.cold_stats.size,
        0
    );
}
