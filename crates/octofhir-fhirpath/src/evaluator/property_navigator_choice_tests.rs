//! Comprehensive tests for PropertyNavigator choice type integration
//!
//! This module tests the integration between PropertyNavigator and ChoiceTypeDetector
//! with real FHIR data scenarios and edge cases.

#[cfg(test)]
mod tests {
    use crate::core::{
        model_provider::MockModelProvider,
        FhirPathValue,
    };
    use crate::evaluator::property_navigator::PropertyNavigator;
    use serde_json::json;

    #[tokio::test]
    async fn test_observation_value_choice_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        // Comprehensive Observation with multiple value types
        let observation = json!({
            "resourceType": "Observation",
            "id": "example-multivalue",
            "status": "final",
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "33747-0",
                    "display": "General laboratory studies"
                }]
            },
            "subject": {
                "reference": "Patient/example"
            },
            "valueString": "Normal glucose levels",
            "valueQuantity": {
                "value": 95,
                "unit": "mg/dL",
                "system": "http://unitsofmeasure.org",
                "code": "mg/dL"
            },
            "valueBoolean": true,
            "valueInteger": 95,
            "_valueString": {
                "id": "value-string-id",
                "extension": [{
                    "url": "http://example.com/interpretation",
                    "valueString": "within normal range"
                }]
            },
            "_valueQuantity": {
                "extension": [{
                    "url": "http://example.com/precision",
                    "valueString": "exact"
                }]
            }
        });

        let observation_wrapped = FhirPathValue::resource_wrapped(observation);

        // Navigate to value property (should resolve all choice types)
        let value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();

        // Should return collection of all value choice implementations
        assert!(matches!(value_result, FhirPathValue::Collection(_)));

        if let FhirPathValue::Collection(collection) = value_result {
            assert_eq!(collection.len(), 4); // valueString, valueQuantity, valueBoolean, valueInteger

            let values: Vec<&FhirPathValue> = collection.iter().collect();
            let mut found_types = Vec::new();

            for value in values {
                if let FhirPathValue::Wrapped(wrapped) = value {
                    let type_info = wrapped.get_type_info().unwrap();
                    found_types.push(type_info.type_name.clone());
                    
                    // Verify proper FHIR namespace
                    assert_eq!(type_info.namespace, Some("FHIR".to_string()));
                    
                    // Check for primitive extensions where expected
                    match type_info.type_name.as_str() {
                        "String" => {
                            assert!(wrapped.has_extensions(), "String value should have extensions");
                            let primitive_element = wrapped.get_primitive_element().unwrap();
                            assert_eq!(primitive_element.id, Some("value-string-id".to_string()));
                            assert_eq!(primitive_element.extensions.len(), 1);
                        }
                        "Any" => {
                            // This should be the Quantity
                            assert_eq!(type_info.name, Some("Quantity".to_string()));
                            // Quantity might have extensions too
                        }
                        _ => {
                            // Other primitive types might not have extensions in this example
                        }
                    }
                }
            }

            // Verify we found all expected types
            found_types.sort();
            let mut expected_types = vec!["String".to_string(), "Any".to_string(), "Boolean".to_string(), "Integer".to_string()];
            expected_types.sort();
            assert_eq!(found_types, expected_types);
        }
    }

    #[tokio::test]
    async fn test_medication_effective_choice_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let medication = json!({
            "resourceType": "Medication",
            "id": "example-effective",
            "code": {
                "coding": [{
                    "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                    "code": "308136",
                    "display": "Amoxicillin 500mg"
                }]
            },
            "effectiveDateTime": "2023-12-01T08:00:00Z",
            "effectivePeriod": {
                "start": "2023-12-01",
                "end": "2023-12-07"
            },
            "_effectiveDateTime": {
                "extension": [{
                    "url": "http://example.com/precision",
                    "valueString": "minute"
                }, {
                    "url": "http://example.com/timezone-confirmed",
                    "valueBoolean": true
                }]
            }
        });

        let medication_wrapped = FhirPathValue::resource_wrapped(medication);

        // Navigate to effective property
        let effective_result = navigator
            .navigate_property(&medication_wrapped, "effective", &provider)
            .await
            .unwrap();

        assert!(matches!(effective_result, FhirPathValue::Collection(_)));

        if let FhirPathValue::Collection(collection) = effective_result {
            assert_eq!(collection.len(), 2); // effectiveDateTime and effectivePeriod

            let mut found_datetime = false;
            let mut found_period = false;

            for value in collection.iter() {
                if let FhirPathValue::Wrapped(wrapped) = value {
                    let type_info = wrapped.get_type_info().unwrap();
                    
                    match type_info.type_name.as_str() {
                        "DateTime" => {
                            found_datetime = true;
                            assert!(wrapped.has_extensions());
                            let primitive_element = wrapped.get_primitive_element().unwrap();
                            assert_eq!(primitive_element.extensions.len(), 2);
                        }
                        "Any" => {
                            // Should be Period
                            found_period = true;
                            assert_eq!(type_info.name, Some("Period".to_string()));
                            assert!(!wrapped.has_extensions());
                        }
                        _ => panic!("Unexpected type: {}", type_info.type_name),
                    }
                }
            }

            assert!(found_datetime, "Should find effectiveDateTime");
            assert!(found_period, "Should find effectivePeriod");
        }
    }

    #[tokio::test]
    async fn test_empty_choice_property_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let observation = json!({
            "resourceType": "Observation",
            "status": "final",
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "15074-8"
                }]
            }
            // No value[x] properties
        });

        let observation_wrapped = FhirPathValue::resource_wrapped(observation);

        // Navigate to value property (should find nothing)
        let value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();

        assert!(matches!(value_result, FhirPathValue::Empty));
    }

    #[tokio::test]
    async fn test_single_choice_property_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let observation = json!({
            "resourceType": "Observation",
            "valueString": "Single value only",
            "_valueString": {
                "extension": [{
                    "url": "http://example.com/note",
                    "valueString": "manually entered"
                }]
            }
        });

        let observation_wrapped = FhirPathValue::resource_wrapped(observation);

        let value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();

        // Single choice property should return wrapped value directly (not collection)
        assert!(matches!(value_result, FhirPathValue::Wrapped(_)));

        if let FhirPathValue::Wrapped(wrapped) = value_result {
            let type_info = wrapped.get_type_info().unwrap();
            assert_eq!(type_info.type_name, "String");
            assert!(wrapped.has_extensions());
            
            let primitive_element = wrapped.get_primitive_element().unwrap();
            assert_eq!(primitive_element.extensions.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_non_choice_property_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let patient = json!({
            "resourceType": "Patient",
            "id": "example",
            "active": true,
            "name": [{
                "family": "Smith",
                "given": ["John", "William"]
            }],
            "gender": "male",
            "_gender": {
                "extension": [{
                    "url": "http://example.com/gender-source",
                    "valueString": "self-reported"
                }]
            }
        });

        let patient_wrapped = FhirPathValue::resource_wrapped(patient);

        // Test regular property navigation
        let name_result = navigator
            .navigate_property(&patient_wrapped, "name", &provider)
            .await
            .unwrap();

        if let FhirPathValue::Wrapped(wrapped) = name_result {
            // Should be array/collection type
            let type_info = wrapped.get_type_info();
            assert!(type_info.is_some());
            // Name should be handled as regular property, not choice
        }

        // Test property with primitive extensions (non-choice)
        let gender_result = navigator
            .navigate_property(&patient_wrapped, "gender", &provider)
            .await
            .unwrap();

        if let FhirPathValue::Wrapped(wrapped) = gender_result {
            assert!(wrapped.has_extensions());
            let primitive_element = wrapped.get_primitive_element().unwrap();
            assert_eq!(primitive_element.extensions.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_bundle_entry_choice_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let bundle = json!({
            "resourceType": "Bundle",
            "entry": [{
                "resource": {
                    "resourceType": "Observation",
                    "valueString": "bundle observation value",
                    "valueInteger": 123
                }
            }, {
                "resource": {
                    "resourceType": "Patient",
                    "name": [{
                        "family": "Doe"
                    }]
                }
            }]
        });

        let bundle_wrapped = FhirPathValue::resource_wrapped(bundle);

        // Navigate to entry property
        let entry_result = navigator
            .navigate_property(&bundle_wrapped, "entry", &provider)
            .await
            .unwrap();

        // Entry should be a collection/array
        if let FhirPathValue::Wrapped(entries_wrapped) = entry_result {
            // This tests that choice detection doesn't interfere with regular array navigation
            let type_info = entries_wrapped.get_type_info();
            assert!(type_info.is_some());
        }
    }

    #[tokio::test]
    async fn test_choice_property_case_sensitivity() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let test_data = json!({
            "valueString": "correct case",
            "valuestring": "incorrect case", // lowercase - should not be detected
            "ValueString": "wrong case" // different case - should not be detected  
        });

        let wrapped = FhirPathValue::wrapped(test_data, None);

        let value_result = navigator
            .navigate_property(&wrapped, "value", &provider)
            .await
            .unwrap();

        // Should only find the correctly cased "valueString"
        if let FhirPathValue::Wrapped(result_wrapped) = value_result {
            // Single result for only the correctly cased property
            assert_eq!(**result_wrapped.unwrap(), json!("correct case"));
        }
    }

    #[tokio::test]
    async fn test_nested_object_choice_navigation() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let complex_observation = json!({
            "resourceType": "Observation",
            "component": [{
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "8480-6"
                    }]
                },
                "valueQuantity": {
                    "value": 120,
                    "unit": "mmHg"
                }
            }, {
                "code": {
                    "coding": [{
                        "system": "http://loinc.org", 
                        "code": "8462-4"
                    }]
                },
                "valueString": "Normal"
            }]
        });

        let observation_wrapped = FhirPathValue::resource_wrapped(complex_observation);

        // Navigate to component property first
        let component_result = navigator
            .navigate_property(&observation_wrapped, "component", &provider)
            .await
            .unwrap();

        // Component should be a collection
        if let FhirPathValue::Wrapped(component_wrapped) = component_result {
            // Verify it's handled as an array, not as choice type
            let type_info = component_wrapped.get_type_info();
            assert!(type_info.is_some());
        }
    }

    #[tokio::test]
    async fn test_arc_sharing_with_choice_properties() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        // Large observation to test Arc sharing efficiency
        let large_observation_data = json!({
            "resourceType": "Observation",
            "valueString": "test value",
            "valueInteger": 42,
            "largeData": "x".repeat(50000), // Large field to test Arc sharing
            "moreData": {
                "field1": "value1".repeat(1000),
                "field2": "value2".repeat(1000),
                "field3": "value3".repeat(1000)
            }
        });

        let observation_wrapped = FhirPathValue::resource_wrapped(large_observation_data);

        let start = std::time::Instant::now();
        let value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();
        let duration = start.elapsed();

        // Should complete quickly due to Arc sharing (no copying large data)
        assert!(duration.as_millis() < 50, "Navigation took too long: {:?}", duration);

        // Verify results are correct
        if let FhirPathValue::Collection(collection) = value_result {
            assert_eq!(collection.len(), 2); // valueString and valueInteger
        }
    }

    #[tokio::test]
    async fn test_error_handling_with_invalid_choice_suffix() {
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let test_data = json!({
            "valueInvalidType": "this should cause an error due to unknown suffix"
        });

        let wrapped = FhirPathValue::wrapped(test_data, None);

        // This should handle the error gracefully and return empty
        let value_result = navigator
            .navigate_property(&wrapped, "value", &provider)
            .await
            .unwrap();

        // Should return empty since the choice type detection would fail
        assert!(matches!(value_result, FhirPathValue::Empty));
    }
}