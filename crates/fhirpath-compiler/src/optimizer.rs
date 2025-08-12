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

//! Bytecode optimization passes
//!
//! This module provides optimization passes that can be applied to bytecode
//! or AST expressions to improve performance.

use fhirpath_ast::{BinaryOperator, ExpressionNode, LiteralValue, UnaryOperator};
use fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Result type for optimization operations
pub type OptimizationResult<T> = Result<T, OptimizationError>;

/// Errors that can occur during optimization
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationError {
    /// Cannot fold expression due to runtime dependency
    NotConstant(String),
    /// Arithmetic error during constant folding
    ArithmeticError(String),
    /// Type error during constant evaluation
    TypeError {
        /// Expected type name
        expected: String,
        /// Actual type name encountered
        actual: String,
    },
    /// Division by zero in constant folding
    DivisionByZero,
    /// Unsupported operation for constant folding
    UnsupportedOperation(String),
}

impl std::fmt::Display for OptimizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationError::NotConstant(msg) => write!(f, "Not constant: {msg}"),
            OptimizationError::ArithmeticError(msg) => write!(f, "Arithmetic error: {msg}"),
            OptimizationError::TypeError { expected, actual } => {
                write!(f, "Type error: expected {expected}, got {actual}")
            }
            OptimizationError::DivisionByZero => write!(f, "Division by zero"),
            OptimizationError::UnsupportedOperation(op) => {
                write!(f, "Unsupported operation: {op}")
            }
        }
    }
}

impl std::error::Error for OptimizationError {}

/// Configuration for optimization passes
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable constant folding optimization
    pub constant_folding: bool,
    /// Enable dead code elimination
    pub dead_code_elimination: bool,
    /// Enable strength reduction
    pub strength_reduction: bool,
    /// Maximum depth for recursive optimization
    pub max_depth: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            strength_reduction: true,
            max_depth: 32,
        }
    }
}

/// Expression optimizer that applies various optimization passes
pub struct ExpressionOptimizer {
    config: OptimizationConfig,
    /// Cache for previously computed constant expressions
    constant_cache: HashMap<String, Option<FhirPathValue>>,
}

impl ExpressionOptimizer {
    /// Create a new optimizer with default configuration
    pub fn new() -> Self {
        Self {
            config: OptimizationConfig::default(),
            constant_cache: HashMap::new(),
        }
    }

    /// Create a new optimizer with custom configuration
    pub fn with_config(config: OptimizationConfig) -> Self {
        Self {
            config,
            constant_cache: HashMap::new(),
        }
    }

    /// Optimize an expression using all enabled passes
    pub fn optimize(&mut self, expr: ExpressionNode) -> ExpressionNode {
        let mut result = expr;

        // Apply optimization passes in order
        if self.config.constant_folding {
            result = self.constant_fold_recursive(result, 0);
        }

        if self.config.strength_reduction {
            result = self.strength_reduce(result);
        }

        if self.config.dead_code_elimination {
            result = self.eliminate_dead_code(result);
        }

        result
    }

    /// Apply constant folding recursively to an expression
    fn constant_fold_recursive(&mut self, expr: ExpressionNode, depth: usize) -> ExpressionNode {
        if depth >= self.config.max_depth {
            return expr;
        }

        // First, recursively optimize children
        let expr = self.optimize_children(expr, depth + 1);

        // Then try to fold this expression
        match self.constant_fold(&expr) {
            Ok(value) => {
                // Convert the constant value back to a literal expression
                self.value_to_literal(value)
            }
            Err(_) => expr, // Cannot fold, return as-is
        }
    }

    /// Optimize child expressions recursively
    fn optimize_children(&mut self, expr: ExpressionNode, depth: usize) -> ExpressionNode {
        match expr {
            ExpressionNode::BinaryOp(mut data) => {
                data.left = self.constant_fold_recursive(data.left, depth);
                data.right = self.constant_fold_recursive(data.right, depth);
                ExpressionNode::BinaryOp(data)
            }
            ExpressionNode::UnaryOp { op, operand } => {
                let operand = Box::new(self.constant_fold_recursive(*operand, depth));
                ExpressionNode::UnaryOp { op, operand }
            }
            ExpressionNode::FunctionCall(mut data) => {
                data.args = data
                    .args
                    .into_iter()
                    .map(|arg| self.constant_fold_recursive(arg, depth))
                    .collect();
                ExpressionNode::FunctionCall(data)
            }
            other => other, // Literals, identifiers, etc. don't have children
        }
    }

    /// Attempt to fold a constant expression into a value
    pub fn constant_fold(&mut self, expr: &ExpressionNode) -> OptimizationResult<FhirPathValue> {
        // Check cache first
        let expr_key = self.expression_key(expr);
        if let Some(cached) = self.constant_cache.get(&expr_key) {
            return match cached {
                Some(value) => Ok(value.clone()),
                None => Err(OptimizationError::NotConstant(
                    "cached as non-constant".to_string(),
                )),
            };
        }

        let result = self.constant_fold_impl(expr);

        // Cache the result (both success and failure)
        self.constant_cache
            .insert(expr_key, result.as_ref().ok().cloned());

        result
    }

    /// Implementation of constant folding
    fn constant_fold_impl(&self, expr: &ExpressionNode) -> OptimizationResult<FhirPathValue> {
        match expr {
            ExpressionNode::Literal(lit) => Ok(self.literal_to_value(lit)),

            ExpressionNode::BinaryOp(data) => {
                let left_val = self.constant_fold_impl(&data.left)?;
                let right_val = self.constant_fold_impl(&data.right)?;
                self.evaluate_binary_operation(data.op, left_val, right_val)
            }

            ExpressionNode::UnaryOp { op, operand } => {
                let operand_val = self.constant_fold_impl(operand)?;
                self.evaluate_unary_operation(*op, operand_val)
            }

            // Non-constant expressions
            ExpressionNode::Identifier(_) => {
                Err(OptimizationError::NotConstant("identifier".to_string()))
            }
            ExpressionNode::Path { .. } => Err(OptimizationError::NotConstant(
                "path navigation".to_string(),
            )),
            ExpressionNode::FunctionCall(_) => {
                Err(OptimizationError::NotConstant("function call".to_string()))
            }
            ExpressionNode::Index { .. } => {
                Err(OptimizationError::NotConstant("indexer".to_string()))
            }
            ExpressionNode::Lambda(_) => Err(OptimizationError::NotConstant("lambda".to_string())),
            _ => Err(OptimizationError::NotConstant(
                "other expression".to_string(),
            )),
        }
    }

    /// Convert a literal value to FhirPathValue
    fn literal_to_value(&self, lit: &LiteralValue) -> FhirPathValue {
        match lit {
            LiteralValue::Boolean(b) => FhirPathValue::Boolean(*b),
            LiteralValue::Integer(i) => FhirPathValue::Integer(*i),
            LiteralValue::Decimal(d) => {
                // Parse decimal string to Decimal type
                match d.parse::<Decimal>() {
                    Ok(decimal) => FhirPathValue::Decimal(decimal),
                    Err(_) => FhirPathValue::String(d.clone().into()), // Fallback to string
                }
            }
            LiteralValue::String(s) => FhirPathValue::interned_string(s),
            LiteralValue::Date(d) => {
                // Parse date string to NaiveDate
                match chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                    Ok(date) => FhirPathValue::Date(date),
                    Err(_) => FhirPathValue::String(d.clone().into()), // Fallback to string
                }
            }
            LiteralValue::DateTime(dt) => {
                // Parse datetime string to DateTime<FixedOffset>
                match chrono::DateTime::parse_from_rfc3339(dt) {
                    Ok(datetime) => FhirPathValue::DateTime(
                        datetime
                            .with_timezone(&chrono::Utc)
                            .with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
                    ),
                    Err(_) => FhirPathValue::String(dt.clone().into()), // Fallback to string
                }
            }
            LiteralValue::Time(t) => {
                // Parse time string to NaiveTime
                match chrono::NaiveTime::parse_from_str(t, "%H:%M:%S") {
                    Ok(time) => FhirPathValue::Time(time),
                    Err(_) => FhirPathValue::String(t.clone().into()), // Fallback to string
                }
            }
            LiteralValue::Quantity { value, unit } => {
                // Parse value string to Decimal
                match value.parse::<Decimal>() {
                    Ok(decimal) => FhirPathValue::quantity(decimal, Some(unit.clone())),
                    Err(_) => FhirPathValue::String(format!("{value} {unit}").into()), // Fallback to string
                }
            }
            LiteralValue::Null => FhirPathValue::Empty,
        }
    }

    /// Convert a FhirPathValue back to a literal expression
    fn value_to_literal(&self, value: FhirPathValue) -> ExpressionNode {
        let literal = match value {
            FhirPathValue::Boolean(b) => LiteralValue::Boolean(b),
            FhirPathValue::Integer(i) => LiteralValue::Integer(i),
            FhirPathValue::Decimal(d) => LiteralValue::Decimal(d.to_string()),
            FhirPathValue::String(s) => LiteralValue::String(s.as_ref().to_string()),
            FhirPathValue::Date(d) => LiteralValue::Date(d.format("%Y-%m-%d").to_string()),
            FhirPathValue::DateTime(dt) => LiteralValue::DateTime(dt.to_rfc3339()),
            FhirPathValue::Time(t) => LiteralValue::Time(t.format("%H:%M:%S").to_string()),
            FhirPathValue::Quantity(ref q) => LiteralValue::Quantity {
                value: q.value.to_string(),
                unit: q.unit.as_ref().unwrap_or(&"".to_string()).clone(),
            },
            // Non-literal values cannot be converted back
            _ => return ExpressionNode::Literal(LiteralValue::Boolean(false)), // Fallback
        };

        ExpressionNode::Literal(literal)
    }

    /// Evaluate a binary operation on constant values
    fn evaluate_binary_operation(
        &self,
        op: BinaryOperator,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match op {
            // Arithmetic operations
            BinaryOperator::Add => self.add_values(left, right),
            BinaryOperator::Subtract => self.subtract_values(left, right),
            BinaryOperator::Multiply => self.multiply_values(left, right),
            BinaryOperator::Divide => self.divide_values(left, right),
            BinaryOperator::Modulo => self.modulo_values(left, right),

            // Comparison operations
            BinaryOperator::Equal => Ok(FhirPathValue::Boolean(self.values_equal(&left, &right))),
            BinaryOperator::NotEqual => {
                Ok(FhirPathValue::Boolean(!self.values_equal(&left, &right)))
            }
            BinaryOperator::LessThan => self.less_than_values(left, right),
            BinaryOperator::LessThanOrEqual => self.less_than_or_equal_values(left, right),
            BinaryOperator::GreaterThan => self.greater_than_values(left, right),
            BinaryOperator::GreaterThanOrEqual => self.greater_than_or_equal_values(left, right),

            // Logical operations
            BinaryOperator::And => self.and_values(left, right),
            BinaryOperator::Or => self.or_values(left, right),
            BinaryOperator::Xor => self.xor_values(left, right),

            // String operations
            BinaryOperator::Concatenate => self.concatenate_values(left, right),

            // Type operations
            BinaryOperator::Is => Err(OptimizationError::UnsupportedOperation("is".to_string())),

            // Collection operations
            BinaryOperator::Union => {
                Err(OptimizationError::UnsupportedOperation("union".to_string()))
            }
            BinaryOperator::In => Err(OptimizationError::UnsupportedOperation("in".to_string())),
            BinaryOperator::Contains => Err(OptimizationError::UnsupportedOperation(
                "contains".to_string(),
            )),

            // Additional operations
            BinaryOperator::IntegerDivide => self.divide_values(left, right), // Same as divide for now
            BinaryOperator::Equivalent => {
                Ok(FhirPathValue::Boolean(self.values_equal(&left, &right)))
            }
            BinaryOperator::NotEquivalent => {
                Ok(FhirPathValue::Boolean(!self.values_equal(&left, &right)))
            }
            BinaryOperator::Implies => Err(OptimizationError::UnsupportedOperation(
                "implies".to_string(),
            )),
        }
    }

    /// Evaluate a unary operation on a constant value
    fn evaluate_unary_operation(
        &self,
        op: UnaryOperator,
        operand: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match op {
            UnaryOperator::Not => match operand {
                FhirPathValue::Boolean(b) => Ok(FhirPathValue::Boolean(!b)),
                _ => Err(OptimizationError::TypeError {
                    expected: "Boolean".to_string(),
                    actual: operand.type_name().to_string(),
                }),
            },
            UnaryOperator::Minus => match operand {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(-i)),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
                _ => Err(OptimizationError::TypeError {
                    expected: "Number".to_string(),
                    actual: operand.type_name().to_string(),
                }),
            },
            UnaryOperator::Plus => match operand {
                FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(i)),
                FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(d)),
                _ => Err(OptimizationError::TypeError {
                    expected: "Number".to_string(),
                    actual: operand.type_name().to_string(),
                }),
            },
        }
    }

    // Arithmetic helper methods
    fn add_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a + b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a + b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(Decimal::from(a) + b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Decimal(a + Decimal::from(b)))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Number".to_string(),
                actual: format!("{} + {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn subtract_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a - b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a - b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(Decimal::from(a) - b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Decimal(a - Decimal::from(b)))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Number".to_string(),
                actual: format!("{} - {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn multiply_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Integer(a * b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a * b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(Decimal::from(a) * b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                Ok(FhirPathValue::Decimal(a * Decimal::from(b)))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Number".to_string(),
                actual: format!("{} * {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn divide_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if b == 0 {
                    return Err(OptimizationError::DivisionByZero);
                }
                // Integer division in FHIRPath returns decimal
                Ok(FhirPathValue::Decimal(Decimal::from(a) / Decimal::from(b)))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OptimizationError::DivisionByZero);
                }
                Ok(FhirPathValue::Decimal(a / b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OptimizationError::DivisionByZero);
                }
                Ok(FhirPathValue::Decimal(Decimal::from(a) / b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if b == 0 {
                    return Err(OptimizationError::DivisionByZero);
                }
                Ok(FhirPathValue::Decimal(a / Decimal::from(b)))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Number".to_string(),
                actual: format!("{} / {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn modulo_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if b == 0 {
                    return Err(OptimizationError::DivisionByZero);
                }
                Ok(FhirPathValue::Integer(a % b))
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if b.is_zero() {
                    return Err(OptimizationError::DivisionByZero);
                }
                Ok(FhirPathValue::Decimal(a % b))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Number".to_string(),
                actual: format!("{} mod {}", left.type_name(), right.type_name()),
            }),
        }
    }

    // Comparison helper methods
    fn values_equal(&self, left: &FhirPathValue, right: &FhirPathValue) -> bool {
        // Implement FHIRPath equality semantics
        match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a == b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a == b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(*a) == *b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => *a == Decimal::from(*b),
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a == b,
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a == b,
            _ => false, // Different types are not equal
        }
    }

    fn less_than_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a < b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(a) < b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a < Decimal::from(b),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a < b,
            (left, right) => {
                return Err(OptimizationError::TypeError {
                    expected: "Comparable".to_string(),
                    actual: format!("{} < {}", left.type_name(), right.type_name()),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }

    fn less_than_or_equal_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a <= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a <= b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(a) <= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a <= Decimal::from(b),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a <= b,
            (left, right) => {
                return Err(OptimizationError::TypeError {
                    expected: "Comparable".to_string(),
                    actual: format!("{} <= {}", left.type_name(), right.type_name()),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }

    fn greater_than_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a > b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a > b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(a) > b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a > Decimal::from(b),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a > b,
            (left, right) => {
                return Err(OptimizationError::TypeError {
                    expected: "Comparable".to_string(),
                    actual: format!("{} > {}", left.type_name(), right.type_name()),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }

    fn greater_than_or_equal_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a >= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a >= b,
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(a) >= b,
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a >= Decimal::from(b),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a >= b,
            (left, right) => {
                return Err(OptimizationError::TypeError {
                    expected: "Comparable".to_string(),
                    actual: format!("{} >= {}", left.type_name(), right.type_name()),
                });
            }
        };
        Ok(FhirPathValue::Boolean(result))
    }

    // Logical helper methods
    fn and_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                Ok(FhirPathValue::Boolean(a && b))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Boolean".to_string(),
                actual: format!("{} and {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn or_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                Ok(FhirPathValue::Boolean(a || b))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Boolean".to_string(),
                actual: format!("{} or {}", left.type_name(), right.type_name()),
            }),
        }
    }

    fn xor_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => {
                Ok(FhirPathValue::Boolean(a ^ b))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "Boolean".to_string(),
                actual: format!("{} xor {}", left.type_name(), right.type_name()),
            }),
        }
    }

    // String helper methods
    fn concatenate_values(
        &self,
        left: FhirPathValue,
        right: FhirPathValue,
    ) -> OptimizationResult<FhirPathValue> {
        match (left, right) {
            (FhirPathValue::String(a), FhirPathValue::String(b)) => {
                Ok(FhirPathValue::String(format!("{a}{b}").into()))
            }
            (left, right) => Err(OptimizationError::TypeError {
                expected: "String".to_string(),
                actual: format!("{} & {}", left.type_name(), right.type_name()),
            }),
        }
    }

    /// Apply strength reduction optimizations
    fn strength_reduce(&self, expr: ExpressionNode) -> ExpressionNode {
        match expr {
            ExpressionNode::BinaryOp(data) => {
                match (data.op, &data.left, &data.right) {
                    // x + 0 => x
                    (BinaryOperator::Add, _, ExpressionNode::Literal(LiteralValue::Integer(0))) => {
                        data.left
                    }
                    (BinaryOperator::Add, ExpressionNode::Literal(LiteralValue::Integer(0)), _) => {
                        data.right
                    }

                    // x * 1 => x
                    (
                        BinaryOperator::Multiply,
                        _,
                        ExpressionNode::Literal(LiteralValue::Integer(1)),
                    ) => data.left,
                    (
                        BinaryOperator::Multiply,
                        ExpressionNode::Literal(LiteralValue::Integer(1)),
                        _,
                    ) => data.right,

                    // x * 0 => 0
                    (
                        BinaryOperator::Multiply,
                        _,
                        ExpressionNode::Literal(LiteralValue::Integer(0)),
                    ) => ExpressionNode::Literal(LiteralValue::Integer(0)),
                    (
                        BinaryOperator::Multiply,
                        ExpressionNode::Literal(LiteralValue::Integer(0)),
                        _,
                    ) => ExpressionNode::Literal(LiteralValue::Integer(0)),

                    // x / 1 => x
                    (
                        BinaryOperator::Divide,
                        _,
                        ExpressionNode::Literal(LiteralValue::Integer(1)),
                    ) => data.left,

                    // x - 0 => x
                    (
                        BinaryOperator::Subtract,
                        _,
                        ExpressionNode::Literal(LiteralValue::Integer(0)),
                    ) => data.left,

                    // x or true => true
                    (
                        BinaryOperator::Or,
                        _,
                        ExpressionNode::Literal(LiteralValue::Boolean(true)),
                    ) => ExpressionNode::Literal(LiteralValue::Boolean(true)),
                    (
                        BinaryOperator::Or,
                        ExpressionNode::Literal(LiteralValue::Boolean(true)),
                        _,
                    ) => ExpressionNode::Literal(LiteralValue::Boolean(true)),

                    // x or false => x
                    (
                        BinaryOperator::Or,
                        _,
                        ExpressionNode::Literal(LiteralValue::Boolean(false)),
                    ) => data.left,
                    (
                        BinaryOperator::Or,
                        ExpressionNode::Literal(LiteralValue::Boolean(false)),
                        _,
                    ) => data.right,

                    // x and true => x
                    (
                        BinaryOperator::And,
                        _,
                        ExpressionNode::Literal(LiteralValue::Boolean(true)),
                    ) => data.left,
                    (
                        BinaryOperator::And,
                        ExpressionNode::Literal(LiteralValue::Boolean(true)),
                        _,
                    ) => data.right,

                    // x and false => false
                    (
                        BinaryOperator::And,
                        _,
                        ExpressionNode::Literal(LiteralValue::Boolean(false)),
                    ) => ExpressionNode::Literal(LiteralValue::Boolean(false)),
                    (
                        BinaryOperator::And,
                        ExpressionNode::Literal(LiteralValue::Boolean(false)),
                        _,
                    ) => ExpressionNode::Literal(LiteralValue::Boolean(false)),

                    _ => ExpressionNode::BinaryOp(data),
                }
            }
            ExpressionNode::UnaryOp { op, operand } => {
                match (op, operand.as_ref()) {
                    // --x => x (double negative)
                    (
                        UnaryOperator::Minus,
                        ExpressionNode::UnaryOp {
                            op: UnaryOperator::Minus,
                            operand: inner,
                        },
                    ) => *inner.clone(),

                    // not not x => x (double negation)
                    (
                        UnaryOperator::Not,
                        ExpressionNode::UnaryOp {
                            op: UnaryOperator::Not,
                            operand: inner,
                        },
                    ) => *inner.clone(),

                    _ => ExpressionNode::UnaryOp { op, operand },
                }
            }
            other => other,
        }
    }

    /// Eliminate dead code (currently a placeholder)
    fn eliminate_dead_code(&self, expr: ExpressionNode) -> ExpressionNode {
        // For now, just return the expression as-is
        // In a more complete implementation, this would:
        // - Remove unused variable assignments
        // - Remove unreachable code after returns
        // - Remove conditions that are always true/false
        expr
    }

    /// Generate a cache key for an expression
    fn expression_key(&self, expr: &ExpressionNode) -> String {
        // Simple string representation for caching
        // In a production implementation, this would be more sophisticated
        format!("{expr:?}")
    }
}

impl Default for ExpressionOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
