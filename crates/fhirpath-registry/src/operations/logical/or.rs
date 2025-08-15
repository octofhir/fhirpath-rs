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

//! Logical OR operator implementation

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Logical OR operator
#[derive(Debug, Clone)]
pub struct OrOperation;

impl OrOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("or", OperationType::BinaryOperator {
            precedence: 1,
            associativity: Associativity::Left,
        })
            .description("Logical OR operator with three-valued logic")
            .example("true or false")
            .example("false or false")
            .example("empty() or true")
            .parameter("left", TypeConstraint::Specific(FhirPathType::Boolean), false)
            .parameter("right", TypeConstraint::Specific(FhirPathType::Boolean), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    /// Unwrap single-item collections to their contained value
    fn unwrap_single_collection(&self, value: &FhirPathValue) -> FhirPathValue {
        match value {
            FhirPathValue::Collection(items) if items.len() == 1 => {
                items.first().unwrap().clone()
            }
            _ => value.clone()
        }
    }

    fn to_boolean(value: &FhirPathValue) -> Result<Option<bool>> {
        match value {
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(None)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    // Multi-element collections in logical operations result in empty (not error)
                    Ok(None)
                }
            },
            // Non-boolean values in logical operations result in empty (not error)
            // This follows FHIRPath specification for type conversion in boolean context
            _ => Ok(None),
        }
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

        let left_bool = match Self::to_boolean(left) {
            Ok(b) => b,
            Err(_) => return None, // Fallback to async for type errors
        };
        
        let right_bool = match Self::to_boolean(right) {
            Ok(b) => b,
            Err(_) => return None, // Fallback to async for type errors
        };

        // Three-valued logic for OR
        let result = match (left_bool, right_bool) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (Some(false), Some(false)) => Some(false),
            _ => None, // If either is empty/null and the other is not true
        };

        // Wrap result in collection as per FHIRPath spec
        Some(match result {
            Some(b) => Ok(FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(b)]))),
            None => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
        })
    }
}

#[async_trait]
impl FhirPathOperation for OrOperation {
    fn identifier(&self) -> &str {
        "or"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 1,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            OrOperation::create_metadata()
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

        // Unwrap single-item collections
        let left_unwrapped = self.unwrap_single_collection(&args[0]);
        let right_unwrapped = self.unwrap_single_collection(&args[1]);
        
        // Try sync path first
        if let Some(result) = self.evaluate_binary_sync(&left_unwrapped, &right_unwrapped) {
            return result;
        }

        // Handle complex cases (async fallback)
        let left = Self::to_boolean(&left_unwrapped)?;
        let right = Self::to_boolean(&right_unwrapped)?;

        // Three-valued logic for OR
        let result = match (left, right) {
            (Some(true), _) | (_, Some(true)) => Some(true),
            (Some(false), Some(false)) => Some(false),
            _ => None, // If either is empty/null and the other is not true
        };

        // Wrap result in collection as per FHIRPath spec
        match result {
            Some(b) => Ok(FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(b)]))),
            None => Ok(FhirPathValue::Collection(Collection::from(vec![]))),
        }
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 2 {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 2, 
                actual: args.len() 
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::MockModelProvider;
        use crate::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
    }

    #[tokio::test]
    async fn test_or_operation() {
        let op = OrOperation::new();
        let ctx = create_test_context();

        // Test true or false
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test false or false
        let args = vec![FhirPathValue::Boolean(false), FhirPathValue::Boolean(false)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)])));

        // Test true or false
        let args = vec![FhirPathValue::Boolean(false), FhirPathValue::Boolean(true)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test true or true
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(true)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test true or empty (should return true)
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Empty];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test empty or true (should return true)
        let args = vec![FhirPathValue::Empty, FhirPathValue::Boolean(true)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test false or empty (should return empty collection)
        let args = vec![FhirPathValue::Boolean(false), FhirPathValue::Empty];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test empty or false (should return empty collection)
        let args = vec![FhirPathValue::Empty, FhirPathValue::Boolean(false)];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test empty or empty (should return empty collection)
        let args = vec![FhirPathValue::Empty, FhirPathValue::Empty];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[tokio::test]
    async fn test_or_operation_with_collections() {
        let op = OrOperation::new();
        let ctx = create_test_context();

        // Test single-element collection inputs
        let args = vec![
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])),
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(false)]))
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));

        // Test empty collection with boolean
        let args = vec![
            FhirPathValue::Collection(Collection::from(vec![])),
            FhirPathValue::Boolean(true)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test boolean with empty collection
        let args = vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::Collection(Collection::from(vec![]))
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));

        // Test multi-element collection (should return empty)
        let args = vec![
            FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)])),
            FhirPathValue::Boolean(true)
        ];
        let result = op.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Collection(Collection::from(vec![])));
    }

    #[test]
    fn test_sync_evaluation() {
        let op = OrOperation::new();
        let ctx = create_test_context();

        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Boolean(false)];
        let sync_result = op.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(sync_result, FhirPathValue::Collection(Collection::from(vec![FhirPathValue::Boolean(true)])));
        assert!(op.supports_sync());
    }
}