//! Comprehensive tests for FHIRPath type functions
//!
//! Tests cover type checking, casting, filtering, and all type system functionality
//! including proper FHIR type hierarchy and subtype relationships.

#[cfg(test)]
mod tests {
    use super::super::types::{FhirPathType, TypeChecker};
    use super::super::type_utils::TypeUtils;
    use crate::core::FhirPathValue;
    use crate::registry::{FunctionRegistry, FunctionContext};
    use crate::{MockModelProvider, FunctionDispatcher};
    use std::collections::HashMap;
    use serde_json::json;

    fn create_test_context_with_globals<'a>(
        input: &'a [FhirPathValue],
        arguments: &'a [FhirPathValue],
    ) -> FunctionContext<'a> {
        use std::sync::OnceLock;
        
        static MODEL_PROVIDER: OnceLock<MockModelProvider> = OnceLock::new();
        static VARIABLES: OnceLock<HashMap<String, FhirPathValue>> = OnceLock::new();
        
        let mp = MODEL_PROVIDER.get_or_init(|| MockModelProvider::default());
        let vars = VARIABLES.get_or_init(|| HashMap::new());
        
        FunctionContext {
            input,
            arguments,
            model_provider: mp,
            variables: vars,
            resource_context: None,
            terminology: None,
        }
    }
    
    macro_rules! create_test_context {
        ($input:expr, $args:expr) => {
            create_test_context_with_globals($input, $args)
        };
    }

    fn create_test_registry() -> FunctionRegistry {
        let registry = FunctionRegistry::new();
        registry.register_type_functions().unwrap();
        registry
    }

    #[test]
    fn test_fhir_path_type_names() {
        // Test type name consistency
        assert_eq!(FhirPathType::Boolean.type_name(), "Boolean");
        assert_eq!(FhirPathType::Integer.type_name(), "Integer");
        assert_eq!(FhirPathType::String.type_name(), "String");
        assert_eq!(FhirPathType::Patient.type_name(), "Patient");
        assert_eq!(FhirPathType::Code.type_name(), "code");
        assert_eq!(FhirPathType::Uri.type_name(), "uri");
    }

    #[test]
    fn test_type_name_parsing() {
        // Test parsing from strings
        assert_eq!(FhirPathType::from_type_name("Boolean"), Some(FhirPathType::Boolean));
        assert_eq!(FhirPathType::from_type_name("Integer"), Some(FhirPathType::Integer));
        assert_eq!(FhirPathType::from_type_name("Patient"), Some(FhirPathType::Patient));
        assert_eq!(FhirPathType::from_type_name("code"), Some(FhirPathType::Code));
        assert_eq!(FhirPathType::from_type_name("NonExistent"), None);
    }

    #[test]
    fn test_subtype_relationships() {
        // Test resource hierarchy
        assert!(FhirPathType::Patient.is_subtype_of(&FhirPathType::DomainResource));
        assert!(FhirPathType::Patient.is_subtype_of(&FhirPathType::Resource));
        assert!(FhirPathType::Patient.is_subtype_of(&FhirPathType::Any));
        assert!(FhirPathType::DomainResource.is_subtype_of(&FhirPathType::Resource));

        // Test FHIR primitive types
        assert!(FhirPathType::Code.is_subtype_of(&FhirPathType::String));
        assert!(FhirPathType::Uri.is_subtype_of(&FhirPathType::String));
        assert!(FhirPathType::Instant.is_subtype_of(&FhirPathType::DateTime));

        // Test self-subtyping
        assert!(FhirPathType::Patient.is_subtype_of(&FhirPathType::Patient));

        // Test Any type  
        assert!(FhirPathType::String.is_subtype_of(&FhirPathType::Any));
        assert!(FhirPathType::Integer.is_subtype_of(&FhirPathType::Any));

        // Test negative cases
        assert!(!FhirPathType::String.is_subtype_of(&FhirPathType::Integer));
        assert!(!FhirPathType::Patient.is_subtype_of(&FhirPathType::Observation));
    }

    #[test]
    fn test_type_checker_get_type() {
        // Test primitive types
        assert_eq!(TypeChecker::get_type(&FhirPathValue::Boolean(true)), FhirPathType::Boolean);
        assert_eq!(TypeChecker::get_type(&FhirPathValue::Integer(42)), FhirPathType::Integer);
        assert_eq!(TypeChecker::get_type(&FhirPathValue::String("test".to_string())), FhirPathType::String);

        // Test FHIR resource object
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "example"
        });
        let patient = FhirPathValue::Resource(patient_json);
        assert_eq!(TypeChecker::get_type(&patient), FhirPathType::Patient);

        // Test complex object inference
        let coding_json = json!({
            "system": "http://snomed.info/sct",
            "code": "123456"
        });
        let coding = FhirPathValue::JsonValue(coding_json);
        assert_eq!(TypeChecker::get_type(&coding), FhirPathType::Coding);

        let human_name_json = json!({
            "family": "Doe",
            "given": ["John"]
        });
        let human_name = FhirPathValue::JsonValue(human_name_json);
        assert_eq!(TypeChecker::get_type(&human_name), FhirPathType::HumanName);
    }

    #[test]
    fn test_type_checker_is_type() {
        // Test direct type checking
        let value = FhirPathValue::Integer(42);
        assert!(TypeChecker::is_type(&value, &FhirPathType::Integer));
        assert!(!TypeChecker::is_type(&value, &FhirPathType::String));

        // Test subtype checking
        let patient_json = json!({"resourceType": "Patient", "id": "example"});
        let patient = FhirPathValue::Resource(patient_json);
        assert!(TypeChecker::is_type(&patient, &FhirPathType::Patient));
        assert!(TypeChecker::is_type(&patient, &FhirPathType::DomainResource));
        assert!(TypeChecker::is_type(&patient, &FhirPathType::Resource));
        assert!(TypeChecker::is_type(&patient, &FhirPathType::Any));
    }

    #[test]
    fn test_type_checker_cast_to_type() {
        // Test successful casts
        let result = TypeChecker::cast_to_type(&FhirPathValue::Integer(42), &FhirPathType::Decimal);
        assert!(result.is_ok());
        
        let result = TypeChecker::cast_to_type(&FhirPathValue::String("123".to_string()), &FhirPathType::Integer);
        assert!(result.is_ok());
        if let Ok(FhirPathValue::Integer(i)) = result {
            assert_eq!(i, 123);
        }

        let result = TypeChecker::cast_to_type(&FhirPathValue::String("true".to_string()), &FhirPathType::Boolean);
        assert!(result.is_ok());
        if let Ok(FhirPathValue::Boolean(b)) = result {
            assert_eq!(b, true);
        }

        // Test string conversion
        let result = TypeChecker::cast_to_type(&FhirPathValue::Integer(42), &FhirPathType::String);
        assert!(result.is_ok());
        if let Ok(FhirPathValue::String(s)) = result {
            assert_eq!(s, "42");
        }

        // Test failed casts
        let result = TypeChecker::cast_to_type(&FhirPathValue::String("not_a_number".to_string()), &FhirPathType::Integer);
        assert!(result.is_err());

        let result = TypeChecker::cast_to_type(&FhirPathValue::String("invalid".to_string()), &FhirPathType::Boolean);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_function() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Test integer type checking
        let input = vec![FhirPathValue::Integer(42)];
        let arguments = vec![FhirPathValue::String("Integer".to_string())];
        let context = create_test_context!(&input, &arguments);
        
        let result = dispatcher.dispatch_sync("is", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test wrong type
        let arguments = vec![FhirPathValue::String("String".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("is", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(false));

        // Test subtype checking with Patient
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "example"
        });
        let patient = FhirPathValue::Resource(patient_json);
        let input = vec![patient];
        let arguments = vec![FhirPathValue::String("Resource".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("is", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));

        // Test DomainResource subtype
        let arguments = vec![FhirPathValue::String("DomainResource".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("is", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Boolean(true));
    }

    #[test]
    fn test_is_function_errors() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Test error on multiple values
        let input = vec![FhirPathValue::Integer(1), FhirPathValue::Integer(2)];
        let arguments = vec![FhirPathValue::String("Integer".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("is", &context);
        assert!(result.is_err());

        // Test error on no arguments
        let input = vec![FhirPathValue::Integer(42)];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("is", &context);
        assert!(result.is_err());

        // Test error on invalid type name
        let arguments = vec![FhirPathValue::String("NonExistentType".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("is", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_as_function() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Test successful cast
        let input = vec![FhirPathValue::String("123".to_string())];
        let arguments = vec![FhirPathValue::String("Integer".to_string())];
        let context = create_test_context!(&input, &arguments);
        
        let result = dispatcher.dispatch_sync("as", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(123));

        // Test failed cast (should return empty)
        let input = vec![FhirPathValue::String("not_a_number".to_string())];
        let arguments = vec![FhirPathValue::String("Integer".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("as", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test subtype preservation
        let patient_json = json!({"resourceType": "Patient", "id": "example"});
        let patient = FhirPathValue::Resource(patient_json);
        let input = vec![patient.clone()];
        let arguments = vec![FhirPathValue::String("Resource".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("as", &context).unwrap();
        assert_eq!(result[0], patient);
    }

    #[test]
    fn test_oftype_function() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Test filtering mixed collection by type
        let input = vec![
            FhirPathValue::String("hello".to_string()),
            FhirPathValue::Integer(42),
            FhirPathValue::Boolean(true),
            FhirPathValue::String("world".to_string()),
            FhirPathValue::Decimal(rust_decimal::Decimal::new(314, 2)),
        ];
        let arguments = vec![FhirPathValue::String("String".to_string())];
        let context = create_test_context!(&input, &arguments);
        
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], FhirPathValue::String("hello".to_string()));
        assert_eq!(result[1], FhirPathValue::String("world".to_string()));

        // Test with Integer type
        let arguments = vec![FhirPathValue::String("Integer".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], FhirPathValue::Integer(42));

        // Test empty result
        let arguments = vec![FhirPathValue::String("Patient".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_oftype_function_with_subtypes() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Create a collection with different resource types
        let patient_json = json!({"resourceType": "Patient", "id": "p1"});
        let obs_json = json!({"resourceType": "Observation", "id": "o1"});
        let bundle_json = json!({"resourceType": "Bundle", "id": "b1"});

        let input = vec![
            FhirPathValue::Resource(patient_json),
            FhirPathValue::Resource(obs_json),
            FhirPathValue::Resource(bundle_json),
            FhirPathValue::String("not_a_resource".to_string()),
        ];

        // Filter by Resource type (should get all resources)
        let arguments = vec![FhirPathValue::String("Resource".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 3); // Patient, Observation, Bundle are all Resources

        // Filter by DomainResource (should get Patient and Observation)
        let arguments = vec![FhirPathValue::String("DomainResource".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 2); // Patient and Observation are DomainResources

        // Filter by specific type
        let arguments = vec![FhirPathValue::String("Patient".to_string())];
        let context = create_test_context!(&input, &arguments);
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_type_function() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Test getting type of integer
        let input = vec![FhirPathValue::Integer(42)];
        let context = create_test_context!(&input, &[]);
        
        let result = dispatcher.dispatch_sync("type", &context).unwrap();
        if let FhirPathValue::JsonValue(type_info) = &result[0] {
            assert_eq!(type_info["namespace"], "System");
            assert_eq!(type_info["name"], "Integer");
        } else {
            panic!("Expected JsonValue with TypeInfo structure");
        }

        // Test getting type of string
        let input = vec![FhirPathValue::String("hello".to_string())];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("type", &context).unwrap();
        if let FhirPathValue::JsonValue(type_info) = &result[0] {
            assert_eq!(type_info["namespace"], "System");
            assert_eq!(type_info["name"], "String");
        } else {
            panic!("Expected JsonValue with TypeInfo structure");
        }

        // Test getting type of FHIR resource
        let patient_json = json!({"resourceType": "Patient", "id": "example"});
        let patient = FhirPathValue::Resource(patient_json);
        let input = vec![patient];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("type", &context).unwrap();
        if let FhirPathValue::JsonValue(type_info) = &result[0] {
            assert_eq!(type_info["namespace"], "FHIR");
            assert_eq!(type_info["name"], "Patient");
        } else {
            panic!("Expected JsonValue with TypeInfo structure");
        }

        // Test getting type of complex object
        let coding_json = json!({"system": "http://example.com", "code": "test"});
        let coding = FhirPathValue::JsonValue(coding_json);
        let input = vec![coding];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("type", &context).unwrap();
        if let FhirPathValue::JsonValue(type_info) = &result[0] {
            assert_eq!(type_info["namespace"], "FHIR");
            assert_eq!(type_info["name"], "Coding");
        } else {
            panic!("Expected JsonValue with TypeInfo structure");
        }
    }

    #[test]
    fn test_type_utilities() {
        // Test compatible types
        let patient_json = json!({"resourceType": "Patient", "id": "example"});
        let patient = FhirPathValue::Resource(patient_json);
        let compatible_types = TypeUtils::get_compatible_types(&patient);
        
        assert!(compatible_types.contains(&FhirPathType::Patient));
        assert!(compatible_types.contains(&FhirPathType::DomainResource));
        assert!(compatible_types.contains(&FhirPathType::Resource));
        assert!(compatible_types.contains(&FhirPathType::Any));

        // Test common type finding
        let values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ];
        let common_type = TypeUtils::find_common_type(&values);
        assert_eq!(common_type, FhirPathType::Integer);

        let mixed_values = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::String("test".to_string()),
        ];
        let common_type = TypeUtils::find_common_type(&mixed_values);
        assert_eq!(common_type, FhirPathType::Any);

        // Test safe conversions
        assert!(TypeUtils::is_safe_conversion(&FhirPathType::Integer, &FhirPathType::Decimal));
        assert!(TypeUtils::is_safe_conversion(&FhirPathType::Integer, &FhirPathType::String));
        assert!(!TypeUtils::is_safe_conversion(&FhirPathType::String, &FhirPathType::Integer));

        // Test type categories
        assert!(TypeUtils::is_primitive_type(&FhirPathType::Integer));
        assert!(TypeUtils::is_fhir_primitive_type(&FhirPathType::Code));
        assert!(TypeUtils::is_complex_type(&FhirPathType::HumanName));
        assert!(TypeUtils::is_resource_type(&FhirPathType::Patient));
    }

    #[test]
    fn test_type_name_validation() {
        assert!(TypeUtils::is_valid_type_name("Integer"));
        assert!(TypeUtils::is_valid_type_name("Patient"));
        assert!(TypeUtils::is_valid_type_name("code"));
        assert!(!TypeUtils::is_valid_type_name("NonExistentType"));
        assert!(!TypeUtils::is_valid_type_name(""));
    }

    #[test]
    fn test_type_compatibility() {
        // Test numeric compatibility
        assert!(TypeUtils::are_comparable(&FhirPathType::Integer, &FhirPathType::Decimal));
        assert!(TypeUtils::are_comparable(&FhirPathType::Integer, &FhirPathType::Quantity));
        
        // Test date/time compatibility
        assert!(TypeUtils::are_comparable(&FhirPathType::Date, &FhirPathType::DateTime));
        assert!(TypeUtils::are_comparable(&FhirPathType::DateTime, &FhirPathType::Instant));
        
        // Test subtype compatibility
        assert!(TypeUtils::are_comparable(&FhirPathType::Patient, &FhirPathType::Resource));
        assert!(TypeUtils::are_comparable(&FhirPathType::Code, &FhirPathType::String));
        
        // Test incompatible types
        assert!(!TypeUtils::are_comparable(&FhirPathType::Integer, &FhirPathType::String));
        assert!(!TypeUtils::are_comparable(&FhirPathType::Patient, &FhirPathType::Observation));
    }

    #[test]
    fn test_unicode_support() {
        // Test type checking with Unicode strings
        let unicode_string = FhirPathValue::String("Héllo Wörld 🌍".to_string());
        assert_eq!(TypeChecker::get_type(&unicode_string), FhirPathType::String);
        
        // Test casting with Unicode
        let result = TypeChecker::cast_to_type(&unicode_string, &FhirPathType::String);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_cases() {
        let registry = create_test_registry();
        let dispatcher = FunctionDispatcher::new(registry);

        // Test with empty collection for ofType
        let arguments = [FhirPathValue::String("Integer".to_string())];
        let context = create_test_context!(&[], &arguments);
        let result = dispatcher.dispatch_sync("ofType", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test type() with quantity
        let quantity = FhirPathValue::Quantity { 
            value: rust_decimal::Decimal::from(42), 
            unit: Some("kg".to_string()),
            ucum_unit: None,
            calendar_unit: None,
        };
        let input = vec![quantity];
        let context = create_test_context!(&input, &[]);
        let result = dispatcher.dispatch_sync("type", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::String("Quantity".to_string()));
    }
}
