//! Comprehensive test suite for enhanced terminology provider functionality

#[cfg(test)]
mod tests {
    use super::super::terminology_provider::{
        TerminologyProvider, DefaultTerminologyProvider, MockTerminologyProvider, ConceptDetails
    };
    use super::super::terminology_utils::{
        TerminologyUtils, Coding, ConceptTranslation, ConceptDesignation
    };
    use crate::core::FhirPathValue;
    use serde_json::json;

    /// Test basic Coding creation and validation
    #[test]
    fn test_coding_creation_and_validation() {
        let coding = Coding::new("http://loinc.org", "789-8")
            .with_display("Red blood cell count")
            .with_version("2.74");
            
        assert_eq!(coding.system, "http://loinc.org");
        assert_eq!(coding.code, "789-8");
        assert_eq!(coding.display.as_ref().unwrap(), "Red blood cell count");
        assert_eq!(coding.version.as_ref().unwrap(), "2.74");
        
        // Test validation
        assert!(TerminologyUtils::validate_coding(&coding).is_ok());
        
        // Test invalid coding
        let invalid_coding = Coding::new("", "test");
        assert!(TerminologyUtils::validate_coding(&invalid_coding).is_err());
    }

    /// Test Coding extraction from FHIR values
    #[test]
    fn test_coding_extraction_from_fhir_values() {
        // Test direct Coding extraction
        let coding_json = json!({
            "system": "http://loinc.org",
            "code": "789-8",
            "display": "Erythrocytes [#/volume] in Blood by Automated count",
            "version": "2.74"
        });

        let fhir_value = FhirPathValue::Resource(coding_json);
        let coding = TerminologyUtils::extract_coding(&fhir_value).unwrap();
        
        assert_eq!(coding.system, "http://loinc.org");
        assert_eq!(coding.code, "789-8");
        assert_eq!(coding.display.as_ref().unwrap(), "Erythrocytes [#/volume] in Blood by Automated count");
        assert_eq!(coding.version.as_ref().unwrap(), "2.74");

        // Test CodeableConcept extraction (should get first coding)
        let codeable_concept_json = json!({
            "coding": [
                {
                    "system": "http://loinc.org",
                    "code": "789-8",
                    "display": "Erythrocytes [#/volume] in Blood by Automated count"
                },
                {
                    "system": "http://snomed.info/sct",
                    "code": "26453005",
                    "display": "Erythrocyte count"
                }
            ],
            "text": "Red blood cell count"
        });

        let fhir_value = FhirPathValue::Resource(codeable_concept_json.clone());
        let coding = TerminologyUtils::extract_coding(&fhir_value).unwrap();
        
        assert_eq!(coding.system, "http://loinc.org");
        assert_eq!(coding.code, "789-8");

        // Test extracting all codings
        let codings = TerminologyUtils::extract_all_codings(&FhirPathValue::Resource(codeable_concept_json)).unwrap();
        assert_eq!(codings.len(), 2);
        assert_eq!(codings[0].system, "http://loinc.org");
        assert_eq!(codings[0].code, "789-8");
        assert_eq!(codings[1].system, "http://snomed.info/sct");
        assert_eq!(codings[1].code, "26453005");
    }

    /// Test ConceptTranslation utility functions
    #[test]
    fn test_concept_translation_utilities() {
        let translation = ConceptTranslation {
            equivalence: "equivalent".to_string(),
            concept: Coding::new("http://hl7.org/fhir/administrative-gender", "male")
                .with_display("Male"),
            comment: Some("Standard gender mapping".to_string()),
        };

        let fhir_value = TerminologyUtils::translation_to_fhir_value(&translation);
        
        if let FhirPathValue::Resource(json_val) = fhir_value {
            let obj = json_val.as_object().unwrap();
            assert_eq!(obj.get("equivalence").unwrap().as_str().unwrap(), "equivalent");
            assert!(obj.contains_key("concept"));
            assert_eq!(obj.get("comment").unwrap().as_str().unwrap(), "Standard gender mapping");
            
            let concept = obj.get("concept").unwrap().as_object().unwrap();
            assert_eq!(concept.get("system").unwrap().as_str().unwrap(), "http://hl7.org/fhir/administrative-gender");
            assert_eq!(concept.get("code").unwrap().as_str().unwrap(), "male");
            assert_eq!(concept.get("display").unwrap().as_str().unwrap(), "Male");
        } else {
            panic!("Expected Resource value");
        }
    }

    /// Test ConceptDesignation utility functions
    #[test]
    fn test_concept_designation_utilities() {
        let designation = ConceptDesignation {
            language: Some("es".to_string()),
            use_coding: Some(Coding::new("http://terminology.hl7.org/CodeSystem/designation-usage", "display")),
            value: "Masculino".to_string(),
        };

        let fhir_value = TerminologyUtils::designation_to_fhir_value(&designation);
        
        if let FhirPathValue::Resource(json_val) = fhir_value {
            let obj = json_val.as_object().unwrap();
            assert_eq!(obj.get("value").unwrap().as_str().unwrap(), "Masculino");
            assert_eq!(obj.get("language").unwrap().as_str().unwrap(), "es");
            assert!(obj.contains_key("use"));
            
            let use_obj = obj.get("use").unwrap().as_object().unwrap();
            assert_eq!(use_obj.get("system").unwrap().as_str().unwrap(), "http://terminology.hl7.org/CodeSystem/designation-usage");
            assert_eq!(use_obj.get("code").unwrap().as_str().unwrap(), "display");
        } else {
            panic!("Expected Resource value");
        }
    }

    /// Test coding equality functions
    #[test]
    fn test_coding_equality() {
        let coding1 = Coding::new("http://loinc.org", "789-8").with_version("2.74");
        let coding2 = Coding::new("http://loinc.org", "789-8").with_version("2.74");
        let coding3 = Coding::new("http://loinc.org", "789-8"); // No version
        let coding4 = Coding::new("http://loinc.org", "123-4");
        
        assert!(TerminologyUtils::codings_equal(&coding1, &coding2));
        assert!(!TerminologyUtils::codings_equal(&coding1, &coding3)); // Different versions
        assert!(!TerminologyUtils::codings_equal(&coding1, &coding4)); // Different codes
    }

    /// Test MockTerminologyProvider functionality
    #[tokio::test]
    async fn test_mock_terminology_provider() {
        let provider = MockTerminologyProvider;

        // Test ValueSet membership
        let coding = Coding::new("http://hl7.org/fhir/administrative-gender", "male");
        let is_member = provider.check_valueset_membership(&coding, "http://hl7.org/fhir/ValueSet/administrative-gender").await.unwrap();
        assert!(is_member);

        let other_coding = Coding::new("http://example.org", "unknown");
        let is_not_member = provider.check_valueset_membership(&other_coding, "http://hl7.org/fhir/ValueSet/administrative-gender").await.unwrap();
        assert!(!is_not_member);

        // Test concept translation
        let source_coding = Coding::new("http://example.org/gender", "M");
        let translations = provider.translate_concept(&source_coding, "http://example.org/ConceptMap/gender-mapping", false).await.unwrap();
        assert_eq!(translations.len(), 1);
        
        if let FhirPathValue::Resource(json_val) = &translations[0] {
            let obj = json_val.as_object().unwrap();
            assert_eq!(obj.get("equivalence").unwrap().as_str().unwrap(), "equivalent");
            
            let concept = obj.get("concept").unwrap().as_object().unwrap();
            assert_eq!(concept.get("code").unwrap().as_str().unwrap(), "male");
        } else {
            panic!("Expected translation result");
        }

        // Test code validation
        let is_valid = provider.validate_code("http://hl7.org/fhir/administrative-gender", "male").await.unwrap();
        assert!(is_valid);

        let is_invalid = provider.validate_code("http://hl7.org/fhir/administrative-gender", "unknown").await.unwrap();
        assert!(!is_invalid);

        // Test subsumption
        let parent_coding = Coding::new("http://example.org/codes", "parent");
        let child_coding = Coding::new("http://example.org/codes", "child");
        let subsumes = provider.check_subsumption(&parent_coding, &child_coding).await.unwrap();
        assert!(subsumes);

        let unrelated_coding = Coding::new("http://example.org/codes", "other");
        let does_not_subsume = provider.check_subsumption(&parent_coding, &unrelated_coding).await.unwrap();
        assert!(!does_not_subsume);

        // Test designations
        let male_coding = Coding::new("http://hl7.org/fhir/administrative-gender", "male");
        let designations_en = provider.get_designations(&male_coding, Some("en"), None).await.unwrap();
        assert_eq!(designations_en.len(), 1);
        
        if let FhirPathValue::Resource(json_val) = &designations_en[0] {
            let obj = json_val.as_object().unwrap();
            assert_eq!(obj.get("value").unwrap().as_str().unwrap(), "Male");
            assert_eq!(obj.get("language").unwrap().as_str().unwrap(), "en");
        }

        let designations_es = provider.get_designations(&male_coding, Some("es"), None).await.unwrap();
        assert_eq!(designations_es.len(), 1);
        
        if let FhirPathValue::Resource(json_val) = &designations_es[0] {
            let obj = json_val.as_object().unwrap();
            assert_eq!(obj.get("value").unwrap().as_str().unwrap(), "Masculino");
            assert_eq!(obj.get("language").unwrap().as_str().unwrap(), "es");
        }

        // Test properties
        let properties = provider.get_concept_properties(&male_coding, "definition").await.unwrap();
        assert_eq!(properties.len(), 1);
        
        if let FhirPathValue::String(def) = &properties[0] {
            assert_eq!(def, "Male gender");
        } else {
            panic!("Expected string property value");
        }

        // Test server URL
        let server_url = provider.get_terminology_server_url().await.unwrap();
        assert_eq!(server_url, "http://mock-tx.example.com");

        // Test empty results for unknown concepts
        let unknown_coding = Coding::new("http://example.org", "unknown");
        let empty_designations = provider.get_designations(&unknown_coding, None, None).await.unwrap();
        assert!(empty_designations.is_empty());

        let empty_properties = provider.get_concept_properties(&unknown_coding, "definition").await.unwrap();
        assert!(empty_properties.is_empty());

        let empty_expansions = provider.expand_valueset("http://example.org/ValueSet/empty").await.unwrap();
        assert!(empty_expansions.is_empty());

        let no_concept = provider.lookup_concept(&unknown_coding).await.unwrap();
        assert!(no_concept.is_none());
    }

    /// Test DefaultTerminologyProvider creation
    #[test]
    fn test_default_terminology_provider_creation() {
        let default_provider = DefaultTerminologyProvider::new();
        // We can't test the actual HTTP functionality in unit tests, but we can test construction
        let provider_with_custom_url = DefaultTerminologyProvider::with_server_url("https://custom-tx.example.com/r4");
        
        // These should not panic and should create valid provider instances
        // Real integration testing would require network connectivity
    }

    /// Test error handling for invalid data
    #[test]
    fn test_error_handling() {
        // Test extracting coding from invalid data
        let invalid_fhir_value = FhirPathValue::String("not a coding".to_string());
        assert!(TerminologyUtils::extract_coding(&invalid_fhir_value).is_err());

        // Test validation of invalid coding
        let invalid_coding_empty_system = Coding::new("", "test");
        assert!(TerminologyUtils::validate_coding(&invalid_coding_empty_system).is_err());

        let invalid_coding_empty_code = Coding::new("http://example.org", "");
        assert!(TerminologyUtils::validate_coding(&invalid_coding_empty_code).is_err());

        let invalid_coding_bad_uri = Coding::new("not-a-uri", "test");
        assert!(TerminologyUtils::validate_coding(&invalid_coding_bad_uri).is_err());
    }

    /// Test edge cases and boundary conditions
    #[test]
    fn test_edge_cases() {
        // Test coding with minimal valid data
        let minimal_coding = Coding::new("http://example.org", "test");
        assert!(TerminologyUtils::validate_coding(&minimal_coding).is_ok());

        // Test coding with URN system
        let urn_coding = Coding::new("urn:oid:2.16.840.1.113883.6.1", "789-8");
        assert!(TerminologyUtils::validate_coding(&urn_coding).is_ok());

        // Test empty collections
        let empty_codings: Vec<Coding> = vec![];
        assert!(empty_codings.is_empty());

        // Test coding round-trip conversion
        let original_coding = Coding::new("http://loinc.org", "789-8").with_display("Test");
        let fhir_value = TerminologyUtils::coding_to_value(&original_coding);
        let extracted_coding = TerminologyUtils::extract_coding(&fhir_value).unwrap();
        
        assert!(TerminologyUtils::codings_equal(&original_coding, &extracted_coding));
    }

    /// Test complex scenarios with multiple codings and translations
    #[test]
    fn test_complex_scenarios() {
        // Test CodeableConcept with multiple codings
        let complex_codeable_concept = json!({
            "coding": [
                {
                    "system": "http://loinc.org",
                    "code": "789-8",
                    "display": "Erythrocytes [#/volume] in Blood by Automated count"
                },
                {
                    "system": "http://snomed.info/sct",
                    "code": "26453005",
                    "display": "Erythrocyte count"
                },
                {
                    "system": "http://acme.org/tests",
                    "code": "RBC",
                    "display": "Red Blood Cell Count"
                }
            ],
            "text": "Red blood cell count - multiple coding systems"
        });

        let fhir_value = FhirPathValue::Resource(complex_codeable_concept);
        let all_codings = TerminologyUtils::extract_all_codings(&fhir_value).unwrap();
        
        assert_eq!(all_codings.len(), 3);
        assert_eq!(all_codings[0].system, "http://loinc.org");
        assert_eq!(all_codings[1].system, "http://snomed.info/sct");
        assert_eq!(all_codings[2].system, "http://acme.org/tests");

        // Test first coding extraction from the same data
        let first_coding = TerminologyUtils::extract_coding(&FhirPathValue::Resource(
            json!({
                "coding": [
                    {
                        "system": "http://loinc.org",
                        "code": "789-8",
                        "display": "Erythrocytes [#/volume] in Blood by Automated count"
                    },
                    {
                        "system": "http://snomed.info/sct",
                        "code": "26453005",
                        "display": "Erythrocyte count"
                    }
                ]
            })
        )).unwrap();
        
        assert_eq!(first_coding.system, "http://loinc.org");
        assert_eq!(first_coding.code, "789-8");
    }

    /// Test async functionality with proper error propagation
    #[tokio::test]
    async fn test_async_error_propagation() {
        let provider = MockTerminologyProvider;

        // Test that async methods properly return errors for invalid inputs
        // (The mock provider doesn't generate network errors, but this tests the async structure)
        
        // Test with different systems for subsumption (should return false, not error)
        let coding_a = Coding::new("http://system1.org", "code1");
        let coding_b = Coding::new("http://system2.org", "code2");
        let result = provider.check_subsumption(&coding_a, &coding_b).await.unwrap();
        assert!(!result); // Different systems should not subsume each other
    }

    /// Test concurrent access (basic thread safety verification)
    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;
        use tokio::task;

        let provider = Arc::new(MockTerminologyProvider);
        let coding = Coding::new("http://hl7.org/fhir/administrative-gender", "male");
        
        let mut tasks = Vec::new();
        
        // Create multiple concurrent tasks
        for _ in 0..10 {
            let provider_clone = Arc::clone(&provider);
            let coding_clone = coding.clone();
            
            tasks.push(task::spawn(async move {
                provider_clone.check_valueset_membership(
                    &coding_clone, 
                    "http://hl7.org/fhir/ValueSet/administrative-gender"
                ).await.unwrap()
            }));
        }
        
        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;
        
        // All should return true
        for result in results {
            assert!(result.unwrap());
        }
    }
}