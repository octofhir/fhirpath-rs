//! ConvertsToDateTime function implementation
//!
//! This function tests if a value can be converted to a DateTime.

use crate::core::{Collection, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use std::sync::Arc;

/// ConvertsToDateTime function evaluator
pub struct ConvertsToDateTimeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ConvertsToDateTimeFunctionEvaluator {
    /// Create a new convertsToDateTime function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "convertsToDateTime".to_string(),
                description: "Tests if the input can be converted to a DateTime".to_string(),
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
impl PureFunctionEvaluator for ConvertsToDateTimeFunctionEvaluator {
    async fn evaluate(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        self.evaluate_sync(input, args)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn evaluate_sync(&self, input: Collection, args: Vec<Collection>) -> Result<EvaluationResult> {
        super::require_single("convertsToDateTime", &input)?;
        let format = super::date_format::format_arg(&args, "convertsToDateTime")?;

        let mut results = Vec::new();

        for value in input {
            let can_convert = match &value {
                FhirPathValue::String(s, _, _) if format.is_some() => {
                    super::date_format::parse_datetime_with_format(
                        s,
                        format.as_deref().unwrap_or(""),
                    )
                    .is_some()
                }
                FhirPathValue::String(s, _, _) => {
                    // Test if string can be parsed as DateTime, Date, or partial datetime (which can be converted to DateTime)
                    use crate::core::temporal::{PrecisionDate, PrecisionDateTime};

                    // First try parsing as a full DateTime
                    if PrecisionDateTime::parse(s).is_some() {
                        true
                    } else if PrecisionDate::parse(s).is_some() {
                        // Date can be converted to DateTime
                        true
                    } else {
                        // Try parsing partial datetime strings like "2015-02-04T14" or "2015-02-04T14:28"
                        // These are essentially datetime strings with partial time information
                        s.contains('T') && s.len() >= 13 // Minimum for "2015-02-04T14"
                    }
                }
                FhirPathValue::DateTime(_, _, _) => true, // Already a DateTime
                FhirPathValue::Date(_, _, _) => true,     // Date can be converted to DateTime
                _ => false, // Other types cannot be converted to DateTime
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
