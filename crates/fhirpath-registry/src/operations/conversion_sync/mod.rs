//! Synchronous conversion operations module
//!
//! This module provides sync implementations of all FHIRPath conversion operations,
//! replacing the complex async system with simple sync implementations for pure
//! data transformation operations.

pub mod converts_to_boolean;
pub mod converts_to_date;
pub mod converts_to_datetime;
pub mod converts_to_decimal;
pub mod converts_to_integer;
pub mod converts_to_long;
pub mod converts_to_quantity;
pub mod converts_to_string;
pub mod converts_to_time;
pub mod to_boolean;
pub mod to_date;
pub mod to_datetime;
pub mod to_decimal;
pub mod to_integer;
pub mod to_long;
pub mod to_quantity;
pub mod to_string;
pub mod to_time;

// Re-export available sync conversion operations
pub use converts_to_boolean::ConvertsToBooleanFunction;
pub use converts_to_date::ConvertsToDateFunction;
pub use converts_to_datetime::ConvertsToDateTimeFunction;
pub use converts_to_decimal::ConvertsToDecimalFunction;
pub use converts_to_integer::ConvertsToIntegerFunction;
pub use converts_to_long::ConvertsToLongFunction;
pub use converts_to_quantity::ConvertsToQuantityFunction;
pub use converts_to_string::ConvertsToStringFunction;
pub use converts_to_time::ConvertsToTimeFunction;
pub use to_boolean::ToBooleanFunction;
pub use to_date::ToDateFunction;
pub use to_datetime::ToDateTimeFunction;
pub use to_decimal::ToDecimalFunction;
pub use to_integer::ToIntegerFunction;
pub use to_long::ToLongFunction;
pub use to_quantity::ToQuantityFunction;
pub use to_string::ToStringFunction;
pub use to_time::ToTimeFunction;

use crate::registry::FunctionRegistry;

/// Registry helper for sync conversion operations
pub struct SyncConversionOperations;

impl SyncConversionOperations {
    /// Register all sync conversion operations with the registry
    pub async fn register_all(registry: &FunctionRegistry) {
        // Type checking operations (converts_to_*)
        registry.register_sync(Box::new(ConvertsToBooleanFunction)).await;
        registry.register_sync(Box::new(ConvertsToDateFunction)).await;
        registry.register_sync(Box::new(ConvertsToDateTimeFunction)).await;
        registry.register_sync(Box::new(ConvertsToDecimalFunction)).await;
        registry.register_sync(Box::new(ConvertsToIntegerFunction)).await;
        registry.register_sync(Box::new(ConvertsToLongFunction)).await;
        registry.register_sync(Box::new(ConvertsToStringFunction)).await;
        registry.register_sync(Box::new(ConvertsToTimeFunction)).await;

        // Type conversion operations (to_*)
        registry.register_sync(Box::new(ToBooleanFunction)).await;
        registry.register_sync(Box::new(ToDateFunction)).await;
        registry.register_sync(Box::new(ToDateTimeFunction)).await;
        registry.register_sync(Box::new(ToDecimalFunction)).await;
        registry.register_sync(Box::new(ToIntegerFunction)).await;
        registry.register_sync(Box::new(ToLongFunction)).await;
        registry.register_sync(Box::new(ToStringFunction)).await;
        registry.register_sync(Box::new(ToTimeFunction)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{SyncOperation, EvaluationContext};
    use octofhir_fhirpath_model::{MockModelProvider, FhirPathValue};
    use std::sync::Arc;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, model_provider)
    }

    #[test]
    fn test_all_sync_operations_have_correct_names() {
        // Type checking operations
        assert_eq!(ConvertsToBooleanFunction.name(), "convertsToBoolean");
        assert_eq!(ConvertsToDateFunction.name(), "convertsToDate");
        assert_eq!(ConvertsToDateTimeFunction.name(), "convertsToDateTime");
        assert_eq!(ConvertsToDecimalFunction.name(), "convertsToDecimal");
        assert_eq!(ConvertsToIntegerFunction.name(), "convertsToInteger");
        assert_eq!(ConvertsToLongFunction.name(), "convertsToLong");
        assert_eq!(ConvertsToQuantityFunction.name(), "convertsToQuantity");
        assert_eq!(ConvertsToStringFunction.name(), "convertsToString");
        assert_eq!(ConvertsToTimeFunction.name(), "convertsToTime");

        // Type conversion operations
        assert_eq!(ToBooleanFunction.name(), "toBoolean");
        assert_eq!(ToDateFunction.name(), "toDate");
        assert_eq!(ToDateTimeFunction.name(), "toDateTime");
        assert_eq!(ToDecimalFunction.name(), "toDecimal");
        assert_eq!(ToIntegerFunction.name(), "toInteger");
        assert_eq!(ToLongFunction.name(), "toLong");
        assert_eq!(ToQuantityFunction.name(), "toQuantity");
        assert_eq!(ToStringFunction.name(), "toString");
        assert_eq!(ToTimeFunction.name(), "toTime");
    }

    #[test]
    fn test_all_sync_operations_have_signatures() {
        // Test that all operations have valid signatures
        let operations: Vec<Box<dyn SyncOperation>> = vec![
            // Type checking operations
            Box::new(ConvertsToBooleanFunction),
            Box::new(ConvertsToDateFunction),
            Box::new(ConvertsToDateTimeFunction),
            Box::new(ConvertsToDecimalFunction),
            Box::new(ConvertsToIntegerFunction),
            Box::new(ConvertsToLongFunction),
            Box::new(ConvertsToQuantityFunction),
            Box::new(ConvertsToStringFunction),
            Box::new(ConvertsToTimeFunction),
            
            // Type conversion operations
            Box::new(ToBooleanFunction),
            Box::new(ToDateFunction),
            Box::new(ToDateTimeFunction),
            Box::new(ToDecimalFunction),
            Box::new(ToIntegerFunction),
            Box::new(ToLongFunction),
            Box::new(ToQuantityFunction),
            Box::new(ToStringFunction),
            Box::new(ToTimeFunction),
        ];

        for op in operations {
            let signature = op.signature();
            assert!(!signature.name.is_empty(), "Operation {} has empty name", op.name());
            assert_eq!(signature.name, op.name(), "Signature name doesn't match operation name for {}", op.name());
            
            // All conversion operations should have no parameters
            assert_eq!(signature.parameters.len(), 0, "Conversion operation {} should have no parameters", op.name());
            assert!(!signature.variadic, "Conversion operation {} should not be variadic", op.name());
        }
    }

    #[test]
    fn test_conversion_operations_work_with_empty_input() {
        let context = create_context(FhirPathValue::Empty);

        // Test a few operations with empty input
        let to_string = ToStringFunction;
        let result = to_string.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        let converts_to_string = ConvertsToStringFunction;
        let result = converts_to_string.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true)); // Empty converts to anything per FHIRPath spec
    }
}