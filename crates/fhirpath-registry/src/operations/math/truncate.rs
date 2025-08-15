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

//! Truncate function implementation

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use crate::operations::EvaluationContext;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;
use async_trait::async_trait;

/// Truncate function - removes decimal part
#[derive(Debug, Clone)]
pub struct TruncateFunction;

impl TruncateFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("truncate", OperationType::Function)
            .description("Returns the integer part of a decimal (removes decimal part without rounding)")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .example("(3.14).truncate()")
            .example("(-2.9).truncate()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for TruncateFunction {
    fn identifier(&self) -> &str {
        "truncate"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            TruncateFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 0, 
                actual: args.len() 
            });
        }

        match &context.input {
            FhirPathValue::Integer(n) => Ok(FhirPathValue::Integer(*n)), // Already integer
            FhirPathValue::Decimal(n) => {
                let truncated = n.trunc();
                Ok(FhirPathValue::Integer(truncated.to_i64().unwrap_or(0)))
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(c.first().unwrap().clone(), context.registry.clone(), context.model_provider.clone());
                    self.evaluate(args, &item_context).await
                } else {
                    Err(FhirPathError::TypeError { message: "truncate() can only be applied to single numeric values".to_string() })
                }
            },
            _ => Err(FhirPathError::TypeError { 
                message: format!("truncate() can only be applied to numeric values, got {}", context.input.type_name()) 
            }),
        }
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 0, 
                actual: args.len() 
            }));
        }

        match &context.input {
            FhirPathValue::Integer(n) => Some(Ok(FhirPathValue::Integer(*n))), // Already integer
            FhirPathValue::Decimal(n) => {
                let truncated = n.trunc();
                Some(Ok(FhirPathValue::Integer(truncated.to_i64().unwrap_or(0))))
            },
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(c.first().unwrap().clone(), context.registry.clone(), context.model_provider.clone());
                    self.try_evaluate_sync(args, &item_context)
                } else {
                    Some(Err(FhirPathError::TypeError { message: "truncate() can only be applied to single numeric values".to_string() }))
                }
            },
            _ => Some(Err(FhirPathError::TypeError { 
                message: format!("truncate() can only be applied to numeric values, got {}", context.input.type_name()) 
            })),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_truncate_function() {
        let func = TruncateFunction::new();

        // Test positive decimal
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("3.14").unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));

        // Test negative decimal (truncate towards zero)
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("-2.9").unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(-2)); // Truncate towards zero, not floor

        // Test integer (no change)
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(5), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));

        // Test small positive decimal
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("0.9").unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0));

        // Test small negative decimal
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("-0.9").unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(0)); // Truncate towards zero

        // Test empty
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Empty, registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_truncate_sync() {
        let func = TruncateFunction::new();
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("7.8").unwrap()), registry, model_provider)
        };
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Integer(7));
    }
}