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

//! Exponential function implementation

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use crate::operations::EvaluationContext;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use async_trait::async_trait;

/// Exponential function - returns e raised to the power of the input
#[derive(Debug, Clone)]
pub struct ExpFunction;

impl ExpFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("exp", OperationType::Function)
            .description("Returns e raised to the power of the input")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .example("(1).exp()")
            .example("(2.0).exp()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ExpFunction {
    fn identifier(&self) -> &str {
        "exp"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            ExpFunction::create_metadata()
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
            FhirPathValue::Integer(n) => {
                let result = (*n as f64).exp();
                if result.is_finite() {
                    Ok(FhirPathValue::Decimal(Decimal::try_from(result).unwrap_or_default()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            },
            FhirPathValue::Decimal(n) => {
                let result = n.to_f64().unwrap_or(0.0).exp();
                if result.is_finite() {
                    Ok(FhirPathValue::Decimal(Decimal::try_from(result).unwrap_or_default()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(c.first().unwrap().clone(), context.registry.clone(), context.model_provider.clone());
                    self.evaluate(args, &item_context).await
                } else {
                    Err(FhirPathError::TypeError { message: "exp() can only be applied to single numeric values".to_string() })
                }
            },
            _ => Err(FhirPathError::TypeError { 
                message: format!("exp() can only be applied to numeric values, got {}", context.input.type_name()) 
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
            FhirPathValue::Integer(n) => {
                let result = (*n as f64).exp();
                if result.is_finite() {
                    Some(Ok(FhirPathValue::Decimal(Decimal::try_from(result).unwrap_or_default())))
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            },
            FhirPathValue::Decimal(n) => {
                let result = n.to_f64().unwrap_or(0.0).exp();
                if result.is_finite() {
                    Some(Ok(FhirPathValue::Decimal(Decimal::try_from(result).unwrap_or_default())))
                } else {
                    Some(Ok(FhirPathValue::Empty))
                }
            },
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if c.len() == 1 {
                    let item_context = EvaluationContext::new(c.first().unwrap().clone(), context.registry.clone(), context.model_provider.clone());
                    self.try_evaluate_sync(args, &item_context)
                } else {
                    Some(Err(FhirPathError::TypeError { message: "exp() can only be applied to single numeric values".to_string() }))
                }
            },
            _ => Some(Err(FhirPathError::TypeError { 
                message: format!("exp() can only be applied to numeric values, got {}", context.input.type_name()) 
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
    async fn test_exp_function() {
        let func = ExpFunction::new();

        // Test exp(0) = 1
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(0), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        if let FhirPathValue::Decimal(d) = result {
            assert!((d.to_f64().unwrap() - 1.0).abs() < 0.0001);
        } else {
            panic!("Expected decimal result");
        }

        // Test exp(1) ≈ e ≈ 2.718
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(1), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        if let FhirPathValue::Decimal(d) = result {
            let e_approx = d.to_f64().unwrap();
            assert!(e_approx > 2.71 && e_approx < 2.72);
        } else {
            panic!("Expected decimal result");
        }

        // Test with decimal input
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Decimal(Decimal::try_from(0.5).unwrap()), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert!(matches!(result, FhirPathValue::Decimal(_)));

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
    async fn test_exp_sync() {
        let func = ExpFunction::new();
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(2), registry, model_provider)
        };
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert!(matches!(result, FhirPathValue::Decimal(_)));
    }
}