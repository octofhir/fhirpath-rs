//! not() function - logical negation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::FunctionSignature;

/// not() function - logical negation
pub struct NotFunction;

impl FhirPathFunction for NotFunction {
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

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)]))
            }
            FhirPathValue::Integer(i) => {
                // Per FHIRPath spec: 0 is false, non-zero is true
                let bool_val = *i != 0;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    !bool_val,
                )]))
            }
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection is false, not becomes true
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        true,
                    )]))
                } else if items.len() == 1 {
                    match items.iter().next() {
                        Some(FhirPathValue::Boolean(b)) => {
                            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)]))
                        }
                        Some(FhirPathValue::Integer(i)) => {
                            // Per FHIRPath spec: 0 is false, non-zero is true
                            let bool_val = *i != 0;
                            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                                !bool_val,
                            )]))
                        }
                        _ => Ok(FhirPathValue::Empty),
                    }
                } else {
                    // Multiple items - return empty per FHIRPath spec for not()
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                true,
            )])),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
