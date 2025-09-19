//! Evaluation result types for FHIRPath evaluation
//!
//! This module defines the result types returned by FHIRPath evaluation.

use crate::core::{Collection, FhirPathValue};

/// Evaluation result containing the resulting collection
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Result collection (always a Collection per FHIRPath spec)
    pub value: Collection,
}

impl EvaluationResult {
    /// Create new evaluation result
    pub fn new(value: Collection) -> Self {
        Self { value }
    }

    /// Create evaluation result from values
    pub fn from_values(values: Vec<FhirPathValue>) -> Self {
        Self {
            value: Collection::from(values),
        }
    }

    /// Convert to ModelEvaluationResult for external interface
    pub fn to_evaluation_result(&self) -> octofhir_fhir_model::EvaluationResult {
        // Convert our Collection to ModelEvaluationResult format
        // TODO: Implement proper conversion when needed
        octofhir_fhir_model::EvaluationResult::Empty
    }

    /// Check if result represents true (for boolean evaluation)
    pub fn to_boolean(&self) -> bool {
        // Follow FHIRPath boolean conversion rules
        if self.value.is_empty() {
            false
        } else if self.value.len() == 1 {
            match self.value.iter().next() {
                Some(FhirPathValue::Boolean(b, _, _)) => *b,
                Some(_) => true, // Non-empty single value is truthy
                None => false,
            }
        } else {
            true // Multiple values are truthy
        }
    }
}

/// Evaluation result with comprehensive metadata for CLI debugging
#[derive(Debug, Clone)]
pub struct EvaluationResultWithMetadata {
    /// Core evaluation result
    pub result: EvaluationResult,
    /// Metadata collected during evaluation
    pub metadata: crate::evaluator::metadata_collector::EvaluationSummary,
}

impl EvaluationResultWithMetadata {
    /// Create new result with metadata
    pub fn new(
        result: EvaluationResult,
        metadata: crate::evaluator::metadata_collector::EvaluationSummary,
    ) -> Self {
        Self { result, metadata }
    }

    /// Get the core result
    pub fn result(&self) -> &EvaluationResult {
        &self.result
    }

    /// Get the metadata
    pub fn metadata(&self) -> &crate::evaluator::metadata_collector::EvaluationSummary {
        &self.metadata
    }
}
