//! round() function - rounds to nearest integer

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use rust_decimal::prelude::*;

/// round() function - rounds to nearest integer
pub struct RoundFunction;

impl FhirPathFunction for RoundFunction {
    fn name(&self) -> &str {
        "round"
    }
    fn human_friendly_name(&self) -> &str {
        "Round"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "round",
                vec![ParameterInfo::optional("precision", TypeInfo::Integer)],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // round() is a pure mathematical function
    }

    fn documentation(&self) -> &str {
        "Rounds the decimal to the nearest whole number using a traditional round (i.e. 0.5 or higher will round to 1). If specified, the precision argument determines the decimal place at which the rounding will occur."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle single-element collections (common in method calls like (1.5).round())
        let input_value = match &context.input {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
            other => other,
        };

        match input_value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => {
                if let Some(FhirPathValue::Integer(precision)) = args.first() {
                    Ok(FhirPathValue::Decimal(d.round_dp(*precision as u32)))
                } else {
                    Ok(FhirPathValue::Integer(d.round().to_i64().unwrap_or(0)))
                }
            }
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
