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

//! As operator implementation - type casting that returns the value if it matches the type

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{
        FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
        TypeConstraint,
    },
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// As operator - returns the value if it is of the specified type, otherwise returns empty
#[derive(Debug, Clone)]
pub struct AsOperation;

impl Default for AsOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl AsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("as", OperationType::Function)
            .description(
                "Type casting function - returns the input if it is of the specified type, otherwise empty",
            )
            .example("Observation.value.as(Quantity).unit")
            .example("(Observation.value as Quantity).unit")
            .parameter(
                "type",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Any)
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    /// Check if value is of specified type using the same logic as IsOperation
    pub async fn check_type_with_provider(
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        // Use the same type checking logic as IsOperation
        crate::operations::types::is::IsOperation::check_type_with_provider(
            value, type_name, context,
        )
        .await
    }
}

#[async_trait]
impl FhirPathOperation for AsOperation {
    fn identifier(&self) -> &str {
        "as"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(AsOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.is_empty() {
            // If no arguments provided, return empty collection
            return Ok(FhirPathValue::Empty);
        }

        // Handle both function-style (1 arg) and binary-style (2 args) calls
        let (value_to_check, type_name) = if args.len() == 1 {
            // Function-style: value.as(Type) - use context.input as the value
            let type_name = context
                .model_provider
                .extract_type_name(&args[0])
                .map_err(|e| FhirPathError::TypeError {
                    message: format!("as operator {e}"),
                })?;
            (&context.input, type_name)
        } else if args.len() == 2 {
            // Binary-style: value as Type - use first arg as value, second as type
            // Check if the value is an empty collection - if so, return empty
            if let FhirPathValue::Collection(c) = &args[0] {
                if c.is_empty() {
                    return Ok(FhirPathValue::Empty);
                }
            }

            let type_name = context
                .model_provider
                .extract_type_name(&args[1])
                .map_err(|e| FhirPathError::TypeError {
                    message: format!("as operator {e}"),
                })?;
            (&args[0], type_name)
        } else {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        };

        let result = match value_to_check {
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    FhirPathValue::Empty
                } else if c.len() == 1 {
                    let value = c.first().unwrap();
                    let matches_type =
                        Self::check_type_with_provider(value, &type_name, context).await?;
                    if matches_type {
                        value.clone()
                    } else {
                        FhirPathValue::Empty
                    }
                } else {
                    // More than one item - return error per FHIRPath spec
                    return Err(FhirPathError::TypeError {
                        message: "as operator requires a single item".to_string(),
                    });
                }
            }
            single_value => {
                let matches_type =
                    Self::check_type_with_provider(single_value, &type_name, context).await?;
                if matches_type {
                    single_value.clone()
                } else {
                    FhirPathValue::Empty
                }
            }
        };

        Ok(result)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Type checking requires async ModelProvider calls, so cannot be done synchronously
        None
    }

    fn supports_sync(&self) -> bool {
        false // Type checking requires async ModelProvider calls
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 1 && args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
