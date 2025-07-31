//! allTrue() function - returns true if all items in collection are true

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::FunctionSignature;

/// allTrue() function - returns true if all items in collection are true
pub struct AllTrueFunction;

impl FhirPathFunction for AllTrueFunction {
    fn name(&self) -> &str {
        "allTrue"
    }
    fn human_friendly_name(&self) -> &str {
        "All True"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("allTrue", vec![], TypeInfo::Boolean)
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // allTrue() is a pure boolean function
    }

    fn documentation(&self) -> &str {
        "Takes a collection of Boolean values and returns `true` if all the items are `true`. If any items are `false`, the result is `false`. If the input is empty (`{ }`), the result is `true`."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    true,
                )]));
            } // Empty collection is vacuously true
            single => {
                // Single item - check if it's a boolean true
                match single {
                    FhirPathValue::Boolean(b) => {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(*b)]));
                    }
                    _ => {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                            false,
                        )]));
                    }
                }
            }
        };

        // All items must be boolean true
        for item in items.iter() {
            match item {
                FhirPathValue::Boolean(true) => continue,
                _ => {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        false,
                    )]));
                }
            }
        }
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            true,
        )]))
    }
}
