//! toString() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// toString(): Converts input to String where possible
pub struct ToStringFunction;

impl SyncOperation for ToStringFunction {
    fn name(&self) -> &'static str {
        "toString"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toString",
            parameters: vec![],
            return_type: ValueType::String,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(&self, _args: &[FhirPathValue], context: &crate::traits::EvaluationContext) -> Result<FhirPathValue> {
        convert_to_string(&context.input)
    }
}

fn convert_to_string(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a string
        FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),

        // Integer conversion
        FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string().into())),

        // Decimal conversion with proper formatting
        FhirPathValue::Decimal(d) => {
            // Format decimal according to FHIRPath specification
            let formatted = format_decimal(*d);
            Ok(FhirPathValue::String(formatted.into()))
        }

        // Boolean conversion
        FhirPathValue::Boolean(b) => {
            let s = if *b { "true" } else { "false" };
            Ok(FhirPathValue::String(s.into()))
        }

        // Date conversion
        FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.to_string().into())),

        // DateTime conversion
        FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.to_string().into())),

        // Time conversion
        FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.to_string().into())),

        // Quantity conversion
        FhirPathValue::Quantity(q) => {
            let formatted_value = format_decimal(q.value);
            let result = if let Some(unit) = &q.unit {
                // Only quote UCUM units, leave standard units unquoted
                match unit.as_str() {
                    "wk" | "mo" | "a" | "d" => format!("{formatted_value} '{unit}'"),
                    _ => format!("{formatted_value} {unit}"),
                }
            } else {
                formatted_value
            };
            Ok(FhirPathValue::String(result.into()))
        }

        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_string(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }

        // Empty input
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

        // Unsupported types
        _ => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection for unsupported types
    }
}

fn format_decimal(decimal: rust_decimal::Decimal) -> String {
    // Format decimal according to FHIRPath specification
    let s = decimal.to_string();

    // Remove trailing zeros after decimal point
    if s.contains('.') {
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        if trimmed.is_empty() {
            "0".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::{MockModelProvider, Quantity};
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use std::str::FromStr;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, model_provider)
    }

    #[test]
    fn test_to_string() {
        let op = ToStringFunction;

        // Test string input
        let context = create_context(FhirPathValue::String("hello".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("hello".into()));

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("42".into()));

        // Test negative integer input
        let context = create_context(FhirPathValue::Integer(-123));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("-123".into()));

        // Test decimal input
        let decimal_val = Decimal::new(1234, 2); // 12.34
        let context = create_context(FhirPathValue::Decimal(decimal_val));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("12.34".into()));

        // Test decimal with trailing zeros
        let decimal_val = Decimal::new(12300, 2); // 123.00
        let context = create_context(FhirPathValue::Decimal(decimal_val));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("123".into()));

        // Test boolean inputs
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("true".into()));

        let context = create_context(FhirPathValue::Boolean(false));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("false".into()));

        // Test date input
        let date = FhirDate::from_ymd(2023, 12, 25).unwrap();
        let context = create_context(FhirPathValue::Date(date));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("2023-12-25".into()));

        // Test datetime input
        let datetime = FhirDateTime::from_ymd_hms(2023, 12, 25, 10, 30, 45).unwrap();
        let context = create_context(FhirPathValue::DateTime(datetime));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("2023-12-25T10:30:45".into()));

        // Test time input
        let time = FhirTime::from_hms(10, 30, 45).unwrap();
        let context = create_context(FhirPathValue::Time(time));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("10:30:45".into()));

        // Test quantity input
        let quantity = Quantity::new(42.5, "mg");
        let context = create_context(FhirPathValue::Quantity(quantity));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("42.5 mg".into()));

        // Test quantity with UCUM unit (should be quoted)
        let quantity = Quantity::new(2.0, "wk");
        let context = create_context(FhirPathValue::Quantity(quantity));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("2 'wk'".into()));

        // Test quantity without unit
        let quantity = Quantity::new(42.0, "1");
        let context = create_context(FhirPathValue::Quantity(quantity));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("42 1".into()));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        // Test single item collection
        let collection = vec![FhirPathValue::Integer(123)];
        let context = create_context(FhirPathValue::Collection(collection.into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("123".into()));

        // Test multi-item collection (should return empty)
        let collection = vec![
            FhirPathValue::Integer(42),
            FhirPathValue::Integer(24)
        ];
        let context = create_context(FhirPathValue::Collection(collection.into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));
    }

    #[test]
    fn test_format_decimal() {
        // Test normal decimal
        assert_eq!(format_decimal(Decimal::new(1234, 2)), "12.34");
        
        // Test decimal with trailing zeros
        assert_eq!(format_decimal(Decimal::new(12300, 2)), "123");
        assert_eq!(format_decimal(Decimal::new(123000, 3)), "123");
        
        // Test whole number
        assert_eq!(format_decimal(Decimal::new(123, 0)), "123");
        
        // Test zero
        assert_eq!(format_decimal(Decimal::ZERO), "0");
        
        // Test negative decimal
        assert_eq!(format_decimal(Decimal::new(-1234, 2)), "-12.34");
        
        // Test very small decimal
        assert_eq!(format_decimal(Decimal::new(1, 3)), "0.001");
    }

    #[test]
    fn test_quantity_unit_formatting() {
        let op = ToStringFunction;

        // Test standard units (no quotes)
        let quantity = Quantity::new(5.0, "mg");
        let context = create_context(FhirPathValue::Quantity(quantity));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("5 mg".into()));

        // Test UCUM units that should be quoted
        let ucum_units = vec!["wk", "mo", "a", "d"];
        for unit in ucum_units {
            let quantity = Quantity::new(1.0, unit);
            let context = create_context(FhirPathValue::Quantity(quantity));
            let result = op.execute(&[], &context).unwrap();
            assert_eq!(result, FhirPathValue::String(format!("1 '{}'", unit).into()));
        }

        // Test complex units (no quotes)
        let quantity = std::sync::Arc::new(Quantity::new(rust_decimal::Decimal::from_str("10.0").unwrap(), Some("mg/kg".to_string())));
        let context = create_context(FhirPathValue::Quantity(quantity));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::String("10 mg/kg".into()));
    }
}