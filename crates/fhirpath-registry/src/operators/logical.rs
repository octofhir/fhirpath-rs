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

//! Logical operators for FHIRPath expressions

use super::super::operator::{
    Associativity, FhirPathOperator, OperatorError, OperatorRegistry, OperatorResult,
};
use crate::signature::OperatorSignature;
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// Logical AND operator
pub struct AndOperator;

impl FhirPathOperator for AndOperator {
    fn symbol(&self) -> &str {
        "and"
    }
    fn human_friendly_name(&self) -> &str {
        "Logical And"
    }
    fn precedence(&self) -> u8 {
        2
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "and",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // FHIRPath logical AND semantics with truthiness:
        // - Uses boolean conversion for non-boolean types
        // - Empty collections are false, non-empty are true
        // - Numbers: 0 is false, others are true
        // - Strings: empty is false, non-empty is true

        // Convert both operands to boolean using FHIRPath truthiness rules
        let left_truthy = self.to_boolean(left);
        let right_truthy = self.to_boolean(right);

        match (left_truthy, right_truthy) {
            // If either side is empty, special handling
            (None, _) | (_, None) => {
                // FHIRPath AND rules for empty:
                // false and empty = false
                // empty and false = false
                // true and empty = empty
                // empty and true = empty
                // empty and empty = empty
                if left_truthy == Some(false) || right_truthy == Some(false) {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                        false,
                    )]))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            // Both have boolean values
            (Some(a), Some(b)) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                a && b,
            )])),
        }
    }
}

impl AndOperator {
    /// Convert a FhirPathValue to boolean using FHIRPath truthiness rules
    /// Returns None if the value is empty (which has special semantics in logical operations)
    fn to_boolean(&self, value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Empty => None,
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Integer(i) => Some(*i != 0),
            FhirPathValue::Decimal(d) => Some(!d.is_zero()),
            FhirPathValue::String(s) => Some(!s.is_empty()),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    None
                } else {
                    Some(true) // Non-empty collections are truthy
                }
            }
            // All other types are considered truthy if they exist
            _ => Some(true),
        }
    }
}

/// Logical OR operator
pub struct OrOperator;

impl FhirPathOperator for OrOperator {
    fn symbol(&self) -> &str {
        "or"
    }
    fn human_friendly_name(&self) -> &str {
        "Logical Or"
    }
    fn precedence(&self) -> u8 {
        1
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "or",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // FHIRPath logical OR semantics:
        // - true or true = true
        // - true or false = true
        // - false or true = true
        // - false or false = false
        // - true or empty = true
        // - false or empty = empty
        // - empty or true = true
        // - empty or false = empty
        // - empty or empty = empty

        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(*a || *b)]),
            ),
            // If left is true, result is always true (short-circuit)
            (FhirPathValue::Boolean(true), _) if right.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(true)],
            )),
            // If right is true, result is always true (short-circuit)
            (_, FhirPathValue::Boolean(true)) if left.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(true)],
            )),
            // If either operand is empty (and the other is not true), result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical XOR operator
pub struct XorOperator;

impl FhirPathOperator for XorOperator {
    fn symbol(&self) -> &str {
        "xor"
    }
    fn human_friendly_name(&self) -> &str {
        "Exclusive Or"
    }
    fn precedence(&self) -> u8 {
        1
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "xor",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(*a ^ *b)]),
            ),
            // If either operand is empty, result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical IMPLIES operator
pub struct ImpliesOperator;

impl FhirPathOperator for ImpliesOperator {
    fn symbol(&self) -> &str {
        "implies"
    }
    fn human_friendly_name(&self) -> &str {
        "Implies"
    }
    fn precedence(&self) -> u8 {
        1
    }
    fn associativity(&self) -> Associativity {
        Associativity::Right
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "implies",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                // A implies B is equivalent to (not A) or B
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                    !*a || *b,
                )]))
            }
            // false implies empty = true (because false implies anything is true)
            (FhirPathValue::Boolean(false), _) if right.is_empty() => Ok(
                FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]),
            ),
            // empty implies true = true (because empty is considered false-like, so !empty || true = true)
            (_, FhirPathValue::Boolean(true)) if left.is_empty() => Ok(FhirPathValue::collection(
                vec![FhirPathValue::Boolean(true)],
            )),
            // If either operand is empty (and not handled above), result is empty
            _ if left.is_empty() || right.is_empty() => Ok(FhirPathValue::Empty),
            _ => Err(OperatorError::InvalidOperandTypes {
                operator: self.symbol().to_string(),
                left_type: left.type_name().to_string(),
                right_type: right.type_name().to_string(),
            }),
        }
    }
}

/// Logical NOT operator
pub struct NotOperator;

impl FhirPathOperator for NotOperator {
    fn symbol(&self) -> &str {
        "not"
    }
    fn human_friendly_name(&self) -> &str {
        "Logical Not"
    }
    fn precedence(&self) -> u8 {
        8
    }
    fn associativity(&self) -> Associativity {
        Associativity::Right
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::unary(
                "not",
                TypeInfo::Boolean,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        _left: &FhirPathValue,
        _right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        Err(OperatorError::EvaluationError {
            operator: self.symbol().to_string(),
            message: "NOT is a unary operator".to_string(),
        })
    }

    fn evaluate_unary(&self, operand: &FhirPathValue) -> OperatorResult<FhirPathValue> {
        match operand {
            FhirPathValue::Boolean(b) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!*b)]))
            }
            _ => Err(OperatorError::InvalidUnaryOperandType {
                operator: self.symbol().to_string(),
                operand_type: operand.type_name().to_string(),
            }),
        }
    }
}

/// Register all logical operators
pub fn register_logical_operators(registry: &mut OperatorRegistry) {
    registry.register(AndOperator);
    registry.register(OrOperator);
    registry.register(XorOperator);
    registry.register(ImpliesOperator);
    registry.register(NotOperator);
}
