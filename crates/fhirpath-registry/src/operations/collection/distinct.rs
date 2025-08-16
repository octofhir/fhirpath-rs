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

//! Distinct function implementation

use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;

/// Distinct function - removes duplicate values from collection
#[derive(Debug, Clone)]
pub struct DistinctFunction;

impl Default for DistinctFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DistinctFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("distinct", OperationType::Function)
            .description("Returns a collection with duplicate values removed")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("(1 | 2 | 1 | 3).distinct()")
            .example("Patient.name.family.distinct()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for DistinctFunction {
    fn identifier(&self) -> &str {
        "distinct"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(DistinctFunction::create_metadata);
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
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                let mut seen = HashSet::new();
                let mut result = Vec::new();

                for item in c.iter() {
                    // Basic deduplication - this is a placeholder implementation
                    // TODO: Implement proper FHIRPath value comparison
                    let key = format!("{item:?}");
                    if seen.insert(key) {
                        result.push(item.clone());
                    }
                }

                if result.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if result.len() == 1 {
                    Ok(result.into_iter().next().unwrap())
                } else {
                    Ok(FhirPathValue::collection(result))
                }
            }
            single => Ok(single.clone()), // Single values are already distinct
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
            FhirPathValue::Empty => Some(Ok(FhirPathValue::Empty)),
            FhirPathValue::Collection(c) => {
                let mut seen = HashSet::new();
                let mut result = Vec::new();

                for item in c.iter() {
                    let key = format!("{item:?}");
                    if seen.insert(key) {
                        result.push(item.clone());
                    }
                }

                if result.is_empty() {
                    Some(Ok(FhirPathValue::Empty))
                } else if result.len() == 1 {
                    Some(Ok(result.into_iter().next().unwrap()))
                } else {
                    Some(Ok(FhirPathValue::collection(result)))
                }
            }
            single => Some(Ok(single.clone())),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
