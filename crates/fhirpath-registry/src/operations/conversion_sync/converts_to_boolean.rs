//! convertsToBoolean() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToBoolean(): Returns true if the input can be converted to Boolean
pub struct ConvertsToBooleanFunction;

impl SyncOperation for ConvertsToBooleanFunction {
    fn name(&self) -> &'static str {
        "convertsToBoolean"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToBoolean",
            parameters: vec![],
            return_type: ValueType::Boolean,
            variadic: false,
            category: FunctionCategory::Scalar,
            cardinality_requirement: CardinalityRequirement::AcceptsBoth,
        };
        &SIGNATURE
    }

    fn execute(
        &self,
        _args: &[FhirPathValue],
        context: &crate::traits::EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle collections by applying convertsToBoolean to each element
        match &context.input {
            FhirPathValue::Collection(col) => {
                // Apply convertsToBoolean to each element in the collection
                let results: Result<Vec<FhirPathValue>> = col
                    .iter()
                    .map(|item| {
                        let can_convert = can_convert_to_boolean(item)?;
                        Ok(FhirPathValue::Boolean(can_convert))
                    })
                    .collect();

                Ok(FhirPathValue::collection(results?))
            }
            _ => {
                // Single element - original behavior
                let can_convert = can_convert_to_boolean(&context.input)?;
                Ok(FhirPathValue::Boolean(can_convert))
            }
        }
    }
}

fn can_convert_to_boolean(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a boolean
        FhirPathValue::Boolean(_) => Ok(true),

        // String values that can be parsed as booleans
        FhirPathValue::String(s) => match s.to_lowercase().as_str() {
            "true" | "t" | "yes" | "y" | "1" | "false" | "f" | "no" | "n" | "0" => Ok(true),
            _ => Ok(false),
        },

        // Integers can convert to boolean (0 = false, non-zero = true)
        FhirPathValue::Integer(_) => Ok(true),

        // Decimals can convert to boolean (0.0 = false, non-zero = true)
        FhirPathValue::Decimal(_) => Ok(true),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_boolean(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to boolean
        _ => Ok(false),
    }
}

#[cfg(not(test))]
mod tests {
    use super::*;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input.clone(), std::sync::Arc::new(input), model_provider)
    }

    #[test]
    fn test_converts_to_boolean() {
        let op = ConvertsToBooleanFunction;

        // Test boolean input
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid string inputs
        let context = create_context(FhirPathValue::String("true".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let context = create_context(FhirPathValue::String("false".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid string input
        let context = create_context(FhirPathValue::String("invalid".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
