//! Simple integration test for core functionality
//!
//! This test verifies basic functionality works without external dependencies

use octofhir_fhirpath::model::{
    ChoiceTypeResolver, MockModelProvider, ModelProvider, ProfileResolver, TypeReflectionInfo,
};

#[tokio::test]
async fn test_mock_provider_basic() {
    let provider = MockModelProvider::new();

    // Test Patient type resolution
    if let Some(type_info) = provider.get_type_reflection("Patient").await {
        match type_info {
            TypeReflectionInfo::ClassInfo {
                name, base_type, ..
            } => {
                assert_eq!(name, "Patient");
                assert_eq!(base_type, Some("DomainResource".to_string()));
            }
            _ => panic!("Expected Patient to be ClassInfo"),
        }
    } else {
        panic!("Could not resolve Patient type");
    }
}

#[test]
fn test_choice_type_resolver() {
    let resolver = ChoiceTypeResolver::new();

    // Test valueString resolution
    if let Some(resolution) = resolver.resolve_choice_type("Observation.valueString") {
        assert_eq!(resolution.concrete_type, "string");
        assert_eq!(resolution.base_path, "Observation.value");
        assert_eq!(resolution.suffix, "String");
        assert!(resolution.is_valid);
    } else {
        panic!("Could not resolve Observation.valueString");
    }

    // Test invalid choice type
    if let Some(resolution) = resolver.resolve_choice_type("Observation.valueInvalidType") {
        assert!(!resolution.is_valid);
    }
}

#[test]
fn test_profile_resolver_creation() {
    let resolver = ProfileResolver::new();
    let (profile_stats, constraint_stats) = resolver.cache_stats();
    assert_eq!(profile_stats.size, 0);
    assert_eq!(constraint_stats.size, 0);
}

#[tokio::test]
async fn test_mock_provider_inheritance() {
    let provider = MockModelProvider::new();

    // Test inheritance relationships
    assert!(provider.is_subtype_of("Patient", "DomainResource").await);
    assert!(provider.is_subtype_of("Patient", "Resource").await);
    assert!(provider.is_subtype_of("Patient", "Patient").await); // Same type
    assert!(!provider.is_subtype_of("DomainResource", "Patient").await); // Reverse

    // Test with complex types
    assert!(provider.is_subtype_of("HumanName", "Element").await);
    assert!(!provider.is_subtype_of("HumanName", "Resource").await);
}

#[tokio::test]
async fn test_navigation_validation() {
    let provider = MockModelProvider::new();

    // Test valid navigation
    let validation = provider.validate_navigation_path("Patient", "name").await;
    match validation {
        Ok(nav_result) => {
            assert!(nav_result.is_valid);
            assert!(nav_result.result_type.is_some());
        }
        Err(e) => panic!("Navigation validation failed: {e}"),
    }

    // Test invalid navigation
    let validation = provider
        .validate_navigation_path("Patient", "nonExistentProperty")
        .await;
    match validation {
        Ok(nav_result) => {
            assert!(!nav_result.is_valid);
            assert!(!nav_result.messages.is_empty());
        }
        Err(_) => {
            // Also acceptable - some implementations might return errors
        }
    }
}
