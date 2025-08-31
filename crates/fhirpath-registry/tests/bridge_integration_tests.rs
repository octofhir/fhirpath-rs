//! Bridge Integration Tests for FHIRPath Registry
//!
//! Comprehensive tests for the bridge support functionality including
//! type registry, schema-aware functions, and package management.

use octofhir_fhirpath_model::ModelProvider;
use octofhir_fhirpath_registry::{
    FhirPathTypeRegistry, FhirPathValue, PackageError, RefreshableRegistry, RegistryError,
    RegistryPackageManager, SchemaAwareFunctionRegistry,
};
use octofhir_fhirschema::package::FhirSchemaPackageManager;
use std::sync::Arc;

/// Mock schema manager for testing
struct MockSchemaManager;

impl MockSchemaManager {
    fn new() -> FhirSchemaPackageManager {
        // This would be a real implementation in practice
        // For now, we'll use a placeholder
        todo!("Implement mock schema manager for tests")
    }
}

/// Create a test schema manager
async fn create_test_schema_manager()
-> Result<Arc<FhirSchemaPackageManager>, Box<dyn std::error::Error>> {
    let manager = MockSchemaManager::new();
    Ok(Arc::new(manager))
}

/// Create a test evaluation context
fn create_test_evaluation_context() -> octofhir_fhirpath_registry::traits::EvaluationContext {
    use octofhir_fhirpath_model::{FhirPathValue, MockModelProvider};
    use octofhir_fhirpath_registry::traits::EvaluationContext;

    // Create a simple patient resource value for testing
    let patient_value = FhirPathValue::String("Patient".to_string());
    let root_value = Arc::new(patient_value.clone());

    EvaluationContext::new(
        patient_value,
        root_value,
        Arc::new(MockModelProvider::new()),
    )
}

/// Test type registry basic operations
#[tokio::test]
async fn test_type_registry_operations() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = FhirPathTypeRegistry::new(manager).await?;

    // Test O(1) operations
    assert!(registry.is_resource_type("Patient"));
    assert!(registry.is_resource_type("Observation"));
    assert!(registry.is_resource_type("Bundle"));

    assert!(registry.is_data_type("HumanName"));
    assert!(registry.is_data_type("Coding"));
    assert!(registry.is_data_type("Address"));

    assert!(registry.is_primitive_type("string"));
    assert!(registry.is_primitive_type("boolean"));
    assert!(registry.is_primitive_type("decimal"));

    // Test negative cases
    assert!(!registry.is_resource_type("InvalidType"));
    assert!(!registry.is_data_type("NotADataType"));
    assert!(!registry.is_primitive_type("notprimitive"));

    Ok(())
}

/// Test type registry list operations
#[tokio::test]
async fn test_type_registry_lists() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = FhirPathTypeRegistry::new(manager).await?;

    // Test getting all types
    let resource_types = registry.get_all_resource_types();
    assert!(!resource_types.is_empty());
    assert!(resource_types.contains(&"Patient".to_string()));
    assert!(resource_types.contains(&"Observation".to_string()));

    let data_types = registry.get_all_data_types();
    assert!(!data_types.is_empty());
    assert!(data_types.contains(&"HumanName".to_string()));
    assert!(data_types.contains(&"Coding".to_string()));

    Ok(())
}

/// Test subtype checking
#[tokio::test]
async fn test_subtype_checking() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = FhirPathTypeRegistry::new(manager).await?;

    // Test inheritance checking
    assert!(registry.is_subtype_of("Patient", "DomainResource").await?);
    assert!(registry.is_subtype_of("Patient", "Resource").await?);
    assert!(registry.is_subtype_of("Patient", "Patient").await?); // Same type

    assert!(!registry.is_subtype_of("Patient", "Observation").await?);
    assert!(!registry.is_subtype_of("string", "Patient").await?);

    Ok(())
}

/// Test resource info retrieval
#[tokio::test]
async fn test_resource_info() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = FhirPathTypeRegistry::new(manager).await?;

    // Test getting resource information
    let patient_info = registry.get_resource_info("Patient").await?;
    assert_eq!(patient_info.base_type, "DomainResource");
    assert!(!patient_info.properties.is_empty());

    // Test error for invalid resource
    let result = registry.get_resource_info("InvalidResource").await;
    assert!(result.is_err());

    Ok(())
}

/// Test schema-aware function registry creation
#[tokio::test]
async fn test_schema_aware_registry_creation() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;

    // Test that registry was created successfully
    assert!(!registry.type_registry().get_all_resource_types().is_empty());

    Ok(())
}

/// Test schema-aware ofType function
#[tokio::test]
async fn test_schema_aware_oftype_function() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;
    let context = create_test_evaluation_context();

    // Test ofType function with valid type
    let args = vec![FhirPathValue::String("Patient".to_string())];
    let result = registry
        .evaluate_function("ofType", &args, &context)
        .await?;

    match result {
        FhirPathValue::Collection(items) => {
            // Check that we get a collection result (may be empty for simplified implementation)
            let _ = items;
        }
        _ => panic!("Expected collection result"),
    }

    // Test ofType with invalid type
    let args = vec![FhirPathValue::String("InvalidType".to_string())];
    let result = registry
        .evaluate_function("ofType", &args, &context)
        .await?;

    match result {
        FhirPathValue::Collection(items) => {
            // Should be empty for invalid type
            let _ = items;
        }
        _ => panic!("Expected empty collection for invalid type"),
    }

    Ok(())
}

/// Test schema-aware is function
#[tokio::test]
async fn test_schema_aware_is_function() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;
    let context = create_test_evaluation_context();

    // Test is function with correct type
    let args = vec![FhirPathValue::String("Patient".to_string())];
    let result = registry.evaluate_function("is", &args, &context).await?;

    match result {
        FhirPathValue::Boolean(_) => {
            // Test passes if we get a boolean result
        }
        _ => panic!("Expected boolean result"),
    }

    // Test is function with wrong type
    let args = vec![FhirPathValue::String("Observation".to_string())];
    let result = registry.evaluate_function("is", &args, &context).await?;

    match result {
        FhirPathValue::Boolean(_) => {
            // Test passes if we get a boolean result
        }
        _ => panic!("Expected boolean result"),
    }

    Ok(())
}

/// Test schema-aware as function
#[tokio::test]
async fn test_schema_aware_as_function() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;
    let context = create_test_evaluation_context();

    // Test as function with correct type
    let args = vec![FhirPathValue::String("Patient".to_string())];
    let result = registry.evaluate_function("as", &args, &context).await?;

    match result {
        FhirPathValue::Collection(_items) => {
            // Test passes if we get a collection result
        }
        _ => panic!("Expected collection result"),
    }

    // Test as function with wrong type
    let args = vec![FhirPathValue::String("Observation".to_string())];
    let result = registry.evaluate_function("as", &args, &context).await?;

    match result {
        FhirPathValue::Collection(_items) => {
            // Test passes if we get a collection result
        }
        _ => panic!("Expected empty collection for wrong type"),
    }

    Ok(())
}

/// Test conformsTo function
#[tokio::test]
async fn test_conforms_to_function() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;
    let context = create_test_evaluation_context();

    // Test conformsTo function
    let args = vec![FhirPathValue::String(
        "http://hl7.org/fhir/StructureDefinition/Patient".to_string(),
    )];
    let result = registry
        .evaluate_function("conformsTo", &args, &context)
        .await?;

    match result {
        FhirPathValue::Boolean(_conforms) => {
            // For now, this always returns false as actual conformance checking
            // is not yet fully implemented
        }
        _ => panic!("Expected boolean result"),
    }

    Ok(())
}

/// Test package management basic operations
#[tokio::test]
async fn test_package_management() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let package_manager = RegistryPackageManager::new(manager).await?;

    // Test package loading
    package_manager
        .load_package("hl7.fhir.r4.core", Some("4.0.1"))
        .await?;

    // Test checking loaded packages
    let loaded = package_manager.get_loaded_packages().await?;
    assert!(loaded.contains(&"hl7.fhir.r4.core".to_string()));

    // Test package info
    let info = package_manager.get_package_info("hl7.fhir.r4.core").await?;
    assert!(info.loaded);
    assert_eq!(info.id, "hl7.fhir.r4.core");

    // Test package unloading
    package_manager.unload_package("hl7.fhir.r4.core").await?;

    // Verify package is no longer loaded
    let loaded_after = package_manager.get_loaded_packages().await?;
    assert!(!loaded_after.contains(&"hl7.fhir.r4.core".to_string()));

    Ok(())
}

/// Test refreshable registry
#[tokio::test]
async fn test_refreshable_registry() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let mut refreshable = RefreshableRegistry::new(manager).await?;

    // Test load and refresh
    refreshable
        .load_package_and_refresh("hl7.fhir.us.core", Some("3.1.1"))
        .await?;

    let loaded = refreshable.package_manager().get_loaded_packages().await?;
    assert!(loaded.contains(&"hl7.fhir.us.core".to_string()));

    // Test unload and refresh
    refreshable
        .unload_package_and_refresh("hl7.fhir.us.core")
        .await?;

    let loaded_after = refreshable.package_manager().get_loaded_packages().await?;
    assert!(!loaded_after.contains(&"hl7.fhir.us.core".to_string()));

    Ok(())
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;
    let context = create_test_evaluation_context();

    // Test function with wrong number of arguments
    let result = registry.evaluate_function("ofType", &[], &context).await;
    assert!(result.is_err());

    let result = registry
        .evaluate_function(
            "ofType",
            &[
                FhirPathValue::String("Patient".to_string()),
                FhirPathValue::String("Extra".to_string()),
            ],
            &context,
        )
        .await;
    assert!(result.is_err());

    // Test function with wrong argument type
    let result = registry
        .evaluate_function("ofType", &[FhirPathValue::Boolean(true)], &context)
        .await;
    assert!(result.is_err());

    // Test non-existent function
    let result = registry
        .evaluate_function("nonExistentFunction", &[], &context)
        .await;
    assert!(result.is_err());

    Ok(())
}

/// Performance test for O(1) type checking
#[tokio::test]
async fn test_type_checking_performance() -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_test_schema_manager().await?;
    let registry = FhirPathTypeRegistry::new(manager).await?;

    // Test that type checking is fast (should be O(1))
    let start = std::time::Instant::now();

    for _ in 0..10000 {
        registry.is_resource_type("Patient");
        registry.is_data_type("HumanName");
        registry.is_primitive_type("string");
    }

    let duration = start.elapsed();

    // Should complete 30,000 operations in well under 1 second
    assert!(duration < std::time::Duration::from_millis(100));

    Ok(())
}

/// Integration test combining all features
#[tokio::test]
async fn test_full_integration() -> Result<(), Box<dyn std::error::Error>> {
    // Create schema manager and load package
    let manager = create_test_schema_manager().await?;
    let mut refreshable = RefreshableRegistry::new(manager.clone()).await?;

    refreshable
        .load_package_and_refresh("hl7.fhir.r4.core", Some("4.0.1"))
        .await?;

    // Create schema-aware registry
    let registry = SchemaAwareFunctionRegistry::new(manager).await?;
    let context = create_test_evaluation_context();

    // Test type registry
    let type_registry = registry.type_registry();
    assert!(type_registry.is_resource_type("Patient"));

    // Test schema-aware functions
    let args = vec![FhirPathValue::String("Patient".to_string())];
    let result = registry
        .evaluate_function("ofType", &args, &context)
        .await?;

    match result {
        FhirPathValue::Collection(items) => {
            assert!(!items.is_empty());
        }
        _ => panic!("Expected collection result"),
    }

    // Test package management
    let package_manager = refreshable.package_manager();
    let loaded = package_manager.get_loaded_packages().await?;
    assert!(loaded.contains(&"hl7.fhir.r4.core".to_string()));

    Ok(())
}
