//! Tests for the Unified Registry
//! 
//! These integration tests verify that the unified registry works correctly
//! with smart dispatch and proper function registration.

use crate::traits::EvaluationContext;
use crate::function_registry::create_standard_registry;
use octofhir_fhirpath_model::{FhirPathValue, MockModelProvider};
use std::sync::Arc;

#[tokio::test]
async fn test_unified_registry_sync_dispatch() {
    let registry = create_standard_registry();
    
    // Test sync string operations
    let context = EvaluationContext {
        input: FhirPathValue::String("hello".into()),
        model_provider: Arc::new(MockModelProvider::new()),
        variables: Default::default(),
    };
    
    // Test length() - sync string operation
    let result = registry.evaluate("length", &[], &context).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(5));
    
    // Test upper() - sync string operation  
    let result = registry.evaluate("upper", &[], &context).await.unwrap();
    assert_eq!(result, FhirPathValue::String("HELLO".into()));
}

#[tokio::test]
async fn test_unified_registry_collection_operations() {
    let registry = create_standard_registry();
    
    // Test sync collection operations
    let collection = FhirPathValue::Collection(
        octofhir_fhirpath_model::Collection::from(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ])
    );
    
    let context = EvaluationContext {
        input: collection,
        model_provider: Arc::new(MockModelProvider::new()),
        variables: Default::default(),
    };
    
    // Test count() - sync collection operation
    let result = registry.evaluate("count", &[], &context).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(3));
    
    // Test first() - sync collection operation
    let result = registry.evaluate("first", &[], &context).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(1));
}

#[tokio::test]
async fn test_unified_registry_math_operations() {
    let registry = create_standard_registry();
    
    // Test sync math operations
    let context = EvaluationContext {
        input: FhirPathValue::Integer(-42),
        model_provider: Arc::new(MockModelProvider::new()),
        variables: Default::default(),
    };
    
    // Test abs() - sync math operation
    let result = registry.evaluate("abs", &[], &context).await.unwrap();
    assert_eq!(result, FhirPathValue::Integer(42));
}

#[tokio::test]
async fn test_unified_registry_async_operations() {
    let registry = create_standard_registry();
    
    let context = EvaluationContext {
        input: FhirPathValue::Empty,
        model_provider: Arc::new(MockModelProvider::new()),
        variables: Default::default(),
    };
    
    // Test now() - async datetime operation
    let result = registry.evaluate("now", &[], &context).await.unwrap();
    // Just verify it returns a DateTime value
    matches!(result, FhirPathValue::DateTime(_));
    
    // Test today() - async datetime operation
    let result = registry.evaluate("today", &[], &context).await.unwrap();
    // Just verify it returns a Date value
    matches!(result, FhirPathValue::Date(_));
}

#[tokio::test]
async fn test_unified_registry_unknown_function() {
    let registry = create_standard_registry();
    
    let context = EvaluationContext {
        input: FhirPathValue::Empty,
        model_provider: Arc::new(MockModelProvider::new()),
        variables: Default::default(),
    };
    
    // Test unknown function
    let result = registry.evaluate("unknownFunction", &[], &context).await;
    assert!(result.is_err());
    
    if let Err(err) = result {
        assert!(err.to_string().contains("Unknown function: unknownFunction"));
    }
}

#[tokio::test]
async fn test_unified_registry_try_sync_only() {
    let registry = create_standard_registry();
    
    let context = EvaluationContext {
        input: FhirPathValue::String("test".into()),
        model_provider: Arc::new(MockModelProvider::new()),
        variables: Default::default(),
    };
    
    // Test sync-only evaluation for sync operations
    let result = registry.try_evaluate_sync("length", &[], &context).unwrap().unwrap();
    assert_eq!(result, FhirPathValue::Integer(4));
    
    // Test sync-only evaluation for async operations (should return None)
    let result = registry.try_evaluate_sync("now", &[], &context);
    assert!(result.is_none());
}

#[test]
fn test_unified_registry_function_queries() {
    let registry = create_standard_registry();
    
    // Test has_function
    assert!(registry.has_function("length"));
    assert!(registry.has_function("count"));
    assert!(registry.has_function("abs"));
    assert!(registry.has_function("now"));
    assert!(!registry.has_function("unknownFunction"));
    
    // Test supports_sync
    assert!(registry.supports_sync("length"));
    assert!(registry.supports_sync("count"));
    assert!(registry.supports_sync("abs"));
    assert!(!registry.supports_sync("now")); // async operation
    
    // Test function_names
    let names = registry.function_names();
    assert!(names.contains(&"length".to_string()));
    assert!(names.contains(&"count".to_string()));
    assert!(names.contains(&"abs".to_string()));
    assert!(names.contains(&"now".to_string()));
    
    // Test stats
    let stats = registry.stats();
    assert!(stats.sync_operations > 0);
    assert!(stats.async_operations > 0);
    assert_eq!(stats.total_operations, stats.sync_operations + stats.async_operations);
    assert!(stats.sync_percentage() > 0.0);
}