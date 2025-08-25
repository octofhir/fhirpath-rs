//! toLong() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// toLong(): Converts input to Long (64-bit integer) where possible
pub struct ToLongFunction;

impl SyncOperation for ToLongFunction {
    fn name(&self) -> &'static str {
        "toLong"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toLong",
            parameters: vec![],
            return_type: ValueType::Integer, // Long maps to Integer in our type system
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(&self, _args: &[FhirPathValue], context: &crate::traits::EvaluationContext) -> Result<FhirPathValue> {
        convert_to_long(&context.input)
    }
}

fn convert_to_long(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already an integer (which is i64 in our implementation)
        FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
        
        // Decimal can be converted if it's a whole number within i64 range
        FhirPathValue::Decimal(d) => {
            if d.fract().is_zero() {
                match d.to_i64() {
                    Some(i) => {
                        // Ensure it's within i64 range
                        if i >= i64::MIN && i <= i64::MAX {
                            Ok(FhirPathValue::Integer(i))
                        } else {
                            Err(FhirPathError::ConversionError {
                                from: format!("Decimal value {} is out of Long range", d),
                                to: "Long".to_string(),
                            })
                        }
                    }
                    None => Err(FhirPathError::ConversionError {
                        from: format!("Cannot convert decimal {} to Long", d),
                        to: "Long".to_string(),
                    }),
                }
            } else {
                Err(FhirPathError::ConversionError {
                    from: format!("Cannot convert decimal {} to Long (has fractional part)", d),
                    to: "Long".to_string(),
                })
            }
        },
        
        // String conversion with proper long parsing
        FhirPathValue::String(s) => {
            match s.trim().parse::<i64>() {
                Ok(i) => Ok(FhirPathValue::Integer(i)),
                Err(_) => Err(FhirPathError::ConversionError {
                    from: format!("Cannot convert string '{}' to Long", s),
                    to: "Long".to_string(),
                }),
            }
        },
        
        // Boolean conversion (true = 1, false = 0)
        FhirPathValue::Boolean(b) => {
            let i = if *b { 1i64 } else { 0i64 };
            Ok(FhirPathValue::Integer(i))
        },
        
        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
        
        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_long(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }
        
        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: format!("Cannot convert {} to Long", value.type_name()),
            to: "Long".to_string(),
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
    fn test_to_long() {
        let op = ToLongFunction;

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test large integer input
        let large_value = i64::MAX;
        let context = create_context(FhirPathValue::Integer(large_value));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(large_value));

        // Test negative integer input
        let context = create_context(FhirPathValue::Integer(i64::MIN));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(i64::MIN));

        // Test whole decimal input
        let whole_decimal = Decimal::new(42, 0); // 42.0
        let context = create_context(FhirPathValue::Decimal(whole_decimal));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test fractional decimal input (should fail)
        let fractional_decimal = Decimal::new(425, 2); // 4.25
        let context = create_context(FhirPathValue::Decimal(fractional_decimal));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test valid long string
        let context = create_context(FhirPathValue::String("9223372036854775807".into())); // i64::MAX
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(i64::MAX));

        // Test valid negative long string
        let context = create_context(FhirPathValue::String("-9223372036854775808".into())); // i64::MIN
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(i64::MIN));

        // Test long string with whitespace
        let context = create_context(FhirPathValue::String("  123456789  ".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(123456789));

        // Test out of range string (should fail)
        let context = create_context(FhirPathValue::String("9223372036854775808".into())); // i64::MAX + 1
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test decimal string (should fail)
        let context = create_context(FhirPathValue::String("42.5".into()));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test invalid string
        let context = create_context(FhirPathValue::String("invalid".into()));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test boolean inputs
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));

        let context = create_context(FhirPathValue::Boolean(false));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        // Test single item collection
        let collection = vec![FhirPathValue::String("123456".into())];
        let context = create_context(FhirPathValue::Collection(collection.into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(123456));

        // Test multi-item collection (should return empty)
        let collection = vec![FhirPathValue::Integer(42), FhirPathValue::Integer(24)];
        let context = create_context(FhirPathValue::Collection(collection.into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));
    }

    #[test]
    fn test_large_decimal_to_long() {
        let op = ToLongFunction;

        // Test large decimal that fits in i64
        let large_decimal = Decimal::from(i64::MAX);
        let context = create_context(FhirPathValue::Decimal(large_decimal));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(i64::MAX));

        // Test small decimal that fits in i64
        let small_decimal = Decimal::from(i64::MIN);
        let context = create_context(FhirPathValue::Decimal(small_decimal));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(i64::MIN));
    }
}