//! ConvertsToDate function implementation
//!
//! The convertsToDate function tests whether a value can be converted to a date.
//! Syntax: value.convertsToDate()

use std::sync::Arc;

use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};

/// ConvertsToDate function evaluator
pub struct ConvertsToDateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToDateFunctionEvaluator {
    /// Create a new convertsToDate function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToDate".to_string(),
                description: "Tests whether a value can be converted to a date".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "format".to_string(),
                        parameter_type: vec!["String".to_string()],
                        optional: true,
                        is_expression: false,
                        description: "Optional date/time format string".to_string(),
                        default_value: None,
                    }],
                    return_type: "Boolean".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Conversion,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ConvertsToDateFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        super::require_single("convertsToDate", &input)?;
        let format = super::date_format::format_arg(&args, "convertsToDate")?;

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::Date(_, _, _) => true,
                FhirPathValue::DateTime(_, _, _) => true,
                FhirPathValue::String(str_val, _, _) if format.is_some() => {
                    super::date_format::parse_date_with_format(
                        str_val,
                        format.as_deref().unwrap_or(""),
                    )
                    .is_some()
                }
                FhirPathValue::String(str_val, _, _) => {
                    // Try to parse the string as a date using the temporal module
                    use crate::core::temporal::PrecisionDate;
                    PrecisionDate::parse(str_val).is_some()
                }
                _ => false, // Other types cannot be converted to date
            };

            results.push(FhirPathValue::boolean(can_convert));
        }

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
