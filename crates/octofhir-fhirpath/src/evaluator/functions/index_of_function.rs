//! IndexOf function implementation
//!
//! The indexOf function finds the index of a substring in a string or element in a collection.
//! Returns -1 if not found (following JavaScript convention).
//! Syntax: string.indexOf(substring) or collection.indexOf(item)

use std::sync::Arc;

use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, NullPropagationStrategy, PureFunctionEvaluator,
};
use crate::evaluator::EvaluationResult;

/// IndexOf function evaluator
pub struct IndexOfFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl IndexOfFunctionEvaluator {
    /// Create a new indexOf function evaluator
    pub fn create() -> Arc<dyn PureFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "indexOf".to_string(),
                description: "Returns the index of the first occurrence of a substring in a string or an item in a collection. Returns -1 if not found.".to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "searchValue".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "The substring to search for in a string, or the item to find in a collection".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Integer".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::StringManipulation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Find substring index in string
    fn find_substring_index(haystack: &str, needle: &str) -> i64 {
        match haystack.find(needle) {
            Some(index) => index as i64,
            None => -1,
        }
    }

    /// Find item index in collection
    fn find_item_index(collection: &[FhirPathValue], search_item: &FhirPathValue) -> i64 {
        for (index, item) in collection.iter().enumerate() {
            if Self::values_equal(item, search_item) {
                return index as i64;
            }
        }
        -1
    }

    /// Compare two FhirPathValues for equality (simplified comparison)
    fn values_equal(a: &FhirPathValue, b: &FhirPathValue) -> bool {
        match (a, b) {
            (FhirPathValue::String(s1, _, _), FhirPathValue::String(s2, _, _)) => s1 == s2,
            (FhirPathValue::Integer(i1, _, _), FhirPathValue::Integer(i2, _, _)) => i1 == i2,
            (FhirPathValue::Decimal(d1, _, _), FhirPathValue::Decimal(d2, _, _)) => d1 == d2,
            (FhirPathValue::Boolean(b1, _, _), FhirPathValue::Boolean(b2, _, _)) => b1 == b2,
            (FhirPathValue::Date(d1, _, _), FhirPathValue::Date(d2, _, _)) => d1 == d2,
            (FhirPathValue::DateTime(dt1, _, _), FhirPathValue::DateTime(dt2, _, _)) => dt1 == dt2,
            (FhirPathValue::Time(t1, _, _), FhirPathValue::Time(t2, _, _)) => t1 == t2,
            (FhirPathValue::Empty, FhirPathValue::Empty) => true,
            // Cross-type numeric comparisons
            (FhirPathValue::Integer(i, _, _), FhirPathValue::Decimal(d, _, _)) => {
                rust_decimal::Decimal::from(*i) == *d
            }
            (FhirPathValue::Decimal(d, _, _), FhirPathValue::Integer(i, _, _)) => {
                *d == rust_decimal::Decimal::from(*i)
            }
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl PureFunctionEvaluator for IndexOfFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                "indexOf function requires exactly one argument (searchValue)".to_string(),
            ));
        }

        // Handle empty input - propagate empty collections
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        // Get search value argument from pre-evaluated args
        let search_values = &args[0];

        // Handle empty search parameter - propagate empty collections
        if search_values.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::empty(),
            });
        }

        if search_values.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0056,
                "indexOf function search argument must evaluate to a single value".to_string(),
            ));
        }

        let search_value = &search_values[0];

        // Handle different input types
        let index = if input.len() == 1 {
            // Single input - could be string search or single-item collection
            match &input[0] {
                FhirPathValue::String(haystack, _, _) => {
                    // String search
                    if let FhirPathValue::String(needle, _, _) = search_value {
                        Self::find_substring_index(haystack, needle)
                    } else {
                        // Searching for non-string in string always returns -1
                        -1
                    }
                }
                _ => {
                    // Single item collection search
                    Self::find_item_index(&input, search_value)
                }
            }
        } else {
            // Multi-item collection search
            Self::find_item_index(&input, search_value)
        };

        Ok(EvaluationResult {
            value: crate::core::Collection::from(vec![FhirPathValue::integer(index)]),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
