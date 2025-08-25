//! toQuantity() sync implementation

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Quantity};
use rust_decimal::Decimal;
use std::sync::Arc;

/// toQuantity(): Converts input to Quantity where possible
pub struct ToQuantityFunction;

impl SyncOperation for ToQuantityFunction {
    fn name(&self) -> &'static str {
        "toQuantity"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "toQuantity",
            parameters: vec![],
            return_type: ValueType::Quantity,
            variadic: false,
        };
        &SIGNATURE
    }

    fn execute(&self, _args: &[FhirPathValue], context: &crate::traits::EvaluationContext) -> Result<FhirPathValue> {
        convert_to_quantity(&context.input)
    }
}

fn convert_to_quantity(value: &FhirPathValue) -> Result<FhirPathValue> {
    match value {
        // Already a quantity
        FhirPathValue::Quantity(q) => Ok(FhirPathValue::Quantity(q.clone())),
        
        // Integer can be converted (becomes quantity with unit "1")
        FhirPathValue::Integer(i) => {
            let quantity = Quantity::new(Decimal::new(*i, 0), Some("1".to_string()));
            Ok(FhirPathValue::Quantity(Arc::new(quantity)))
        },
        
        // Decimal can be converted (becomes quantity with unit "1")
        FhirPathValue::Decimal(d) => {
            let quantity = Quantity::new(*d, Some("1".to_string()));
            Ok(FhirPathValue::Quantity(Arc::new(quantity)))
        },
        
        // String conversion with quantity parsing
        FhirPathValue::String(s) => {
            match parse_quantity_string(s) {
                Some((value, unit)) => {
                    let quantity = Quantity::new(Decimal::from_f64_retain(value).unwrap_or(Decimal::ZERO), unit);
                    Ok(FhirPathValue::Quantity(Arc::new(quantity)))
                },
                None => Err(FhirPathError::ConversionError {
                    from: format!("Cannot convert string '{}' to Quantity", s),
                    to: "Quantity".to_string(),
                }),
            }
        },
        
        // Empty input returns empty collection
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
        
        // Collection handling
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(FhirPathValue::Collection(vec![].into()))
            } else if c.len() == 1 {
                convert_to_quantity(c.first().unwrap())
            } else {
                // Multiple items - return empty collection per FHIRPath spec
                Ok(FhirPathValue::Collection(vec![].into()))
            }
        }
        
        // Unsupported types
        _ => Err(FhirPathError::ConversionError {
            from: format!("Cannot convert {} to Quantity", value.type_name()),
            to: "Quantity".to_string(),
        }),
    }
}

fn parse_quantity_string(s: &str) -> Option<(f64, Option<String>)> {
    let s = s.trim();
    
    // Try to parse just a number (no unit)
    if let Ok(value) = s.parse::<f64>() {
        return Some((value, None));
    }
    
    // Try to parse "value unit" format
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() == 2 {
        if let Ok(value) = parts[0].parse::<f64>() {
            let unit = parts[1].to_string();
            // Remove quotes from unit if present
            let unit = if unit.starts_with('\'') && unit.ends_with('\'') {
                unit[1..unit.len()-1].to_string()
            } else {
                unit
            };
            return Some((value, Some(unit)));
        }
    }
    
    // Try to parse formats like "5mg", "10.5kg", etc.
    let mut split_pos = None;
    for (i, c) in s.char_indices() {
        if c.is_alphabetic() || c == '\'' {
            split_pos = Some(i);
            break;
        }
    }
    
    if let Some(pos) = split_pos {
        let (value_part, unit_part) = s.split_at(pos);
        if let Ok(value) = value_part.parse::<f64>() {
            let unit = unit_part.trim();
            let unit = if unit.starts_with('\'') && unit.ends_with('\'') {
                unit[1..unit.len()-1].to_string()
            } else {
                unit.to_string()
            };
            return Some((value, Some(unit)));
        }
    }
    
    None
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
    fn test_to_quantity() {
        let op = ToQuantityFunction;

        // Test quantity input
        let quantity = Quantity::new(42.5, "mg");
        let context = create_context(FhirPathValue::Quantity(quantity.clone()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Quantity(quantity));

        // Test integer input (should become quantity with unit "1")
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(42.0, "1");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        // Test decimal input (should become quantity with unit "1")
        let decimal_val = Decimal::new(425, 1); // 42.5
        let context = create_context(FhirPathValue::Decimal(decimal_val));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(42.5, "1");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        // Test valid quantity string formats
        let context = create_context(FhirPathValue::String("42.5".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(42.5, "1"); // No unit defaults to "1"
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        let context = create_context(FhirPathValue::String("42.5 mg".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(42.5, "mg");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        let context = create_context(FhirPathValue::String("42mg".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(42.0, "mg");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        let context = create_context(FhirPathValue::String("42 'kg'".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(42.0, "kg");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        // Test negative quantities
        let context = create_context(FhirPathValue::String("-10.5 m".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(-10.5, "m");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        // Test invalid quantity string
        let context = create_context(FhirPathValue::String("invalid".into()));
        let result = op.execute(&[], &context);
        assert!(result.is_err());

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Collection(vec![].into()));

        // Test single item collection
        let collection = vec![FhirPathValue::String("123.45 g".into())];
        let context = create_context(FhirPathValue::Collection(collection.into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(123.45, "g");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

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
    fn test_parse_quantity_string() {
        // Test number only
        assert_eq!(parse_quantity_string("42.5"), Some((42.5, None)));
        assert_eq!(parse_quantity_string("-10.3"), Some((-10.3, None)));
        
        // Test number with unit
        assert_eq!(parse_quantity_string("42.5 mg"), Some((42.5, Some("mg".to_string()))));
        assert_eq!(parse_quantity_string("100.0 kg"), Some((100.0, Some("kg".to_string()))));
        
        // Test number with quoted unit
        assert_eq!(parse_quantity_string("42 'kg'"), Some((42.0, Some("kg".to_string()))));
        assert_eq!(parse_quantity_string("5.5 'wk'"), Some((5.5, Some("wk".to_string()))));
        
        // Test concatenated format
        assert_eq!(parse_quantity_string("42mg"), Some((42.0, Some("mg".to_string()))));
        assert_eq!(parse_quantity_string("10.5kg"), Some((10.5, Some("kg".to_string()))));
        assert_eq!(parse_quantity_string("100'wk'"), Some((100.0, Some("wk".to_string()))));
        
        // Test scientific notation
        assert_eq!(parse_quantity_string("1.23e2"), Some((123.0, None)));
        assert_eq!(parse_quantity_string("1.5E-3 mg"), Some((0.0015, Some("mg".to_string()))));
        
        // Test invalid formats
        assert_eq!(parse_quantity_string("invalid"), None);
        assert_eq!(parse_quantity_string("mg42"), None);
        assert_eq!(parse_quantity_string(""), None);
    }

    #[test]
    fn test_complex_units() {
        let op = ToQuantityFunction;

        // Test complex UCUM units
        let context = create_context(FhirPathValue::String("5.0 mg/kg".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(5.0, "mg/kg");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));

        // Test units with numbers
        let context = create_context(FhirPathValue::String("25 mg/m2".into()));
        let result = op.execute(&[], &context).unwrap();
        let expected_quantity = Quantity::new(25.0, "mg/m2");
        assert_eq!(result, FhirPathValue::Quantity(expected_quantity));
    }
}