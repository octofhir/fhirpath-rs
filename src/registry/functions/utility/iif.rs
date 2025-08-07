//! iif() function - conditional expression (if-then-else)

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

/// iif() function - conditional expression (if-then-else)
pub struct IifFunction;

#[async_trait]
impl AsyncFhirPathFunction for IifFunction {
    fn name(&self) -> &str {
        "iif"
    }
    fn human_friendly_name(&self) -> &str {
        "If"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "iif",
                vec![
                    ParameterInfo::required("condition", TypeInfo::Any),
                    ParameterInfo::required("true_value", TypeInfo::Any),
                    ParameterInfo::optional("false_value", TypeInfo::Any),
                ],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn documentation(&self) -> &str {
        "An immediate if function that returns the `true_value` if the `condition` evaluates to `true`, or the `false_value` otherwise. If `false_value` is not provided and the condition is false, an empty collection is returned."
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Check context collection constraints first
        // iif can only work on empty collections or single items
        match &context.input {
            FhirPathValue::Collection(coll) => {
                if coll.len() > 1 {
                    // Collections with more than one item return empty
                    return Ok(FhirPathValue::Empty);
                }
                // Empty collections or single items continue
            }
            _ => {
                // Single values are fine
            }
        }

        // Handle condition using FHIRPath truthiness rules
        let condition = match &args[0] {
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    // Multi-item collections make the whole iif return empty
                    return Ok(FhirPathValue::Empty);
                } else if items.is_empty() {
                    false // Empty collection is falsy
                } else {
                    // Single item collection - evaluate the item
                    match items.first().unwrap() {
                        FhirPathValue::Boolean(b) => *b,
                        FhirPathValue::Integer(i) => *i != 0,
                        FhirPathValue::Decimal(d) => !d.is_zero(),
                        FhirPathValue::String(s) => !s.is_empty(),
                        FhirPathValue::Empty => false,
                        _ => true, // Most other types are truthy when present
                    }
                }
            }
            FhirPathValue::Integer(i) => *i != 0,
            FhirPathValue::Decimal(d) => !d.is_zero(),
            FhirPathValue::String(s) => !s.is_empty(),
            _ => true, // Most other types are truthy when present
        };

        if condition {
            Ok(args[1].clone())
        } else {
            Ok(args.get(2).cloned().unwrap_or(FhirPathValue::Empty))
        }
    }
}
