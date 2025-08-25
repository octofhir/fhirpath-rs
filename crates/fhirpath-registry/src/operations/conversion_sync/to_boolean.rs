//! toBoolean() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// toBoolean(): Converts input to Boolean where possible
pub struct ToBooleanFunction;

impl SyncOperation for ToBooleanFunction {
    fn name(&self) -> &'static str {
        "toBoolean"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toBoolean",
            parameters: vec![],
            return_type: ValueType::Boolean,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(&self, _args: &[FhirPathValue], context: &crate::traits::EvaluationContext) -> Result<FhirPathValue> {
        convert_to_boolean(&context.input)
    }
}

fn convert_to_boolean(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a boolean
        FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
        
        // String conversion following FHIRPath rules
        FhirPathValue::String(s) => {
            match s.to_lowercase().as_str() {
                "true" | "t" | "yes" | "y" | "1" => Ok(FhirPathValue::Boolean(true)),
                "false" | "f" | "no" | "n" | "0" => Ok(FhirPathValue::Boolean(false)),
                _ => Err(FhirPathError::ConversionError {
                    from: format!("String('{}')", s),
                    to: "Boolean".to_string(),
                }),
            }
        },
        
        // Integer conversion (0 = false, non-zero = true)
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Boolean(*i != 0)),
        
        // Decimal conversion (0.0 = false, non-zero = true)
        FhirPathValue::Decimal(d) => Ok(FhirPathValue::Boolean(!d.is_zero())),
        
        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
        
        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_boolean(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }
        
        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: "Unsupported type".to_string(),
            to: "Boolean".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::MockModelProvider;
    use rust_decimal::Decimal;
    use std::sync::Arc;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, model_provider)
    }

    #[test]
    fn test_to_boolean() {
        let op = ToBooleanFunction;

        // Test boolean input
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test valid string inputs
        let context = create_context(FhirPathValue::String("true".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let context = create_context(FhirPathValue::String("FALSE".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        let context = create_context(FhirPathValue::String("1".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test invalid string input
        let context = create_context(FhirPathValue::String("invalid".into()));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test integer inputs
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let context = create_context(FhirPathValue::Integer(0));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test decimal inputs
        let context = create_context(FhirPathValue::Decimal(Decimal::new(123, 2))); // 1.23
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        let context = create_context(FhirPathValue::Decimal(Decimal::ZERO));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));
    }
}