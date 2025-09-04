//! Tests for collection functions

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::{FhirPathValue, ModelProvider};
    use crate::mock_provider::MockModelProvider;
    use std::collections::HashMap;

    fn create_test_context<'a>(
        input: &'a [FhirPathValue],
        arguments: &'a [FhirPathValue],
    ) -> FunctionContext<'a> {
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();

        FunctionContext {
            input,
            arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
        }
    }

    #[test]
    fn test_first_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test with non-empty collection
        let input = vec![
            FhirPathValue::String("first".to_string()),
            FhirPathValue::String("second".to_string()),
            FhirPathValue::String("third".to_string()),
        ];
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("first", &context).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], FhirPathValue::String("first".to_string()));

        // Test with empty collection
        let empty_input = vec![];
        let context = create_test_context(&empty_input, &[]);
        let result = dispatcher.dispatch_sync("first", &context).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_last_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("first".to_string()),
            FhirPathValue::String("second".to_string()),
            FhirPathValue::String("third".to_string()),
        ];
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("last", &context).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], FhirPathValue::String("third".to_string()));
    }

    #[test]
    fn test_tail_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("first".to_string()),
            FhirPathValue::String("second".to_string()),
            FhirPathValue::String("third".to_string()),
        ];
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("tail", &context).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], FhirPathValue::String("second".to_string()));
        assert_eq!(result[1], FhirPathValue::String("third".to_string()));

        // Test with empty collection
        let empty_input = vec![];
        let context = create_test_context(&empty_input, &[]);
        let result = dispatcher.dispatch_sync("tail", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test with single item
        let single_input = vec![FhirPathValue::String("only".to_string())];
        let context = create_test_context(&single_input, &[]);
        let result = dispatcher.dispatch_sync("tail", &context).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_count_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("c".to_string()),
        ];
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("count", &context).unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], FhirPathValue::Integer(3));

        // Test with empty collection
        let empty_input = vec![];
        let context = create_test_context(&empty_input, &[]);
        let result = dispatcher.dispatch_sync("count", &context).unwrap();
        assert_eq!(result[0], FhirPathValue::Integer(0));
    }

    #[test]
    fn test_skip_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("first".to_string()),
            FhirPathValue::String("second".to_string()),
            FhirPathValue::String("third".to_string()),
        ];

        // Test skip(1)
        let arguments = vec![FhirPathValue::Integer(1)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("skip", &context).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], FhirPathValue::String("second".to_string()));
        assert_eq!(result[1], FhirPathValue::String("third".to_string()));

        // Test skip(0)
        let arguments = vec![FhirPathValue::Integer(0)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("skip", &context).unwrap();
        assert_eq!(result.len(), 3);

        // Test skip(10) - more than collection size
        let arguments = vec![FhirPathValue::Integer(10)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("skip", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test skip with no arguments (should error)
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("skip", &context);
        assert!(result.is_err());

        // Test skip with negative argument (should error)
        let arguments = vec![FhirPathValue::Integer(-1)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("skip", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_take_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("first".to_string()),
            FhirPathValue::String("second".to_string()),
            FhirPathValue::String("third".to_string()),
        ];

        // Test take(2)
        let arguments = vec![FhirPathValue::Integer(2)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("take", &context).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], FhirPathValue::String("first".to_string()));
        assert_eq!(result[1], FhirPathValue::String("second".to_string()));

        // Test take(0)
        let arguments = vec![FhirPathValue::Integer(0)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("take", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test take(10) - more than collection size
        let arguments = vec![FhirPathValue::Integer(10)];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("take", &context).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_distinct_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("a".to_string()), // duplicate
            FhirPathValue::String("c".to_string()),
            FhirPathValue::String("b".to_string()), // duplicate
        ];
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("distinct", &context).unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], FhirPathValue::String("a".to_string()));
        assert_eq!(result[1], FhirPathValue::String("b".to_string()));
        assert_eq!(result[2], FhirPathValue::String("c".to_string()));

        // Test with different value types
        let mixed_input = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::String("test".to_string()),
            FhirPathValue::Integer(1), // duplicate
            FhirPathValue::Boolean(true),
            FhirPathValue::String("test".to_string()), // duplicate
        ];
        let context = create_test_context(&mixed_input, &[]);
        let result = dispatcher.dispatch_sync("distinct", &context).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_single_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        // Test with single item
        let single_input = vec![FhirPathValue::String("only".to_string())];
        let context = create_test_context(&single_input, &[]);
        let result = dispatcher.dispatch_sync("single", &context).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], FhirPathValue::String("only".to_string()));

        // Test with empty collection
        let empty_input = vec![];
        let context = create_test_context(&empty_input, &[]);
        let result = dispatcher.dispatch_sync("single", &context).unwrap();
        assert_eq!(result.len(), 0);

        // Test with multiple items (should error)
        let multi_input = vec![
            FhirPathValue::String("first".to_string()),
            FhirPathValue::String("second".to_string()),
        ];
        let context = create_test_context(&multi_input, &[]);
        let result = dispatcher.dispatch_sync("single", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_union_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
        ];
        let arguments = vec![
            FhirPathValue::String("b".to_string()), // duplicate
            FhirPathValue::String("c".to_string()),
        ];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("union", &context).unwrap();
        
        assert_eq!(result.len(), 3);
        assert!(result.contains(&FhirPathValue::String("a".to_string())));
        assert!(result.contains(&FhirPathValue::String("b".to_string())));
        assert!(result.contains(&FhirPathValue::String("c".to_string())));

        // Test union with empty collections
        let empty_input = vec![];
        let context = create_test_context(&empty_input, &arguments);
        let result = dispatcher.dispatch_sync("union", &context).unwrap();
        assert_eq!(result.len(), 2);

        // Test union without arguments (should error)
        let context = create_test_context(&input, &[]);
        let result = dispatcher.dispatch_sync("union", &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_intersect_function() {
        let registry = FunctionRegistry::default();
        let dispatcher = dispatcher::FunctionDispatcher::new(registry);

        let input = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("c".to_string()),
        ];
        let arguments = vec![
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("c".to_string()),
            FhirPathValue::String("d".to_string()),
        ];
        let context = create_test_context(&input, &arguments);
        let result = dispatcher.dispatch_sync("intersect", &context).unwrap();
        
        assert_eq!(result.len(), 2);
        assert!(result.contains(&FhirPathValue::String("b".to_string())));
        assert!(result.contains(&FhirPathValue::String("c".to_string())));

        // Test intersect with no common elements
        let no_common_input = vec![
            FhirPathValue::String("x".to_string()),
            FhirPathValue::String("y".to_string()),
        ];
        let context = create_test_context(&no_common_input, &arguments);
        let result = dispatcher.dispatch_sync("intersect", &context).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_collection_utils() {
        use super::super::collection::CollectionUtils;

        // Test value_hash_key
        let value1 = FhirPathValue::String("test".to_string());
        let value2 = FhirPathValue::Integer(42);
        let hash1 = CollectionUtils::value_hash_key(&value1);
        let hash2 = CollectionUtils::value_hash_key(&value2);
        assert_ne!(hash1, hash2);
        assert!(hash1.starts_with("str:"));
        assert!(hash2.starts_with("int:"));

        // Test remove_duplicates
        let collection = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("a".to_string()),
        ];
        let unique = CollectionUtils::remove_duplicates(&collection);
        assert_eq!(unique.len(), 2);

        // Test union_collections
        let first = vec![FhirPathValue::String("a".to_string())];
        let second = vec![FhirPathValue::String("b".to_string())];
        let union = CollectionUtils::union_collections(&first, &second);
        assert_eq!(union.len(), 2);

        // Test intersect_collections
        let first = vec![
            FhirPathValue::String("a".to_string()),
            FhirPathValue::String("b".to_string()),
        ];
        let second = vec![
            FhirPathValue::String("b".to_string()),
            FhirPathValue::String("c".to_string()),
        ];
        let intersection = CollectionUtils::intersect_collections(&first, &second);
        assert_eq!(intersection.len(), 1);
        assert_eq!(intersection[0], FhirPathValue::String("b".to_string()));
    }
}