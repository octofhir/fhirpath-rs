//! Tests for the function registry system

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::MockModelProvider;
    use crate::core::{FhirPathValue, ModelProvider};
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    #[test]
    fn test_function_registration() {
        let mut registry = FunctionRegistry::new();

        let test_function = Arc::new(|_context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
            Ok(vec![FhirPathValue::boolean(true)])
        });

        let metadata = FunctionMetadata {
            name: "test".to_string(),
            category: FunctionCategory::Utility,
            description: "Test function".to_string(),
            parameters: vec![],
            return_type: Some("boolean".to_string()),
            is_async: false,
            examples: vec![],
            requires_model_provider: false,
            requires_terminology_provider: false,
            does_not_propagate_empty: false,
        };

        assert!(
            registry
                .register_sync_function("test", test_function, metadata)
                .is_ok()
        );
        assert!(registry.get_sync_function("test").is_some());
        assert_eq!(registry.is_function_async("test"), Some(false));
    }

    #[test]
    fn test_duplicate_function_registration() {
        let mut registry = FunctionRegistry::new();

        let test_function1 = Arc::new(|_: &FunctionContext| -> Result<Vec<FhirPathValue>> {
            Ok(vec![FhirPathValue::boolean(true)])
        });

        let test_function2 = Arc::new(|_: &FunctionContext| -> Result<Vec<FhirPathValue>> {
            Ok(vec![FhirPathValue::boolean(false)])
        });

        let metadata = FunctionMetadata {
            name: "test".to_string(),
            category: FunctionCategory::Utility,
            description: "Test function".to_string(),
            parameters: vec![],
            return_type: Some("boolean".to_string()),
            is_async: false,
            examples: vec![],
            requires_model_provider: false,
            requires_terminology_provider: false,
            does_not_propagate_empty: false,
        };

        assert!(
            registry
                .register_sync_function("test", test_function1, metadata.clone())
                .is_ok()
        );
        assert!(
            registry
                .register_sync_function("test", test_function2, metadata)
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_async_function_registration() {
        let mut registry = FunctionRegistry::new();

        let async_function: AsyncFunction = Arc::new(|_context| {
            Box::pin(async { Ok(vec![FhirPathValue::string("async_result".to_string())]) })
        });

        let metadata = FunctionMetadata {
            name: "async_test".to_string(),
            category: FunctionCategory::Utility,
            description: "Async test function".to_string(),
            parameters: vec![],
            return_type: Some("string".to_string()),
            is_async: true,
            examples: vec![],
            requires_model_provider: false,
            requires_terminology_provider: false,
            does_not_propagate_empty: false,
        };

        assert!(
            registry
                .register_async_function("async_test", async_function, metadata)
                .is_ok()
        );
        assert!(registry.get_async_function("async_test").is_some());
        assert_eq!(registry.is_function_async("async_test"), Some(true));
    }

    #[test]
    fn test_function_metadata() {
        let registry = FunctionRegistry::default();

        let metadata = registry.get_function_metadata("empty");
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "empty");
        assert_eq!(metadata.category, FunctionCategory::Utility);
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_list_functions() {
        let registry = FunctionRegistry::default();

        let functions = registry.list_functions();
        assert!(!functions.is_empty());

        // Should have at least the default functions (empty, exists)
        assert!(functions.len() >= 2);
    }

    #[test]
    fn test_list_functions_by_category() {
        let registry = FunctionRegistry::default();

        let utility_functions = registry.list_functions_by_category(FunctionCategory::Utility);
        assert!(!utility_functions.is_empty());

        for func in utility_functions {
            assert_eq!(func.category, FunctionCategory::Utility);
        }
    }

    fn create_test_context_parts() -> (MockModelProvider, HashMap<String, FhirPathValue>) {
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        (model_provider, variables)
    }

    fn create_test_context<'a>(
        input: &'a [FhirPathValue],
        arguments: &'a [FhirPathValue],
        model_provider: &'a MockModelProvider,
        variables: &'a HashMap<String, FhirPathValue>,
    ) -> FunctionContext<'a> {
        FunctionContext {
            input,
            arguments,
            model_provider,
            variables,
            resource_context: None,
            terminology: None,
        }
    }
}
