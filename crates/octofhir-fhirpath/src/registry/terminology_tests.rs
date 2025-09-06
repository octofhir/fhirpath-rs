//! Comprehensive test suite for terminology functions

#[cfg(test)]
mod tests {
    use super::super::terminology_utils::{Coding, TerminologyUtils};
    use super::super::{FunctionCategory, FunctionRegistry};
    use crate::core::FhirPathValue;
    use serde_json::json;

    #[test]
    fn test_terminology_utils_extract_coding() {
        // Test direct Coding extraction
        let coding_json = json!({
            "system": "http://loinc.org",
            "code": "789-8",
            "display": "Erythrocytes [#/volume] in Blood by Automated count"
        });

        let fhir_value = FhirPathValue::Resource(coding_json);
        let coding = TerminologyUtils::extract_coding(&fhir_value).unwrap();

        assert_eq!(coding.system, "http://loinc.org");
        assert_eq!(coding.code, "789-8");
        assert_eq!(
            coding.display.as_ref().unwrap(),
            "Erythrocytes [#/volume] in Blood by Automated count"
        );
    }

    #[test]
    fn test_terminology_utils_extract_codeable_concept() {
        // Test CodeableConcept extraction
        let codeable_concept_json = json!({
            "coding": [{
                "system": "http://loinc.org",
                "code": "789-8",
                "display": "Erythrocytes [#/volume] in Blood by Automated count"
            }],
            "text": "Red blood cell count"
        });

        let fhir_value = FhirPathValue::Resource(codeable_concept_json);
        let coding = TerminologyUtils::extract_coding(&fhir_value).unwrap();

        assert_eq!(coding.system, "http://loinc.org");
        assert_eq!(coding.code, "789-8");
    }

    #[test]
    fn test_terminology_utils_validate_coding() {
        let valid_coding = Coding::new("http://loinc.org", "789-8");
        assert!(TerminologyUtils::validate_coding(&valid_coding).is_ok());

        let invalid_coding_no_system = Coding::new("", "test");
        assert!(TerminologyUtils::validate_coding(&invalid_coding_no_system).is_err());

        let invalid_coding_no_code = Coding::new("http://loinc.org", "");
        assert!(TerminologyUtils::validate_coding(&invalid_coding_no_code).is_err());

        let invalid_coding_bad_system = Coding::new("not-a-uri", "test");
        assert!(TerminologyUtils::validate_coding(&invalid_coding_bad_system).is_err());
    }

    #[test]
    fn test_terminology_utils_codings_equal() {
        let coding1 = Coding::new("http://loinc.org", "789-8");
        let coding2 = Coding::new("http://loinc.org", "789-8");
        let coding3 = Coding::new("http://loinc.org", "123-4");

        assert!(TerminologyUtils::codings_equal(&coding1, &coding2));
        assert!(!TerminologyUtils::codings_equal(&coding1, &coding3));
    }

    #[test]
    fn test_terminology_utils_extract_all_codings() {
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

        let fhir_value = FhirPathValue::Resource(codeable_concept_json);
        let codings = TerminologyUtils::extract_all_codings(&fhir_value).unwrap();

        assert_eq!(codings.len(), 2);
        assert_eq!(codings[0].system, "http://loinc.org");
        assert_eq!(codings[0].code, "789-8");
        assert_eq!(codings[1].system, "http://snomed.info/sct");
        assert_eq!(codings[1].code, "26453005");
    }

    #[test]
    fn test_function_metadata() {
        let registry = FunctionRegistry::new();
        registry.register_terminology_functions().unwrap();

        // Test that all terminology functions are registered with correct metadata
        let functions = registry.list_functions_by_category(FunctionCategory::Terminology);
        let function_names: Vec<&str> = functions.iter().map(|f| f.name.as_str()).collect();

        assert!(function_names.contains(&"memberOf"));
        assert!(function_names.contains(&"translate"));
        assert!(function_names.contains(&"validateCode"));
        assert!(function_names.contains(&"subsumes"));
        assert!(function_names.contains(&"designation"));
        assert!(function_names.contains(&"property"));

        // Check that functions are marked as async
        for func in functions {
            assert!(
                func.is_async,
                "Terminology function {} should be async",
                func.name
            );
        }
    }

    #[test]
    fn test_coding_builder_methods() {
        let coding = Coding::new("http://loinc.org", "789-8")
            .with_display("Red blood cell count")
            .with_version("2.74");

        assert_eq!(coding.system, "http://loinc.org");
        assert_eq!(coding.code, "789-8");
        assert_eq!(coding.display.as_ref().unwrap(), "Red blood cell count");
        assert_eq!(coding.version.as_ref().unwrap(), "2.74");
    }

    #[test]
    fn test_coding_to_fhir_value() {
        let coding = Coding::new("http://loinc.org", "789-8").with_display("Red blood cell count");

        let fhir_value = TerminologyUtils::coding_to_value(&coding);

        if let FhirPathValue::Resource(json_val) = fhir_value {
            let obj = json_val.as_object().unwrap();
            assert_eq!(
                obj.get("system").unwrap().as_str().unwrap(),
                "http://loinc.org"
            );
            assert_eq!(obj.get("code").unwrap().as_str().unwrap(), "789-8");
            assert_eq!(
                obj.get("display").unwrap().as_str().unwrap(),
                "Red blood cell count"
            );
        } else {
            panic!("Expected Resource value");
        }
    }
}
