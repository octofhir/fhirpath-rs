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

    /// Normalize type names to handle various namespace formats per FHIRPath specification
    /// Uses same logic as IsOperation for consistency
    fn normalize_type_name(type_name: &str) -> String {
        // Handle backticks first
        let cleaned = type_name.trim_matches('`');

        // Handle various namespace prefixes per FHIRPath specification
        if let Some(stripped) = cleaned.strip_prefix("FHIR.") {
            stripped.to_string()
        } else if let Some(stripped) = cleaned.strip_prefix("fhir.") {
            stripped.to_string()
        } else if let Some(stripped) = cleaned.strip_prefix("System.") {
            stripped.to_string()
        } else if let Some(stripped) = cleaned.strip_prefix("system.") {
            stripped.to_string()
        } else {
            cleaned.to_string()
        }
    }

    /// Try to cast value to specified type using ModelProvider's advanced casting
    pub async fn try_cast_to_type(
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> Result<Option<FhirPathValue>> {
        // Normalize type name first
        let normalized_type = Self::normalize_type_name(type_name);

        // Use ModelProvider's advanced type casting which supports:
        // - Inheritance (upcast/downcast)
        // - Primitive type conversions
        // - Abstract type handling
        let cast_result = context
            .model_provider
            .try_cast_value(value, &normalized_type)
            .await
            .map_err(|e| FhirPathError::TypeError {
                message: format!("Type casting failed: {e}"),
            })?;
        Ok(cast_result)
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
                    // Try to cast using ModelProvider's advanced casting
                    match Self::try_cast_to_type(value, &type_name, context).await? {
                        Some(cast_value) => cast_value,
                        None => FhirPathValue::Empty,
                    }
                } else {
                    // More than one item - return error per FHIRPath spec
                    return Err(FhirPathError::TypeError {
                        message: "as operator requires a single item".to_string(),
                    });
                }
            }
            single_value => {
                // Try to cast using ModelProvider's advanced casting
                match Self::try_cast_to_type(single_value, &type_name, context).await? {
                    Some(cast_value) => cast_value,
                    None => FhirPathValue::Empty,
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
