//! Comparable function implementation - sync version

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use crate::{FhirPathValue};
use FhirPathValue::*;

/// Comparable function - checks if two values can be compared
#[derive(Debug, Clone)]
pub struct ComparableFunction;

impl ComparableFunction {
    pub fn new() -> Self {
        Self
    }

    fn can_compare_values(left: &FhirPathValue, right: &FhirPathValue) -> bool {
        match (left, right) {
            (Integer(_), Integer(_)) => true,
            (Decimal(_), Decimal(_)) => true,
            (Integer(_), Decimal(_)) => true,
            (Decimal(_), Integer(_)) => true,
            (String(_), String(_)) => true,
            (Date(_), Date(_)) => true,
            (DateTime(_), DateTime(_)) => true,
            (Time(_), Time(_)) => true,
            (Boolean(_), Boolean(_)) => true,
            (Quantity { .. }, Quantity { .. }) => true, // Simplified - no unit checking
            _ => false,
        }
    }

    // Simplified implementation for JSON-based values
}

impl SyncOperation for ComparableFunction {
    fn name(&self) -> &'static str {
        "comparable"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "comparable",
                parameters: vec![ParameterType::Any],
                return_type: ValueType::Boolean,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "comparable".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let can_compare = Self::can_compare_values(&context.input, &args[0]);
        Ok(FhirPathValue::Boolean(can_compare))
    }
}

impl Default for ComparableFunction {
    fn default() -> Self {
        Self::new()
    }
}
