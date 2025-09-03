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

//! Logical operations evaluator

use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_core::FhirPathValue;

/// Specialized evaluator for logical operations
pub struct LogicalEvaluator;

impl LogicalEvaluator {
    /// Helper to handle collection extraction for logical operations
    fn extract_operands<'a>(
        left: &'a FhirPathValue,
        right: &'a FhirPathValue,
    ) -> (Option<&'a FhirPathValue>, Option<&'a FhirPathValue>) {
        let left_val = match left {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first()
                } else if items.is_empty() {
                    None
                } else {
                    None // Multi-element collections not supported for logical operations
                }
            }
            FhirPathValue::Empty => None,
            val => Some(val),
        };

        let right_val = match right {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first()
                } else if items.is_empty() {
                    None
                } else {
                    None // Multi-element collections not supported for logical operations
                }
            }
            FhirPathValue::Empty => None,
            val => Some(val),
        };

        (left_val, right_val)
    }

    /// Helper to extract boolean value or convert to boolean per FHIRPath rules
    fn to_boolean(value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Integer(i) => Some(*i != 0),
            FhirPathValue::Decimal(d) => Some(!d.is_zero()),
            FhirPathValue::String(s) => Some(!s.is_empty()),
            FhirPathValue::Collection(items) => Some(!items.is_empty()),
            FhirPathValue::Empty => None,
            _ => Some(true), // Other types are considered truthy in FHIRPath
        }
    }

    /// Evaluate logical AND operation (FHIRPath three-valued logic)
    pub async fn evaluate_and(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (
            left_val.and_then(Self::to_boolean),
            right_val.and_then(Self::to_boolean),
        ) {
            (Some(false), _) | (_, Some(false)) => Ok(FhirPathValue::Boolean(false)), // Short-circuit: false AND anything = false
            (Some(true), Some(true)) => Ok(FhirPathValue::Boolean(true)),
            _ => Ok(FhirPathValue::Empty), // Any empty/null operand with non-false other = empty
        }
    }

    /// Evaluate logical OR operation (FHIRPath three-valued logic)
    pub async fn evaluate_or(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (
            left_val.and_then(Self::to_boolean),
            right_val.and_then(Self::to_boolean),
        ) {
            (Some(true), _) | (_, Some(true)) => Ok(FhirPathValue::Boolean(true)), // Short-circuit: true OR anything = true
            (Some(false), Some(false)) => Ok(FhirPathValue::Boolean(false)),
            _ => Ok(FhirPathValue::Empty), // Any empty/null operand with non-true other = empty
        }
    }

    /// Evaluate logical XOR operation
    pub async fn evaluate_xor(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        match (
            left_val.and_then(Self::to_boolean),
            right_val.and_then(Self::to_boolean),
        ) {
            (Some(left_bool), Some(right_bool)) => {
                Ok(FhirPathValue::Boolean(left_bool != right_bool))
            }
            _ => Ok(FhirPathValue::Empty), // Any empty operand = empty result
        }
    }

    /// Evaluate logical NOT operation
    pub async fn evaluate_not(operand: &FhirPathValue) -> EvaluationResult<FhirPathValue> {
        let value = match operand {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    items.first().unwrap()
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            val => val,
        };

        match Self::to_boolean(value) {
            Some(b) => Ok(FhirPathValue::Boolean(!b)),
            None => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate implies operation (FHIRPath logical implication)
    pub async fn evaluate_implies(
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> EvaluationResult<FhirPathValue> {
        let (left_val, right_val) = Self::extract_operands(left, right);

        // Logical implication: A implies B = (not A) or B
        // In three-valued logic:
        // - false implies anything = true
        // - true implies true = true
        // - true implies false = false
        // - empty implies true = true
        // - empty implies false = empty
        // - empty implies empty = empty
        // - true implies empty = empty
        match (
            left_val.and_then(Self::to_boolean),
            right_val.and_then(Self::to_boolean),
        ) {
            (Some(false), _) => Ok(FhirPathValue::Boolean(true)), // false implies anything = true
            (None, Some(true)) => Ok(FhirPathValue::Boolean(true)), // empty implies true = true
            (None, Some(false)) => Ok(FhirPathValue::Empty),      // empty implies false = empty
            (None, None) => Ok(FhirPathValue::Empty),             // empty implies empty = empty
            (Some(true), Some(true)) => Ok(FhirPathValue::Boolean(true)),
            (Some(true), Some(false)) => Ok(FhirPathValue::Boolean(false)),
            (Some(true), None) => Ok(FhirPathValue::Empty), // true implies empty = empty
        }
    }
}
