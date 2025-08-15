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

//! Less than operator (<) implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::{EvaluationContext, binary_operator_utils};
use async_trait::async_trait;

/// Less than operator (<)
#[derive(Debug, Clone)]
pub struct LessThanOperation;

impl LessThanOperation {
    pub fn new() -> Self {
        Self
    }

    pub fn compare_less_than(left: &FhirPathValue, right: &FhirPathValue) -> Result<bool> {
        match (left, right) {
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => Ok(false),
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => Ok(a < b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => Ok(a < b),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                let a_decimal = rust_decimal::Decimal::from(*a);
                Ok(a_decimal < *b)
            },
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                let b_decimal = rust_decimal::Decimal::from(*b);
                Ok(*a < b_decimal)
            },
            (FhirPathValue::String(a), FhirPathValue::String(b)) => Ok(a < b),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => Ok(a < b),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => Ok(a < b),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => Ok(a < b),
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // Compare quantities with unit conversion
                match (a.has_compatible_dimensions(b), &a.unit, &b.unit) {
                    (true, Some(unit_a), Some(_)) => {
                        // Try to convert b to a's unit and compare
                        match b.convert_to_compatible_unit(unit_a) {
                            Ok(converted_b) => Ok(a.value < converted_b.value),
                            Err(_) => Ok(false), // If conversion fails, comparison is false
                        }
                    }
                    (true, None, None) => {
                        // Both unitless quantities
                        Ok(a.value < b.value)
                    }
                    _ => Ok(false), // Incompatible units
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!("Cannot compare {} < {}", left.type_name(), right.type_name())
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for LessThanOperation {
    fn identifier(&self) -> &str {
        "<"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            MetadataBuilder::new("<", OperationType::BinaryOperator {
                precedence: 6,
                associativity: Associativity::Left,
            })
            .description("Less than comparison operator")
            .example("1 < 2")
            .example("@2023-01-01 < @2023-12-31")
            .parameter("left", TypeConstraint::Any, false)
            .parameter("right", TypeConstraint::Any, false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len()
            });
        }

        binary_operator_utils::evaluate_binary_operator(&args[0], &args[1], Self::compare_less_than)
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len()
            }));
        }

        Some(binary_operator_utils::evaluate_binary_operator(&args[0], &args[1], Self::compare_less_than))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use rust_decimal::Decimal;
    use octofhir_fhirpath_model::MockModelProvider;
    use crate::FhirPathRegistry;

    fn create_test_context() -> EvaluationContext {
        use std::sync::Arc;

        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_less_than_operation() {
        let op = LessThanOperation::new();
        let ctx = create_test_context();

        // Test integer comparison
        let args = vec![FhirPathValue::Integer(3), FhirPathValue::Integer(5)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test false case
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(3)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test mixed integer/decimal
        let args = vec![FhirPathValue::Integer(3), FhirPathValue::Decimal(Decimal::from_str("3.5").unwrap())];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
