//! comparable() function - checks if two quantities have compatible units

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

/// comparable() function - checks if two quantities have compatible units
pub struct ComparableFunction;

#[async_trait]
impl AsyncFhirPathFunction for ComparableFunction {
    fn name(&self) -> &str {
        "comparable"
    }
    fn human_friendly_name(&self) -> &str {
        "Comparable"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "comparable",
                vec![ParameterInfo::required("other", TypeInfo::Quantity)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let this_quantity = match &context.input {
            FhirPathValue::Quantity(q) => q,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Quantity".to_string(),
                    actual: context.input.type_name().to_string(),
                });
            }
        };

        let other_quantity = match &args[0] {
            FhirPathValue::Quantity(q) => q,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Quantity".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Check if quantities have compatible dimensions using existing method
        let result = this_quantity.has_compatible_dimensions(other_quantity);
        Ok(FhirPathValue::Boolean(result))
    }
}
