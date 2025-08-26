//! convertsToDecimal() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToDecimal(): Returns true if the input can be converted to Decimal
pub struct ConvertsToDecimalFunction;

impl SyncOperation for ConvertsToDecimalFunction {
    fn name(&self) -> &'static str {
        "convertsToDecimal"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToDecimal",
            parameters: vec![],
            return_type: ValueType::Boolean,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(
        &self,
        _args: &[FhirPathValue],
        context: &crate::traits::EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Handle collections by applying convertsToDecimal to each element
        match &context.input {
            FhirPathValue::Collection(col) => {
                // Apply convertsToDecimal to each element in the collection
                let results: Result<Vec<FhirPathValue>> = col
                    .iter()
                    .map(|item| {
                        let can_convert = can_convert_to_decimal(item)?;
                        Ok(FhirPathValue::Boolean(can_convert))
                    })
                    .collect();

                Ok(FhirPathValue::collection(results?))
            }
            _ => {
                // Single element - original behavior
                let can_convert = can_convert_to_decimal(&context.input)?;
                Ok(FhirPathValue::Boolean(can_convert))
            }
        }
    }
}

fn can_convert_to_decimal(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a decimal
        FhirPathValue::Decimal(_) => Ok(true),

        // Integer can be converted to decimal
        FhirPathValue::Integer(_) => Ok(true),

        // String values that can be parsed as decimal
        FhirPathValue::String(s) => {
            use rust_decimal::Decimal;
            use std::str::FromStr;
            Ok(Decimal::from_str(s.trim()).is_ok())
        }

        // Boolean can be converted (true = 1.0, false = 0.0)
        FhirPathValue::Boolean(_) => Ok(true),

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_decimal(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other types cannot convert to decimal
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
    fn test_converts_to_decimal() {
        let op = ConvertsToDecimalFunction;

        // Test decimal input
        let context = create_context(FhirPathValue::Decimal(Decimal::new(123, 2))); // 1.23
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid decimal string
        let context = create_context(FhirPathValue::String("123.45".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test integer string
        let context = create_context(FhirPathValue::String("123".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid decimal string
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
