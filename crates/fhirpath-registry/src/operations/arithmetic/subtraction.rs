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

//! Subtraction operation (-) implementation for FHIRPath

use crate::metadata::{
    MetadataBuilder, OperationType, TypeConstraint, FhirPathType,
    OperationMetadata, PerformanceComplexity, Associativity,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use rust_decimal::Decimal;

/// Subtraction operation (-) - supports both binary and unary operations
pub struct SubtractionOperation;

impl SubtractionOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("-", OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        })
            .description("Binary subtraction operation and unary minus")
            .example("5 - 2")
            .example("-42")
            .returns(TypeConstraint::OneOf(vec![
                FhirPathType::Integer, 
                FhirPathType::Decimal
            ]))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn evaluate_binary_sync(&self, left: &FhirPathValue, right: &FhirPathValue) -> Option<Result<FhirPathValue>> {
        // Handle empty collections per FHIRPath spec
        match (left, right) {
            (FhirPathValue::Collection(l), FhirPathValue::Collection(r)) => {
                if l.is_empty() || r.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                if l.len() > 1 || r.len() > 1 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                // Single element collections - unwrap and proceed
                let left_val = l.first().unwrap();
                let right_val = r.first().unwrap();
                return self.evaluate_binary_sync(left_val, right_val);
            }
            (FhirPathValue::Collection(l), other) => {
                if l.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                if l.len() > 1 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                let left_val = l.first().unwrap();
                return self.evaluate_binary_sync(left_val, other);
            }
            (other, FhirPathValue::Collection(r)) => {
                if r.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                if r.len() > 1 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                }
                let right_val = r.first().unwrap();
                return self.evaluate_binary_sync(other, right_val);
            }
            _ => {}
        }

        // Actual arithmetic operations on scalar values
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                a.checked_sub(*b)
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::ArithmeticError {
                        message: "Integer overflow in subtraction".to_string()
                    })
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                Ok(FhirPathValue::Decimal(a - b))
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                match Decimal::try_from(*a) {
                    Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal - b)),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    })
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                match Decimal::try_from(*b) {
                    Ok(b_decimal) => Ok(FhirPathValue::Decimal(a - b_decimal)),
                    Err(_) => Err(FhirPathError::ArithmeticError {
                        message: "Cannot convert integer to decimal".to_string()
                    })
                }
            }
            // Quantity subtraction - requires compatible units
            (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
                // For subtraction, quantities must have compatible dimensions
                if a.has_compatible_dimensions(b) {
                    match b.convert_to_compatible_unit(&a.unit.as_ref().unwrap_or(&"1".to_string())) {
                        Ok(converted_b) => {
                            let result_value = a.value - converted_b.value;
                            Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                                octofhir_fhirpath_model::Quantity::new(result_value, a.unit.clone())
                            )))
                        },
                        Err(_) => return None, // Conversion failed, fallback to async
                    }
                } else {
                    return None; // Incompatible units, fallback to async for error handling
                }
            }
            _ => return None, // Fallback to async for complex cases
        };

        // Wrap result in collection as per FHIRPath spec
        Some(result.map(|val| FhirPathValue::Collection(Collection::from(vec![val]))))
    }

    async fn evaluate_binary(&self, left: &FhirPathValue, right: &FhirPathValue, _context: &EvaluationContext) -> Result<FhirPathValue> {
        // Unwrap single-item collections
        let left_unwrapped = self.unwrap_single_collection(left);
        let right_unwrapped = self.unwrap_single_collection(right);
        
        // Try sync path first
        if let Some(result) = self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped) {
            return result;
        }

        // Handle remaining cases
        let result = match (&left_unwrapped, &right_unwrapped) {
            (FhirPathValue::Empty, _) => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            (_, FhirPathValue::Empty) => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            _ => Err(FhirPathError::TypeError {
                message: format!(
                    "Cannot subtract {} from {}",
                    right_unwrapped.type_name(), left_unwrapped.type_name()
                )
            })
        };

        // Wrap result in collection as per FHIRPath spec
        result.map(|val| FhirPathValue::Collection(Collection::from(vec![val])))
    }

    async fn evaluate_unary(&self, value: &FhirPathValue, _context: &EvaluationContext) -> Result<FhirPathValue> {
        // Handle collections first
        let unwrapped = self.unwrap_single_collection(value);
        
        // Unary minus - negate the value
        let result = match &unwrapped {
            FhirPathValue::Integer(i) => {
                i.checked_neg()
                    .map(FhirPathValue::Integer)
                    .ok_or_else(|| FhirPathError::ArithmeticError {
                        message: "Integer overflow in negation".to_string()
                    })
            }
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
            FhirPathValue::String(s) => {
                // Try to convert string to number and negate
                if let Ok(int_val) = s.parse::<i64>() {
                    int_val.checked_neg()
                        .map(FhirPathValue::Integer)
                        .ok_or_else(|| FhirPathError::ArithmeticError {
                            message: "Integer overflow in negation".to_string()
                        })
                } else if let Ok(decimal_val) = s.parse::<Decimal>() {
                    Ok(FhirPathValue::Decimal(-decimal_val))
                } else {
                    Err(FhirPathError::TypeError {
                        message: format!("Cannot apply unary minus to string '{}'", s)
                    })
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Collection(Collection::from(vec![]))),
            FhirPathValue::Collection(items) if items.is_empty() => {
                return Ok(FhirPathValue::Collection(Collection::from(vec![])));
            }
            FhirPathValue::Collection(items) if items.len() > 1 => {
                return Ok(FhirPathValue::Collection(Collection::from(vec![])));
            }
            FhirPathValue::Quantity(q) => {
                // Negate quantity value while preserving unit
                Ok(FhirPathValue::Quantity(std::sync::Arc::new(
                    octofhir_fhirpath_model::Quantity::new(-q.value, q.unit.clone())
                )))
            }
            _ => Err(FhirPathError::TypeError {
                message: format!("Cannot apply unary minus to {}", unwrapped.type_name())
            })
        };

        // Wrap result in collection as per FHIRPath spec
        result.map(|val| FhirPathValue::Collection(Collection::from(vec![val])))
    }
}

#[async_trait]
impl FhirPathOperation for SubtractionOperation {
    fn identifier(&self) -> &str {
        "-"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 6,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            SubtractionOperation::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        match args.len() {
            1 => self.evaluate_unary(&args[0], context).await,
            2 => self.evaluate_binary(&args[0], &args[1], context).await,
            _ => Err(FhirPathError::InvalidArgumentCount {
                function_name: "-".to_string(),
                expected: 1, // Can be 1 or 2
                actual: args.len(),
            }),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        match args.len() {
            1 => {
                // Unary minus sync evaluation
                let unwrapped = self.unwrap_single_collection(&args[0]);
                match &unwrapped {
                    FhirPathValue::Integer(i) => {
                        i.checked_neg()
                            .map(FhirPathValue::Integer)
                            .map(|val| FhirPathValue::Collection(Collection::from(vec![val])))
                            .map(Ok)
                            .or_else(|| Some(Err(FhirPathError::ArithmeticError {
                                message: "Integer overflow in negation".to_string()
                            })))
                    }
                    FhirPathValue::Decimal(d) => Some(Ok(FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Decimal(-d)])))),
                    FhirPathValue::Empty => Some(Ok(FhirPathValue::Collection(Collection::from(vec![])))),
                    _ => None, // Fallback to async for string conversion
                }
            }
            2 => {
                let left_unwrapped = self.unwrap_single_collection(&args[0]);
                let right_unwrapped = self.unwrap_single_collection(&args[1]);
                self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped)
            }
            _ => Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "-".to_string(),
                expected: 1,
                actual: args.len(),
            })),
        }
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl SubtractionOperation {
    /// Unwrap single-item collections to their contained value
    fn unwrap_single_collection(&self, value: &FhirPathValue) -> FhirPathValue {
        match value {
            FhirPathValue::Collection(items) if items.len() == 1 => {
                items.first().unwrap().clone()
            }
            _ => value.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_binary_subtraction() {
        let sub_op = SubtractionOperation::new();

        // Integer subtraction
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(2)];
        let context = create_test_context(FhirPathValue::Empty);
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));

        // Decimal subtraction
        let dec1 = Decimal::from_str("5.5").unwrap();
        let dec2 = Decimal::from_str("2.5").unwrap();
        let args = vec![FhirPathValue::Decimal(dec1), FhirPathValue::Decimal(dec2)];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Decimal(Decimal::from_str("3.0").unwrap())])));

        // Mixed types
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Decimal(Decimal::from_str("2.5").unwrap())];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Decimal(Decimal::from_str("2.5").unwrap())])));
    }

    #[tokio::test]
    async fn test_unary_minus() {
        let sub_op = SubtractionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Unary minus on integer
        let args = vec![FhirPathValue::Integer(42)];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(-42)])));

        // Unary minus on decimal
        let args = vec![FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap())];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Decimal(Decimal::from_str("-3.14").unwrap())])));

        // Unary minus on string number
        let args = vec![FhirPathValue::String("123".into())];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(-123)])));
    }

    #[tokio::test]
    async fn test_subtraction_with_empty() {
        let sub_op = SubtractionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Empty operands
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Empty];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        let args = vec![FhirPathValue::Empty, FhirPathValue::Integer(5)];
        let result = sub_op.evaluate(&args, &context).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let sub_op = SubtractionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        let args = vec![FhirPathValue::Integer(5), FhirPathValue::Integer(2)];
        let sync_result = sub_op.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Integer(3)])));
        assert!(sub_op.supports_sync());
    }

    #[tokio::test]
    async fn test_type_errors() {
        let sub_op = SubtractionOperation::new();
        let context = create_test_context(FhirPathValue::Empty);

        // Cannot subtract string from integer
        let args = vec![FhirPathValue::Integer(5), FhirPathValue::String("hello".into())];
        let result = sub_op.evaluate(&args, &context).await;
        assert!(result.is_err());
    }
}