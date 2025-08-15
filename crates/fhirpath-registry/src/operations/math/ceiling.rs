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

//! Ceiling function implementation

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use crate::operations::EvaluationContext;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;
use async_trait::async_trait;

/// Ceiling function - rounds up to nearest integer
#[derive(Debug, Clone)]
pub struct CeilingFunction;

impl CeilingFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("ceiling", OperationType::Function)
            .description("Returns the smallest integer greater than or equal to the input")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .example("(1.5).ceiling()")
            .example("(-1.5).ceiling()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for CeilingFunction {
    fn identifier(&self) -> &str {
        "ceiling"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            CeilingFunction::create_metadata()
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
            FhirPathValue::Decimal(n) => Ok(FhirPathValue::Integer(n.ceil().to_i64().unwrap_or(0))),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(c.first().unwrap().clone(), context.registry.clone(), context.model_provider.clone());
                    self.evaluate(args, &item_context).await
                } else {
                    Err(FhirPathError::TypeError { message: "ceiling() can only be applied to single numeric values".to_string() })
                }
            },
            _ => Err(FhirPathError::TypeError { 
                message: format!("ceiling() can only be applied to numeric values, got {}", context.input.type_name()) 
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
            FhirPathValue::Integer(n) => Some(Ok(FhirPathValue::Integer(*n))),
            FhirPathValue::Decimal(n) => Some(Ok(FhirPathValue::Integer(n.ceil().to_i64().unwrap_or(0)))),
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(c.first().unwrap().clone(), context.registry.clone(), context.model_provider.clone());
                    self.try_evaluate_sync(args, &item_context)
                } else {
                    Some(Err(FhirPathError::TypeError { message: "ceiling() can only be applied to single numeric values".to_string() }))
                }
            },
            _ => Some(Err(FhirPathError::TypeError { 
                message: format!("ceiling() can only be applied to numeric values, got {}", context.input.type_name()) 
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

    #[tokio::test]
    async fn test_ceiling_function() {
        let func = CeilingFunction::new();

        // Test positive decimal
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("1.5").unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(2));

        // Test negative decimal
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("-1.5").unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(-1));

        // Test integer (no change)
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(5), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Integer(5));

        // Test empty
        let ctx = EvaluationContext::new(FhirPathValue::Empty, context.registry.clone(), context.model_provider.clone());
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_ceiling_sync() {
        let func = CeilingFunction::new();
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::from_str("2.3").unwrap()), registry, model_provider)
        };
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Integer(3));
    }
}