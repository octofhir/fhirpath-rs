//! Performance tests for choice type detection with large FHIR resources
//!
//! This module contains performance benchmarks and stress tests for the choice type
//! detection system to ensure it scales well with large Bundle resources and
//! complex nested structures.

#[cfg(test)]
mod performance_tests {
    use crate::core::model_provider::MockModelProvider;
    use crate::evaluator::choice_types::ChoiceTypeDetector;
    use crate::evaluator::property_navigator::PropertyNavigator;
    use crate::core::FhirPathValue;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Instant;

    #[tokio::test]
    async fn test_large_bundle_choice_detection_performance() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Create a large Bundle with many Observation resources
        let mut entries = Vec::new();
        for i in 0..1000 {
            entries.push(json!({
                "resource": {
                    "resourceType": "Observation",
                    "id": format!("obs-{}", i),
                    "status": "final",
                    "valueString": format!("Value {}", i),
                    "valueInteger": i,
                    "valueQuantity": {
                        "value": i as f64 * 1.5,
                        "unit": "mg/dL"
                    },
                    "_valueString": {
                        "extension": [{
                            "url": "http://example.com/source",
                            "valueString": "automated"
                        }]
                    }
                }
            }));
        }

        let large_bundle = json!({
            "resourceType": "Bundle",
            "id": "large-bundle",
            "type": "collection", 
            "entry": entries
        });

        // Test choice detection performance on multiple resources
        let start = Instant::now();
        let mut total_choice_properties = 0;

        for entry in large_bundle["entry"].as_array().unwrap() {
            let resource = &entry["resource"];
            let resolution = detector
                .detect_choice_properties(resource, "value")
                .await
                .unwrap();
            
            if resolution.is_choice {
                total_choice_properties += resolution.resolved_properties.len();
            }
        }

        let duration = start.elapsed();

        // Each observation should have 3 choice properties (valueString, valueInteger, valueQuantity)
        assert_eq!(total_choice_properties, 3000);
        
        // Should complete in reasonable time (less than 2 seconds for 1000 resources)
        assert!(duration.as_secs() < 2, "Choice detection took too long: {:?}", duration);
        
        println!("Processed 1000 observations with {} choice properties in {:?}", 
                total_choice_properties, duration);
    }

    #[tokio::test]
    async fn test_property_navigator_performance_with_large_resources() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        // Create a single large observation with many choice properties
        let mut large_observation = serde_json::Map::new();
        large_observation.insert("resourceType".to_string(), json!("Observation"));
        large_observation.insert("id".to_string(), json!("large-obs"));
        
        // Add many choice properties
        large_observation.insert("valueString".to_string(), json!("string value"));
        large_observation.insert("valueInteger".to_string(), json!(42));
        large_observation.insert("valueBoolean".to_string(), json!(true));
        large_observation.insert("valueDecimal".to_string(), json!(123.45));
        large_observation.insert("valueQuantity".to_string(), json!({
            "value": 100.0,
            "unit": "mg"
        }));
        large_observation.insert("valueCodeableConcept".to_string(), json!({
            "coding": [{
                "system": "http://loinc.org",
                "code": "LA6113-0"
            }]
        }));

        // Add many non-choice properties to test filtering performance
        for i in 0..5000 {
            large_observation.insert(format!("property{}", i), json!(format!("value{}", i)));
            large_observation.insert(format!("object{}", i), json!({
                "nested": format!("data{}", i),
                "more": {
                    "deeply": {
                        "nested": format!("value{}", i)
                    }
                }
            }));
        }

        // Add some primitive extensions
        large_observation.insert("_valueString".to_string(), json!({
            "extension": [{
                "url": "http://example.com/note",
                "valueString": "test extension"
            }]
        }));

        let observation_wrapped = FhirPathValue::wrapped(
            serde_json::Value::Object(large_observation), 
            None
        );

        // Test navigation performance
        let start = Instant::now();
        let value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();
        let duration = start.elapsed();

        // Should find all choice properties despite large object
        if let FhirPathValue::Collection(collection) = value_result {
            assert_eq!(collection.len(), 6); // All 6 choice properties
        } else {
            panic!("Expected collection result");
        }

        // Should complete quickly despite large object
        assert!(duration.as_millis() < 200, "Navigation took too long: {:?}", duration);
        
        println!("Navigated large observation with 10,000+ properties in {:?}", duration);
    }

    #[tokio::test]
    async fn test_deep_nested_bundle_performance() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Create deeply nested Bundle structure
        let mut deep_bundle = json!({
            "resourceType": "Bundle",
            "entry": []
        });

        // Create nested Bundle entries (Bundle containing Bundles)
        for i in 0..100 {
            let nested_entries: Vec<_> = (0..10).map(|j| {
                json!({
                    "resource": {
                        "resourceType": "Observation",
                        "id": format!("obs-{}-{}", i, j),
                        "valueString": format!("Value {} {}", i, j),
                        "valueInteger": i * 10 + j,
                        "effectiveDateTime": "2023-12-01T10:00:00Z",
                        "_valueString": {
                            "extension": [{
                                "url": "http://example.com/batch",
                                "valueString": format!("batch-{}", i)
                            }]
                        }
                    }
                })
            }).collect();

            deep_bundle["entry"].as_array_mut().unwrap().push(json!({
                "resource": {
                    "resourceType": "Bundle",
                    "id": format!("nested-bundle-{}", i),
                    "entry": nested_entries
                }
            }));
        }

        // Test performance with nested structure
        let start = Instant::now();
        let mut total_detections = 0;

        // Navigate through nested structure
        for outer_entry in deep_bundle["entry"].as_array().unwrap() {
            let nested_bundle = &outer_entry["resource"];
            if nested_bundle["resourceType"] == "Bundle" {
                if let Some(inner_entries) = nested_bundle["entry"].as_array() {
                    for inner_entry in inner_entries {
                        let observation = &inner_entry["resource"];
                        if observation["resourceType"] == "Observation" {
                            let value_resolution = detector
                                .detect_choice_properties(observation, "value")
                                .await
                                .unwrap();
                            
                            let effective_resolution = detector
                                .detect_choice_properties(observation, "effective")
                                .await
                                .unwrap();
                            
                            if value_resolution.is_choice {
                                total_detections += value_resolution.resolved_properties.len();
                            }
                            if effective_resolution.is_choice {
                                total_detections += effective_resolution.resolved_properties.len();
                            }
                        }
                    }
                }
            }
        }

        let duration = start.elapsed();

        // Should find choice properties in all nested observations
        // 100 bundles * 10 observations * (2 value + 1 effective) = 3000 total
        assert_eq!(total_detections, 3000);
        
        // Should complete in reasonable time
        assert!(duration.as_secs() < 3, "Nested detection took too long: {:?}", duration);
        
        println!("Processed deeply nested structure with {} detections in {:?}", 
                total_detections, duration);
    }

    #[tokio::test]
    async fn test_memory_usage_with_large_choice_collections() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Create observation with all possible choice types to test memory usage
        let comprehensive_observation = json!({
            "resourceType": "Observation",
            "valueString": "string value",
            "valueInteger": 42,
            "valueBoolean": true,
            "valueDecimal": 123.45,
            "valueDate": "2023-12-01",
            "valueDateTime": "2023-12-01T10:00:00Z",
            "valueTime": "10:00:00",
            "valueCode": "active",
            "valueUri": "http://example.com",
            "valueUrl": "https://example.com/resource",
            "valueId": "patient-123",
            "valueOid": "1.2.3.4.5",
            "valueUuid": "550e8400-e29b-41d4-a716-446655440000",
            "valueQuantity": {
                "value": 123.45,
                "unit": "mg",
                "system": "http://unitsofmeasure.org",
                "code": "mg"
            },
            "valueCoding": {
                "system": "http://loinc.org", 
                "code": "LA6113-0",
                "display": "Positive"
            },
            "valueCodeableConcept": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "LA6113-0"
                }]
            },
            "valueReference": {
                "reference": "Patient/123"
            },
            "valuePeriod": {
                "start": "2023-01-01",
                "end": "2023-12-31"
            },
            "valueRange": {
                "low": {"value": 10, "unit": "mg"},
                "high": {"value": 20, "unit": "mg"}
            },
            "valueAttachment": {
                "contentType": "image/png",
                "data": "base64data=="
            }
        });

        // Test memory efficiency with Arc sharing
        let start = Instant::now();
        let resolution = detector
            .detect_choice_properties(&comprehensive_observation, "value")
            .await
            .unwrap();
        let detection_duration = start.elapsed();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 20); // All choice types

        // Convert to wrapped values (should use Arc sharing)
        let start = Instant::now();
        let wrapped_values = detector
            .choice_resolution_to_wrapped_values(&resolution)
            .unwrap();
        let conversion_duration = start.elapsed();

        assert_eq!(wrapped_values.len(), 20);

        // Test multiple conversions to verify Arc sharing efficiency
        let start = Instant::now();
        for _ in 0..100 {
            let _more_wrapped = detector
                .choice_resolution_to_wrapped_values(&resolution)
                .unwrap();
        }
        let multiple_conversions_duration = start.elapsed();

        // Arc sharing should make repeated conversions very fast
        assert!(multiple_conversions_duration.as_millis() < 50, 
               "Multiple conversions took too long: {:?}", multiple_conversions_duration);

        println!("Detection: {:?}, Conversion: {:?}, 100x conversions: {:?}",
                detection_duration, conversion_duration, multiple_conversions_duration);
    }

    #[tokio::test]
    async fn test_concurrent_choice_detection() {
        use tokio::task;

        let model_provider = Arc::new(MockModelProvider);
        
        // Create multiple observations for concurrent processing
        let observations: Vec<_> = (0..50).map(|i| {
            json!({
                "resourceType": "Observation",
                "id": format!("concurrent-obs-{}", i),
                "valueString": format!("Concurrent value {}", i),
                "valueInteger": i,
                "valueQuantity": {
                    "value": i as f64,
                    "unit": "mg"
                },
                "_valueString": {
                    "extension": [{
                        "url": "http://example.com/concurrent",
                        "valueInteger": i
                    }]
                }
            })
        }).collect();

        // Test concurrent choice detection
        let start = Instant::now();
        let tasks: Vec<_> = observations.into_iter().map(|obs| {
            let detector = ChoiceTypeDetector::new(model_provider.clone());
            task::spawn(async move {
                detector.detect_choice_properties(&obs, "value").await
            })
        }).collect();

        let mut total_properties = 0;
        for task in tasks {
            let resolution = task.await.unwrap().unwrap();
            if resolution.is_choice {
                total_properties += resolution.resolved_properties.len();
            }
        }
        let duration = start.elapsed();

        // Should find all choice properties across all concurrent tasks
        assert_eq!(total_properties, 150); // 50 observations * 3 properties each
        
        // Concurrent processing should be efficient
        assert!(duration.as_secs() < 2, "Concurrent processing took too long: {:?}", duration);
        
        println!("Concurrent processing of 50 observations completed in {:?}", duration);
    }

    #[tokio::test]
    async fn test_performance_regression_baseline() {
        // This test serves as a baseline for performance regression testing
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let standard_observation = json!({
            "resourceType": "Observation",
            "valueString": "baseline test",
            "valueInteger": 42,
            "valueQuantity": {
                "value": 123.45,
                "unit": "mg"
            }
        });

        // Perform baseline measurement
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _resolution = detector
                .detect_choice_properties(&standard_observation, "value")
                .await
                .unwrap();
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / iterations;

        // Baseline: should average less than 1ms per detection
        assert!(avg_duration.as_micros() < 1000, 
               "Average detection time exceeded baseline: {:?}", avg_duration);
        
        println!("Baseline: {} iterations in {:?} (avg: {:?} per detection)",
                iterations, duration, avg_duration);
    }
}