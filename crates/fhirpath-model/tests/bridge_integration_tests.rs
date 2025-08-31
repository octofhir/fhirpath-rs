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

//! Comprehensive bridge support integration tests

use octofhir_fhirpath_model::provider::TypeReflectionInfo;
use octofhir_fhirpath_model::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_property_navigator_integration() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let navigator = PropertyNavigator::new(manager);

    // Test O(1) resource type operations
    assert!(navigator.has_resource_type("Patient").await);
    assert!(navigator.has_resource_type("Observation").await);
    assert!(navigator.has_resource_type("Bundle").await);
    assert!(!navigator.has_resource_type("InvalidType").await);

    // Test property information retrieval
    let patient_name = navigator.get_property_info("Patient", "name").await;
    assert!(patient_name.is_ok());

    let prop_info = patient_name.unwrap();
    assert_eq!(prop_info.name, "name");

    // Test choice type resolution
    let choice_result = navigator
        .resolve_choice_type("Observation.value[x]", "valueString")
        .await;
    assert!(choice_result.is_ok());

    let choice_info = choice_result.unwrap();
    assert_eq!(choice_info.resolved_type, "valueString");
    assert!(choice_info.is_valid);
}

#[tokio::test]
async fn test_bridge_choice_type_resolver() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let mut resolver = BridgeChoiceTypeResolver::new(manager);

    // Test basic choice type resolution
    let result = resolver
        .resolve_choice_type("Observation.value[x]", "valueString")
        .await;
    assert!(result.is_ok());

    let choice_info = result.unwrap();
    assert_eq!(choice_info.resolved_type, "valueString");
    assert!(choice_info.is_valid);

    // Test caching
    let (initial_cache_size, _) = resolver.get_cache_stats();
    assert_eq!(initial_cache_size, 1);

    // Resolve same type again - should use cache
    let cached_result = resolver
        .resolve_choice_type("Observation.value[x]", "valueString")
        .await;
    assert!(cached_result.is_ok());

    let (cache_size_after, _) = resolver.get_cache_stats();
    assert_eq!(cache_size_after, 1); // Should remain same size

    // Test different choice type
    let different_result = resolver
        .resolve_choice_type("Observation.effective[x]", "effectiveDateTime")
        .await;
    assert!(different_result.is_ok());

    let (final_cache_size, _) = resolver.get_cache_stats();
    assert_eq!(final_cache_size, 2); // Should now have 2 entries
}

#[tokio::test]
async fn test_fhirschema_provider_bridge_integration() {
    let provider = FhirSchemaModelProvider::new().await.unwrap();

    // Test bridge-based resource type checking
    assert!(provider.is_resource_type("Patient").await);
    assert!(provider.is_resource_type("Observation").await);
    assert!(provider.is_resource_type("Bundle").await);
    assert!(!provider.is_resource_type("NonExistentType").await);

    // Test type reflection with bridge support
    let patient_reflection = provider.get_type_reflection("Patient").await;
    assert!(patient_reflection.is_some());

    let reflection_info = patient_reflection.unwrap();
    match reflection_info {
        TypeReflectionInfo::ClassInfo {
            name, namespace, ..
        } => {
            assert_eq!(name, "Patient");
            assert_eq!(namespace, "FHIR");
        }
        _ => panic!("Expected ClassInfo for Patient type"),
    }
}

#[tokio::test]
async fn test_performance_improvements() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let navigator = PropertyNavigator::new(manager);

    // Test O(1) operations performance
    let start = std::time::Instant::now();

    // Perform multiple resource type checks
    for _ in 0..100 {
        // Reduced for async operations
        navigator.has_resource_type("Patient").await;
        navigator.has_resource_type("Observation").await;
        navigator.has_resource_type("Bundle").await;
    }

    let duration = start.elapsed();

    // O(1) operations should complete quickly even with 300 async calls
    assert!(
        duration.as_millis() < 1000,
        "O(1) operations should be fast"
    );
}

#[tokio::test]
async fn test_schema_operations() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let navigator = PropertyNavigator::new(manager);

    // Test schema retrieval
    let patient_schema = navigator
        .get_schema("http://hl7.org/fhir/StructureDefinition/Patient")
        .await;
    assert!(patient_schema.is_ok());

    let schema = patient_schema.unwrap();
    if let Some(name) = &schema.name {
        assert_eq!(name, "Patient");
    }

    // Test schemas by type
    let patient_schemas = navigator.get_schemas_by_type("Patient").await;
    assert!(patient_schemas.is_ok());

    let schemas = patient_schemas.unwrap();
    assert!(!schemas.is_empty());

    // Test resource types enumeration
    let resource_types = navigator.get_resource_types().await;
    assert!(resource_types.is_ok());

    let types = resource_types.unwrap();
    assert!(types.contains(&"Patient".to_string()));
    assert!(types.contains(&"Observation".to_string()));
}

#[tokio::test]
async fn test_choice_type_utils() {
    use octofhir_fhirpath_model::choice_type_bridge::utils::*;

    // Test utility functions
    assert_eq!(extract_base_property("value[x]"), "value");
    assert_eq!(extract_base_property("effective[x]"), "effective");
    assert_eq!(extract_base_property("regularProperty"), "regularProperty");

    assert!(is_choice_type("value[x]"));
    assert!(is_choice_type("effective[x]"));
    assert!(!is_choice_type("regularProperty"));

    assert_eq!(
        generate_choice_property_name("value[x]", "String"),
        "valueString"
    );
    assert_eq!(
        generate_choice_property_name("effective[x]", "DateTime"),
        "effectiveDateTime"
    );
}

#[tokio::test]
async fn test_bridge_validation() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let navigator = PropertyNavigator::new(manager);

    // Test FHIRPath constraint validation
    let validation_result = navigator
        .validate_fhirpath_constraint("Patient.name.exists()")
        .await;

    assert!(validation_result.is_ok());

    let result = validation_result.unwrap();
    assert!(result.is_valid);
}

#[tokio::test]
async fn test_concurrent_operations() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let navigator = PropertyNavigator::new(manager);

    // Test concurrent resource type checks
    let tasks: Vec<_> = (0..10)
        .map(|_| {
            let nav = navigator.clone();
            tokio::spawn(async move {
                nav.has_resource_type("Patient").await
                    && nav.has_resource_type("Observation").await
                    && !nav.has_resource_type("InvalidType").await
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All tasks should complete successfully
    for result in results {
        assert!(result.unwrap());
    }
}

#[tokio::test]
async fn test_error_handling() {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = Arc::new(
        FhirSchemaPackageManager::new(fcm_config, config)
            .await
            .unwrap(),
    );
    let navigator = PropertyNavigator::new(manager.clone());
    let mut resolver = BridgeChoiceTypeResolver::new(manager);

    // Test invalid property lookup
    let invalid_prop = navigator
        .get_property_info("InvalidType", "invalidProperty")
        .await;
    assert!(invalid_prop.is_err());

    // Test invalid choice type resolution
    let invalid_choice = resolver
        .resolve_choice_type("InvalidType.invalid[x]", "invalidType")
        .await;
    assert!(invalid_choice.is_err());

    // Test invalid schema lookup
    let invalid_schema = navigator
        .get_schema("http://invalid.url/StructureDefinition/Invalid")
        .await;
    assert!(invalid_schema.is_err());
}
