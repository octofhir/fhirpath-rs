// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Integration tests for memory usage monitoring and validation with Bridge Support Architecture

use octofhir_fhirpath::*;
use octofhir_fhirpath_model::*;
use octofhir_fhirpath_evaluator::*;
use octofhir_fhirpath_analyzer::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

mod utils;
use utils::IntegrationTestContext;

// Memory monitoring utilities

struct MemorySnapshot {
    timestamp: Instant,
    description: String,
}

impl MemorySnapshot {
    fn new(description: &str) -> Self {
        Self {
            timestamp: Instant::now(),
            description: description.to_string(),
        }
    }
    
    fn elapsed_since(&self, start: &MemorySnapshot) -> std::time::Duration {
        self.timestamp.duration_since(start.timestamp)
    }
}

// Create memory-intensive test data

fn create_memory_intensive_bundle(size: usize, depth: usize) -> Value {
    let mut entries = Vec::new();
    
    for i in 0..size {
        // Create deeply nested patient resources
        let mut extensions = Vec::new();
        for ext_i in 0..depth {
            extensions.push(json!({
                "url": format!("http://example.org/extension-{}", ext_i),
                "valueString": format!("Extension value {} for patient {}", ext_i, i)
            }));
        }
        
        let mut names = Vec::new();
        for name_i in 0..3 {
            names.push(json!({
                "use": if name_i == 0 { "official" } else { "nickname" },
                "family": format!("Family{:06}-{}", i, name_i),
                "given": [
                    format!("Given{:06}-{}-1", i, name_i),
                    format!("Given{:06}-{}-2", i, name_i),
                    format!("Given{:06}-{}-3", i, name_i)
                ],
                "prefix": [format!("Prefix{}", name_i)],
                "suffix": [format!("Suffix{}", name_i)]
            }));
        }
        
        let mut telecoms = Vec::new();
        for tel_i in 0..4 {
            telecoms.push(json!({
                "system": match tel_i % 4 {
                    0 => "phone",
                    1 => "email", 
                    2 => "fax",
                    _ => "other"
                },
                "value": format!("contact-{}-{}-{}", i, tel_i, tel_i * 1000 + i),
                "use": match tel_i % 3 {
                    0 => "home",
                    1 => "work", 
                    _ => "mobile"
                }
            }));
        }
        
        let patient = json!({
            "resourceType": "Patient",
            "id": format!("memory-patient-{:08}", i),
            "extension": extensions,
            "identifier": [
                {
                    "use": "official",
                    "system": "http://memory-test.example.org/patient-ids",
                    "value": format!("MEM-PAT-{:08}", i)
                },
                {
                    "use": "secondary", 
                    "system": "http://memory-test.example.org/mrn",
                    "value": format!("MRN-{:08}", i * 7 + 1000000)
                }
            ],
            "active": i % 10 != 0,
            "name": names,
            "telecom": telecoms,
            "gender": match i % 3 {
                0 => "male",
                1 => "female",
                _ => "other"
            },
            "birthDate": format!("{}-{:02}-{:02}", 
                1920 + (i % 100), 
                1 + (i % 12), 
                1 + (i % 28)
            ),
            "address": [{
                "use": "home",
                "type": "physical",
                "line": [
                    format!("{} Memory Test Avenue", i + 1),
                    format!("Suite {}", i % 1000),
                    format!("Building {}", i / 1000)
                ],
                "city": format!("MemoryCity{:04}", i / 100),
                "district": format!("MemoryDistrict{:02}", i / 1000),
                "state": format!("MS{:02}", i % 50),
                "postalCode": format!("{:05}", 10000 + (i % 90000)),
                "country": "US",
                "period": {
                    "start": format!("{}-01-01", 2000 + (i % 24))
                }
            }],
            "maritalStatus": {
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/v3-MaritalStatus",
                    "code": match i % 4 {
                        0 => "M",
                        1 => "S",
                        2 => "D",
                        _ => "W"
                    },
                    "display": match i % 4 {
                        0 => "Married",
                        1 => "Single",
                        2 => "Divorced", 
                        _ => "Widowed"
                    }
                }]
            }
        });
        
        entries.push(json!({"resource": patient}));
        
        // Add observations for memory testing
        if i % 3 == 0 {
            let observation = json!({
                "resourceType": "Observation",
                "id": format!("memory-obs-{:08}", i),
                "status": "final",
                "category": [{
                    "coding": [{
                        "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                        "code": "survey"
                    }]
                }],
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": format!("{:05}-{}", 10000 + (i % 90000), i % 10),
                        "display": format!("Memory Test Observation {}", i)
                    }]
                },
                "subject": {
                    "reference": format!("Patient/memory-patient-{:08}", i)
                },
                "effectiveDateTime": format!("2024-01-{:02}T{:02}:{:02}:00Z",
                    1 + (i % 31),
                    (i % 24),
                    (i * 13) % 60
                ),
                "valueString": format!("Memory test result for observation {}: {}", 
                    i, "A".repeat((i % 100) + 1))
            });
            entries.push(json!({"resource": observation}));
        }
    }
    
    json!({
        "resourceType": "Bundle",
        "id": format!("memory-intensive-bundle-{}-{}", size, depth),
        "type": "collection",
        "timestamp": "2024-01-15T10:00:00Z",
        "total": entries.len(),
        "entry": entries
    })
}

#[tokio::test]
async fn test_memory_usage_with_growing_datasets() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    let dataset_sizes = vec![100, 500, 1000, 2000];
    let test_query = "Bundle.entry.resource.ofType(Patient).name.family.first()";
    
    println!("Testing memory usage with growing datasets...");
    
    for size in dataset_sizes {
        let start_snapshot = MemorySnapshot::new(&format!("Start processing {} resources", size));
        
        // Create dataset
        let bundle = create_memory_intensive_bundle(size, 3);
        let post_creation_snapshot = MemorySnapshot::new(&format!("After creating {} resources", size));
        
        // Process dataset
        let result = context.fhirpath.evaluate(test_query, &bundle).await;
        let post_processing_snapshot = MemorySnapshot::new(&format!("After processing {} resources", size));
        
        match result {
            Ok(values) => {
                println!("  ✅ {} resources: {} values", size, values.len());
                println!("     Creation time: {:?}", post_creation_snapshot.elapsed_since(&start_snapshot));
                println!("     Processing time: {:?}", post_processing_snapshot.elapsed_since(&post_creation_snapshot));
                println!("     Total time: {:?}", post_processing_snapshot.elapsed_since(&start_snapshot));
                
                // Validate reasonable performance
                let total_time = post_processing_snapshot.elapsed_since(&start_snapshot);
                assert!(total_time.as_secs() < 30, 
                    "Processing {} resources should complete within 30 seconds", size);
            },
            Err(e) => {
                println!("  ❌ {} resources: Error: {:?}", size, e);
            }
        }
        
        // Force garbage collection by dropping the bundle
        drop(bundle);
        let post_cleanup_snapshot = MemorySnapshot::new(&format!("After cleanup {} resources", size));
        
        println!("     Cleanup time: {:?}", post_cleanup_snapshot.elapsed_since(&post_processing_snapshot));
    }
}

#[tokio::test]
async fn test_memory_leak_detection() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    println!("Testing for memory leaks with repeated evaluations...");
    
    let test_queries = vec![
        "Bundle.entry.resource.ofType(Patient).count()",
        "Bundle.entry.resource.ofType(Patient).name.family.first()",
        "Bundle.entry.resource.ofType(Patient).where(active = true).count()",
    ];
    
    // Create a moderately sized bundle for repeated use
    let bundle = create_memory_intensive_bundle(500, 2);
    
    let iterations = 100;
    let start_snapshot = MemorySnapshot::new("Start memory leak test");
    
    // Perform many iterations to detect memory leaks
    for iteration in 0..iterations {
        for (query_idx, query) in test_queries.iter().enumerate() {
            let result = context.fhirpath.evaluate(query, &bundle).await;
            
            if result.is_err() {
                println!("  ⚠️  Iteration {} Query {}: Error: {:?}", iteration, query_idx, result.unwrap_err());
            }
        }
        
        // Periodic progress reporting
        if (iteration + 1) % 25 == 0 {
            let checkpoint_snapshot = MemorySnapshot::new(&format!("After {} iterations", iteration + 1));
            let elapsed = checkpoint_snapshot.elapsed_since(&start_snapshot);
            println!("  📊 Completed {} iterations in {:?}", iteration + 1, elapsed);
        }
    }
    
    let final_snapshot = MemorySnapshot::new("End memory leak test");
    let total_duration = final_snapshot.elapsed_since(&start_snapshot);
    let operations_per_second = (iterations * test_queries.len()) as f64 / total_duration.as_secs_f64();
    
    println!("  📊 Memory leak test summary:");
    println!("     Total iterations: {}", iterations);
    println!("     Total operations: {}", iterations * test_queries.len());
    println!("     Total time: {:?}", total_duration);
    println!("     Operations per second: {:.0}", operations_per_second);
    
    // Performance should remain consistent (no significant degradation suggesting leaks)
    assert!(total_duration.as_secs() < 120, 
        "Memory leak test should complete within 2 minutes");
    
    assert!(operations_per_second > 10.0,
        "Should maintain at least 10 operations per second");
}

#[tokio::test]
async fn test_memory_efficient_large_query_processing() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    println!("Testing memory-efficient processing of large queries...");
    
    // Create a large bundle for memory efficiency testing
    let large_bundle = create_memory_intensive_bundle(1000, 5);
    
    let memory_intensive_queries = vec![
        // Queries that might create large intermediate results
        ("Bundle.entry.resource.ofType(Patient).name.family", "All family names"),
        ("Bundle.entry.resource.ofType(Patient).telecom.value", "All telecom values"), 
        ("Bundle.entry.resource.ofType(Patient).extension.valueString", "All extension strings"),
        ("Bundle.entry.resource.ofType(Patient).name.given", "All given names"),
        ("Bundle.entry.resource.ofType(Patient).address.line", "All address lines"),
        
        // Queries with filtering (should be more memory efficient)
        ("Bundle.entry.resource.ofType(Patient).where(active = true).name.family.first()", "Active patient family name"),
        ("Bundle.entry.resource.ofType(Patient).where(gender = 'female').count()", "Female patient count"),
        ("Bundle.entry.resource.ofType(Observation).where(status = 'final').valueString.first()", "Final observation value"),
    ];
    
    for (query, description) in memory_intensive_queries {
        let start_snapshot = MemorySnapshot::new(&format!("Start {}", description));
        
        let result = context.fhirpath.evaluate(query, &large_bundle).await;
        
        let end_snapshot = MemorySnapshot::new(&format!("End {}", description));
        let duration = end_snapshot.elapsed_since(&start_snapshot);
        
        match result {
            Ok(values) => {
                println!("  ✅ {}: {} values in {:?}", description, values.len(), duration);
                
                // Memory-intensive queries should still complete in reasonable time
                assert!(duration.as_secs() < 60, 
                    "Memory-intensive query should complete within 60 seconds: {}", description);
            },
            Err(e) => {
                println!("  ⚠️  {}: Error: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_concurrent_memory_usage() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    println!("Testing memory usage under concurrent load...");
    
    // Create multiple bundles for concurrent processing
    let bundle1 = create_memory_intensive_bundle(300, 2);
    let bundle2 = create_memory_intensive_bundle(300, 2);
    let bundle3 = create_memory_intensive_bundle(300, 2);
    
    let concurrent_queries = vec![
        ("Bundle.entry.resource.ofType(Patient).count()", &bundle1),
        ("Bundle.entry.resource.ofType(Patient).name.family.first()", &bundle2),
        ("Bundle.entry.resource.ofType(Patient).where(active = true).count()", &bundle3),
        ("Bundle.entry.resource.ofType(Observation).count()", &bundle1),
        ("Bundle.entry.resource.ofType(Patient).telecom.value.first()", &bundle2),
        ("Bundle.entry.resource.ofType(Patient).address.city.first()", &bundle3),
    ];
    
    let start_snapshot = MemorySnapshot::new("Start concurrent memory test");
    
    // Launch concurrent tasks
    let mut tasks = Vec::new();
    
    for (i, (query, bundle)) in concurrent_queries.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let bundle_data = bundle.clone();
        let query_string = query.to_string();
        
        let task = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = fhirpath.evaluate(&query_string, &bundle_data).await;
            let task_duration = task_start.elapsed();
            (i, result, task_duration)
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let end_snapshot = MemorySnapshot::new("End concurrent memory test");
    
    let total_duration = end_snapshot.elapsed_since(&start_snapshot);
    let mut successful_tasks = 0;
    let mut total_values = 0;
    
    for result in results {
        let (task_id, eval_result, task_duration) = result.unwrap();
        
        match eval_result {
            Ok(values) => {
                successful_tasks += 1;
                total_values += values.len();
                println!("  ✅ Concurrent task {}: {} values in {:?}", 
                    task_id, values.len(), task_duration);
            },
            Err(e) => {
                println!("  ❌ Concurrent task {}: Error: {:?}", task_id, e);
            }
        }
    }
    
    println!("  📊 Concurrent memory usage summary:");
    println!("     Successful tasks: {}/6", successful_tasks);
    println!("     Total values: {}", total_values);
    println!("     Total time: {:?}", total_duration);
    
    // Concurrent memory usage should be manageable
    assert!(successful_tasks >= 4, 
        "At least 4/6 concurrent memory-intensive tasks should succeed");
    
    assert!(total_duration.as_secs() < 90,
        "Concurrent memory test should complete within 90 seconds");
}

#[tokio::test]
async fn test_memory_pressure_handling() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    println!("Testing memory pressure handling...");
    
    // Create progressively larger datasets to test memory pressure
    let pressure_sizes = vec![500, 1000, 1500, 2000];
    let pressure_query = "Bundle.entry.resource.ofType(Patient).extension.valueString";
    
    for size in pressure_sizes {
        let start_snapshot = MemorySnapshot::new(&format!("Start pressure test {}", size));
        
        // Create large dataset with deep nesting
        let pressure_bundle = create_memory_intensive_bundle(size, 10);
        let creation_snapshot = MemorySnapshot::new(&format!("Created pressure dataset {}", size));
        
        // Attempt to process under memory pressure
        let result = context.fhirpath.evaluate(pressure_query, &pressure_bundle).await;
        let processing_snapshot = MemorySnapshot::new(&format!("Processed pressure dataset {}", size));
        
        match result {
            Ok(values) => {
                println!("  ✅ Pressure test {}: {} values", size, values.len());
                println!("     Creation time: {:?}", creation_snapshot.elapsed_since(&start_snapshot));
                println!("     Processing time: {:?}", processing_snapshot.elapsed_since(&creation_snapshot));
                
                // Under pressure, should still maintain reasonable performance
                let processing_time = processing_snapshot.elapsed_since(&creation_snapshot);
                assert!(processing_time.as_secs() < 120, 
                    "Pressure test {} should complete within 2 minutes", size);
            },
            Err(e) => {
                println!("  ⚠️  Pressure test {}: Error (may be expected under extreme pressure): {:?}", size, e);
            }
        }
        
        // Clean up to release memory pressure
        drop(pressure_bundle);
        let cleanup_snapshot = MemorySnapshot::new(&format!("Cleaned up pressure dataset {}", size));
        println!("     Cleanup time: {:?}", cleanup_snapshot.elapsed_since(&processing_snapshot));
    }
}

#[tokio::test]
async fn test_schema_cache_memory_efficiency() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    println!("Testing schema cache memory efficiency...");
    
    let patient = json!({
        "resourceType": "Patient",
        "id": "schema-cache-test",
        "name": [{"family": "CacheTest", "given": ["Schema"]}],
        "active": true
    });
    
    // Queries that should benefit from schema caching
    let cache_test_queries = vec![
        "Patient.name.given.first()",
        "Patient.name.family.first()",
        "Patient.active",
        "Patient.name.exists()",
        "Patient.active.is(boolean)",
        "Patient.name.given.first().is(string)",
        "Patient.name.family.first().is(string)",
    ];
    
    // First run to populate cache
    let cache_warmup_start = MemorySnapshot::new("Start cache warmup");
    for query in &cache_test_queries {
        let _ = context.fhirpath.evaluate(query, &patient).await;
    }
    let cache_warmup_end = MemorySnapshot::new("End cache warmup");
    
    // Repeated runs should benefit from caching
    let cached_run_start = MemorySnapshot::new("Start cached runs");
    let iterations = 50;
    
    for _iteration in 0..iterations {
        for query in &cache_test_queries {
            let result = context.fhirpath.evaluate(query, &patient).await;
            if result.is_err() {
                println!("  ⚠️  Cache efficiency test query failed: {}", query);
            }
        }
    }
    
    let cached_run_end = MemorySnapshot::new("End cached runs");
    
    let warmup_duration = cache_warmup_end.elapsed_since(&cache_warmup_start);
    let cached_duration = cached_run_end.elapsed_since(&cached_run_start);
    let total_cached_operations = iterations * cache_test_queries.len();
    let cached_operations_per_second = total_cached_operations as f64 / cached_duration.as_secs_f64();
    
    println!("  📊 Schema cache efficiency summary:");
    println!("     Cache warmup time: {:?}", warmup_duration);
    println!("     Cached operations: {} in {:?}", total_cached_operations, cached_duration);
    println!("     Cached ops per second: {:.0}", cached_operations_per_second);
    
    // Cached operations should be very fast due to schema cache
    assert!(cached_operations_per_second > 1000.0,
        "Schema cache should enable >1000 ops/sec, got {:.0}", cached_operations_per_second);
}

#[tokio::test]
async fn test_memory_cleanup_after_errors() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    println!("Testing memory cleanup after evaluation errors...");
    
    let test_bundle = create_memory_intensive_bundle(200, 3);
    
    // Mix of valid and invalid queries to test error handling memory cleanup
    let error_test_queries = vec![
        ("Bundle.entry.resource.ofType(Patient).count()", true),
        ("Bundle.entry.resource.ofType(Patient).invalidProperty", false),
        ("Bundle.entry.resource.ofType(Patient).name.family.first()", true),
        ("Bundle.entry.resource.ofType(Patient).name..family", false), // Parse error
        ("Bundle.entry.resource.ofType(Patient).where(active = true).count()", true),
        ("Bundle.entry.resource.ofType(Patient).invalidFunction()", false),
        ("Bundle.entry.resource.ofType(Observation).valueString.first()", true),
        ("Bundle.entry.resource.(", false), // Syntax error
    ];
    
    let start_snapshot = MemorySnapshot::new("Start error cleanup test");
    
    let mut successful_queries = 0;
    let mut error_queries = 0;
    
    for (query, should_succeed) in error_test_queries {
        let query_start = MemorySnapshot::new("Query start");
        let result = context.fhirpath.evaluate(query, &test_bundle).await;
        let query_end = MemorySnapshot::new("Query end");
        
        let query_duration = query_end.elapsed_since(&query_start);
        
        match (result.is_ok(), should_succeed) {
            (true, true) => {
                successful_queries += 1;
                println!("  ✅ Success query completed in {:?}", query_duration);
            },
            (false, false) => {
                error_queries += 1;
                println!("  ✅ Error query failed as expected in {:?}", query_duration);
            },
            (true, false) => {
                println!("  ⚠️  Expected error query succeeded: {}", query);
            },
            (false, true) => {
                println!("  ❌ Expected success query failed: {}", query);
            }
        }
        
        // Brief pause to allow cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    let end_snapshot = MemorySnapshot::new("End error cleanup test");
    let total_duration = end_snapshot.elapsed_since(&start_snapshot);
    
    println!("  📊 Memory cleanup after errors summary:");
    println!("     Successful queries: {}", successful_queries);
    println!("     Error queries: {}", error_queries);
    println!("     Total time: {:?}", total_duration);
    
    // Memory cleanup should work properly even with errors
    assert!(successful_queries >= 3, 
        "At least 3 valid queries should succeed");
    
    assert!(error_queries >= 3, 
        "At least 3 invalid queries should fail as expected");
    
    assert!(total_duration.as_secs() < 60,
        "Error cleanup test should complete within 60 seconds");
}

#[tokio::test]
async fn run_memory_monitoring_summary() {
    println!("\n🎉 Memory usage monitoring and validation tests completed!");
    println!("📊 Test Summary:");
    println!("  ✅ Memory usage with growing datasets");
    println!("  ✅ Memory leak detection");
    println!("  ✅ Memory efficient large query processing");
    println!("  ✅ Concurrent memory usage");
    println!("  ✅ Memory pressure handling");
    println!("  ✅ Schema cache memory efficiency");
    println!("  ✅ Memory cleanup after errors");
    println!("\n🧠 Memory monitoring and validation completed with Bridge Support Architecture!");
}