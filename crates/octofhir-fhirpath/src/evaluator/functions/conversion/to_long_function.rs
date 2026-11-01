//! toLong() and convertsToLong() (FHIRPath §1.5.5.3.3, §1.5.5.3.4, STU)
//!
//! An Integer or Long converts as is, a Boolean to 1 or 0, and a String when it matches
//! `(\+|-)?\d+` and fits in 64 bits. Anything else gives empty (`toLong`) or false
//! (`convertsToLong`).

use std::sync::Arc;

use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// The Long a single value converts to, if any.
pub(crate) fn to_long(value: &FhirPathValue) -> Option<i64> {
    match value {
        FhirPathValue::Integer(i, _, _) => Some(*i),
        FhirPathValue::Boolean(b, _, _) => Some(i64::from(*b)),
        FhirPathValue::String(s, _, _) => {
            let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            s.strip_prefix('+').unwrap_or(s).parse::<i64>().ok()
        }
        _ => None,
    }
}

fn metadata(name: &str, description: &str, return_type: &str) -> FunctionMetadata {
    FunctionMetadata {
        name: name.to_string(),
        description: description.to_string(),
        signature: FunctionSignature {
            input_type: "Any".to_string(),
            parameters: vec![],
            return_type: return_type.to_string(),
            polymorphic: false,
            min_params: 0,
            max_params: Some(0),
        },
        argument_evaluation: ArgumentEvaluationStrategy::Current,
        null_propagation: NullPropagationStrategy::Focus,
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::Conversion,
        requires_terminology: false,
        requires_model: false,
    }
}

/// The single input item, empty for an empty input, an error for several.
fn single_input<'a>(name: &str, input: &'a Collection) -> Result<Option<&'a FhirPathValue>> {
    match input.len() {
        0 => Ok(None),
        1 => Ok(input.first()),
        _ => Err(FhirPathError::evaluation_error(
            crate::core::error_code::FP0054,
            format!("{name} can only be called on a single item"),
        )),
    }
}

/// toLong() function evaluator
pub struct ToLongFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToLongFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: metadata("toLong", "Converts a value to a Long", "Long"),
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ToLongFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, _args: Vec<Collection>) -> Result<EvaluationResult> {
        let value = single_input("toLong", &input)?
            .and_then(to_long)
            .map(FhirPathValue::long);
        Ok(EvaluationResult {
            value: value
                .map(Collection::single)
                .unwrap_or_else(Collection::empty),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

/// convertsToLong() function evaluator
pub struct ConvertsToLongFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToLongFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: metadata(
                "convertsToLong",
                "Whether a value converts to a Long",
                "Boolean",
            ),
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToLongFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, _args: Vec<Collection>) -> Result<EvaluationResult> {
        let value = single_input("convertsToLong", &input)?
            .map(|value| FhirPathValue::boolean(to_long(value).is_some()));
        Ok(EvaluationResult {
            value: value
                .map(Collection::single)
                .unwrap_or_else(Collection::empty),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_per_spec() {
        assert_eq!(to_long(&FhirPathValue::integer(42)), Some(42));
        assert_eq!(to_long(&FhirPathValue::boolean(true)), Some(1));
        assert_eq!(to_long(&FhirPathValue::string("+123")), Some(123));
        assert_eq!(
            to_long(&FhirPathValue::string("-9223372036854775808")),
            Some(i64::MIN)
        );
        assert_eq!(to_long(&FhirPathValue::string("9223372036854775808")), None);
        assert_eq!(to_long(&FhirPathValue::string("1.0")), None);
        assert_eq!(to_long(&FhirPathValue::string(" 1")), None);
        assert_eq!(to_long(&FhirPathValue::string("abc")), None);
        assert_eq!(to_long(&FhirPathValue::decimal(1)), None);
    }
}
