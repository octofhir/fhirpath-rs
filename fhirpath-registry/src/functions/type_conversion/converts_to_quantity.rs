//! convertsToQuantity() function - checks if value can be converted to quantity

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// convertsToQuantity() function - checks if value can be converted to quantity
pub struct ConvertsToQuantityFunction;

impl FhirPathFunction for ConvertsToQuantityFunction {
    fn name(&self) -> &str {
        "convertsToQuantity"
    }
    fn human_friendly_name(&self) -> &str {
        "Converts To Quantity"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("convertsToQuantity", vec![], TypeInfo::Boolean)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // convertsToQuantity() is a pure type conversion function
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

        let can_convert = match input_item {
            FhirPathValue::Quantity(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::String(s) => {
                // Check if string can be parsed as a number
                s.parse::<f64>().is_ok()
            }
            _ => false,
        };
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            can_convert,
        )]))
    }
}