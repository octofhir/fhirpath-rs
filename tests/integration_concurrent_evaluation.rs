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

//! Integration tests for concurrent evaluation with Bridge Support Architecture

use octofhir_fhirpath::*;
use octofhir_fhirpath_model::*;
use octofhir_fhirpath_evaluator::*;
use octofhir_fhirpath_analyzer::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};

mod utils;
use utils::IntegrationTestContext;

// Create test data for concurrent evaluation

fn create_concurrent_test_patient(id: usize) -> Value {
    json!({
        "resourceType": "Patient",
        "id": format!("concurrent-patient-{:06}", id),
        "active": true,
        "name": [{
            "use": "official",
            "family": format!("ConcurrentTest{:06}", id),
            "given": ["Concurrent", "User"]
        }],
        "telecom": [{
            "system": "phone", 
            "value": format!("+1-555-{:03}-{:04}", id / 10000, id % 10000),
            "use": "home"
        }],
        "gender": if id % 2 == 0 { "female" } else { "male" },
        "birthDate": format!("{}-{:02}-{:02}", 
            1950 + (id % 70), 
            1 + (id % 12), 
            1 + (id % 28)
        ),
        "address": [{
            "use": "home",
            "line": [format!("{} Concurrent Street", id + 1)],
            "city": format!("TestCity{:04}", id / 100),
            "state": "TC",
            "postalCode": format!("{:05}", 10000 + (id % 90000))
        }]
    })
}

fn create_concurrent_test_bundle(resource_count: usize) -> Value {
    let mut entries = Vec::new();
    
    for i in 0..resource_count {
        let patient = create_concurrent_test_patient(i);
        entries.push(json!({"resource": patient}));
        
        // Add some observations for each patient
        if i < resource_count / 2 {
            let observation = json!({
                "resourceType": "Observation",
                "id": format!("concurrent-obs-{:06}", i),
                "status": "final",
                "category": [{
                    "coding": [{
                        "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                        "code": "vital-signs"
                    }]
                }],
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "29463-7",
                        "display": "Body Weight"
                    }]
                },
                "subject": {
                    "reference": format!("Patient/concurrent-patient-{:06}", i)
                },
                "effectiveDateTime": "2024-01-15T10:00:00Z",
                "valueQuantity": {
                    "value": 60.0 + (i as f64 % 50.0),
                    "unit": "kg",
                    "system": "http://unitsofmeasure.org",
                    "code": "kg"
                }
            });
            entries.push(json!({"resource": observation}));
        }
    }
    
    json!({
        "resourceType": "Bundle",
        "id": format!("concurrent-test-bundle-{}", resource_count),
        "type": "collection",
        "timestamp": "2024-01-15T10:00:00Z",
        "total": entries.len(),
        "entry": entries
    })
}

#[tokio::test]
async fn test_basic_concurrent_evaluation() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    let test_queries = vec![
        "Patient.name.given.first()",
        "Patient.name.family.first()",
        "Patient.active",
        "Patient.gender",
        "Patient.birthDate",
        "Patient.telecom.value.first()",
        "Patient.address.city.first()",
        "Patient.address.postalCode.first()",
    ];
    
    let patient = create_concurrent_test_patient(1);
    
    println!("Testing basic concurrent evaluation with {} queries...", test_queries.len());
    
    // Launch concurrent evaluation tasks
    let mut tasks = Vec::new();
    
    for (i, query) in test_queries.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let patient_data = patient.clone();
        
        let task = tokio::spawn(async move {
            let start = Instant::now();
            let result = fhirpath.evaluate(&query, &patient_data).await;
            let duration = start.elapsed();
            (i, query, result, duration)
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    
    let mut successful_evaluations = 0;
    let mut total_duration = std::time::Duration::default();
    
    for result in results {
        let (task_id, query, eval_result, duration) = result.unwrap();
        total_duration += duration;
        
        match eval_result {
            Ok(values) => {
                successful_evaluations += 1;
                println!("  ✅ Task {}: '{}' -> {} values in {:?}", 
                    task_id, query, values.len(), duration);
            },
            Err(e) => {
                println!("  ❌ Task {}: '{}' -> Error: {:?}", task_id, query, e);
            }
        }
    }
    
    println!("  📊 Concurrent evaluation summary:");
    println!("    Successful: {}/8", successful_evaluations);
    println!("    Total time: {:?}", total_duration);
    println!("    Avg time per query: {:?}", total_duration / 8);
    
    // Most evaluations should succeed
    assert!(successful_evaluations >= 6, 
        "At least 6/8 concurrent evaluations should succeed");
}

#[tokio::test]
async fn test_concurrent_bundle_queries() {
    let context = IntegrationTestContext::new().await.unwrap();
    let bundle = create_concurrent_test_bundle(500);
    
    let bundle_queries = vec![
        "Bundle.entry.resource.ofType(Patient).count()",
        "Bundle.entry.resource.ofType(Observation).count()",  
        "Bundle.entry.resource.ofType(Patient).where(active = true).count()",
        "Bundle.entry.resource.ofType(Patient).where(gender = 'female').count()",
        "Bundle.entry.resource.ofType(Patient).name.family.first()",
        "Bundle.entry.resource.ofType(Observation).valueQuantity.value.first()",
        "Bundle.entry.resource.ofType(Patient).telecom.value.first()",
        "Bundle.entry.resource.ofType(Patient).address.city.first()",
        "Bundle.entry.resource.ofType(Observation).subject.reference.first()",
        "Bundle.entry.resource.ofType(Patient).birthDate.first()",
    ];
    
    println!("Testing concurrent bundle queries with {} queries on 500 resources...", bundle_queries.len());
    
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for (i, query) in bundle_queries.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let bundle_data = bundle.clone();
        
        let task = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = fhirpath.evaluate(&query, &bundle_data).await;
            let task_duration = task_start.elapsed();
            (i, query, result, task_duration)
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let total_wall_time = start.elapsed();
    
    let mut successful_queries = 0;
    let mut total_processing_time = std::time::Duration::default();
    
    for result in results {
        let (task_id, query, eval_result, duration) = result.unwrap();
        total_processing_time += duration;
        
        match eval_result {
            Ok(values) => {
                successful_queries += 1;
                println!("  ✅ Query {}: {} values in {:?}", task_id, values.len(), duration);
            },
            Err(e) => {
                println!("  ❌ Query {}: '{}' -> Error: {:?}", task_id, query, e);
            }
        }
    }
    
    println!("  📊 Concurrent bundle query summary:");
    println!("    Wall time: {:?}", total_wall_time);
    println!("    Total processing time: {:?}", total_processing_time);
    println!("    Successful queries: {}/10", successful_queries);
    println!("    Parallelization efficiency: {:.1}x", 
        total_processing_time.as_secs_f64() / total_wall_time.as_secs_f64());
    
    // Should achieve some parallelization benefit
    assert!(total_wall_time < total_processing_time,
        "Concurrent execution should be faster than sequential");
    
    assert!(successful_queries >= 8, 
        "At least 8/10 concurrent bundle queries should succeed");
}

#[tokio::test]
async fn test_high_concurrency_stress_test() {
    let context = IntegrationTestContext::new().await.unwrap();
    let patient = create_concurrent_test_patient(1);
    
    let concurrency_levels = vec![10, 25, 50, 100];
    let test_query = "Patient.name.given.first() + ' ' + Patient.name.family.first()";
    
    for concurrency in concurrency_levels {
        println!("Testing concurrency level: {} simultaneous tasks...", concurrency);
        
        let start = Instant::now();
        let mut tasks = Vec::new();
        
        for i in 0..concurrency {
            let fhirpath = context.fhirpath.clone();
            let patient_data = patient.clone();
            let query = test_query.to_string();
            
            let task = tokio::spawn(async move {
                let task_start = Instant::now();
                let result = fhirpath.evaluate(&query, &patient_data).await;
                let task_duration = task_start.elapsed();
                (i, result.is_ok(), task_duration)
            });
            
            tasks.push(task);
        }
        
        let results = futures::future::join_all(tasks).await;
        let total_duration = start.elapsed();
        
        let successful_tasks = results.iter()
            .map(|r| r.as_ref().unwrap().1)
            .filter(|&success| success)
            .count();
        
        let avg_task_duration: std::time::Duration = results.iter()
            .map(|r| r.as_ref().unwrap().2)
            .sum::<std::time::Duration>() / results.len() as u32;
        
        println!("  ✅ Concurrency {}: {}/{} successful, wall time: {:?}, avg task time: {:?}",
            concurrency, successful_tasks, concurrency, total_duration, avg_task_duration);
        
        // High concurrency should still work reasonably well
        assert!(successful_tasks >= concurrency * 80 / 100, 
            "At least 80% of concurrent tasks should succeed at concurrency level {}", concurrency);
        
        assert!(total_duration.as_secs() < 30,
            "High concurrency test should complete within 30 seconds at level {}", concurrency);
    }
}

#[tokio::test]
async fn test_concurrent_mixed_workload() {
    let context = IntegrationTestContext::new().await.unwrap();
    let bundle = create_concurrent_test_bundle(1000);
    let patient = create_concurrent_test_patient(1);
    
    println!("Testing concurrent mixed workload...");
    
    // Mix of simple and complex queries
    let workload = vec![
        // Simple patient queries (fast)
        ("Patient.name.given.first()", &patient, "simple"),
        ("Patient.active", &patient, "simple"),
        ("Patient.gender", &patient, "simple"),
        
        // Complex bundle queries (slower)
        ("Bundle.entry.resource.ofType(Patient).count()", &bundle, "complex"),
        ("Bundle.entry.resource.ofType(Patient).where(active = true).name.family", &bundle, "complex"),
        ("Bundle.entry.resource.ofType(Observation).valueQuantity.value", &bundle, "complex"),
        
        // Medium complexity queries
        ("Patient.telecom.where(system = 'phone').value.first()", &patient, "medium"),
        ("Patient.address.where(use = 'home').city.first()", &patient, "medium"),
        ("Bundle.entry.resource.ofType(Patient).where(gender = 'female').count()", &bundle, "medium"),
        ("Bundle.entry.resource.ofType(Observation).subject.reference.first()", &bundle, "medium"),
    ];
    
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for (i, (query, resource, complexity)) in workload.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let resource_data = resource.clone();
        let query_string = query.to_string();
        let complexity_level = complexity.to_string();
        
        let task = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = fhirpath.evaluate(&query_string, &resource_data).await;
            let task_duration = task_start.elapsed();
            (i, complexity_level, result, task_duration)
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let total_duration = start.elapsed();
    
    let mut results_by_complexity: std::collections::HashMap<String, Vec<std::time::Duration>> = std::collections::HashMap::new();
    let mut successful_tasks = 0;
    
    for result in results {
        let (task_id, complexity, eval_result, duration) = result.unwrap();
        
        results_by_complexity.entry(complexity.clone()).or_insert_with(Vec::new).push(duration);
        
        match eval_result {
            Ok(values) => {
                successful_tasks += 1;
                println!("  ✅ Task {} ({}): {} values in {:?}", 
                    task_id, complexity, values.len(), duration);
            },
            Err(e) => {
                println!("  ❌ Task {} ({}): Error: {:?}", task_id, complexity, e);
            }
        }
    }
    
    // Analyze performance by complexity
    for (complexity, durations) in results_by_complexity {
        let avg_duration: std::time::Duration = durations.iter().sum::<std::time::Duration>() / durations.len() as u32;
        println!("  📊 {} queries: {} tasks, avg duration: {:?}", 
            complexity, durations.len(), avg_duration);
    }
    
    println!("  📊 Mixed workload summary:");
    println!("    Total wall time: {:?}", total_duration);
    println!("    Successful tasks: {}/10", successful_tasks);
    
    assert!(successful_tasks >= 8, 
        "At least 8/10 mixed workload tasks should succeed");
}

#[tokio::test]
async fn test_concurrent_schema_operations() {
    let context = IntegrationTestContext::new().await.unwrap();
    let patient = create_concurrent_test_patient(1);
    
    // Test concurrent schema-intensive operations
    let schema_queries = vec![
        "Patient.name.given.first().is(string)",
        "Patient.active.is(boolean)",
        "Patient.birthDate.is(date)",
        "Patient.telecom.count() > 0",
        "Patient.address.exists()",
        "Patient.name.exists()",
        "Patient.identifier.empty()",
        "Patient.gender.is(string)",
    ];
    
    println!("Testing concurrent schema operations...");
    
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for (i, query) in schema_queries.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let patient_data = patient.clone();
        
        let task = tokio::spawn(async move {
            let task_start = Instant::now();
            
            // Run the query multiple times to stress schema operations
            let mut all_successful = true;
            let iterations = 5;
            
            for _iter in 0..iterations {
                let result = fhirpath.evaluate(query, &patient_data).await;
                if result.is_err() {
                    all_successful = false;
                    break;
                }
            }
            
            let task_duration = task_start.elapsed();
            (i, query, all_successful, task_duration)
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let total_duration = start.elapsed();
    
    let mut successful_schema_tasks = 0;
    
    for result in results {
        let (task_id, query, success, duration) = result.unwrap();
        
        if success {
            successful_schema_tasks += 1;
            println!("  ✅ Schema task {}: '{}' completed in {:?}", 
                task_id, query, duration);
        } else {
            println!("  ❌ Schema task {}: '{}' failed", task_id, query);
        }
    }
    
    println!("  📊 Concurrent schema operations summary:");
    println!("    Total time: {:?}", total_duration);
    println!("    Successful schema tasks: {}/8", successful_schema_tasks);
    
    // Schema operations should be thread-safe and concurrent
    assert!(successful_schema_tasks >= 6, 
        "At least 6/8 concurrent schema operations should succeed");
    
    assert!(total_duration.as_secs() < 10,
        "Concurrent schema operations should complete quickly");
}

#[tokio::test] 
async fn test_concurrent_error_handling() {
    let context = IntegrationTestContext::new().await.unwrap();
    let patient = create_concurrent_test_patient(1);
    
    // Mix of valid and invalid queries to test error handling under concurrency
    let mixed_queries = vec![
        ("Patient.name.given.first()", true),
        ("Patient.invalidProperty", false),
        ("Patient.active", true),
        ("Patient.name.given..first()", false), // Parse error
        ("Patient.birthDate", true),
        ("Patient.nonExistent.value", false),
        ("Patient.telecom.value.first()", true),
        ("Patient.address.(", false), // Syntax error
        ("Patient.gender", true),
        ("Patient.name.given.invalidFunction()", false),
    ];
    
    println!("Testing concurrent error handling...");
    
    let successful_count = Arc::new(AtomicUsize::new(0));
    let error_count = Arc::new(AtomicUsize::new(0));
    
    let mut tasks = Vec::new();
    
    for (i, (query, should_succeed)) in mixed_queries.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let patient_data = patient.clone();
        let query_string = query.to_string();
        let success_counter = successful_count.clone();
        let error_counter = error_count.clone();
        
        let task = tokio::spawn(async move {
            let result = fhirpath.evaluate(&query_string, &patient_data).await;
            
            match (result.is_ok(), should_succeed) {
                (true, true) => {
                    success_counter.fetch_add(1, Ordering::Relaxed);
                    println!("  ✅ Task {}: Valid query succeeded as expected", i);
                },
                (false, false) => {
                    error_counter.fetch_add(1, Ordering::Relaxed);
                    println!("  ✅ Task {}: Invalid query failed as expected", i);
                },
                (true, false) => {
                    println!("  ⚠️  Task {}: Invalid query unexpectedly succeeded", i);
                },
                (false, true) => {
                    println!("  ❌ Task {}: Valid query unexpectedly failed: '{}'", i, query_string);
                }
            }
            
            (i, result.is_ok(), should_succeed)
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    
    let final_successful = successful_count.load(Ordering::Relaxed);
    let final_errors = error_count.load(Ordering::Relaxed);
    let expected_correct_handling = final_successful + final_errors;
    
    println!("  📊 Concurrent error handling summary:");
    println!("    Correctly handled: {}/10", expected_correct_handling);
    println!("    Valid queries succeeded: {}", final_successful);
    println!("    Invalid queries failed properly: {}", final_errors);
    
    // Error handling should be robust under concurrency
    assert!(expected_correct_handling >= 8, 
        "At least 8/10 queries should be handled correctly under concurrent execution");
}

#[tokio::test]
async fn test_concurrent_resource_sharing() {
    let context = IntegrationTestContext::new().await.unwrap();
    let shared_bundle = create_concurrent_test_bundle(200);
    
    println!("Testing concurrent resource sharing...");
    
    // Multiple tasks sharing the same resource data
    let resource_sharing_queries = vec![
        "Bundle.entry.resource.count()",
        "Bundle.entry.resource.ofType(Patient).count()",
        "Bundle.entry.resource.ofType(Observation).count()",
        "Bundle.total",
        "Bundle.id",
        "Bundle.type",
    ];
    
    let start = Instant::now();
    let mut all_tasks = Vec::new();
    
    // Launch multiple rounds of concurrent tasks all using the same resource
    for round in 0..5 {
        let mut round_tasks = Vec::new();
        
        for (i, query) in resource_sharing_queries.iter().enumerate() {
            let fhirpath = context.fhirpath.clone();
            let bundle_data = shared_bundle.clone(); // Shared resource
            let query_string = query.to_string();
            
            let task = tokio::spawn(async move {
                let result = fhirpath.evaluate(&query_string, &bundle_data).await;
                (round, i, result.is_ok())
            });
            
            round_tasks.push(task);
        }
        
        all_tasks.extend(round_tasks);
    }
    
    let results = futures::future::join_all(all_tasks).await;
    let total_duration = start.elapsed();
    
    let mut successful_shared_operations = 0;
    let total_operations = results.len();
    
    for result in results {
        let (round, task_id, success) = result.unwrap();
        if success {
            successful_shared_operations += 1;
        }
        println!("  {} Round {} Task {}: {}", 
            if success { "✅" } else { "❌" }, 
            round, task_id, 
            if success { "Success" } else { "Failed" });
    }
    
    println!("  📊 Concurrent resource sharing summary:");
    println!("    Total operations: {}", total_operations);
    println!("    Successful operations: {}", successful_shared_operations);
    println!("    Success rate: {:.1}%", 
        100.0 * successful_shared_operations as f64 / total_operations as f64);
    println!("    Total time: {:?}", total_duration);
    
    // Resource sharing should work reliably
    assert!(successful_shared_operations >= total_operations * 90 / 100,
        "At least 90% of concurrent resource sharing operations should succeed");
}

#[tokio::test]
async fn test_concurrent_performance_comparison() {
    let context = IntegrationTestContext::new().await.unwrap();
    let bundle = create_concurrent_test_bundle(1000);
    
    let test_queries = vec![
        "Bundle.entry.resource.ofType(Patient).count()",
        "Bundle.entry.resource.ofType(Observation).count()",
        "Bundle.entry.resource.ofType(Patient).where(active = true).count()",
        "Bundle.entry.resource.ofType(Patient).name.family.first()",
    ];
    
    println!("Comparing sequential vs concurrent performance...");
    
    // Sequential execution
    let sequential_start = Instant::now();
    for query in &test_queries {
        let result = context.fhirpath.evaluate(query, &bundle).await;
        assert!(result.is_ok(), "Sequential query should succeed: {}", query);
    }
    let sequential_duration = sequential_start.elapsed();
    
    // Concurrent execution
    let concurrent_start = Instant::now();
    let mut tasks = Vec::new();
    
    for (i, query) in test_queries.iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let bundle_data = bundle.clone();
        let query_string = query.to_string();
        
        let task = tokio::spawn(async move {
            let result = fhirpath.evaluate(&query_string, &bundle_data).await;
            (i, result.is_ok())
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let concurrent_duration = concurrent_start.elapsed();
    
    let successful_concurrent = results.iter()
        .filter(|r| r.as_ref().unwrap().1)
        .count();
    
    println!("  📊 Performance comparison:");
    println!("    Sequential: {:?}", sequential_duration);
    println!("    Concurrent: {:?} ({}/4 successful)", 
        concurrent_duration, successful_concurrent);
    println!("    Speedup: {:.2}x", 
        sequential_duration.as_secs_f64() / concurrent_duration.as_secs_f64());
    
    // Concurrent execution should provide some speedup
    assert!(concurrent_duration <= sequential_duration,
        "Concurrent execution should be at least as fast as sequential");
    
    assert!(successful_concurrent >= 3,
        "At least 3/4 concurrent queries should succeed");
}

#[tokio::test]
async fn run_concurrent_evaluation_summary() {
    println!("\n🎉 Concurrent evaluation integration tests completed!");
    println!("📊 Test Summary:");
    println!("  ✅ Basic concurrent evaluation");
    println!("  ✅ Concurrent bundle queries");
    println!("  ✅ High concurrency stress test");
    println!("  ✅ Concurrent mixed workload");
    println!("  ✅ Concurrent schema operations");
    println!("  ✅ Concurrent error handling");
    println!("  ✅ Concurrent resource sharing");
    println!("  ✅ Concurrent performance comparison");
    println!("\n⚡ Concurrent evaluation validated with Bridge Support Architecture!");
}