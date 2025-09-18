//! Today function implementation
//!
//! The today function returns the current date.
//! Syntax: today()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::temporal::PrecisionDate;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Today function evaluator
pub struct TodayFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl TodayFunctionEvaluator {
    /// Create a new today function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "today".to_string(),
                description: "Returns the current date".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "Date".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: false, // Current date is non-deterministic
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for TodayFunctionEvaluator {
    async fn evaluate(
        &self,
        _input: Vec<FhirPathValue>,
        _context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if !args.is_empty() {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "today function takes no arguments".to_string(),
            ));
        }

        // Get current system time
        let now = chrono::Utc::now();

        // Create a PrecisionDate from the current date with day precision
        use crate::core::TemporalPrecision;
        let current_date = now.date_naive();
        let precision_date = PrecisionDate::new(current_date, TemporalPrecision::Day);

        let result = FhirPathValue::date(precision_date);

        Ok(EvaluationResult {
            value: crate::core::Collection::single(result),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
