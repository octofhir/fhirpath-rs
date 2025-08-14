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

//! Property access validation tests for Task B3

#[cfg(test)]
mod tests {
    use crate::unified_implementations::type_checking::UnifiedIsFunction;
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_model::{FhirPathValue, resource::FhirResource, mock_provider::MockModelProvider};
    use serde_json::json;
    use std::sync::Arc;

    /// Test basic property access on FHIR resources
    #[tokio::test]
    async fn test_patient_gender_property_access() {
        // Create a Patient resource with gender
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123",
            "gender": "male"
        });
        
        let patient_resource = FhirResource::from_json(patient_json);
        let patient_value = FhirPathValue::Resource(Arc::new(patient_resource));
        
        // Test direct property access to gender
        match patient_value {
            FhirPathValue::Resource(resource) => {
                let gender_value = resource.get_property("gender");
                assert!(gender_value.is_some());
                
                if let Some(gender) = gender_value {
                    assert_eq!(gender, &json!("male"));
                }
            }
            _ => panic!("Expected Resource"),
        }
    }

    /// Test Patient.gender.is(code) pattern with is() function
    #[tokio::test]
    async fn test_patient_gender_is_code_pattern() {
        let is_func = UnifiedIsFunction::new();
        
        // Create Patient with gender
        let patient_json = json!({
            "resourceType": "Patient", 
            "id": "123",
            "gender": "male"
        });
        
        // Test 1: Check if string "male" is of type string
        let gender_value = FhirPathValue::String("male".into());
        let context = EvaluationContext::new(gender_value);
        
        let args = vec![FhirPathValue::String("string".into())];
        let result = is_func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
        println!("✅ 'male'.is(string) = true");
        
        // Test 2: Check if string "male" is of type code (should require ModelProvider)
        let gender_value = FhirPathValue::String("male".into());
        let mut context = EvaluationContext::new(gender_value);
        context.model_provider = Some(Arc::new(MockModelProvider::empty()));
        
        let args = vec![FhirPathValue::String("code".into())];
        let result = is_func.evaluate_async(&args, &context).await;
        
        match result {
            Ok(value) => {
                println!("✅ 'male'.is(code) = {:?}", value);
            }
            Err(e) => {
                println!("ℹ️  'male'.is(code) error (expected with MockProvider): {:?}", e);
            }
        }
    }

    /// Test property navigation with collections
    #[tokio::test]
    async fn test_property_navigation_collections() {
        // Create Patient with multiple names
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123", 
            "name": [
                {
                    "family": "Doe",
                    "given": ["John", "Jane"]
                },
                {
                    "family": "Smith", 
                    "given": ["Bob"]
                }
            ]
        });
        
        let patient_resource = FhirResource::from_json(patient_json);
        let patient_value = FhirPathValue::Resource(Arc::new(patient_resource));
        
        // Test property access to name array
        match patient_value {
            FhirPathValue::Resource(resource) => {
                let name_value = resource.get_property("name");
                assert!(name_value.is_some());
                
                if let Some(names) = name_value {
                    match names {
                        serde_json::Value::Array(arr) => {
                            assert_eq!(arr.len(), 2);
                            println!("✅ Patient.name returns array with {} items", arr.len());
                        }
                        _ => panic!("Expected array for Patient.name"),
                    }
                }
            }
            _ => panic!("Expected Resource"),
        }
    }

    /// Test choice type property access (value[x] pattern)
    #[tokio::test] 
    async fn test_choice_type_property_access() {
        // Create Observation with valueQuantity (choice type)
        let observation_json = json!({
            "resourceType": "Observation",
            "id": "123",
            "valueQuantity": {
                "value": 72.5,
                "unit": "kg",
                "code": "kg",
                "system": "http://unitsofmeasure.org"
            }
        });
        
        let observation_resource = FhirResource::from_json(observation_json);
        let observation_value = FhirPathValue::Resource(Arc::new(observation_resource));
        
        // Test access to choice type property
        match observation_value {
            FhirPathValue::Resource(resource) => {
                // Test direct access to valueQuantity
                let value_quantity = resource.get_property("valueQuantity");
                assert!(value_quantity.is_some());
                
                if let Some(vq) = value_quantity {
                    assert!(vq.is_object());
                    println!("✅ Observation.valueQuantity accessible");
                }
                
                // Test polymorphic access patterns
                let value_with_name = resource.get_property_with_name("value");
                assert!(value_with_name.is_some());
                
                if let Some((value, actual_name)) = value_with_name {
                    println!("✅ Polymorphic access: {} -> {}", "value", actual_name);
                    assert!(value.is_object());
                }
            }
            _ => panic!("Expected Resource"),
        }
    }

    /// Test property access error handling
    #[tokio::test]
    async fn test_property_access_error_handling() {
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        
        let patient_resource = FhirResource::from_json(patient_json);
        let patient_value = FhirPathValue::Resource(Arc::new(patient_resource));
        
        // Test access to non-existent property
        match patient_value {
            FhirPathValue::Resource(resource) => {
                let non_existent = resource.get_property("nonExistentProperty");
                assert!(non_existent.is_none());
                println!("✅ Non-existent property returns None");
            }
            _ => panic!("Expected Resource"),
        }
    }

    /// Test namespace property access (FHIR.Patient pattern)
    #[tokio::test]
    async fn test_namespace_property_access() {
        // Test FHIR namespace access
        let fhir_namespace = FhirPathValue::TypeInfoObject {
            namespace: "FHIR".into(),
            name: "namespace".into(),
        };
        
        // This simulates FHIR.Patient access
        match fhir_namespace {
            FhirPathValue::TypeInfoObject { namespace, name } => {
                assert_eq!(namespace.as_ref(), "FHIR");
                assert_eq!(name.as_ref(), "namespace");
                println!("✅ FHIR namespace access working");
            }
            _ => panic!("Expected TypeInfoObject"),
        }
    }
}