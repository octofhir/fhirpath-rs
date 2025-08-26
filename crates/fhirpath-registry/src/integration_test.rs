//! Integration test for the complete Unified Registry
//!
//! This tests the full functionality of the unified registry with all enabled operations

#[cfg(test)]
mod tests {
    use crate::function_registry::create_standard_registry;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::{FhirPathValue, MockModelProvider};
    use std::sync::Arc;

    use std::str::FromStr;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input.clone(), Arc::new(input), model_provider)
    }

    #[tokio::test]
    async fn test_unified_registry_comprehensive() {
        let registry = create_standard_registry().await;

        // Test sync string operations
        let context = create_test_context(FhirPathValue::String("hello world".into()));

        let result = registry.evaluate("length", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(11));

        let result = registry.evaluate("upper", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::String("HELLO WORLD".into()));
    }

    #[tokio::test]
    async fn test_sync_math_operations() {
        let registry = create_standard_registry().await;

        // Test math operations
        let context = create_test_context(FhirPathValue::Integer(-42));
        let result = registry.evaluate("abs", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        let context = create_test_context(FhirPathValue::Decimal(
            rust_decimal::Decimal::from_str("3.7").unwrap(),
        ));
        let result = registry.evaluate("ceiling", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(4));
    }

    #[tokio::test]
    async fn test_sync_collection_operations() {
        let registry = create_standard_registry().await;

        // Test collection operations
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
            ]));
        let context = create_test_context(collection);

        let result = registry.evaluate("count", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        let result = registry.evaluate("first", &[], &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
    }

    #[tokio::test]
    async fn test_async_datetime_operations() {
        let registry = create_standard_registry().await;

        let context = create_test_context(FhirPathValue::Empty);

        // Test async operations
        let result = registry.evaluate("now", &[], &context).await;
        assert!(result.is_ok()); // Should return a DateTime value

        let result = registry.evaluate("today", &[], &context).await;
        assert!(result.is_ok()); // Should return a Date value
    }

    #[tokio::test]
    async fn test_sync_first_async_fallback() {
        let registry = create_standard_registry().await;

        let context = create_test_context(FhirPathValue::String("test".into()));

        // Test that sync operations can be evaluated synchronously
        let sync_result = registry.try_evaluate_sync("length", &[], &context).await;
        assert!(sync_result.is_some());
        assert_eq!(sync_result.unwrap().unwrap(), FhirPathValue::Integer(4));

        // Test that async operations return None for sync evaluation
        let async_result = registry.try_evaluate_sync("now", &[], &context).await;
        assert!(async_result.is_none());
    }

    #[tokio::test]
    async fn test_registry_metadata() {
        let registry = create_standard_registry().await;

        // Test registry queries
        assert!(registry.has_function("length").await);
        assert!(registry.has_function("count").await);
        assert!(registry.has_function("abs").await);
        assert!(registry.has_function("now").await);
        assert!(!registry.has_function("unknownFunction").await);

        // Test sync support queries
        assert!(registry.supports_sync("length").await);
        assert!(registry.supports_sync("count").await);
        assert!(registry.supports_sync("abs").await);
        assert!(!registry.supports_sync("now").await); // async operation

        // Test function enumeration
        let names = registry.function_names().await;
        assert!(names.len() > 50); // We should have many operations registered

        // Test stats
        let stats = registry.stats().await;
        assert!(stats.sync_operations > 0);
        assert!(stats.async_operations > 0);
        assert_eq!(
            stats.total_operations,
            stats.sync_operations + stats.async_operations
        );

        println!(
            "Registry stats: {} sync, {} async, {:.1}% sync",
            stats.sync_operations,
            stats.async_operations,
            stats.sync_percentage()
        );
    }
}
