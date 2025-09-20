//! ToDate function implementation (stub)
//! TODO: Complete implementation

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::EvaluationResult;
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use std::sync::Arc;

pub struct ToDateFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl ToDateFunctionEvaluator {
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "toDate".to_string(),
                description: "Converts a value to a date".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Date".to_string(),
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
            },
        })
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for ToDateFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "toDate function takes no arguments".to_string(),
            ));
        }

        if input.len() != 1 {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        let result = match &input[0] {
            FhirPathValue::DateTime(precision_datetime, _, _) => {
                // Convert DateTime to Date by extracting just the date part
                let date = precision_datetime.date();
                Some(FhirPathValue::date(date))
            }
            FhirPathValue::Date(_precision_date, _, _) => {
                // Date is already a date, return as-is
                Some(input[0].clone())
            }
            FhirPathValue::String(s, _, _) => {
                // Try to parse string as date or datetime
                use crate::core::temporal::PrecisionDate;

                // First try parsing as a Date
                if let Some(precision_date) = PrecisionDate::parse(s) {
                    Some(FhirPathValue::date(precision_date))
                } else {
                    // Try parsing as a DateTime and extract date part
                    use crate::core::temporal::PrecisionDateTime;
                    if let Some(precision_datetime) = PrecisionDateTime::parse(s) {
                        let date = precision_datetime.date();
                        Some(FhirPathValue::date(date))
                    } else {
                        // String is not a valid date or datetime
                        None
                    }
                }
            }
            _ => {
                // Other types cannot be converted to date
                None
            }
        };

        Ok(EvaluationResult {
            value: match result {
                Some(date_value) => crate::core::Collection::from(vec![date_value]),
                None => crate::core::Collection::empty(),
            },
        })
    }
    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
