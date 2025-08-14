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

//! Unified operator trait with enhanced metadata support

use crate::enhanced_operator_metadata::EnhancedOperatorMetadata;
use async_trait::async_trait;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use crate::function::EvaluationContext;
use octofhir_fhirpath_model::FhirPathValue;

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Associativity {
    /// Left-to-right associativity (most operators)
    Left,
    /// Right-to-left associativity (rare)
    Right,
}

/// Result type for operator operations
pub type OperatorResult<T> = Result<T, OperatorError>;

/// Operator evaluation errors
#[derive(Debug, Clone, PartialEq)]
pub enum OperatorError {
    /// Invalid operand types for binary operation
    InvalidOperandTypes {
        operator: String,
        left_type: String,
        right_type: String,
    },
    /// Invalid operand type for unary operation
    InvalidUnaryOperandType {
        operator: String,
        operand_type: String,
    },
    /// General evaluation error
    EvaluationError {
        operator: String,
        message: String,
    },
    /// Incompatible units for quantity operations
    IncompatibleUnits {
        left_unit: String,
        right_unit: String,
    },
}

impl From<OperatorError> for EvaluationError {
    fn from(error: OperatorError) -> Self {
        match error {
            OperatorError::InvalidOperandTypes { operator, left_type, right_type } => {
                EvaluationError::Operator(format!(
                    "Operator '{}' cannot be applied to types {} and {}",
                    operator, left_type, right_type
                ))
            }
            OperatorError::InvalidUnaryOperandType { operator, operand_type } => {
                EvaluationError::Operator(format!(
                    "Operator '{}' cannot be applied to type {}",
                    operator, operand_type
                ))
            }
            OperatorError::EvaluationError { operator, message } => {
                EvaluationError::Operator(format!(
                    "Error evaluating operator '{}': {}",
                    operator, message
                ))
            }
            OperatorError::IncompatibleUnits { left_unit, right_unit } => {
                EvaluationError::InvalidOperation {
                    message: format!(
                        "Cannot perform operation with incompatible units: {} and {}",
                        left_unit, right_unit
                    )
                }
            }
        }
    }
}

/// Unified trait for FHIRPath operators with enhanced metadata
#[async_trait]
pub trait UnifiedFhirPathOperator: Send + Sync {
    /// Get the enhanced metadata for this operator
    fn metadata(&self) -> &EnhancedOperatorMetadata;

    /// Get the operator symbol (e.g., "+", "-", "=")
    fn symbol(&self) -> &str {
        &self.metadata().basic.symbol
    }

    /// Get a human-friendly name for the operator
    fn display_name(&self) -> &str {
        &self.metadata().basic.display_name
    }

    /// Get the operator precedence (higher values bind tighter)
    fn precedence(&self) -> u8 {
        self.metadata().basic.precedence
    }

    /// Get the operator associativity
    fn associativity(&self) -> Associativity {
        self.metadata().basic.associativity
    }

    /// Check if this operator supports binary operations
    fn supports_binary(&self) -> bool {
        self.metadata().basic.supports_binary
    }

    /// Check if this operator supports unary operations
    fn supports_unary(&self) -> bool {
        self.metadata().basic.supports_unary
    }

    /// Evaluate the operator with two operands
    async fn evaluate_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let _ = (left, right, context);
        Err(EvaluationError::InvalidOperation {
            message: format!(
                "Operator '{}' does not support binary operations",
                self.symbol()
            )
        })
    }

    /// Evaluate the operator with one operand (for unary operators)
    async fn evaluate_unary(
        &self,
        operand: FhirPathValue,
        context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        let _ = (operand, context);
        Err(EvaluationError::InvalidOperation {
            message: format!(
                "Operator '{}' does not support unary operations",
                self.symbol()
            )
        })
    }

    /// Check if operand types are valid for this operator
    fn validates_types(&self, left_type: Option<&str>, right_type: &str) -> bool {
        // Default implementation checks against type signatures in metadata
        self.metadata().types.type_signatures.iter().any(|sig| {
            match (&sig.left_type, left_type) {
                (Some(expected_left), Some(actual_left)) => {
                    expected_left == actual_left && sig.right_type == right_type
                }
                (None, None) => sig.right_type == right_type,
                _ => false,
            }
        })
    }

    /// Get the expected result type for given operand types
    fn result_type(&self, left_type: Option<&str>, right_type: &str) -> Option<String> {
        self.metadata()
            .types
            .type_signatures
            .iter()
            .find(|sig| {
                match (&sig.left_type, left_type) {
                    (Some(expected_left), Some(actual_left)) => {
                        expected_left == actual_left && sig.right_type == right_type
                    }
                    (None, None) => sig.right_type == right_type,
                    _ => false,
                }
            })
            .map(|sig| sig.result_type.clone())
            .or_else(|| self.metadata().types.default_result_type.clone())
    }

    /// Check if this operator can be optimized
    fn is_optimizable(&self) -> bool {
        self.metadata().performance.optimizable
    }

    /// Check if this operator can short-circuit evaluation
    fn can_short_circuit(&self) -> bool {
        self.metadata().performance.short_circuits
    }

    /// Check if this operator is pure (deterministic with no side effects)
    fn is_pure(&self) -> bool {
        self.metadata().basic.is_pure
    }

    /// Check if this operator is commutative (a op b = b op a)
    fn is_commutative(&self) -> bool {
        self.metadata().basic.is_commutative
    }
}

/// Helper trait for implementing arithmetic operations
pub trait ArithmeticOperator: UnifiedFhirPathOperator {
    /// Perform integer arithmetic operation
    fn apply_integer(&self, left: i64, right: i64) -> OperatorResult<i64>;

    /// Perform decimal arithmetic operation
    fn apply_decimal(&self, left: rust_decimal::Decimal, right: rust_decimal::Decimal) -> OperatorResult<rust_decimal::Decimal>;

    /// Default binary evaluation for arithmetic operators
    async fn evaluate_arithmetic_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        match (left, right) {
            (Integer(l), Integer(r)) => {
                match self.apply_integer(l, r) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Integer(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Decimal(l), Decimal(r)) => {
                match self.apply_decimal(l, r) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Integer(l), Decimal(r)) => {
                let left_decimal = rust_decimal::Decimal::from(l);
                match self.apply_decimal(left_decimal, r) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (Decimal(l), Integer(r)) => {
                let right_decimal = rust_decimal::Decimal::from(r);
                match self.apply_decimal(l, right_decimal) {
                    Ok(result) => Ok(FhirPathValue::collection(vec![Decimal(result)])),
                    Err(e) => Err(e.into()),
                }
            }
            (left_val, right_val) => {
                Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left_val.type_name().to_string(),
                    right_type: right_val.type_name().to_string(),
                }.into())
            }
        }
    }
}

/// Helper trait for implementing comparison operations
pub trait ComparisonOperator: UnifiedFhirPathOperator {
    /// Perform the comparison operation
    fn compare(&self, ordering: std::cmp::Ordering) -> bool;
    
    /// Check if two elements are equal (used for collection comparisons)
    fn elements_equal(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use octofhir_fhirpath_model::FhirPathValue::*;
        
        match (left, right) {
            (Empty, Empty) => true,
            (Integer(l), Integer(r)) => l == r,
            (Decimal(l), Decimal(r)) => l == r,
            (Integer(l), Decimal(r)) => rust_decimal::Decimal::from(*l) == *r,
            (Decimal(l), Integer(r)) => *l == rust_decimal::Decimal::from(*r),
            (String(l), String(r)) => l == r,
            (Boolean(l), Boolean(r)) => l == r,
            (Date(l), Date(r)) => l == r,
            (DateTime(l), DateTime(r)) => l == r,
            (Time(l), Time(r)) => l == r,
            (Quantity(l), Quantity(r)) => l.equals_with_conversion(r).unwrap_or(false),
            (TypeInfoObject { namespace: ln, name: lname }, TypeInfoObject { namespace: rn, name: rname }) => {
                ln == rn && lname == rname
            },
            // Collections can be compared recursively, but this is handled at higher level
            (Collection(l), Collection(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| self.elements_equal(a, b))
            },
            _ => false,
        }
    }

    /// Default binary evaluation for comparison operators
    async fn evaluate_comparison_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;

        // Handle Empty values per FHIRPath specification
        match (&left, &right) {
            (Empty, _) | (_, Empty) => {
                // Per FHIRPath spec: comparisons with empty return empty
                // But we need to distinguish between empty collections and empty single values
                return Ok(Empty);
            }
            _ => {} // Continue with normal comparison
        }

        let ordering = match (&left, &right) {
            (Integer(l), Integer(r)) => l.cmp(r),
            (Decimal(l), Decimal(r)) => l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal),
            (Integer(l), Decimal(r)) => rust_decimal::Decimal::from(*l).partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal),
            (Decimal(l), Integer(r)) => l.partial_cmp(&rust_decimal::Decimal::from(*r)).unwrap_or(std::cmp::Ordering::Equal),
            (String(l), String(r)) => l.cmp(r),
            (Boolean(l), Boolean(r)) => l.cmp(r),
            (Date(l), Date(r)) => l.cmp(r),
            (DateTime(l), DateTime(r)) => l.cmp(r),
            (Time(l), Time(r)) => l.cmp(r),
            (Quantity(l), Quantity(r)) => {
                // Use Quantity's equals_with_conversion for comparison
                match l.equals_with_conversion(r) {
                    Ok(true) => std::cmp::Ordering::Equal,
                    Ok(false) => {
                        // For non-equal quantities, we can't determine ordering without unit conversion
                        // This is primarily for equals operator, other comparisons need more work
                        std::cmp::Ordering::Greater // Fallback for non-equal
                    },
                    Err(_) => std::cmp::Ordering::Greater, // Incompatible units
                }
            },
            // Handle Collection comparisons per FHIRPath specification
            (Collection(l), Collection(r)) => {
                // Filter out Empty values from both collections for comparison
                let l_filtered: Vec<_> = l.iter().filter(|v| !matches!(v, Empty)).collect();
                let r_filtered: Vec<_> = r.iter().filter(|v| !matches!(v, Empty)).collect();
                
                // Collections are equal if they contain the same elements (order doesn't matter for =)
                if l_filtered.len() != r_filtered.len() {
                    std::cmp::Ordering::Greater // Different sizes = not equal
                } else if l_filtered.is_empty() && r_filtered.is_empty() {
                    std::cmp::Ordering::Equal // Both empty after filtering
                } else {
                    // For proper collection equality, need to check both directions:
                    // 1. All elements in left are in right
                    // 2. All elements in right are in left
                    let mut left_in_right = true;
                    for left_item in l_filtered.iter() {
                        let found = r_filtered.iter().any(|right_item| {
                            self.elements_equal(left_item, right_item)
                        });
                        if !found {
                            left_in_right = false;
                            break;
                        }
                    }
                    
                    let mut right_in_left = true;
                    for right_item in r_filtered.iter() {
                        let found = l_filtered.iter().any(|left_item| {
                            self.elements_equal(left_item, right_item)
                        });
                        if !found {
                            right_in_left = false;
                            break;
                        }
                    }
                    
                    if left_in_right && right_in_left {
                        std::cmp::Ordering::Equal
                    } else {
                        std::cmp::Ordering::Greater // Not equal
                    }
                }
            },
            // Handle Single value vs Collection comparisons per FHIRPath spec
            (Collection(col), single_val) => {
                // For equals operator: collection = single means single is in collection
                if self.symbol() == "=" {
                    let contains = col.iter().any(|item| self.elements_equal(item, single_val));
                    return Ok(FhirPathValue::collection(vec![Boolean(contains)]));
                } else {
                    // Other comparison operators with collections need special handling
                    return Ok(Empty);
                }
            },
            (single_val, Collection(col)) => {
                // For equals operator: single = collection means single is in collection  
                if self.symbol() == "=" {
                    let contains = col.iter().any(|item| self.elements_equal(single_val, item));
                    return Ok(FhirPathValue::collection(vec![Boolean(contains)]));
                } else {
                    // Other comparison operators with collections need special handling
                    return Ok(Empty);
                }
            },
            // Handle TypeInfoObject comparisons for type() function results
            (TypeInfoObject { namespace: ln, name: lname }, TypeInfoObject { namespace: rn, name: rname }) => {
                // First compare namespaces, then names
                match ln.cmp(rn) {
                    std::cmp::Ordering::Equal => lname.cmp(rname),
                    other => other,
                }
            },
            _ => {
                return Err(OperatorError::InvalidOperandTypes {
                    operator: self.symbol().to_string(),
                    left_type: left.type_name().to_string(),
                    right_type: right.type_name().to_string(),
                }.into());
            }
        };

        // All FHIRPath evaluation results must be collections per the specification
        Ok(FhirPathValue::collection(vec![Boolean(self.compare(ordering))]))
    }
}

/// Helper trait for implementing logical operations
pub trait LogicalOperator: UnifiedFhirPathOperator {
    /// Apply the logical operation to two boolean values
    fn apply_logical(&self, left: bool, right: bool) -> bool;

    /// Convert FhirPathValue to boolean according to FHIRPath rules
    fn to_boolean(&self, value: &FhirPathValue) -> bool {
        use octofhir_fhirpath_model::FhirPathValue::*;
        match value {
            Boolean(b) => *b,
            Integer(i) => *i != 0,
            Decimal(d) => !d.is_zero(),
            String(s) => !s.is_empty(),
            Collection(c) => !c.is_empty(),
            _ => false,
        }
    }

    /// Default binary evaluation for logical operators with three-valued logic
    async fn evaluate_logical_binary(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
        _context: &EvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_model::FhirPathValue::*;
        
        // Implement three-valued logic per FHIRPath specification
        // Empty collections should be propagated properly for different logical operators
        match (self.symbol(), &left, &right) {
            // AND operator: returns true if both true, false if either false, empty otherwise
            ("and", Empty, _) | ("and", _, Empty) => {
                // Check if the other operand is explicitly false
                let other_is_false = match (&left, &right) {
                    (Empty, other) | (other, Empty) => {
                        match other {
                            Boolean(false) => true,
                            Integer(0) => true,
                            Decimal(d) if d.is_zero() => true,
                            String(s) if s.is_empty() => true,
                            Collection(c) if c.is_empty() => false, // Empty collection, not false
                            _ => false,
                        }
                    }
                    _ => false,
                };
                
                if other_is_false {
                    Ok(FhirPathValue::collection(vec![Boolean(false)]))
                } else {
                    Ok(Empty) // Empty collection propagates
                }
            }
            
            // OR operator: returns false if both false, true if either true, empty otherwise  
            ("or", Empty, _) | ("or", _, Empty) => {
                // Check if the other operand is explicitly true
                let other_is_true = match (&left, &right) {
                    (Empty, other) | (other, Empty) => {
                        match other {
                            Empty => false,
                            Collection(c) if c.is_empty() => false,
                            _ => self.to_boolean(other),
                        }
                    }
                    _ => false,
                };
                
                if other_is_true {
                    Ok(FhirPathValue::collection(vec![Boolean(true)]))
                } else {
                    Ok(Empty) // Empty collection propagates
                }
            }
            
            // XOR operator: empty with anything returns empty
            ("xor", Empty, _) | ("xor", _, Empty) => Ok(Empty),
            
            // IMPLIES operator: FHIRPath three-valued logic for empty collections  
            ("implies", Empty, _) => {
                // Empty (false) implies - check what we're implying
                match &right {
                    Boolean(true) => Ok(FhirPathValue::collection(vec![Boolean(true)])), // false implies true = true
                    Boolean(false) => Ok(Empty), // false implies false = empty (indeterminate)
                    Empty => Ok(Empty), // false implies empty = empty (indeterminate)
                    Collection(c) if c.is_empty() => Ok(Empty), // false implies empty collection = empty (indeterminate)
                    _ => {
                        // For other types, convert to boolean and apply logic
                        let right_bool = self.to_boolean(&right);
                        if right_bool {
                            Ok(FhirPathValue::collection(vec![Boolean(true)])) // false implies true = true
                        } else {
                            Ok(Empty) // false implies false = empty (indeterminate)
                        }
                    }
                }
            }
            ("implies", _, Empty) => {
                let left_bool = self.to_boolean(&left);
                if !left_bool {
                    // false implies empty = true (false implies anything is true)
                    Ok(FhirPathValue::collection(vec![Boolean(true)]))
                } else {
                    // true implies empty (false) = empty (indeterminate in three-valued logic)
                    Ok(Empty)
                }
            }
            
            // No empty collections, use regular boolean logic
            _ => {
                let left_bool = self.to_boolean(&left);
                let right_bool = self.to_boolean(&right);
                let result = self.apply_logical(left_bool, right_bool);
                Ok(FhirPathValue::collection(vec![Boolean(result)]))
            }
        }
    }
}
