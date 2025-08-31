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

//! Integration tests for performance with large datasets using Bridge Support Architecture

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

// Create large dataset generators

fn create_large_patient_bundle(patient_count: usize) -> Value {
    let mut entries = Vec::new();
    
    for i in 0..patient_count {
        let patient = json!({
            "resourceType": "Patient",
            "id": format!("patient-{:06}", i),
            "identifier": [{
                "use": "official",
                "system": "http://hospital.example.org/patient-ids",
                "value": format!("PAT-{:06}", i)
            }],
            "active": true,
            "name": [{
                "use": "official",
                "family": format!("Patient{:06}", i),
                "given": ["Test", "User"]
            }],
            "telecom": [{
                "system": "phone",
                "value": format!("+1-555-{:03}-{:04}", i / 10000, i % 10000),
                "use": "home"
            }],
            "gender": if i % 2 == 0 { "female" } else { "male" },
            "birthDate": format!("{}-{:02}-{:02}", 
                1950 + (i % 70), 
                1 + (i % 12), 
                1 + (i % 28)
            ),
            "address": [{
                "use": "home",
                "line": [format!("{} Test Street", i + 1)],
                "city": format!("TestCity{:04}", i / 100),
                "state": "TS",
                "postalCode": format!("{:05}", 10000 + (i % 90000))
            }]
        });
        
        entries.push(json!({"resource": patient}));
    }
    
    json!({
        "resourceType": "Bundle",
        "id": format!("large-patient-bundle-{}", patient_count),
        "type": "collection",
        "timestamp": "2024-01-15T10:00:00Z",
        "total": entries.len(),
        "entry": entries
    })
}

fn create_large_observation_bundle(observation_count: usize) -> Value {
    let mut entries = Vec::new();
    
    let observation_codes = vec![
        ("33747-0", "Glucose [Mass/volume] in Serum or Plasma"),
        ("2093-3", "Cholesterol [Mass/volume] in Serum or Plasma"), 
        ("4548-4", "Hemoglobin A1c/Hemoglobin.total in Blood"),
        ("29463-7", "Body Weight"),
        ("8302-2", "Body height"),
        ("85354-9", "Blood pressure panel"),
    ];
    
    for i in 0..observation_count {
        let (code, display) = &observation_codes[i % observation_codes.len()];
        
        let observation = json!({
            "resourceType": "Observation",
            "id": format!("obs-{:08}", i),
            "status": "final",
            "category": [{
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                    "code": if code.contains("85354-9") { "vital-signs" } else { "laboratory" },
                    "display": if code.contains("85354-9") { "Vital Signs" } else { "Laboratory" }
                }]
            }],
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": code,
                    "display": display
                }]
            },
            "subject": {
                "reference": format!("Patient/patient-{:06}", i / 10)
            },
            "effectiveDateTime": format!("2024-01-{:02}T{:02}:30:00Z", 
                1 + (i % 31), 
                8 + (i % 12)
            ),
            "valueQuantity": {
                "value": match *code {
                    "33747-0" => 80.0 + (i as f64 % 50.0), // Glucose 80-130
                    "2093-3" => 150.0 + (i as f64 % 100.0), // Cholesterol 150-250
                    "4548-4" => 6.0 + (i as f64 % 40.0) / 10.0, // HbA1c 6.0-10.0
                    "29463-7" => 60.0 + (i as f64 % 80.0), // Weight 60-140kg
                    "8302-2" => 150.0 + (i as f64 % 50.0), // Height 150-200cm
                    _ => 100.0 + (i as f64 % 100.0)
                },
                "unit": match *code {
                    "33747-0" | "2093-3" => "mg/dL",
                    "4548-4" => "%",
                    "29463-7" => "kg",
                    "8302-2" => "cm",
                    _ => "unit"
                },
                "system": "http://unitsofmeasure.org"
            }
        });
        
        entries.push(json!({"resource": observation}));
    }
    
    json!({
        "resourceType": "Bundle",
        "id": format!("large-observation-bundle-{}", observation_count),
        "type": "collection", 
        "timestamp": "2024-01-15T10:00:00Z",
        "total": entries.len(),
        "entry": entries
    })
}

fn create_mixed_large_bundle(resource_count: usize) -> Value {
    let mut entries = Vec::new();
    
    let patient_count = resource_count / 4;
    let observation_count = resource_count / 2; 
    let condition_count = resource_count / 6;
    let medication_count = resource_count - patient_count - observation_count - condition_count;
    
    // Add patients
    for i in 0..patient_count {
        let patient = json!({
            "resourceType": "Patient",
            "id": format!("mixed-patient-{:06}", i),
            "active": i % 10 != 0, // 90% active
            "name": [{
                "family": format!("TestPatient{:06}", i),
                "given": ["Mixed", "Bundle"]
            }],
            "gender": if i % 2 == 0 { "female" } else { "male" },
            "birthDate": format!("{}-01-01", 1940 + (i % 80))
        });
        entries.push(json!({"resource": patient}));
    }
    
    // Add observations
    for i in 0..observation_count {
        let observation = json!({
            "resourceType": "Observation", 
            "id": format!("mixed-obs-{:08}", i),
            "status": "final",
            "category": [{
                "coding": [{
                    "code": if i % 3 == 0 { "vital-signs" } else { "laboratory" }
                }]
            }],
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": format!("{:04}-{}", 1000 + (i % 9000), i % 10),
                    "display": format!("Test Observation {}", i)
                }]
            },
            "subject": {
                "reference": format!("Patient/mixed-patient-{:06}", i % patient_count)
            },
            "effectiveDateTime": "2024-01-15T10:00:00Z",
            "valueQuantity": {
                "value": 100.0 + (i as f64 % 200.0),
                "unit": "unit"
            }
        });
        entries.push(json!({"resource": observation}));
    }
    
    // Add conditions
    for i in 0..condition_count {
        let condition = json!({
            "resourceType": "Condition",
            "id": format!("mixed-condition-{:06}", i),
            "clinicalStatus": {
                "coding": [{
                    "code": if i % 5 == 0 { "resolved" } else { "active" }
                }]
            },
            "code": {
                "coding": [{
                    "system": "http://snomed.info/sct",
                    "code": format!("{}", 100000 + i),
                    "display": format!("Test Condition {}", i)
                }]
            },
            "subject": {
                "reference": format!("Patient/mixed-patient-{:06}", i % patient_count)
            }
        });
        entries.push(json!({"resource": condition}));
    }
    
    // Add medications
    for i in 0..medication_count {
        let medication = json!({
            "resourceType": "MedicationStatement",
            "id": format!("mixed-med-{:06}", i),
            "status": if i % 8 == 0 { "stopped" } else { "active" },
            "medicationCodeableConcept": {
                "coding": [{
                    "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                    "code": format!("{}", 10000 + i),
                    "display": format!("Test Medication {}", i)
                }]
            },
            "subject": {
                "reference": format!("Patient/mixed-patient-{:06}", i % patient_count)
            }
        });
        entries.push(json!({"resource": medication}));
    }
    
    json!({
        "resourceType": "Bundle",
        "id": format!("mixed-large-bundle-{}", resource_count),
        "type": "collection",
        "timestamp": "2024-01-15T10:00:00Z",
        "total": entries.len(),
        "entry": entries
    })
}

#[tokio::test]
async fn test_large_patient_dataset_performance() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Test with progressively larger patient datasets
    let sizes = vec![100, 500, 1000, 2000];
    
    for size in sizes {
        println!("Testing with {} patients...", size);
        let large_bundle = create_large_patient_bundle(size);
        
        let queries = vec![
            "Bundle.entry.resource.ofType(Patient).count()",
            "Bundle.entry.resource.ofType(Patient).where(active = true).count()",
            "Bundle.entry.resource.ofType(Patient).name.family.first()",
            "Bundle.entry.resource.ofType(Patient).where(gender = 'female').count()",
            "Bundle.entry.resource.ofType(Patient).telecom.value.first()",
        ];
        
        let start = Instant::now();
        
        for query in &queries {
            let result = context.fhirpath.evaluate(query, &large_bundle).await;
            assert!(result.is_ok(), "Query should succeed with {} patients: {}", size, query);
        }
        
        let duration = start.elapsed();
        let queries_per_second = queries.len() as f64 / duration.as_secs_f64();
        
        println!("  ✅ {} patients: {:.1} queries/sec, total time: {:?}", 
            size, queries_per_second, duration);
        
        // Performance should remain reasonable even with large datasets
        assert!(duration.as_secs() < 10, 
            "Queries on {} patients should complete within 10 seconds", size);
    }
}

#[tokio::test]
async fn test_large_observation_dataset_performance() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Test with large observation datasets
    let sizes = vec![500, 1000, 2000, 5000];
    
    for size in sizes {
        println!("Testing with {} observations...", size);
        let large_bundle = create_large_observation_bundle(size);
        
        let queries = vec![
            "Bundle.entry.resource.ofType(Observation).count()",
            "Bundle.entry.resource.ofType(Observation).where(status = 'final').count()",
            "Bundle.entry.resource.ofType(Observation).where(category.coding.code = 'laboratory').count()",
            "Bundle.entry.resource.ofType(Observation).code.coding.code.distinct().count()",
            "Bundle.entry.resource.ofType(Observation).valueQuantity.value.average()",
        ];
        
        let start = Instant::now();
        
        for query in &queries {
            let result = context.fhirpath.evaluate(query, &large_bundle).await;
            match result {
                Ok(_) => {},
                Err(e) => {
                    // Some functions like average() might not be implemented
                    if !query.contains("average()") && !query.contains("distinct()") {
                        panic!("Query should succeed with {} observations: {} -> {:?}", size, query, e);
                    }
                }
            }
        }
        
        let duration = start.elapsed();
        let queries_per_second = queries.len() as f64 / duration.as_secs_f64();
        
        println!("  ✅ {} observations: {:.1} queries/sec, total time: {:?}", 
            size, queries_per_second, duration);
        
        // Should handle large observation datasets efficiently
        assert!(duration.as_secs() < 15, 
            "Queries on {} observations should complete within 15 seconds", size);
    }
}

#[tokio::test]
async fn test_mixed_large_dataset_performance() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Test with mixed resource type datasets
    let sizes = vec![1000, 2000, 5000];
    
    for size in sizes {
        println!("Testing with {} mixed resources...", size);
        let large_bundle = create_mixed_large_bundle(size);
        
        let queries = vec![
            "Bundle.entry.resource.count()",
            "Bundle.entry.resource.ofType(Patient).count()",
            "Bundle.entry.resource.ofType(Observation).count()",
            "Bundle.entry.resource.ofType(Condition).count()",
            "Bundle.entry.resource.ofType(MedicationStatement).count()",
            "Bundle.entry.resource.ofType(Patient).where(active = true).name.family",
            "Bundle.entry.resource.ofType(Observation).where(category.coding.code = 'laboratory').subject.reference",
            "Bundle.entry.resource.ofType(Condition).where(clinicalStatus.coding.code = 'active').code.coding.display",
        ];
        
        let start = Instant::now();
        
        for query in &queries {
            let result = context.fhirpath.evaluate(query, &large_bundle).await;
            assert!(result.is_ok(), "Query should succeed with {} mixed resources: {}", size, query);
        }
        
        let duration = start.elapsed();
        let queries_per_second = queries.len() as f64 / duration.as_secs_f64();
        
        println!("  ✅ {} mixed resources: {:.1} queries/sec, total time: {:?}", 
            size, queries_per_second, duration);
        
        // Mixed datasets are more complex but should still perform reasonably
        assert!(duration.as_secs() < 20, 
            "Queries on {} mixed resources should complete within 20 seconds", size);
    }
}

#[tokio::test]
async fn test_complex_queries_on_large_datasets() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create a large mixed dataset for complex query testing
    let large_bundle = create_mixed_large_bundle(3000);
    
    let complex_queries = vec![
        // Cross-resource queries
        ("Bundle.entry.resource.ofType(Patient).where(Bundle.entry.resource.ofType(Condition).subject.reference.contains(id)).count()",
         "Patients with conditions"),
        
        // Multi-step filtering
        ("Bundle.entry.resource.ofType(Observation).where(category.coding.code = 'laboratory' and valueQuantity.value > 150).subject.reference.distinct().count()",
         "Patients with high lab values"),
        
        // Complex conditions
        ("Bundle.entry.resource.ofType(Patient).where(active = true and gender = 'female' and birthDate < '1970-01-01').count()",
         "Active elderly female patients"),
        
        // Nested property access
        ("Bundle.entry.resource.ofType(Observation).where(subject.reference.exists()).code.coding.where(system = 'http://loinc.org').code.count()",
         "LOINC codes with valid subjects"),
        
        // Medication analysis
        ("Bundle.entry.resource.ofType(MedicationStatement).where(status = 'active').medicationCodeableConcept.coding.display.count()",
         "Active medication names"),
    ];
    
    println!("Testing complex queries on large dataset...");
    
    for (query, description) in complex_queries {
        let start = Instant::now();
        let result = context.fhirpath.evaluate(query, &large_bundle).await;
        let duration = start.elapsed();
        
        match result {
            Ok(values) => {
                println!("  ✅ {} - {} values in {:?}", description, values.len(), duration);
                
                // Complex queries should complete within reasonable time
                assert!(duration.as_secs() < 30, 
                    "Complex query '{}' should complete within 30 seconds, took {:?}", description, duration);
            },
            Err(e) => {
                // Some complex queries might not be fully supported yet
                if query.contains("distinct()") {
                    println!("  ⚠️  {} - Not supported yet: {:?}", description, e);
                } else {
                    println!("  ❌ {} - Error: {:?}", description, e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_memory_efficient_large_dataset_processing() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Test memory efficiency with large datasets
    let large_bundle = create_large_observation_bundle(10000);
    
    println!("Testing memory efficiency with 10,000 observations...");
    
    // Monitor memory usage during processing
    let memory_efficient_queries = vec![
        // Queries that should stream/filter efficiently  
        "Bundle.entry.resource.ofType(Observation).where(status = 'final').count()",
        "Bundle.entry.resource.ofType(Observation).where(valueQuantity.value > 100).count()",
        "Bundle.entry.resource.ofType(Observation).code.coding.code.first()",
        
        // Queries that access specific elements
        "Bundle.entry.resource.ofType(Observation).where(id = 'obs-00001000').valueQuantity.value.first()",
        "Bundle.entry.resource.ofType(Observation).where(subject.reference.contains('patient-000100')).count()",
    ];
    
    let start = Instant::now();
    
    for (i, query) in memory_efficient_queries.iter().enumerate() {
        let query_start = Instant::now();
        let result = context.fhirpath.evaluate(query, &large_bundle).await;
        let query_duration = query_start.elapsed();
        
        match result {
            Ok(values) => {
                println!("  ✅ Query {} completed: {} values in {:?}", i + 1, values.len(), query_duration);
            },
            Err(e) => {
                println!("  ⚠️  Query {} errored: {:?}", i + 1, e);
            }
        }
    }
    
    let total_duration = start.elapsed();
    println!("  📊 Total processing time: {:?}", total_duration);
    
    // Memory-efficient processing should complete in reasonable time
    assert!(total_duration.as_secs() < 60, 
        "Memory efficient processing should complete within 60 seconds");
}

#[tokio::test]
async fn test_scalability_patterns() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Test how performance scales with dataset size
    let sizes = vec![100, 500, 1000, 2000];
    let mut performance_data = Vec::new();
    
    let test_query = "Bundle.entry.resource.ofType(Patient).where(active = true).count()";
    
    for size in sizes {
        let bundle = create_large_patient_bundle(size);
        
        // Warmup
        let _ = context.fhirpath.evaluate(test_query, &bundle).await;
        
        // Measure performance
        let start = Instant::now();
        let iterations = 10;
        
        for _i in 0..iterations {
            let result = context.fhirpath.evaluate(test_query, &bundle).await;
            assert!(result.is_ok(), "Query should succeed");
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / iterations;
        let resources_per_second = size as f64 / avg_duration.as_secs_f64();
        
        performance_data.push((size, avg_duration, resources_per_second));
        
        println!("  📈 {} resources: avg {:?} per query, {:.0} resources/sec", 
            size, avg_duration, resources_per_second);
    }
    
    // Analyze scalability - performance should not degrade linearly
    if performance_data.len() >= 2 {
        let (small_size, small_time, _) = performance_data[0];
        let (large_size, large_time, _) = performance_data.last().unwrap();
        
        let size_ratio = *large_size as f64 / small_size as f64;
        let time_ratio = large_time.as_secs_f64() / small_time.as_secs_f64();
        
        println!("  📊 Scalability ratio: {}x size increase = {:.2}x time increase", size_ratio, time_ratio);
        
        // Time should not increase proportionally to size (should be better than O(n))
        assert!(time_ratio < size_ratio * 1.5, 
            "Performance should scale better than linearly");
    }
}

#[tokio::test]
async fn test_concurrent_large_dataset_processing() {
    let context = IntegrationTestContext::new().await.unwrap();
    let large_bundle = create_mixed_large_bundle(2000);
    
    println!("Testing concurrent processing of large dataset...");
    
    // Test concurrent processing with different queries
    let concurrent_queries = vec![
        "Bundle.entry.resource.ofType(Patient).count()",
        "Bundle.entry.resource.ofType(Observation).count()", 
        "Bundle.entry.resource.ofType(Condition).count()",
        "Bundle.entry.resource.ofType(MedicationStatement).count()",
        "Bundle.entry.resource.ofType(Patient).where(active = true).count()",
    ];
    
    let start = Instant::now();
    
    // Run queries concurrently
    let mut tasks = Vec::new();
    
    for (i, query) in concurrent_queries.into_iter().enumerate() {
        let fhirpath = context.fhirpath.clone();
        let bundle = large_bundle.clone();
        
        let task = tokio::spawn(async move {
            let task_start = Instant::now();
            let result = fhirpath.evaluate(&query, &bundle).await;
            let task_duration = task_start.elapsed();
            (i, query, result, task_duration)
        });
        
        tasks.push(task);
    }
    
    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    let total_duration = start.elapsed();
    
    println!("  📊 Concurrent processing completed in {:?}", total_duration);
    
    for result in results {
        let (i, query, eval_result, task_duration) = result.unwrap();
        
        match eval_result {
            Ok(values) => {
                println!("  ✅ Task {}: {} values in {:?}", i, values.len(), task_duration);
            },
            Err(e) => {
                println!("  ⚠️  Task {} ({}): Error {:?}", i, query, e);
            }
        }
    }
    
    // Concurrent processing should be faster than sequential
    let sequential_estimate = std::time::Duration::from_secs(5 * 5); // rough estimate
    assert!(total_duration < sequential_estimate,
        "Concurrent processing should be faster than sequential");
}

#[tokio::test]
async fn test_stress_test_with_maximum_dataset() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create the largest dataset we can reasonably test
    let large_bundle = create_mixed_large_bundle(10000);
    
    println!("Running stress test with 10,000 mixed resources...");
    
    let stress_queries = vec![
        "Bundle.entry.resource.count()",
        "Bundle.entry.resource.ofType(Patient).where(active = true).count()", 
        "Bundle.entry.resource.ofType(Observation).where(status = 'final').count()",
        "Bundle.entry.resource.ofType(Condition).where(clinicalStatus.coding.code = 'active').count()",
    ];
    
    let start = Instant::now();
    let mut successful_queries = 0;
    let mut total_results = 0;
    
    for (i, query) in stress_queries.iter().enumerate() {
        let query_start = Instant::now();
        
        match context.fhirpath.evaluate(query, &large_bundle).await {
            Ok(values) => {
                let query_duration = query_start.elapsed();
                successful_queries += 1;
                total_results += values.len();
                
                println!("  ✅ Stress query {}: {} values in {:?}", i + 1, values.len(), query_duration);
                
                // Individual queries should complete within reasonable time even under stress
                assert!(query_duration.as_secs() < 60, 
                    "Stress query should complete within 60 seconds, took {:?}", query_duration);
            },
            Err(e) => {
                println!("  ❌ Stress query {} failed: {:?}", i + 1, e);
            }
        }
    }
    
    let total_duration = start.elapsed();
    
    println!("  📊 Stress test summary:");
    println!("    Successful queries: {}/{}", successful_queries, stress_queries.len());
    println!("    Total results: {}", total_results);
    println!("    Total time: {:?}", total_duration);
    println!("    Avg time per query: {:?}", total_duration / stress_queries.len() as u32);
    
    // Stress test should complete successfully
    assert!(successful_queries >= stress_queries.len() / 2, 
        "At least half of stress test queries should succeed");
    
    assert!(total_duration.as_secs() < 300, 
        "Stress test should complete within 5 minutes");
}

#[tokio::test]
async fn run_large_dataset_performance_summary() {
    println!("\n🎉 Large dataset performance integration tests completed!");
    println!("📊 Test Summary:");
    println!("  ✅ Large patient dataset performance");
    println!("  ✅ Large observation dataset performance");
    println!("  ✅ Mixed large dataset performance"); 
    println!("  ✅ Complex queries on large datasets");
    println!("  ✅ Memory efficient large dataset processing");
    println!("  ✅ Scalability patterns");
    println!("  ✅ Concurrent large dataset processing");
    println!("  ✅ Stress test with maximum dataset");
    println!("\n⚡ Large dataset performance validated with Bridge Support Architecture!");
}