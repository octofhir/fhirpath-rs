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

//! Exists function implementation

use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;

/// Exists function - checks if any items exist (optionally matching criteria)
#[derive(Debug, Clone)]
pub struct ExistsFunction;

impl Default for ExistsFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ExistsFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("exists", OperationType::Function)
            .description("Returns true if the collection is not empty")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .example("Patient.name.exists()")
            .example("Patient.telecom.exists()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ExistsFunction {
    fn identifier(&self) -> &str {
        "exists"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ExistsFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() > 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Basic implementation - just check if collection is not empty
        // TODO: Add support for criteria expression evaluation
        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
            FhirPathValue::Collection(c) => Ok(FhirPathValue::Boolean(!c.is_empty())),
            _ => Ok(FhirPathValue::Boolean(true)),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if args.len() > 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        match &context.input {
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Boolean(false))),
            FhirPathValue::Collection(c) => Some(Ok(FhirPathValue::Boolean(!c.is_empty()))),
            _ => Some(Ok(FhirPathValue::Boolean(true))),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
