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

//! conformsTo function implementation - validates resource against StructureDefinition

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// conformsTo function - validates if resource conforms to a StructureDefinition
#[derive(Debug, Clone)]
pub struct ConformsToFunction;

impl Default for ConformsToFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConformsToFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("conformsTo", OperationType::Function)
            .description(
                "Tests whether the current resource conforms to the given StructureDefinition",
            )
            .parameter("url", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .example("conformsTo('http://hl7.org/fhir/StructureDefinition/Patient')")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ConformsToFunction {
    fn identifier(&self) -> &str {
        "conformsTo"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ConformsToFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let profile_url = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(coll) if coll.len() == 1 => {
                match coll.iter().next().unwrap() {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(FhirPathError::TypeError {
                            message: "conformsTo() profile URL argument must be a string"
                                .to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(FhirPathError::TypeError {
                    message: "conformsTo() profile URL argument must be a string".to_string(),
                });
            }
        };

        // Use ModelProvider to validate resource against profile
        let conforms = context
            .model_provider
            .validates_resource_against_profile(&context.input, profile_url)
            .await;

        match conforms {
            Ok(result) => Ok(FhirPathValue::Boolean(result)),
            Err(_) => {
                // If validation fails (e.g., invalid profile URL), return empty
                Ok(FhirPathValue::Empty)
            }
        }
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // conformsTo() requires ModelProvider which is async, so force async evaluation
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
