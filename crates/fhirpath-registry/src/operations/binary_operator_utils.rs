// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Utility functions for binary operators following FHIRPath specification

use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Extract singleton values from FHIRPath values according to FHIRPath specification.
///
/// This is used for arithmetic and logical operators that require singleton collections.
/// Per the FHIRPath specification:
/// - If either operand is empty, the function should handle this appropriately
/// - If either operand has more than one element, an error should be thrown
/// - Only singleton collections or single values should be processed
///
/// Returns a tuple of (left_value, right_value) where each value is either:
/// - FhirPathValue::Empty if the collection was empty
/// - The single value if the collection had exactly one element
/// - The original value if it wasn't a collection
pub fn extract_singleton_values(
    left: &FhirPathValue,
    right: &FhirPathValue,
) -> Result<(FhirPathValue, FhirPathValue)> {
    let left_val = match left {
        FhirPathValue::Collection(items) => {
            match items.len() {
                0 => FhirPathValue::Empty,
                1 => items.get(0).unwrap().clone(),
                _ => return Err(FhirPathError::EvaluationError {
                    message: "Binary operators require singleton collections (collections with 0 or 1 elements)".to_string(),
                }),
            }
        },
        single => single.clone(),
    };

    let right_val = match right {
        FhirPathValue::Collection(items) => {
            match items.len() {
                0 => FhirPathValue::Empty,
                1 => items.get(0).unwrap().clone(),
                _ => return Err(FhirPathError::EvaluationError {
                    message: "Binary operators require singleton collections (collections with 0 or 1 elements)".to_string(),
                }),
            }
        },
        single => single.clone(),
    };

    Ok((left_val, right_val))
}

/// Extract values for collection-aware operators (like equality) according to FHIRPath specification.
///
/// This is used for operators that can handle multi-element collections by comparing them element-by-element.
/// Returns the values as-is, allowing the operator implementation to handle collection logic.
pub fn extract_collection_values(
    left: &FhirPathValue,
    right: &FhirPathValue,
) -> (FhirPathValue, FhirPathValue) {
    // For collection-aware operators, we return the values as-is
    // The operator implementation handles the collection comparison logic
    (left.clone(), right.clone())
}

/// Evaluate a binary operator with proper singleton handling according to FHIRPath specification.
///
/// This function:
/// 1. Extracts singleton values from the operands
/// 2. Returns empty if either operand is empty
/// 3. Calls the provided comparison function with the extracted values
///
/// # Arguments
/// * `left` - Left operand (may be a collection)
/// * `right` - Right operand (may be a collection)
/// * `compare_fn` - Function that performs the actual comparison on single values
///
/// # Returns
/// * `FhirPathValue::Empty` if either operand is empty
/// * `FhirPathValue::Boolean(result)` if both operands are singleton values
/// * Error if either operand is a multi-element collection
pub fn evaluate_binary_operator<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    compare_fn: F,
) -> Result<FhirPathValue>
where
    F: FnOnce(&FhirPathValue, &FhirPathValue) -> Result<bool>,
{
    // Handle collections according to FHIRPath specification
    let (left_val, right_val) = extract_singleton_values(left, right)?;

    // If either operand is empty, return empty
    if matches!(left_val, FhirPathValue::Empty) || matches!(right_val, FhirPathValue::Empty) {
        return Ok(FhirPathValue::Empty);
    }

    let result = compare_fn(&left_val, &right_val)?;
    Ok(FhirPathValue::singleton(FhirPathValue::Boolean(result)))
}

/// Evaluate a binary operator that returns Optional<bool> results
pub fn evaluate_binary_operator_optional<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    compare_fn: F,
) -> Result<FhirPathValue>
where
    F: FnOnce(&FhirPathValue, &FhirPathValue) -> Result<Option<bool>>,
{
    // Handle collections according to FHIRPath specification
    let (left_val, right_val) = extract_singleton_values(left, right)?;

    // If either operand is empty, return empty
    if matches!(left_val, FhirPathValue::Empty) || matches!(right_val, FhirPathValue::Empty) {
        return Ok(FhirPathValue::Empty);
    }

    let result = compare_fn(&left_val, &right_val)?;
    match result {
        Some(bool_result) => Ok(FhirPathValue::singleton(FhirPathValue::Boolean(
            bool_result,
        ))),
        None => Ok(FhirPathValue::Empty),
    }
}

/// Evaluate an arithmetic binary operator with proper singleton handling according to FHIRPath specification.
///
/// This function is similar to evaluate_binary_operator but returns Empty collection for empty operands,
/// as per FHIRPath arithmetic operation behavior.
///
/// # Arguments
/// * `left` - Left operand (may be a collection)
/// * `right` - Right operand (may be a collection)
/// * `arithmetic_fn` - Function that performs the actual arithmetic operation on single values
///
/// # Returns
/// * `FhirPathValue::Collection(empty)` if either operand is empty
/// * `FhirPathValue::Collection([result])` containing the arithmetic result
/// * Error if either operand is a multi-element collection
pub fn evaluate_arithmetic_operator<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    arithmetic_fn: F,
) -> Result<FhirPathValue>
where
    F: FnOnce(&FhirPathValue, &FhirPathValue) -> Result<FhirPathValue>,
{
    use octofhir_fhirpath_model::Collection;

    // Handle collections according to FHIRPath specification
    let (left_val, right_val) = extract_singleton_values(left, right)?;

    // If either operand is empty, return empty collection (per FHIRPath arithmetic behavior)
    if matches!(left_val, FhirPathValue::Empty) || matches!(right_val, FhirPathValue::Empty) {
        return Ok(FhirPathValue::Collection(Collection::from(vec![])));
    }

    let result = arithmetic_fn(&left_val, &right_val)?;

    // If the arithmetic function returns Empty (e.g., division by zero), return empty collection
    match result {
        FhirPathValue::Empty => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
        value => Ok(FhirPathValue::Collection(Collection::from(vec![value]))),
    }
}

/// Evaluate a logical binary operator with proper singleton handling according to FHIRPath specification.
///
/// Logical operators in FHIRPath follow three-valued logic (true, false, empty).
/// This function handles the special logic for empty values according to the operator type.
///
/// # Arguments
/// * `left` - Left operand (may be a collection)
/// * `right` - Right operand (may be a collection)
/// * `logical_fn` - Function that performs the actual logical operation, returning Option<bool>
///   where None represents the empty value in three-valued logic
///
/// # Returns
/// * `FhirPathValue::Boolean(result)` for definite true/false results
/// * `FhirPathValue::Empty` for indeterminate results
/// * Error if either operand is a multi-element collection
pub fn evaluate_logical_operator<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    logical_fn: F,
) -> Result<FhirPathValue>
where
    F: FnOnce(&FhirPathValue, &FhirPathValue) -> Result<Option<bool>>,
{
    // Handle collections according to FHIRPath specification
    let (left_val, right_val) = extract_singleton_values(left, right)?;

    let result = logical_fn(&left_val, &right_val)?;
    match result {
        Some(bool_result) => Ok(FhirPathValue::singleton(FhirPathValue::Boolean(
            bool_result,
        ))),
        None => Ok(FhirPathValue::Empty),
    }
}

/// Evaluate a collection-aware binary operator according to FHIRPath specification.
///
/// This is used for operators like equality (=) that can handle multi-element collections
/// by comparing them element-by-element according to specific rules.
///
/// # Arguments
/// * `left` - Left operand (may be a collection)
/// * `right` - Right operand (may be a collection)
/// * `collection_fn` - Function that performs the actual comparison with full collection handling
///
/// # Returns
/// * `FhirPathValue::Boolean(result)` for definite true/false results
/// * `FhirPathValue::Empty` for indeterminate results
pub fn evaluate_collection_aware_operator<F>(
    left: &FhirPathValue,
    right: &FhirPathValue,
    collection_fn: F,
) -> Result<FhirPathValue>
where
    F: FnOnce(&FhirPathValue, &FhirPathValue) -> Result<Option<bool>>,
{
    // For collection-aware operators, we pass the values directly to the operator
    // which implements the full FHIRPath collection comparison logic
    let (left_val, right_val) = extract_collection_values(left, right);

    let result = collection_fn(&left_val, &right_val)?;
    match result {
        Some(bool_result) => Ok(FhirPathValue::Boolean(bool_result)),
        None => Ok(FhirPathValue::Empty),
    }
}
