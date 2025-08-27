//! convertsToInteger() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToInteger(): Returns true if the input can be converted to Integer
pub struct ConvertsToIntegerFunction;

impl SyncOperation for ConvertsToIntegerFunction {
    fn name(&self) -> &'static str {
        "convertsToInteger"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToInteger",
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
        // Handle collections by applying convertsToInteger to each element
        match &context.input {
            FhirPathValue::Collection(col) => {
                // Apply convertsToInteger to each element in the collection
                let results: Result<Vec<FhirPathValue>> = col
                    .iter()
                    .map(|item| {
                        let can_convert = can_convert_to_integer(item)?;
                        Ok(FhirPathValue::Boolean(can_convert))
                    })
                    .collect();

                Ok(FhirPathValue::collection(results?))
            }
            _ => {
                // Single element - original behavior
                let can_convert = can_convert_to_integer(&context.input)?;
                Ok(FhirPathValue::Boolean(can_convert))
            }
        }
    }
}

fn can_convert_to_integer(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already an integer
        FhirPathValue::Integer(_) => Ok(true),

        // Decimal can be converted if it's a whole number
        FhirPathValue::Decimal(d) => Ok(d.fract().is_zero()),

        // String values that can be parsed as integer
        FhirPathValue::String(s) => Ok(s.trim().parse::<i64>().is_ok()),

        // Boolean can be converted (true = 1, false = 0)
        FhirPathValue::Boolean(_) => Ok(true),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_integer(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to integer
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
    fn test_converts_to_integer() {
        let op = ConvertsToIntegerFunction;

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test whole decimal input
        let context = create_context(FhirPathValue::Decimal(Decimal::new(42, 0))); // 42.0
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test fractional decimal input (cannot convert)
        let context = create_context(FhirPathValue::Decimal(Decimal::new(425, 2))); // 4.25
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test valid integer string
        let context = create_context(FhirPathValue::String("42".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test decimal string (cannot convert to integer)
        let context = create_context(FhirPathValue::String("42.5".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test invalid string
        let context = create_context(FhirPathValue::String("invalid".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test boolean input
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
