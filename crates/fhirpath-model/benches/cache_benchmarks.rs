use divan::Bencher;
use octofhir_fhir_model::reflection::TypeReflectionInfo;
use octofhir_fhirpath_model::*;
use std::sync::Arc;

fn create_test_type_info(name: &str) -> TypeReflectionInfo {
    TypeReflectionInfo::SimpleType {
        namespace: "Test".to_string(),
        name: name.to_string(),
        base_type: None,
    }
}

#[divan::bench]
fn bench_cache_put(bencher: Bencher) {
    let config = CacheConfig::default();
    let cache = CacheManager::new(config);

    bencher.bench_local(|| {
        let type_info = Arc::new(create_test_type_info("TestType"));
        cache.put("TestType".to_string(), type_info);
    });
}

#[divan::bench]
fn bench_cache_get_hot(bencher: Bencher) {
    let config = CacheConfig {
        hot_cache_size: 1000,
        ..Default::default()
    };
    let cache = CacheManager::new(config);

    // Pre-populate with data that will be in hot cache
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("HotType{i}")));
        cache.put(format!("HotType{i}"), type_info);
        // Access multiple times to promote to hot cache
        for _ in 0..20 {
            let _ = cache.get(&format!("HotType{i}"));
        }
    }

    bencher.bench_local(|| {
        let _ = cache.get("HotType50");
    });
}

#[divan::bench]
fn bench_cache_get_warm(bencher: Bencher) {
    let config = CacheConfig {
        promotion_threshold: 100, // Make it hard to promote to hot
        ..Default::default()
    };
    let cache = CacheManager::new(config);

    // Pre-populate with data that will stay in warm cache
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("WarmType{i}")));
        cache.put(format!("WarmType{i}"), type_info);
        // Access a few times to keep in warm
        for _ in 0..5 {
            let _ = cache.get(&format!("WarmType{i}"));
        }
    }

    bencher.bench_local(|| {
        let _ = cache.get("WarmType50");
    });
}

#[divan::bench]
fn bench_cache_get_cold(bencher: Bencher) {
    let config = CacheConfig::default();
    let cache = CacheManager::new(config);

    // Pre-populate with data that will be in cold storage
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("ColdType{i}")));
        cache.put(format!("ColdType{i}"), type_info);
        // Don't access them, so they stay cold
    }

    bencher.bench_local(|| {
        let _ = cache.get("ColdType50");
    });
}

#[divan::bench]
fn bench_lock_free_cache_get(bencher: Bencher) {
    let cache = LockFreeCache::<String, String>::new(1000);

    // Pre-populate
    for i in 0..100 {
        cache.insert(format!("Key{i}"), format!("Value{i}"));
    }

    bencher.bench_local(|| {
        let _ = cache.get(&"Key50".to_string());
    });
}

#[divan::bench]
fn bench_lock_free_cache_insert(bencher: Bencher) {
    let cache = LockFreeCache::<String, String>::new(1000);
    let mut counter = 0;

    bencher.bench_local(|| {
        cache.insert(format!("Key{counter}"), format!("Value{counter}"));
        counter += 1;
    });
}

#[divan::bench]
fn bench_access_pattern_tracker_record(bencher: Bencher) {
    let tracker = AccessPatternTracker::new();
    let mut counter = 0;

    bencher.bench_local(|| {
        tracker.record_access(
            &format!("Type{}", counter % 10),
            AccessSource::TypeReflection,
        );
        counter += 1;
    });
}

#[divan::bench]
fn bench_access_pattern_tracker_frequency(bencher: Bencher) {
    let tracker = AccessPatternTracker::new();

    // Pre-populate with some access records
    for i in 0..100 {
        tracker.record_access(&format!("Type{}", i % 10), AccessSource::TypeReflection);
    }

    bencher.bench_local(|| {
        let _ = tracker.get_access_frequency("Type5");
    });
}

#[divan::bench]
fn bench_cache_concurrent_mixed_workload(bencher: Bencher) {
    let config = CacheConfig::default();
    let cache = Arc::new(CacheManager::new(config));

    // Pre-populate with some data
    for i in 0..50 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.put(format!("Type{i}"), type_info);
    }

    bencher.with_inputs(|| cache.clone()).bench_refs(|cache| {
        // Mixed workload: 70% reads, 30% writes
        if fastrand::f32() < 0.7 {
            // Read operation
            let key = format!("Type{}", fastrand::usize(0..50));
            let _ = cache.get(&key);
        } else {
            // Write operation
            let key = format!("NewType{}", fastrand::usize(0..1000));
            let type_info = Arc::new(create_test_type_info(&key));
            cache.put(key, type_info);
        }
    });
}

#[divan::bench]
fn bench_cache_stats_collection(bencher: Bencher) {
    let config = CacheConfig::default();
    let cache = CacheManager::new(config);

    // Pre-populate with some data and activity
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.put(format!("Type{i}"), type_info);
        let _ = cache.get(&format!("Type{i}"));
    }

    bencher.bench_local(|| {
        let _ = cache.get_comprehensive_stats();
    });
}

fn main() {
    divan::main();
}
