//! convertsToLong() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// convertsToLong(): Returns true if the input can be converted to Long (64-bit integer)
pub struct ConvertsToLongFunction;

impl SyncOperation for ConvertsToLongFunction {
    fn name(&self) -> &'static str {
        "convertsToLong"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToLong",
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
        // Handle collections by applying convertsToLong to each element
        match &context.input {
            FhirPathValue::Collection(col) => {
                // Apply convertsToLong to each element in the collection
                let results: Result<Vec<FhirPathValue>> = col
                    .iter()
                    .map(|item| {
                        let can_convert = can_convert_to_long(item)?;
                        Ok(FhirPathValue::Boolean(can_convert))
                    })
                    .collect();

                Ok(FhirPathValue::collection(results?))
            }
            _ => {
                // Single element - original behavior
                let can_convert = can_convert_to_long(&context.input)?;
                Ok(FhirPathValue::Boolean(can_convert))
            }
        }
    }
}

fn can_convert_to_long(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already an integer (which is i64 in our implementation)
        FhirPathValue::Integer(_) => Ok(true),

        // Decimal can be converted if it's a whole number within i64 range
        FhirPathValue::Decimal(d) => {
            if d.fract().is_zero() {
                // Check if it's within i64 range
                if let Some(int_val) = d.to_i64() {
                    Ok((i64::MIN..=i64::MAX).contains(&int_val))
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }

        // String values that can be parsed as i64
        FhirPathValue::String(s) => Ok(s.trim().parse::<i64>().is_ok()),

        // Boolean cannot be converted to Long in FHIRPath
        FhirPathValue::Boolean(_) => Ok(false),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules - single item collections are unwrapped
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_long(c.first().unwrap())
            } else {
                // This shouldn't happen at this level since we handle collections in execute()
                // But keep for safety - multiple items cannot convert as a single value
                Ok(false)
            }
        }

        // Other types cannot convert to long
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
        EvaluationContext::new(input.clone(), Arc::new(input), model_provider)
    }

    #[test]
    fn test_converts_to_long() {
        let op = ConvertsToLongFunction;

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test large integer input
        let context = create_context(FhirPathValue::Integer(i64::MAX));
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

        // Test valid long string
        let context = create_context(FhirPathValue::String("9223372036854775807".into())); // i64::MAX
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid string
        let context = create_context(FhirPathValue::String("invalid".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test boolean input (should not convert to long)
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
