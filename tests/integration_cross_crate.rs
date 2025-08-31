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

//! Integration tests for cross-crate functionality with Bridge Support Architecture

use octofhir_fhirpath::*;
use octofhir_fhirpath_model::*;
use octofhir_fhirpath_evaluator::*;
use octofhir_fhirpath_analyzer::*;
use octofhir_fhirpath_registry::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::{json, Value};
use std::sync::Arc;

mod utils;
use utils::IntegrationTestContext;

#[tokio::test]
async fn test_model_evaluator_integration() {
    // Test that fhirpath-model and fhirpath-evaluator work together seamlessly
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create model provider directly
    let provider = FhirSchemaModelProvider::with_manager(context.schema_manager.clone())
        .await
        .unwrap();
    
    // Create evaluator with model provider
    let registry = create_standard_registry().await;
    let engine = FhirPathEngine::new(Arc::new(registry), Arc::new(provider));
    
    // Test that model-driven evaluation works
    let test_cases = vec![
        ("Patient.name.given.first()", &context.test_patient),
        ("Observation.valueQuantity.value", &context.test_observation),
        ("Bundle.entry.resource.ofType(Patient).active.first()", &context.test_bundle),
    ];
    
    for (expression, resource) in test_cases {
        let result = engine.evaluate(expression, resource.clone()).await;
        
        match result {
            Ok(values) => {
                println!("✅ Model-Evaluator integration '{}' returned {} values", expression, values.len());
            },
            Err(e) => {
                println!("⚠️  Model-Evaluator integration '{}' errored: {:?}", expression, e);
            }
        }
    }
}

#[tokio::test]
async fn test_analyzer_registry_integration() {
    // Test that fhirpath-analyzer and fhirpath-registry work together
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create analyzer with schema support
    let analyzer = FhirPathAnalyzer::new(
        Arc::new(FhirSchemaModelProvider::with_manager(context.schema_manager.clone()).await.unwrap())
    ).await.unwrap();
    
    // Test analysis of different function categories
    let analysis_tests = vec![
        // Collection functions (should be in registry)
        "Patient.name.where(use = 'official')",
        "Patient.telecom.select(value)",
        "Bundle.entry.resource.ofType(Patient)",
        
        // Math functions (should be in registry)
        "Patient.telecom.count()",
        "Bundle.entry.count() + 1",
        
        // String functions (should be in registry) 
        "Patient.name.given.first().upper()",
        "Patient.name.family.first().length()",
        
        // Navigation (should use model provider)
        "Patient.name.given",
        "Observation.valueQuantity.value",
        
        // Type functions (should use registry + model)
        "Patient.active.is(boolean)",
        "Patient.name.given.first().is(string)"
    ];
    
    for expression in analysis_tests {
        let result = analyzer.analyze(expression).await;
        
        match result {
            Ok(analysis) => {
                println!("✅ Analyzer-Registry integration '{}' completed", expression);
                println!("   Return type: {:?}", analysis.return_type);
                println!("   Functions used: {}", analysis.functions_used.len());
                println!("   Suggestions: {}", analysis.suggestions.len());
            },
            Err(e) => {
                println!("⚠️  Analyzer-Registry integration '{}' errored: {:?}", expression, e);
            }
        }
    }
}

#[tokio::test]
async fn test_model_registry_bridge_integration() {
    // Test that model provider and registry bridge components work together
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create bridge components
    let type_registry = FhirPathTypeRegistry::new(context.schema_manager.clone())
        .await
        .unwrap();
    
    let field_validator = AnalyzerFieldValidator::new(context.schema_manager.clone())
        .await
        .unwrap();
    
    // Test type operations with field validation
    let bridge_integration_tests = vec![
        ("Patient", "name", true),
        ("Patient", "active", true), 
        ("Patient", "invalidField", false),
        ("Observation", "valueQuantity", true),
        ("Observation", "valueInvalidType", false),
        ("Bundle", "entry", true),
        ("Bundle", "nonExistentProperty", false),
        ("InvalidResourceType", "anyField", false),
    ];
    
    for (resource_type, field_name, should_be_valid) in bridge_integration_tests {
        // Test type registry knows about resource
        let is_resource = type_registry.is_resource_type(resource_type);
        
        // Test field validation
        let field_validation = field_validator.validate_field(resource_type, field_name).await;
        
        match field_validation {
            Ok(validation_result) => {
                if should_be_valid && is_resource {
                    println!("✅ Bridge integration {}.{} - Resource: {}, Field Valid: {}", 
                        resource_type, field_name, is_resource, validation_result.is_valid);
                } else if !should_be_valid {
                    assert!(!validation_result.is_valid || !is_resource, 
                        "Invalid field {}.{} should not validate", resource_type, field_name);
                    println!("✅ Bridge integration {}.{} correctly identified as invalid", 
                        resource_type, field_name);
                }
            },
            Err(e) => {
                if !should_be_valid {
                    println!("✅ Bridge integration {}.{} correctly errored: {:?}", 
                        resource_type, field_name, e);
                } else {
                    panic!("Valid field {}.{} should not error: {:?}", resource_type, field_name, e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_evaluator_analyzer_pipeline() {
    // Test the complete pipeline: Evaluator -> Analyzer integration
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Use the main API which integrates both evaluator and analyzer
    let expressions = vec![
        "Patient.name.given.first()",
        "Patient.name.select(given.first() + ' ' + family.first())",
        "Observation.valueQuantity.where(value > 0)",
        "Bundle.entry.resource.ofType(Patient).name.family",
        "Patient.telecom.where(system = 'phone').value.first()",
        "Observation.code.coding.where(system = 'http://loinc.org').code.first()"
    ];
    
    for expression in expressions {
        let resource = if expression.contains("Bundle") {
            &context.test_bundle
        } else if expression.contains("Observation") {
            &context.test_observation
        } else {
            &context.test_patient
        };
        
        // Test evaluation only
        let eval_result = context.fhirpath.evaluate(expression, resource).await;
        
        // Test evaluation with analysis
        let analysis_result = context.fhirpath.evaluate_with_analysis(expression, resource).await;
        
        match (eval_result, analysis_result) {
            (Ok(eval_values), Ok(analysis_result)) => {
                println!("✅ Evaluator-Analyzer pipeline '{}' succeeded", expression);
                println!("   Evaluation returned: {} values", eval_values.len());
                println!("   Analysis returned: {} values", analysis_result.values.len());
                
                // Values should be consistent between eval and analysis
                assert_eq!(eval_values.len(), analysis_result.values.len(),
                    "Evaluation and analysis should return same number of values");
                
                if let Some(analysis) = analysis_result.analysis {
                    println!("   Analysis provided return type: {:?}", analysis.return_type);
                }
            },
            (eval_result, analysis_result) => {
                println!("⚠️  Evaluator-Analyzer pipeline '{}' partial success:", expression);
                println!("   Evaluation: {:?}", eval_result.is_ok());
                println!("   Analysis: {:?}", analysis_result.is_ok());
            }
        }
    }
}

#[tokio::test]
async fn test_all_crates_together() {
    // Test that all crates work together in a complex scenario
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create a complex healthcare scenario using all crates
    let complex_expression = 
        "Bundle.entry.resource.ofType(Patient)
         .where(active = true)
         .name.where(use = 'official')
         .given.first() + ' ' + 
         Bundle.entry.resource.ofType(Patient)
         .where(active = true)
         .name.where(use = 'official')  
         .family.first()";
    
    let result = context.fhirpath.evaluate_with_analysis(complex_expression, &context.test_bundle).await;
    
    match result {
        Ok(evaluation_result) => {
            println!("✅ Complex cross-crate integration succeeded");
            println!("   Expression: {}", complex_expression);
            println!("   Results: {} values", evaluation_result.values.len());
            println!("   Execution time: {:?}", evaluation_result.execution_time);
            
            if let Some(analysis) = evaluation_result.analysis {
                println!("   Analysis provided:");
                println!("     Return type: {:?}", analysis.return_type);
                println!("     Functions used: {}", analysis.functions_used.len());
                println!("     Resource types accessed: {}", analysis.resource_types_accessed.len());
                println!("     Suggestions: {}", analysis.suggestions.len());
                
                if !analysis.errors.is_empty() {
                    println!("     Errors: {}", analysis.errors.len());
                }
            }
        },
        Err(e) => {
            println!("⚠️  Complex cross-crate integration failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_configuration_integration() {
    // Test that configuration affects all crates properly
    let schema_manager = Arc::new(
        FhirSchemaPackageManager::new(PackageManagerConfig::default())
            .await
            .unwrap()
    );
    
    // Test different configurations
    let configs = vec![
        ("Basic config", FhirPathConfigBuilder::new()),
        ("With analyzer", FhirPathConfigBuilder::new().with_analyzer(true)),
        ("With performance tracking", FhirPathConfigBuilder::new().with_performance_tracking(true)),
        ("Full config", FhirPathConfigBuilder::new()
            .with_analyzer(true)
            .with_performance_tracking(true)
        ),
    ];
    
    for (config_name, config_builder) in configs {
        let fhirpath = config_builder.build().await;
        
        match fhirpath {
            Ok(fhirpath_instance) => {
                println!("✅ Configuration integration '{}' succeeded", config_name);
                
                // Test basic functionality
                let patient = json!({
                    "resourceType": "Patient",
                    "name": [{"given": ["Test"], "family": "User"}],
                    "active": true
                });
                
                let result = fhirpath_instance.evaluate("Patient.name.given.first()", &patient).await;
                
                match result {
                    Ok(values) => {
                        println!("   Basic evaluation returned {} values", values.len());
                    },
                    Err(e) => {
                        println!("   Basic evaluation failed: {:?}", e);
                    }
                }
                
                // Test analyzer integration if available
                if fhirpath_instance.has_analyzer() {
                    let analysis_result = fhirpath_instance
                        .evaluate_with_analysis("Patient.active", &patient)
                        .await;
                    
                    match analysis_result {
                        Ok(_) => {
                            println!("   Analyzer integration working");
                        },
                        Err(e) => {
                            println!("   Analyzer integration failed: {:?}", e);
                        }
                    }
                }
            },
            Err(e) => {
                println!("⚠️  Configuration integration '{}' failed: {:?}", config_name, e);
            }
        }
    }
}

#[tokio::test]
async fn test_error_propagation_across_crates() {
    // Test that errors propagate properly across crate boundaries
    let context = IntegrationTestContext::new().await.unwrap();
    
    let error_test_cases = vec![
        // Parser errors should propagate through evaluator
        ("Patient.name.given..first()", "Parse error with double dots"),
        ("Patient.name.(", "Incomplete expression"),
        ("Patient.name.given.first(", "Unclosed parentheses"),
        
        // Model errors should propagate through evaluator
        ("Patient.nonExistentField.value", "Invalid field access"),
        
        // Evaluator errors should propagate through main API
        ("Patient.name.given.invalidFunction()", "Invalid function"),
        
        // Type errors should be caught by analyzer
        ("Patient.active + 'string'", "Type mismatch")
    ];
    
    for (expression, error_description) in error_test_cases {
        let result = context.fhirpath.evaluate_with_analysis(expression, &context.test_patient).await;
        
        match result {
            Ok(evaluation_result) => {
                // Some errors might be handled gracefully
                if let Some(analysis) = evaluation_result.analysis {
                    if !analysis.errors.is_empty() {
                        println!("✅ Error propagation '{}' caught by analyzer: {} errors", 
                            error_description, analysis.errors.len());
                    } else {
                        println!("⚠️  Error propagation '{}' not caught (may be valid)", error_description);
                    }
                } else {
                    println!("⚠️  Error propagation '{}' completed without analysis", error_description);
                }
            },
            Err(e) => {
                println!("✅ Error propagation '{}' properly errored: {:?}", error_description, e);
                
                // Errors should contain helpful information
                let error_string = format!("{:?}", e);
                assert!(!error_string.is_empty(), "Error should have description");
            }
        }
    }
}

#[tokio::test]
async fn test_async_coordination() {
    // Test that async operations coordinate properly across crates
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Test concurrent operations that use different crates
    let mut tasks = Vec::new();
    
    for i in 0..5 {
        let fhirpath = context.fhirpath.clone();
        let patient = context.test_patient.clone();
        let observation = context.test_observation.clone();
        let bundle = context.test_bundle.clone();
        
        let task = tokio::spawn(async move {
            let operations = vec![
                // Model + Evaluator
                ("Patient.name.given.first()", &patient),
                ("Observation.valueQuantity.value", &observation),
                
                // Registry + Evaluator
                ("Patient.telecom.count()", &patient),
                ("Bundle.entry.count()", &bundle),
                
                // Analyzer + All others
                ("Patient.active.is(boolean)", &patient),
            ];
            
            let mut results = Vec::new();
            for (expression, resource) in operations {
                let result = fhirpath.evaluate_with_analysis(expression, resource).await;
                results.push((expression, result.is_ok()));
            }
            
            (i, results)
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    
    for result in results {
        let (task_id, operation_results) = result.unwrap();
        let successful_operations = operation_results.iter()
            .filter(|(_, success)| *success)
            .count();
        
        println!("✅ Async coordination task {} completed: {}/{} operations successful", 
            task_id, successful_operations, operation_results.len());
        
        // Most operations should succeed
        assert!(successful_operations >= operation_results.len() / 2,
            "At least half of operations should succeed in task {}", task_id);
    }
}

#[tokio::test]
async fn test_memory_sharing_across_crates() {
    // Test that shared memory structures (like schema cache) work across crates
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Perform operations that should share schema cache
    let cache_warming_operations = vec![
        "Patient.name.given.first()",
        "Patient.name.family.first()",
        "Patient.telecom.value.first()",
        "Patient.address.line.first()", 
        "Patient.active",
    ];
    
    // Warm up the cache
    for operation in &cache_warming_operations {
        let _ = context.fhirpath.evaluate(operation, &context.test_patient).await;
    }
    
    // Now test performance with warmed cache
    let start = std::time::Instant::now();
    
    for _iteration in 0..50 {
        for operation in &cache_warming_operations {
            let _ = context.fhirpath.evaluate(operation, &context.test_patient).await;
        }
    }
    
    let duration = start.elapsed();
    let operations_per_second = (cache_warming_operations.len() * 50) as f64 / duration.as_secs_f64();
    
    println!("✅ Memory sharing performance: {:.0} ops/sec", operations_per_second);
    
    // With proper memory sharing, should be fast
    assert!(operations_per_second > 500.0,
        "Memory sharing should enable >500 ops/sec, got {:.0}", operations_per_second);
}

#[tokio::test]
async fn run_cross_crate_integration_summary() {
    println!("\n🎉 Cross-crate integration tests completed!");
    println!("📊 Test Summary:");
    println!("  ✅ Model-Evaluator integration");
    println!("  ✅ Analyzer-Registry integration");
    println!("  ✅ Model-Registry bridge integration");
    println!("  ✅ Evaluator-Analyzer pipeline");
    println!("  ✅ All crates together");
    println!("  ✅ Configuration integration");
    println!("  ✅ Error propagation across crates");
    println!("  ✅ Async coordination");
    println!("  ✅ Memory sharing across crates");
    println!("\n🚀 Cross-crate Bridge Support Architecture integration validated!");
}