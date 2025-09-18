//! Now function implementation
//!
//! The now function returns the current date and time.
//! Syntax: now()

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::temporal::PrecisionDateTime;
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    EmptyPropagation, FunctionCategory, FunctionEvaluator, FunctionMetadata, FunctionSignature,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Now function evaluator
pub struct NowFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl NowFunctionEvaluator {
    /// Create a new now function evaluator
    pub fn create() -> Arc<dyn FunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "now".to_string(),
                description: "Returns the current date and time".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![],
                    return_type: "DateTime".to_string(),
                    polymorphic: false,
                    min_params: 0,
                    max_params: Some(0),
                },
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: false, // Current time is non-deterministic
                category: FunctionCategory::Utility,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl FunctionEvaluator for NowFunctionEvaluator {
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
                "now function takes no arguments".to_string(),
            ));
        }

        // Get current system time as PrecisionDateTime
        let now = chrono::Utc::now();

        // Convert to fixed offset (UTC)
        let fixed_offset_dt = now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

        // Create a PrecisionDateTime from the current time with full precision
        let precision_dt = PrecisionDateTime::from_datetime(fixed_offset_dt);

        let result = FhirPathValue::datetime(precision_dt);

        Ok(EvaluationResult {
            value: crate::core::Collection::single(result),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
