//! Tests with real FHIR data examples from the specification
//!
//! This module tests choice type detection and property navigation using actual
//! FHIR examples from the official specification to ensure real-world compatibility.

#[cfg(test)]
mod real_fhir_tests {
    use crate::core::{
        model_provider::MockModelProvider,
        FhirPathValue,
    };
    use crate::evaluator::{
        choice_types::ChoiceTypeDetector,
        property_navigator::PropertyNavigator,
    };
    use serde_json::json;
    use std::sync::Arc;

    // Test data based on real FHIR examples
    fn get_observation_vitalsigns() -> serde_json::Value {
        json!({
          "resourceType": "Observation",
          "id": "vitalsigns-2",
          "meta": {
            "versionId": "1",
            "lastUpdated": "2023-12-01T10:30:00Z"
          },
          "status": "final",
          "category": [
            {
              "coding": [
                {
                  "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                  "code": "vital-signs",
                  "display": "Vital Signs"
                }
              ]
            }
          ],
          "code": {
            "coding": [
              {
                "system": "http://loinc.org",
                "code": "85354-9",
                "display": "Blood pressure panel with all children optional"
              }
            ],
            "text": "Blood pressure systolic and diastolic"
          },
          "subject": {
            "reference": "Patient/example"
          },
          "effectiveDateTime": "2023-12-01T10:30:00Z",
          "valueQuantity": {
            "value": 120,
            "unit": "mmHg",
            "system": "http://unitsofmeasure.org",
            "code": "mm[Hg]"
          },
          "_effectiveDateTime": {
            "extension": [
              {
                "url": "http://hl7.org/fhir/StructureDefinition/observation-effectiveTime-precision",
                "valueString": "minute"
              }
            ]
          },
          "component": [
            {
              "code": {
                "coding": [
                  {
                    "system": "http://loinc.org",
                    "code": "8480-6",
                    "display": "Systolic blood pressure"
                  }
                ]
              },
              "valueQuantity": {
                "value": 120,
                "unit": "mmHg",
                "system": "http://unitsofmeasure.org",
                "code": "mm[Hg]"
              }
            },
            {
              "code": {
                "coding": [
                  {
                    "system": "http://loinc.org",
                    "code": "8462-4",
                    "display": "Diastolic blood pressure"
                  }
                ]
              },
              "valueQuantity": {
                "value": 80,
                "unit": "mmHg",
                "system": "http://unitsofmeasure.org", 
                "code": "mm[Hg]"
              }
            }
          ]
        })
    }

    fn get_observation_lab_result() -> serde_json::Value {
        json!({
          "resourceType": "Observation",
          "id": "example-lab",
          "status": "final",
          "category": [
            {
              "coding": [
                {
                  "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                  "code": "laboratory"
                }
              ]
            }
          ],
          "code": {
            "coding": [
              {
                "system": "http://loinc.org",
                "code": "33747-0",
                "display": "General laboratory studies"
              }
            ]
          },
          "subject": {
            "reference": "Patient/example"
          },
          "effectiveDateTime": "2023-12-01T08:30:00Z",
          "valueString": "NEGATIVE for glucose, protein, and ketones",
          "valueCodeableConcept": {
            "coding": [
              {
                "system": "http://snomed.info/sct",
                "code": "260385009",
                "display": "Negative (qualifier value)"
              }
            ]
          },
          "_valueString": {
            "id": "string-value-id",
            "extension": [
              {
                "url": "http://example.com/fhir/extension/lab-method",
                "valueString": "dipstick"
              },
              {
                "url": "http://example.com/fhir/extension/verified-by",
                "valueReference": {
                  "reference": "Practitioner/example"
                }
              }
            ]
          }
        })
    }

    fn get_medication_example() -> serde_json::Value {
        json!({
          "resourceType": "Medication",
          "id": "med0301",
          "meta": {
            "versionId": "1",
            "lastUpdated": "2023-12-01T12:00:00Z"
          },
          "code": {
            "coding": [
              {
                "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                "code": "1594660",
                "display": "Tylenol PM"
              }
            ]
          },
          "manufacturer": {
            "reference": "Organization/mmanu"
          },
          "form": {
            "coding": [
              {
                "system": "http://snomed.info/sct",
                "code": "385055001", 
                "display": "Tablet dose form"
              }
            ]
          },
          "ingredient": [
            {
              "itemCodeableConcept": {
                "coding": [
                  {
                    "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                    "code": "1389",
                    "display": "Acetaminophen"
                  }
                ]
              },
              "strengthRatio": {
                "numerator": {
                  "value": 500,
                  "unit": "mg",
                  "system": "http://unitsofmeasure.org",
                  "code": "mg"
                },
                "denominator": {
                  "value": 1,
                  "unit": "TAB",
                  "system": "http://terminology.hl7.org/CodeSystem/v3-orderableDrugForm",
                  "code": "TAB"
                }
              }
            }
          ]
        })
    }

    fn get_medication_dispense() -> serde_json::Value {
        json!({
          "resourceType": "MedicationDispense",
          "id": "meddisp0301",
          "status": "completed",
          "medicationCodeableConcept": {
            "coding": [
              {
                "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                "code": "1594660",
                "display": "Tylenol PM"
              }
            ]
          },
          "subject": {
            "reference": "Patient/pat1",
            "display": "Donald Duck"
          },
          "quantity": {
            "value": 30,
            "unit": "TAB",
            "system": "http://terminology.hl7.org/CodeSystem/v3-orderableDrugForm",
            "code": "TAB"
          },
          "daysSupply": {
            "value": 10,
            "unit": "Day",
            "system": "http://unitsofmeasure.org",
            "code": "d"
          },
          "whenPrepared": "2023-12-01T15:20:00Z",
          "whenHandedOver": "2023-12-01T15:30:00Z"
        })
    }

    #[tokio::test]
    async fn test_observation_vitalsigns_choice_detection() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let observation = get_observation_vitalsigns();

        // Test value choice detection - should find valueQuantity
        let value_resolution = detector
            .detect_choice_properties(&observation, "value")
            .await
            .unwrap();

        assert!(value_resolution.is_choice);
        assert_eq!(value_resolution.resolved_properties.len(), 1);

        let quantity_prop = value_resolution.first().unwrap();
        assert_eq!(quantity_prop.property_name, "valueQuantity");
        assert_eq!(quantity_prop.type_suffix, "Quantity");
        assert_eq!(quantity_prop.type_info.type_name, "Any");
        assert_eq!(quantity_prop.type_info.name, Some("Quantity".to_string()));

        // Test effective choice detection - should find effectiveDateTime
        let effective_resolution = detector
            .detect_choice_properties(&observation, "effective")
            .await
            .unwrap();

        assert!(effective_resolution.is_choice);
        assert_eq!(effective_resolution.resolved_properties.len(), 1);

        let datetime_prop = effective_resolution.first().unwrap();
        assert_eq!(datetime_prop.property_name, "effectiveDateTime");
        assert_eq!(datetime_prop.type_suffix, "DateTime");
        assert_eq!(datetime_prop.type_info.type_name, "DateTime");
        assert!(datetime_prop.has_extensions());

        // Test PropertyNavigator integration
        let observation_wrapped = FhirPathValue::resource_wrapped(observation);
        
        let nav_value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();

        // Should return single wrapped value for valueQuantity
        if let FhirPathValue::Wrapped(wrapped) = nav_value_result {
            let type_info = wrapped.get_type_info().unwrap();
            assert_eq!(type_info.type_name, "Any");
            assert_eq!(type_info.name, Some("Quantity".to_string()));
        } else {
            panic!("Expected wrapped value");
        }

        let nav_effective_result = navigator
            .navigate_property(&observation_wrapped, "effective", &provider)
            .await
            .unwrap();

        // Should return wrapped effectiveDateTime with extensions
        if let FhirPathValue::Wrapped(wrapped) = nav_effective_result {
            let type_info = wrapped.get_type_info().unwrap();
            assert_eq!(type_info.type_name, "DateTime");
            assert!(wrapped.has_extensions());
        } else {
            panic!("Expected wrapped value with extensions");
        }
    }

    #[tokio::test]
    async fn test_observation_lab_result_multiple_choices() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let lab_observation = get_observation_lab_result();

        // This observation has both valueString and valueCodeableConcept
        let value_resolution = detector
            .detect_choice_properties(&lab_observation, "value")
            .await
            .unwrap();

        assert!(value_resolution.is_choice);
        assert_eq!(value_resolution.resolved_properties.len(), 2);

        // Check valueString with extensions
        let string_prop = value_resolution.get_by_suffix("String").unwrap();
        assert_eq!(string_prop.property_name, "valueString");
        assert!(string_prop.has_extensions());

        let primitive_element = string_prop.primitive_element.as_ref().unwrap();
        assert_eq!(primitive_element.id, Some("string-value-id".to_string()));
        assert_eq!(primitive_element.extensions.len(), 2);

        // Check valueCodeableConcept
        let concept_prop = value_resolution.get_by_suffix("CodeableConcept").unwrap();
        assert_eq!(concept_prop.property_name, "valueCodeableConcept");
        assert_eq!(concept_prop.type_info.type_name, "Any");
        assert!(!concept_prop.has_extensions());

        // Test PropertyNavigator with multiple choices
        let observation_wrapped = FhirPathValue::resource_wrapped(lab_observation);
        
        let nav_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();

        // Should return collection with both choice properties
        if let FhirPathValue::Collection(collection) = nav_result {
            assert_eq!(collection.len(), 2);
            
            let mut found_string = false;
            let mut found_concept = false;
            
            for value in collection.iter() {
                if let FhirPathValue::Wrapped(wrapped) = value {
                    let type_info = wrapped.get_type_info().unwrap();
                    match type_info.type_name.as_str() {
                        "String" => {
                            found_string = true;
                            assert!(wrapped.has_extensions());
                        }
                        "Any" => {
                            found_concept = true;
                            assert_eq!(type_info.name, Some("CodeableConcept".to_string()));
                        }
                        _ => panic!("Unexpected type: {}", type_info.type_name),
                    }
                }
            }
            
            assert!(found_string && found_concept);
        } else {
            panic!("Expected collection with multiple values");
        }
    }

    #[tokio::test]
    async fn test_medication_ingredient_choice_detection() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let medication = get_medication_example();

        // Test ingredient choice detection (itemCodeableConcept vs itemReference)
        for ingredient in medication["ingredient"].as_array().unwrap() {
            let item_resolution = detector
                .detect_choice_properties(ingredient, "item")
                .await
                .unwrap();

            assert!(item_resolution.is_choice);
            assert_eq!(item_resolution.resolved_properties.len(), 1);
            
            let item_prop = item_resolution.first().unwrap();
            assert_eq!(item_prop.property_name, "itemCodeableConcept");
            assert_eq!(item_prop.type_suffix, "CodeableConcept");
            assert_eq!(item_prop.type_info.type_name, "Any");
            assert_eq!(item_prop.type_info.name, Some("CodeableConcept".to_string()));
        }
    }

    #[tokio::test]
    async fn test_medication_dispense_choice_detection() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);
        let navigator = PropertyNavigator;
        let provider = MockModelProvider;

        let med_dispense = get_medication_dispense();

        // Test medication choice detection (medicationCodeableConcept vs medicationReference)
        let medication_resolution = detector
            .detect_choice_properties(&med_dispense, "medication")
            .await
            .unwrap();

        assert!(medication_resolution.is_choice);
        assert_eq!(medication_resolution.resolved_properties.len(), 1);
        
        let med_prop = medication_resolution.first().unwrap();
        assert_eq!(med_prop.property_name, "medicationCodeableConcept");
        assert_eq!(med_prop.type_suffix, "CodeableConcept");

        // Test PropertyNavigator with medication choice
        let dispense_wrapped = FhirPathValue::resource_wrapped(med_dispense);
        
        let nav_result = navigator
            .navigate_property(&dispense_wrapped, "medication", &provider)
            .await
            .unwrap();

        if let FhirPathValue::Wrapped(wrapped) = nav_result {
            let type_info = wrapped.get_type_info().unwrap();
            assert_eq!(type_info.type_name, "Any");
            assert_eq!(type_info.name, Some("CodeableConcept".to_string()));
        } else {
            panic!("Expected wrapped medication choice");
        }
    }

    #[tokio::test]
    async fn test_observation_component_nested_choices() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let observation = get_observation_vitalsigns();

        // Test choice detection in nested component structures
        for component in observation["component"].as_array().unwrap() {
            let value_resolution = detector
                .detect_choice_properties(component, "value")
                .await
                .unwrap();

            assert!(value_resolution.is_choice);
            assert_eq!(value_resolution.resolved_properties.len(), 1);
            
            let quantity_prop = value_resolution.first().unwrap();
            assert_eq!(quantity_prop.property_name, "valueQuantity");
            assert_eq!(quantity_prop.type_suffix, "Quantity");

            // Verify the actual quantity values are different (systolic vs diastolic)
            let quantity_value = quantity_prop.value.as_object().unwrap();
            let pressure_value = quantity_value["value"].as_f64().unwrap();
            assert!(pressure_value == 120.0 || pressure_value == 80.0);
        }
    }

    #[tokio::test]
    async fn test_real_fhir_data_no_false_positives() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let observation = get_observation_vitalsigns();

        // Test that non-choice properties don't trigger false positives
        let non_choice_properties = ["status", "category", "code", "subject", "component"];
        
        for property in non_choice_properties {
            let resolution = detector
                .detect_choice_properties(&observation, property)
                .await
                .unwrap();
            
            assert!(!resolution.is_choice, "Property '{}' should not be detected as choice", property);
            assert!(resolution.resolved_properties.is_empty());
        }
    }

    #[tokio::test]
    async fn test_real_fhir_extension_preservation() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let lab_observation = get_observation_lab_result();

        let value_resolution = detector
            .detect_choice_properties(&lab_observation, "value")
            .await
            .unwrap();

        // Find the valueString property with extensions
        let string_prop = value_resolution.get_by_suffix("String").unwrap();
        assert!(string_prop.has_extensions());

        let primitive_element = string_prop.primitive_element.as_ref().unwrap();
        
        // Verify extension details are preserved
        assert_eq!(primitive_element.id, Some("string-value-id".to_string()));
        assert_eq!(primitive_element.extensions.len(), 2);
        
        // Check that extensions contain the expected URLs
        let extension_urls: Vec<String> = primitive_element.extensions.iter()
            .filter_map(|ext| ext.get("url").and_then(|url| url.as_str()))
            .map(String::from)
            .collect();
        
        assert!(extension_urls.contains(&"http://example.com/fhir/extension/lab-method".to_string()));
        assert!(extension_urls.contains(&"http://example.com/fhir/extension/verified-by".to_string()));
    }

    #[tokio::test]
    async fn test_real_world_performance_with_large_bundle() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Create a realistic large Bundle with mixed resource types
        let mut entries = Vec::new();
        
        // Add 100 observations with various choice types
        for i in 0..100 {
            let mut obs = get_observation_vitalsigns();
            obs["id"] = json!(format!("vitals-{}", i));
            entries.push(json!({"resource": obs}));
        }
        
        // Add 50 lab observations with multiple choice types
        for i in 0..50 {
            let mut lab = get_observation_lab_result();
            lab["id"] = json!(format!("lab-{}", i));
            entries.push(json!({"resource": lab}));
        }
        
        // Add some medications
        for i in 0..25 {
            let mut med = get_medication_example();
            med["id"] = json!(format!("med-{}", i));
            entries.push(json!({"resource": med}));
        }

        let large_bundle = json!({
            "resourceType": "Bundle",
            "id": "real-world-bundle",
            "type": "collection",
            "entry": entries
        });

        // Test performance with realistic data
        let start = std::time::Instant::now();
        let mut total_choice_detections = 0;

        for entry in large_bundle["entry"].as_array().unwrap() {
            let resource = &entry["resource"];
            let resource_type = resource["resourceType"].as_str().unwrap();
            
            match resource_type {
                "Observation" => {
                    let value_res = detector.detect_choice_properties(resource, "value").await.unwrap();
                    let effective_res = detector.detect_choice_properties(resource, "effective").await.unwrap();
                    
                    if value_res.is_choice {
                        total_choice_detections += value_res.resolved_properties.len();
                    }
                    if effective_res.is_choice {
                        total_choice_detections += effective_res.resolved_properties.len();
                    }
                }
                "Medication" => {
                    for ingredient in resource["ingredient"].as_array().unwrap_or(&vec![]) {
                        let item_res = detector.detect_choice_properties(ingredient, "item").await.unwrap();
                        if item_res.is_choice {
                            total_choice_detections += item_res.resolved_properties.len();
                        }
                    }
                }
                _ => {}
            }
        }

        let duration = start.elapsed();

        // Should detect all expected choice properties
        // 100 vitalsigns (1 value + 1 effective each) + 50 labs (2 values + 1 effective each) + 25 meds (1 item each)
        // = 200 + 150 + 25 = 375 total choice detections
        assert_eq!(total_choice_detections, 375);
        
        // Should complete in reasonable time for realistic Bundle size
        assert!(duration.as_secs() < 5, "Real-world Bundle processing took too long: {:?}", duration);
        
        println!("Processed real-world Bundle with {} resources and {} choice detections in {:?}",
                175, total_choice_detections, duration);
    }
}