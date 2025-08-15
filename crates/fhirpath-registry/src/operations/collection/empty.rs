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

//! Empty function implementation

use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;

/// Empty function - checks if collection is empty
#[derive(Debug, Clone)]
pub struct EmptyFunction;

impl EmptyFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("empty", OperationType::Function)
            .description("Returns true if the input collection is empty")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .example("Patient.name.empty()")
            .example("().empty()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for EmptyFunction {
    fn identifier(&self) -> &str {
        "empty"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| EmptyFunction::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(true)),
            FhirPathValue::Collection(c) => Ok(FhirPathValue::Boolean(c.is_empty())),
            _ => Ok(FhirPathValue::Boolean(false)),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            }));
        }

        match &context.input {
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Boolean(true))),
            FhirPathValue::Collection(c) => Some(Ok(FhirPathValue::Boolean(c.is_empty()))),
            _ => Some(Ok(FhirPathValue::Boolean(false))),
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
    async fn test_empty_function() {
        let func = EmptyFunction::new();

        // Test empty collection
        let ctx = EvaluationContext::new(
            FhirPathValue::Empty,
            context.registry.clone(),
            context.model_provider.clone(),
        );
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test non-empty collection
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::collection(vec![FhirPathValue::Integer(1)]), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));

        // Test single value
        let ctx = {
            use std::sync::Arc;
            use octofhir_fhirpath_model::provider::MockModelProvider;
            use octofhir_fhirpath_registry::FhirPathRegistry;
            
            let registry = Arc::new(FhirPathRegistry::new());
            let model_provider = Arc::new(MockModelProvider::new());
            EvaluationContext::new(FhirPathValue::Integer(42), registry, model_provider)
        };
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }
}
