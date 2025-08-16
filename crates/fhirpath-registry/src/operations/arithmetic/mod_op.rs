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

//! Modulo operation (mod) implementation for FHIRPath

use crate::metadata::{
    Associativity, FhirPathType, MetadataBuilder, OperationMetadata, OperationType,
    PerformanceComplexity, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use rust_decimal::Decimal;

/// Modulo operation (mod) - returns remainder of division
pub struct ModOperation;

impl Default for ModOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl ModOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new(
            "mod",
            OperationType::BinaryOperator {
                precedence: 7,
                associativity: Associativity::Left,
            },
        )
        .description("Modulo operation - returns remainder of division")
        .example("10 mod 3")
        .example("7 mod 2")
        .example("7.5 mod 2.0")
        .returns(TypeConstraint::OneOf(vec![
            FhirPathType::Integer,
            FhirPathType::Decimal,
        ]))
        .performance(PerformanceComplexity::Constant, true)
        .build()
    }

    fn evaluate_binary_sync(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> Option<Result<FhirPathValue>> {
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

        // Handle empty values
        match (left, right) {
            (FhirPathValue::Empty, _) | (_, FhirPathValue::Empty) => {
                return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
            }
            _ => {}
        }

        // Actual arithmetic operations on scalar values
        let result = match (left, right) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                } else {
                    Ok(FhirPathValue::Integer(a % b))
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => {
                if *b == Decimal::ZERO {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                } else {
                    Ok(FhirPathValue::Decimal(a % b))
                }
            }
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
                if *b == Decimal::ZERO {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                } else {
                    match Decimal::try_from(*a) {
                        Ok(a_decimal) => Ok(FhirPathValue::Decimal(a_decimal % b)),
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal".to_string(),
                        }),
                    }
                }
            }
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
                if *b == 0 {
                    return Some(Ok(FhirPathValue::Collection(Collection::from(vec![]))));
                } else {
                    match Decimal::try_from(*b) {
                        Ok(b_decimal) => Ok(FhirPathValue::Decimal(a % b_decimal)),
                        Err(_) => Err(FhirPathError::ArithmeticError {
                            message: "Cannot convert integer to decimal".to_string(),
                        }),
                    }
                }
            }
            _ => return None, // Fallback to async for complex cases
        };

        // Wrap result in collection as per FHIRPath spec
        Some(result.map(|val| FhirPathValue::Collection(Collection::from(vec![val]))))
    }

    async fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
        _context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Unwrap single-item collections
        let left_unwrapped = self.unwrap_single_collection(left);
        let right_unwrapped = self.unwrap_single_collection(right);

        // Try sync path first
        if let Some(result) = self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped) {
            return result;
        }

        // Handle other error cases
        Err(FhirPathError::TypeError {
            message: format!(
                "Cannot perform modulo of {} by {}",
                left_unwrapped.type_name(),
                right_unwrapped.type_name()
            ),
        })
    }
}

#[async_trait]
impl FhirPathOperation for ModOperation {
    fn identifier(&self) -> &str {
        "mod"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 7,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ModOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "mod".to_string(),
                expected: 2,
                actual: args.len(),
            });
        }

        self.evaluate_binary(&args[0], &args[1], context).await
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: "mod".to_string(),
                expected: 2,
                actual: args.len(),
            }));
        }

        let left_unwrapped = self.unwrap_single_collection(&args[0]);
        let right_unwrapped = self.unwrap_single_collection(&args[1]);
        self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped)
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ModOperation {
    /// Unwrap single-item collections to their contained value
    fn unwrap_single_collection(&self, value: &FhirPathValue) -> FhirPathValue {
        match value {
            FhirPathValue::Collection(items) if items.len() == 1 => items.first().unwrap().clone(),
            _ => value.clone(),
        }
    }
}
