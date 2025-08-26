//! Simplified collection operations module

// Core operations
pub mod count;
pub mod empty;
pub mod exists;
pub mod first;
pub mod last;
pub mod single;

// Navigation operations
pub mod skip;
pub mod tail;
pub mod take;

// Set operations
pub mod distinct;
pub mod exclude;
pub mod intersect;
pub mod union;

// Boolean operations
pub mod all_false;
pub mod all_true;
pub mod any_false;
pub mod any_true;

// Comparison operations
pub mod is_distinct;
pub mod subset_of;
pub mod superset_of;

// Combine operation
pub mod combine;

// Re-exports
pub use count::SimpleCountFunction;
pub use empty::SimpleEmptyFunction;
pub use exists::SimpleExistsFunction;
pub use first::SimpleFirstFunction;
pub use last::SimpleLastFunction;
pub use single::SimpleSingleFunction;

pub use skip::SimpleSkipFunction;
pub use tail::SimpleTailFunction;
pub use take::SimpleTakeFunction;

pub use distinct::SimpleDistinctFunction;
pub use exclude::SimpleExcludeFunction;
pub use intersect::SimpleIntersectFunction;
pub use union::SimpleUnionFunction;

pub use all_false::SimpleAllFalseFunction;
pub use all_true::SimpleAllTrueFunction;
pub use any_false::SimpleAnyFalseFunction;
pub use any_true::SimpleAnyTrueFunction;

pub use is_distinct::SimpleIsDistinctFunction;
pub use subset_of::SimpleSubsetOfFunction;
pub use superset_of::SimpleSupersetOfFunction;

pub use combine::SimpleCombineFunction;

#[cfg(not(test))]
mod tests {
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(octofhir_fhirpath_model::MockModelProvider::new());
        EvaluationContext::new(input.clone(), Arc::new(input), model_provider)
    }

    #[test]
    fn test_count_function() {
        let func = SimpleCountFunction::new();
        assert_eq!(func.name(), "count");
        assert!(matches!(func.signature().return_type, ValueType::Integer));

        // Test with empty collection
        let empty_collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]));
        let context = create_test_context(empty_collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test with collection of 3 items
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
            ]));
        let context = create_test_context(collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test with single value
        let context = create_test_context(FhirPathValue::Integer(42));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test with empty
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));
    }

    #[test]
    fn test_first_function() {
        let func = SimpleFirstFunction::new();
        assert_eq!(func.name(), "first");

        // Test with collection
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
            ]));
        let context = create_test_context(collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        // Test with empty collection
        let empty_collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]));
        let context = create_test_context(empty_collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[test]
    fn test_empty_function() {
        let func = SimpleEmptyFunction::new();
        assert_eq!(func.name(), "empty");

        // Test with empty collection
        let empty_collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]));
        let context = create_test_context(empty_collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with non-empty collection
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Integer(1),
            ]));
        let context = create_test_context(collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with Empty value
        let context = create_test_context(FhirPathValue::Empty);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with single value
        let context = create_test_context(FhirPathValue::Integer(42));
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_distinct_function() {
        let func = SimpleDistinctFunction::new();
        assert_eq!(func.name(), "distinct");

        // Test with collection with duplicates
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(3),
                FhirPathValue::Integer(2),
            ]));
        let context = create_test_context(collection);
        let result = func.execute(&[], &context).unwrap();

        if let FhirPathValue::Collection(result_collection) = result {
            assert_eq!(result_collection.len(), 3);
            // Should contain 1, 2, 3 in original order
        } else {
            panic!("Expected collection result");
        }
    }

    #[test]
    fn test_all_true_function() {
        let func = SimpleAllTrueFunction::new();
        assert_eq!(func.name(), "allTrue");

        // Test with all true values
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(true),
            ]));
        let context = create_test_context(collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test with one false value
        let collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![
                FhirPathValue::Boolean(true),
                FhirPathValue::Boolean(false),
                FhirPathValue::Boolean(true),
            ]));
        let context = create_test_context(collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test with empty collection
        let empty_collection =
            FhirPathValue::Collection(octofhir_fhirpath_model::Collection::from(vec![]));
        let context = create_test_context(empty_collection);
        let result = func.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
