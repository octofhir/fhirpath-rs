//! ceiling() function - rounds up to nearest integer

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;
use rust_decimal::prelude::*;

/// ceiling() function - rounds up to nearest integer
pub struct CeilingFunction;

#[async_trait]
impl AsyncFhirPathFunction for CeilingFunction {
    fn name(&self) -> &str {
        "ceiling"
    }
    fn human_friendly_name(&self) -> &str {
        "Ceiling"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("ceiling", vec![], TypeInfo::Integer)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // ceiling() is a pure mathematical function
    }

    fn documentation(&self) -> &str {
        "Returns the first integer greater than or equal to the input."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle single-element collections (common in method calls like (1.5).ceiling())
        let input_value = match &context.input {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
            other => other,
        };

        match input_value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Integer(d.ceil().to_i64().unwrap_or(0))),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Number".to_string(),
                actual: format!("{input_value:?}"),
            }),
        }
    }
}
