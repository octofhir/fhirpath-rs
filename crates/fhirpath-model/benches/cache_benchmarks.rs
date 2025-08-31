use divan::Bencher;
use octofhir_fhir_model::reflection::TypeReflectionInfo;
use octofhir_fhirpath_model::*;
use std::sync::Arc;

fn main() {
    divan::main();
}

fn create_test_type_info(name: &str) -> TypeReflectionInfo {
    TypeReflectionInfo::SimpleType {
        namespace: "Test".to_string(),
        name: name.to_string(),
        base_type: None,
    }
}

#[divan::bench]
fn bench_cache_insert(bencher: Bencher) {
    let cache = Cache::new(1000);

    bencher.bench_local(|| {
        let type_info = Arc::new(create_test_type_info("TestType"));
        cache.insert("TestType".to_string(), type_info);
    });
}

#[divan::bench]
fn bench_cache_get_hit(bencher: Bencher) {
    let cache = Cache::new(1000);

    // Pre-populate with data
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.insert(format!("Type{i}"), type_info);
    }

    bencher.bench_local(|| {
        let _ = cache.get(&"Type50".to_string());
    });
}

#[divan::bench]
fn bench_cache_get_miss(bencher: Bencher) {
    let cache = Cache::new(1000);

    // Pre-populate with data
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.insert(format!("Type{i}"), type_info);
    }

    bencher.bench_local(|| {
        // Access a key that doesn't exist
        let _ = cache.get(&"NonExistentType".to_string());
    });
}

#[divan::bench]
fn bench_cache_lru_eviction(bencher: Bencher) {
    let cache = Cache::new(50); // Small cache to force eviction

    // Pre-populate to capacity
    for i in 0..50 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.insert(format!("Type{i}"), type_info);
    }

    bencher.bench_local(|| {
        // This should evict the least recently used item
        let type_info = Arc::new(create_test_type_info("NewType"));
        cache.insert("NewType".to_string(), type_info);
    });
}

#[divan::bench]
fn bench_cache_concurrent_access(bencher: Bencher) {
    let cache = Cache::new(1000);

    // Pre-populate
    for i in 0..100 {
        let type_info = Arc::new(create_test_type_info(&format!("Type{i}")));
        cache.insert(format!("Type{i}"), type_info);
    }

    bencher.bench_local(|| {
        // Mix of reads and writes to test concurrent performance
        for i in 0..10 {
            let _ = cache.get(&format!("Type{}", i % 100));
            if i % 3 == 0 {
                let type_info = Arc::new(create_test_type_info(&format!("NewType{i}")));
                cache.insert(format!("NewType{i}"), type_info);
            }
        }
    });
}
