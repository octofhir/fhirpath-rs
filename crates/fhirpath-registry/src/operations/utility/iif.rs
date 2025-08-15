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

//! IIF (Immediate If) function implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// IIF function - conditional expression (if-then-else)
#[derive(Debug, Clone)]
pub struct IifFunction;

impl IifFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("iif", OperationType::Function)
            .description("Conditional expression: returns second argument if first is true, third argument otherwise")
            .example("iif(true, 'yes', 'no')")
            .example("iif(Patient.active, 'Active', 'Inactive')")
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn to_boolean(value: &FhirPathValue) -> Result<bool> {
        match value {
            FhirPathValue::Boolean(b) => Ok(*b),
            FhirPathValue::Empty => Ok(false),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(false)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    // Multiple items are truthy
                    Ok(true)
                }
            },
            _ => Ok(true), // Non-empty values are truthy
        }
    }
}

#[async_trait]
impl FhirPathOperation for IifFunction {
    fn identifier(&self) -> &str {
        "iif"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            IifFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() < 2 || args.len() > 3 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2, // Updated to show minimum expected
                actual: args.len()
            });
        }

        let condition = Self::to_boolean(&args[0])?;

        if condition {
            Ok(args[1].clone())
        } else {
            // Return third argument if provided, otherwise return empty collection
            if args.len() == 3 {
                Ok(args[2].clone())
            } else {
                Ok(FhirPathValue::Collection(Collection::new()))
            }
        }
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if args.len() != 3 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 3,
                actual: args.len()
            }));
        }

        match Self::to_boolean(&args[0]) {
            Ok(condition) => {
                if condition {
                    Some(Ok(args[1].clone()))
                } else {
                    Some(Ok(args[2].clone()))
                }
            },
            Err(e) => Some(Err(e)),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_iif_function() {
        use octofhir_fhirpath_model::MockModelProvider;
        use crate::FhirPathRegistry;
        use std::sync::Arc;

        let func = IifFunction::new();
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        // Test true condition
        let args = vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::String("yes".into()),
            FhirPathValue::String("no".into())
        ];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::String("yes".into()));

        // Test false condition
        let args = vec![
            FhirPathValue::Boolean(false),
            FhirPathValue::String("yes".into()),
            FhirPathValue::String("no".into())
        ];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::String("no".into()));

        // Test empty condition (false)
        let args = vec![
            FhirPathValue::Empty,
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2)
        ];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));

        // Test non-boolean condition (truthy)
        let args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2)
        ];
        let result = func.evaluate(&args, &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(1));
    }

    #[tokio::test]
    async fn test_iif_sync() {
        use octofhir_fhirpath_model::MockModelProvider;
        use crate::FhirPathRegistry;
        use std::sync::Arc;

        let func = IifFunction::new();
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        let args = vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Integer(42),
            FhirPathValue::Integer(0)
        ];
        let result = func.try_evaluate_sync(&args, &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[tokio::test]
    async fn test_iif_invalid_args() {
        use octofhir_fhirpath_model::MockModelProvider;
        use crate::FhirPathRegistry;
        use std::sync::Arc;

        let func = IifFunction::new();
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        let ctx = EvaluationContext::new(FhirPathValue::Empty, registry, model_provider);

        // Too few arguments
        let args = vec![FhirPathValue::Boolean(true), FhirPathValue::Integer(1)];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());

        // Too many arguments
        let args = vec![
            FhirPathValue::Boolean(true),
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3)
        ];
        let result = func.evaluate(&args, &ctx).await;
        assert!(result.is_err());
    }
}
