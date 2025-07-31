//! toQuantity() function - converts value to quantity

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// toQuantity() function - converts value to quantity  
pub struct ToQuantityFunction;

impl FhirPathFunction for ToQuantityFunction {
    fn name(&self) -> &str {
        "toQuantity"
    }
    fn human_friendly_name(&self) -> &str {
        "To Quantity"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toQuantity", vec![], TypeInfo::Quantity)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toQuantity() is a pure type conversion function
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Quantity(q) => Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(q.clone())])),
            FhirPathValue::Integer(i) => {
                // Convert integer to quantity with unit "1" (dimensionless)
                let quantity = fhirpath_model::Quantity::new(
                    rust_decimal::Decimal::from(*i),
                    Some("1".to_string()),
                );
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(quantity)]))
            }
            FhirPathValue::Decimal(d) => {
                // Convert decimal to quantity with unit "1" (dimensionless)
                let quantity = fhirpath_model::Quantity::new(*d, Some("1".to_string()));
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(quantity)]))
            }
            FhirPathValue::String(s) => {
                // Try to parse string as quantity
                if let Ok(parsed) = s.parse::<f64>() {
                    let quantity = fhirpath_model::Quantity::new(
                        rust_decimal::Decimal::from_f64(parsed).unwrap_or_default(),
                        Some("1".to_string()),
                    );
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(quantity)]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}