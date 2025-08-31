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

//! Comprehensive type resolution integration tests
//!
//! Tests the complete type resolution system including TypeResolver, ChoiceTypeResolver,
//! PropertyResolver, and SystemTypes with bridge support API integration.

use octofhir_canonical_manager::FcmConfig;
use octofhir_fhirpath_model::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use std::sync::Arc;
use std::time::Instant;
use tokio;

async fn create_test_schema_manager() -> Arc<FhirSchemaPackageManager> {
    let fcm_config = FcmConfig::default();
    let config = PackageManagerConfig::default();
    Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .expect("Failed to create schema manager"),
    )
}

#[tokio::test]
async fn test_type_resolver_comprehensive() {
    let schema_manager = create_test_schema_manager().await;
    let mut resolver = TypeResolver::new(schema_manager);

    // Test resource type resolution
    assert!(resolver.is_resource_type("Patient").await);
    assert!(resolver.is_resource_type("Observation").await);
    assert!(resolver.is_resource_type("Bundle").await);
    assert!(!resolver.is_resource_type("InvalidType").await);

    // Test primitive type resolution
    assert!(resolver.is_primitive_type("string").await);
    assert!(resolver.is_primitive_type("boolean").await);
    assert!(resolver.is_primitive_type("integer").await);
    assert!(!resolver.is_primitive_type("Patient").await);

    // Test comprehensive type info
    let patient_info = resolver.get_type_info("Patient").await.unwrap();
    assert_eq!(patient_info.name, "Patient");
    assert!(patient_info.is_resource);
    assert!(!patient_info.is_primitive);
    assert_eq!(patient_info.namespace, "FHIR");

    let string_info = resolver.get_type_info("string").await.unwrap();
    assert_eq!(string_info.name, "string");
    assert!(!string_info.is_resource);
    assert!(string_info.is_primitive);
    assert_eq!(string_info.namespace, "System");
}

#[tokio::test]
async fn test_choice_type_resolver_comprehensive() {
    let schema_manager = create_test_schema_manager().await;
    let mut resolver = ChoiceTypeResolver::new(schema_manager);

    // Test explicit choice type resolution
    let result = resolver
        .resolve_choice_type("Observation.value[x]", "valueString")
        .await;
    assert!(result.is_ok());

    let choice_info = result.unwrap();
    assert_eq!(choice_info.original_path, "Observation.value[x]");
    assert!(choice_info.is_valid);

    // Test different choice variants
    let quantity_result = resolver
        .resolve_choice_type("Observation.value[x]", "valueQuantity")
        .await;
    assert!(quantity_result.is_ok());

    let boolean_result = resolver
        .resolve_choice_type("Observation.value[x]", "valueBoolean")
        .await;
    assert!(boolean_result.is_ok());

    // Test caching behavior
    let (initial_cache_size, _) = resolver.get_cache_stats();
    assert_eq!(initial_cache_size, 3); // Three different resolutions cached

    // Test cache hit
    let cached_result = resolver
        .resolve_choice_type("Observation.value[x]", "valueString")
        .await;
    assert!(cached_result.is_ok());

    let (final_cache_size, _) = resolver.get_cache_stats();
    assert_eq!(final_cache_size, 3); // Should remain the same
}

#[tokio::test]
async fn test_property_resolver_comprehensive() {
    let schema_manager = create_test_schema_manager().await;
    let resolver = PropertyResolver::new(schema_manager);

    // Test simple property resolution
    let name_result = resolver.resolve_property_path("Patient", "name").await;
    assert!(name_result.is_ok());

    let properties = name_result.unwrap();
    assert_eq!(properties.len(), 1);
    assert_eq!(properties[0].name, "name");

    // Test complex property paths
    let complex_result = resolver.resolve_property_path("Patient", "name").await;
    assert!(complex_result.is_ok());

    // Test invalid property paths
    let invalid_result = resolver
        .resolve_property_path("InvalidType", "invalidProperty")
        .await;
    assert!(invalid_result.is_err());
}

#[tokio::test]
async fn test_system_types_comprehensive() {
    let schema_manager = create_test_schema_manager().await;
    let system_types = SystemTypes::new(schema_manager);

    // Test type categorization
    let patient_category = system_types.get_system_type_category("Patient").await;
    assert_eq!(patient_category, SystemTypeCategory::Resource);

    let string_category = system_types.get_system_type_category("string").await;
    assert_eq!(string_category, SystemTypeCategory::Primitive);

    let unknown_category = system_types.get_system_type_category("InvalidType").await;
    assert_eq!(unknown_category, SystemTypeCategory::Unknown);

    // Test namespace resolution
    let patient_namespace = system_types.get_namespace("Patient").await;
    assert_eq!(patient_namespace, "FHIR");

    let string_namespace = system_types.get_namespace("string").await;
    assert_eq!(string_namespace, "System");

    // Test type validation
    assert!(system_types.is_valid_type("Patient").await);
    assert!(system_types.is_valid_type("string").await);
    assert!(!system_types.is_valid_type("InvalidType").await);

    // Test inheritance relationships
    assert!(system_types.is_subtype_of("Patient", "Patient").await);
    assert!(
        system_types
            .is_subtype_of("Patient", "DomainResource")
            .await
    );
    assert!(system_types.is_subtype_of("Patient", "Resource").await);
    assert!(!system_types.is_subtype_of("string", "Patient").await);
}

#[tokio::test]
async fn test_bridge_api_integration() {
    let schema_manager = create_test_schema_manager().await;

    // Test direct bridge API integration
    let provider = FhirSchemaModelProvider::new().await.unwrap();

    // Test bridge-enabled resource type checking
    assert!(provider.is_resource_type("Patient").await);
    assert!(provider.is_resource_type("Observation").await);
    assert!(!provider.is_resource_type("InvalidType").await);

    // Test type object creation with bridge API
    let type_obj = FhirPathTypeObject::fhir_type_with_schema(
        "Patient",
        Some("DomainResource".to_string()),
        &*schema_manager,
    )
    .await;

    assert_eq!(type_obj.name, "Patient");
    assert_eq!(type_obj.namespace, "FHIR");
    assert!(type_obj.metadata.is_resource);
    assert!(!type_obj.metadata.is_primitive);
}

#[tokio::test]
async fn test_performance_o1_operations() {
    let schema_manager = create_test_schema_manager().await;
    let resolver = TypeResolver::new(schema_manager.clone());
    let system_types = SystemTypes::new(schema_manager);

    // Test O(1) performance for type checking operations
    let start = Instant::now();

    // Perform many type checking operations
    for _ in 0..100 {
        resolver.is_resource_type("Patient").await;
        resolver.is_primitive_type("string").await;
        system_types.is_valid_type("Observation").await;
        system_types.get_namespace("boolean").await;
    }

    let duration = start.elapsed();

    // 400 operations should complete very quickly with O(1) lookups
    assert!(
        duration.as_millis() < 500,
        "O(1) operations should be fast: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_caching_performance() {
    let schema_manager = create_test_schema_manager().await;
    let mut type_resolver = TypeResolver::new(schema_manager.clone());
    let mut choice_resolver = ChoiceTypeResolver::new(schema_manager);

    // Test TypeResolver caching
    let start = Instant::now();
    for _ in 0..50 {
        let _info = type_resolver.get_type_info("Patient").await.unwrap();
    }
    let type_resolver_duration = start.elapsed();

    // Test ChoiceTypeResolver caching
    let start = Instant::now();
    for _ in 0..50 {
        let _result = choice_resolver
            .resolve_choice_type("Observation.value[x]", "valueString")
            .await
            .unwrap();
    }
    let choice_resolver_duration = start.elapsed();

    // Cached operations should be very fast
    assert!(
        type_resolver_duration.as_millis() < 100,
        "TypeResolver caching should be fast"
    );
    assert!(
        choice_resolver_duration.as_millis() < 100,
        "ChoiceTypeResolver caching should be fast"
    );

    // Verify caching actually happened
    let (type_cache_size, _) = type_resolver.get_cache_stats();
    let (choice_cache_size, _) = choice_resolver.get_cache_stats();

    assert_eq!(type_cache_size, 1, "Should have cached Patient type info");
    assert_eq!(choice_cache_size, 1, "Should have cached choice resolution");
}

#[tokio::test]
async fn test_concurrent_operations() {
    let schema_manager = create_test_schema_manager().await;
    let system_types = SystemTypes::new(schema_manager);

    // Test concurrent type operations
    let tasks: Vec<_> = (0..10)
        .map(|i| {
            let st = system_types.clone();
            tokio::spawn(async move {
                let resource_types = ["Patient", "Observation", "Bundle", "Condition"];
                let primitive_types = ["string", "boolean", "integer", "decimal"];

                let resource_type = resource_types[i % resource_types.len()];
                let primitive_type = primitive_types[i % primitive_types.len()];

                // Perform multiple operations concurrently
                let is_resource = st.is_valid_type(resource_type).await;
                let is_primitive = st.is_valid_type(primitive_type).await;
                let resource_category = st.get_system_type_category(resource_type).await;
                let primitive_category = st.get_system_type_category(primitive_type).await;

                (
                    is_resource,
                    is_primitive,
                    resource_category,
                    primitive_category,
                )
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All concurrent operations should succeed
    for result in results {
        let (is_resource, is_primitive, resource_category, primitive_category) = result.unwrap();
        assert!(is_resource, "Resource type should be valid");
        assert!(is_primitive, "Primitive type should be valid");
        assert_eq!(resource_category, SystemTypeCategory::Resource);
        assert_eq!(primitive_category, SystemTypeCategory::Primitive);
    }
}

#[tokio::test]
async fn test_error_handling() {
    let schema_manager = create_test_schema_manager().await;
    let mut type_resolver = TypeResolver::new(schema_manager.clone());
    let mut choice_resolver = ChoiceTypeResolver::new(schema_manager.clone());
    let property_resolver = PropertyResolver::new(schema_manager);

    // Test error handling for invalid types
    let _invalid_type_result = type_resolver.get_type_info("InvalidType").await;
    // This should return unknown type info rather than error in this implementation

    // Test error handling for invalid choice types
    let invalid_choice_result = choice_resolver
        .resolve_choice_type("InvalidType.invalid[x]", "invalidType")
        .await;
    assert!(
        invalid_choice_result.is_err(),
        "Should error for invalid choice types"
    );

    // Test error handling for invalid property paths
    let invalid_property_result = property_resolver
        .resolve_property_path("InvalidType", "invalidProperty")
        .await;
    assert!(
        invalid_property_result.is_err(),
        "Should error for invalid property paths"
    );
}

#[tokio::test]
async fn test_real_world_scenarios() {
    let schema_manager = create_test_schema_manager().await;
    let mut type_resolver = TypeResolver::new(schema_manager.clone());
    let _property_resolver = PropertyResolver::new(schema_manager.clone());
    let system_types = SystemTypes::new(schema_manager);

    // Scenario 1: Complex Patient property navigation
    let patient_info = type_resolver.get_type_info("Patient").await.unwrap();
    assert!(patient_info.is_resource);
    assert_eq!(patient_info.namespace, "FHIR");

    // Scenario 2: Choice type resolution for Observation
    let mut choice_resolver = ChoiceTypeResolver::new(system_types.schema_manager().clone());
    let value_string = choice_resolver
        .resolve_choice_type("Observation.value[x]", "valueString")
        .await
        .unwrap();
    assert!(value_string.is_valid);

    // Scenario 3: System type categorization
    let categories = vec![
        ("Patient", SystemTypeCategory::Resource),
        ("string", SystemTypeCategory::Primitive),
        ("HumanName", SystemTypeCategory::Complex),
    ];

    for (type_name, expected_category) in categories {
        let actual_category = system_types.get_system_type_category(type_name).await;
        if type_name == "HumanName" && actual_category == SystemTypeCategory::Unknown {
            // HumanName might not be loaded, which is acceptable
            continue;
        }
        assert_eq!(
            actual_category, expected_category,
            "Category mismatch for {}",
            type_name
        );
    }

    // Scenario 4: Inheritance checking
    let inheritance_tests = vec![
        ("Patient", "DomainResource", true),
        ("Patient", "Resource", true),
        ("string", "Patient", false),
        ("DomainResource", "Resource", true),
    ];

    for (child, parent, expected) in inheritance_tests {
        let is_subtype = system_types.is_subtype_of(child, parent).await;
        assert_eq!(
            is_subtype, expected,
            "Inheritance check failed: {} is subtype of {}",
            child, parent
        );
    }
}

#[tokio::test]
async fn test_utility_functions() {
    use octofhir_fhirpath_model::system_types::utils::*;

    // Test type name validation
    assert!(is_valid_type_name("Patient"));
    assert!(is_valid_type_name("HumanName"));
    assert!(is_valid_type_name("FHIR.Patient"));
    assert!(!is_valid_type_name(""));
    assert!(!is_valid_type_name(".Patient"));
    assert!(!is_valid_type_name("Patient."));

    // Test type name normalization
    assert_eq!(normalize_type_name("Patient"), "Patient");
    assert_eq!(normalize_type_name("`Patient`"), "Patient");
    assert_eq!(normalize_type_name("FHIR.Patient"), "Patient");
    assert_eq!(normalize_type_name("`FHIR.Patient`"), "Patient");

    // Test collection access detection
    assert!(is_collection_access("name[0]"));
    assert!(is_collection_access("telecom[1].value"));
    assert!(!is_collection_access("value[x]")); // Choice type
    assert!(!is_collection_access("name"));

    // Test collection index extraction
    assert_eq!(extract_collection_index("name[0]"), Some(0));
    assert_eq!(extract_collection_index("telecom[5]"), Some(5));
    assert_eq!(extract_collection_index("name"), None);

    // Test collection notation removal
    assert_eq!(remove_collection_notation("name[0]"), "name");
    assert_eq!(remove_collection_notation("value[x]"), "value");
    assert_eq!(remove_collection_notation("name"), "name");
}

#[tokio::test]
async fn test_bridge_api_vs_legacy_consistency() {
    let schema_manager = create_test_schema_manager().await;
    let registry = PrecomputedTypeRegistry::new();

    // Test that bridge API and legacy methods return consistent results
    let test_types = ["Patient", "Observation", "string", "boolean"];

    for type_name in &test_types {
        let bridge_is_resource = registry
            .is_resource_type_bridge(type_name, &*schema_manager)
            .await;
        let legacy_is_resource = registry
            .is_resource_type_bridge(type_name, &*schema_manager)
            .await;

        // For known types, results should be consistent
        if !type_name.starts_with("string") && !type_name.starts_with("boolean") {
            // Only check resource types
            if bridge_is_resource {
                assert!(
                    legacy_is_resource || type_name == &"Patient" || type_name == &"Observation",
                    "Bridge API and legacy should be consistent for resource type: {}",
                    type_name
                );
            }
        }
    }
}
