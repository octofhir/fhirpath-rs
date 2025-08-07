//! not() function - logical negation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;

/// not() function - logical negation
pub struct NotFunction;

#[async_trait]
impl AsyncFhirPathFunction for NotFunction {
    fn name(&self) -> &str {
        "not"
    }
    fn human_friendly_name(&self) -> &str {
        "Not"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("not", vec![], TypeInfo::Boolean));
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // not() is a pure boolean function
    }

    fn documentation(&self) -> &str {
        "Returns `true` if the input collection evaluates to `false`, and `false` if it evaluates to `true`. Otherwise, the result is empty (`{ }`)."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
            FhirPathValue::Integer(i) => {
                // Per FHIRPath spec: 0 is false, non-zero is true
                let bool_val = *i != 0;
                Ok(FhirPathValue::Boolean(!bool_val))
            }
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection is false, not becomes true
                    Ok(FhirPathValue::Boolean(true))
                } else if items.len() == 1 {
                    match items.iter().next() {
                        Some(FhirPathValue::Boolean(b)) => Ok(FhirPathValue::Boolean(!b)),
                        Some(FhirPathValue::Integer(i)) => {
                            // Per FHIRPath spec: 0 is false, non-zero is true
                            let bool_val = *i != 0;
                            Ok(FhirPathValue::Boolean(!bool_val))
                        }
                        _ => Ok(FhirPathValue::Empty),
                    }
                } else {
                    // Multiple items - return empty per FHIRPath spec for not()
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
