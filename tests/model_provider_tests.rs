//! Comprehensive tests for Phase 1: Model Provider Core Infrastructure

use octofhir_fhirpath::model::provider::{FhirVersion, TypeReflectionInfo};
use octofhir_fhirpath::model::{
    CacheManager, MockModelProvider, ModelProvider, TypeInfo, TypeMapper,
};
use std::time::Duration;

#[tokio::test]
async fn test_mock_provider_basic_functionality() {
    let provider = MockModelProvider::new();

    // Test type resolution for Patient
    let patient_type = provider.get_type_reflection("Patient").await;
    assert!(patient_type.is_some());

    if let Some(TypeReflectionInfo::ClassInfo { name, elements, .. }) = patient_type {
        assert_eq!(name, "Patient");
        assert!(!elements.is_empty());

        // Check that Patient has expected properties
        let property_names: Vec<String> = elements.iter().map(|e| e.name.clone()).collect();
        assert!(property_names.contains(&"active".to_string()));
        assert!(property_names.contains(&"name".to_string()));
    }

    // Test property resolution
    let name_type = provider.get_property_type("Patient", "name").await;
    assert!(name_type.is_some());

    let active_type = provider.get_property_type("Patient", "active").await;
    assert!(active_type.is_some());

    if let Some(TypeReflectionInfo::SimpleType { name, .. }) = active_type {
        assert_eq!(name, "Boolean");
    }

    // Test non-existent property
    let invalid_prop = provider
        .get_property_type("Patient", "invalidProperty")
        .await;
    assert!(invalid_prop.is_none());

    // Test non-existent type
    let invalid_type = provider.get_type_reflection("InvalidType").await;
    assert!(invalid_type.is_none());
}

#[tokio::test]
async fn test_mock_provider_human_name_complex_type() {
    let provider = MockModelProvider::new();

    // Test HumanName complex type
    let name_type = provider.get_type_reflection("HumanName").await;
    assert!(name_type.is_some());

    // Test HumanName properties
    let family_type = provider.get_property_type("HumanName", "family").await;
    assert!(family_type.is_some());

    if let Some(TypeReflectionInfo::SimpleType { name, .. }) = family_type {
        assert_eq!(name, "String");
    }

    let given_type = provider.get_property_type("HumanName", "given").await;
    assert!(given_type.is_some());

    // Given should be a list type (collection)
    if let Some(TypeReflectionInfo::ListType { element_type, .. }) = given_type {
        if let TypeReflectionInfo::SimpleType { name, .. } = element_type.as_ref() {
            assert_eq!(name, "String");
        }
    }
}

#[test]
fn test_type_mapper_primitive_types() {
    let mapper = TypeMapper::new();

    // Test primitive type mapping
    assert_eq!(
        mapper.map_primitive_type("boolean"),
        Some(TypeInfo::Boolean)
    );
    assert_eq!(
        mapper.map_primitive_type("integer"),
        Some(TypeInfo::Integer)
    );
    assert_eq!(
        mapper.map_primitive_type("decimal"),
        Some(TypeInfo::Decimal)
    );
    assert_eq!(mapper.map_primitive_type("string"), Some(TypeInfo::String));
    assert_eq!(mapper.map_primitive_type("date"), Some(TypeInfo::Date));
    assert_eq!(
        mapper.map_primitive_type("dateTime"),
        Some(TypeInfo::DateTime)
    );
    assert_eq!(mapper.map_primitive_type("time"), Some(TypeInfo::Time));

    // Test FHIR primitive type aliases
    assert_eq!(mapper.map_primitive_type("code"), Some(TypeInfo::String));
    assert_eq!(mapper.map_primitive_type("id"), Some(TypeInfo::String));
    assert_eq!(mapper.map_primitive_type("uri"), Some(TypeInfo::String));
    assert_eq!(
        mapper.map_primitive_type("positiveInt"),
        Some(TypeInfo::Integer)
    );
    assert_eq!(
        mapper.map_primitive_type("instant"),
        Some(TypeInfo::DateTime)
    );

    // Test unknown primitive type
    assert_eq!(mapper.map_primitive_type("unknown"), None);
}

#[test]
fn test_type_mapper_complex_types() {
    let mapper = TypeMapper::new();

    // Test complex type mapping
    assert_eq!(
        mapper.map_complex_type("CodeableConcept"),
        Some(TypeInfo::Resource("CodeableConcept".to_string()))
    );
    assert_eq!(
        mapper.map_complex_type("Reference"),
        Some(TypeInfo::Resource("Reference".to_string()))
    );
    assert_eq!(
        mapper.map_complex_type("Quantity"),
        Some(TypeInfo::Quantity)
    );
    assert_eq!(
        mapper.map_complex_type("Period"),
        Some(TypeInfo::Resource("Period".to_string()))
    );
    assert_eq!(
        mapper.map_complex_type("HumanName"),
        Some(TypeInfo::Resource("HumanName".to_string()))
    );

    // Test unknown complex type
    assert_eq!(mapper.map_complex_type("UnknownComplexType"), None);
}

#[test]
fn test_type_mapper_resource_types() {
    let mapper = TypeMapper::new();

    // Test resource type mapping
    assert_eq!(
        mapper.map_resource_type("Patient"),
        Some(TypeInfo::Resource("Patient".to_string()))
    );
    assert_eq!(
        mapper.map_resource_type("Observation"),
        Some(TypeInfo::Resource("Observation".to_string()))
    );
    assert_eq!(
        mapper.map_resource_type("Condition"),
        Some(TypeInfo::Resource("Condition".to_string()))
    );
    assert_eq!(
        mapper.map_resource_type("Organization"),
        Some(TypeInfo::Resource("Organization".to_string()))
    );

    // Test foundation resources
    assert_eq!(
        mapper.map_resource_type("Resource"),
        Some(TypeInfo::Resource("Resource".to_string()))
    );
    assert_eq!(
        mapper.map_resource_type("DomainResource"),
        Some(TypeInfo::Resource("DomainResource".to_string()))
    );

    // Test unknown resource type - should still return Some for capitalized names
    assert_eq!(
        mapper.map_resource_type("CustomResource"),
        Some(TypeInfo::Resource("CustomResource".to_string()))
    );

    // Test invalid resource type
    assert_eq!(mapper.map_resource_type("lowercase"), None);
}

#[test]
fn test_cache_basic_operations() {
    use octofhir_fhirpath::model::cache::TypeCache;

    let cache = TypeCache::<String>::new();

    // Test basic put/get
    cache.put("key1".to_string(), "value1".to_string());
    assert_eq!(cache.get("key1"), Some("value1".to_string()));

    // Test cache miss
    assert_eq!(cache.get("nonexistent"), None);

    // Test cache size
    assert_eq!(cache.size(), 1);
    assert!(!cache.is_empty());

    // Test cache clear
    cache.clear();
    assert_eq!(cache.size(), 0);
    assert!(cache.is_empty());
}

#[test]
fn test_cache_eviction_policy() {
    use octofhir_fhirpath::model::cache::{CacheConfig, TypeCache};

    let config = CacheConfig {
        max_size: 2,
        ttl: Duration::from_secs(60),
        enable_stats: true,
    };
    let cache = TypeCache::with_config(config);

    // Fill cache to capacity
    cache.put("key1".to_string(), "value1".to_string());
    cache.put("key2".to_string(), "value2".to_string());

    // Access key1 to make it more recently used
    cache.get("key1");

    // Add third item - should evict key2 (LRU)
    cache.put("key3".to_string(), "value3".to_string());

    // Verify eviction worked correctly
    assert!(cache.get("key1").is_some()); // Most recently used
    assert!(cache.get("key2").is_none()); // Should be evicted
    assert!(cache.get("key3").is_some()); // Newly added
    assert_eq!(cache.size(), 2);
}

#[test]
fn test_cache_statistics() {
    use octofhir_fhirpath::model::cache::TypeCache;

    let cache = TypeCache::<String>::new();

    // Initial stats
    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 0);

    // Generate miss
    cache.get("nonexistent");
    let stats = cache.stats();
    assert_eq!(stats.misses, 1);

    // Generate hit
    cache.put("key1".to_string(), "value1".to_string());
    cache.get("key1");
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.size, 1);

    // Test hit ratio calculation
    assert_eq!(stats.hit_ratio(), 0.5); // 1 hit out of 2 total accesses
}

#[test]
fn test_cache_manager() {
    let manager = CacheManager::new();

    // Test that caches are initialized
    assert!(manager.type_cache.is_empty());
    assert!(manager.element_cache.is_empty());

    // Test adding items to different caches
    let type_info = TypeReflectionInfo::SimpleType {
        namespace: "System".to_string(),
        name: "String".to_string(),
        base_type: None,
    };

    manager
        .type_cache
        .put("String".to_string(), type_info.clone());
    manager
        .element_cache
        .put("Patient.name".to_string(), type_info);

    // Test combined stats
    let combined_stats = manager.combined_stats();
    assert_eq!(combined_stats.size, 2);

    // Test clearing all caches
    manager.clear_all();
    assert!(manager.type_cache.is_empty());
    assert!(manager.element_cache.is_empty());
}

#[tokio::test]
async fn test_model_provider_subtype_checking() {
    let provider = MockModelProvider::new();

    // Test subtype relationships
    assert!(provider.is_subtype_of("Patient", "DomainResource").await);
    assert!(provider.is_subtype_of("Patient", "Patient").await); // Self is subtype
    assert!(provider.is_subtype_of("HumanName", "Element").await);
    assert!(!provider.is_subtype_of("Patient", "Observation").await); // Not related
}

#[test]
fn test_model_provider_fhir_version() {
    let provider = MockModelProvider::new();
    assert_eq!(provider.fhir_version(), FhirVersion::R4);
}

#[tokio::test]
async fn test_model_provider_resource_type_detection() {
    let provider = MockModelProvider::new();

    // Test resource type detection
    assert!(provider.is_resource_type("Patient").await);
    assert!(provider.is_resource_type("Observation").await);
    assert!(!provider.is_resource_type("HumanName").await); // Complex type, not resource
    assert!(!provider.is_resource_type("string").await); // Primitive type
}

#[tokio::test]
async fn test_model_provider_base_types() {
    let provider = MockModelProvider::new();

    // Test base type resolution
    assert_eq!(
        provider.get_base_type("Patient").await,
        Some("DomainResource".to_string())
    );
    assert_eq!(
        provider.get_base_type("HumanName").await,
        Some("Element".to_string())
    );
    assert_eq!(provider.get_base_type("NonExistent").await, None);
}

#[tokio::test]
async fn test_model_provider_properties() {
    let provider = MockModelProvider::new();

    // Test getting all properties for a type
    let patient_props = provider.get_properties("Patient").await;
    assert!(!patient_props.is_empty());

    let prop_names: Vec<String> = patient_props.iter().map(|(name, _)| name.clone()).collect();
    assert!(prop_names.contains(&"active".to_string()));
    assert!(prop_names.contains(&"name".to_string()));
}

#[tokio::test]
async fn test_model_provider_navigation_validation() {
    let provider = MockModelProvider::new();

    // Test valid navigation
    let result = provider
        .validate_navigation_path("Patient", "active")
        .await
        .unwrap();
    assert!(result.is_valid);
    assert!(result.messages.is_empty());
    assert!(result.result_type.is_some());

    // Test invalid navigation
    let result = provider
        .validate_navigation_path("Patient", "invalidProperty")
        .await
        .unwrap();
    assert!(!result.is_valid);
    assert!(!result.messages.is_empty());
    assert!(result.result_type.is_none());
}

#[tokio::test]
async fn test_model_provider_expression_analysis() {
    let provider = MockModelProvider::new();

    // Test expression analysis (basic implementation)
    let result = provider.analyze_expression("Patient.name").await.unwrap();
    assert!(result.referenced_types.is_empty()); // Mock implementation returns empty
    assert!(result.navigation_paths.is_empty());
    assert!(result.type_safety_warnings.is_empty());
}

// Integration test combining all Phase 1 components
#[tokio::test]
async fn test_phase_1_integration() {
    let provider = MockModelProvider::new();
    let mapper = TypeMapper::new();
    let cache_manager = CacheManager::new();

    // Test end-to-end workflow: resolve type -> map to TypeInfo -> cache result

    // 1. Resolve Patient type from provider
    let patient_reflection = provider.get_type_reflection("Patient").await.unwrap();

    // 2. Map to legacy TypeInfo (this simulates how the analyzer would work)
    let patient_type_info = match &patient_reflection {
        TypeReflectionInfo::ClassInfo { name, .. } => TypeInfo::Resource(name.clone()),
        _ => TypeInfo::Any,
    };

    // Verify the type mapping worked
    assert_eq!(patient_type_info, TypeInfo::Resource("Patient".to_string()));

    // 3. Cache the result
    cache_manager
        .type_cache
        .put("Patient".to_string(), patient_reflection.clone());

    // 4. Verify cached retrieval works
    let cached_result = cache_manager.type_cache.get("Patient").unwrap();
    assert_eq!(cached_result, patient_reflection);

    // 5. Test property resolution workflow
    let name_reflection = provider.get_property_type("Patient", "name").await.unwrap();
    let name_key = "Patient.name".to_string();
    cache_manager
        .element_cache
        .put(name_key.clone(), name_reflection.clone());

    let cached_property = cache_manager.element_cache.get(&name_key).unwrap();
    assert_eq!(cached_property, name_reflection);

    // 6. Verify cache statistics show usage
    let stats = cache_manager.combined_stats();
    assert!(stats.hits >= 2); // Should have hits from our cache retrievals
    assert!(stats.size >= 2); // Should have our cached items
}
