//! floor() function - rounds down to nearest integer

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::prelude::*;

/// floor() function - rounds down to nearest integer
pub struct FloorFunction;

impl FhirPathFunction for FloorFunction {
    fn name(&self) -> &str {
        "floor"
    }
    fn human_friendly_name(&self) -> &str {
        "Floor"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("floor", vec![], TypeInfo::Integer));
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // floor() is a pure mathematical function
    }
    
    fn documentation(&self) -> &str {
        "Returns the first integer less than or equal to the input."
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle single-element collections (common in method calls like (1.5).floor())
        let input_value = match &context.input {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
            other => other,
        };

        match input_value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => {
                Ok(FhirPathValue::Integer(d.floor().to_i64().unwrap_or(0)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Number".to_string(),
                actual: format!("{:?}", input_value),
            }),
        }
    }
}