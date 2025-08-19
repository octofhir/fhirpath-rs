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

//! Is operator implementation - type checking

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
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use sonic_rs::JsonValueTrait;

/// Is operator - checks if value is of a specified type
#[derive(Debug, Clone)]
pub struct IsOperation;

impl Default for IsOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl IsOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("is", OperationType::Function)
            .description(
                "Type checking function - returns true if the input is of the specified type",
            )
            .example("Patient.active.is(Boolean)")
            .example("Patient.name.is(Collection)")
            .example("Patient.is(Patient)")
            .parameter(
                "type",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    pub async fn check_type_with_provider(
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        // Normalize type name - handle both FHIR.String and string formats
        let normalized_type = Self::normalize_type_name(type_name);

        // Handle primitive FHIRPath types first (these don't need ModelProvider)
        match normalized_type.to_lowercase().as_str() {
            "boolean" => {
                let result = matches!(value, FhirPathValue::Boolean(_));
                return Ok(result);
            }
            "integer" => return Ok(matches!(value, FhirPathValue::Integer(_))),
            "decimal" => {
                // FHIR decimal can be represented as Integer or Decimal
                return Ok(matches!(
                    value,
                    FhirPathValue::Decimal(_) | FhirPathValue::Integer(_)
                ));
            }
            "string" => return Ok(matches!(value, FhirPathValue::String(_))),
            "date" => return Ok(matches!(value, FhirPathValue::Date(_))),
            "datetime" => return Ok(matches!(value, FhirPathValue::DateTime(_))),
            "time" => return Ok(matches!(value, FhirPathValue::Time(_))),
            "collection" => return Ok(matches!(value, FhirPathValue::Collection(_))),
            "empty" => return Ok(matches!(value, FhirPathValue::Empty)),
            "quantity" => return Ok(matches!(value, FhirPathValue::Quantity(_))),
            // FHIR primitive types that are represented as strings
            "uri" | "url" | "canonical" | "uuid" | "oid" | "id" | "code" | "markdown"
            | "base64binary" | "instant" => {
                return Ok(matches!(value, FhirPathValue::String(_)));
            }
            // FHIR integer types
            "positiveint" | "unsignedint" => {
                return Ok(matches!(value, FhirPathValue::Integer(_)));
            }
            _ => {}
        }

        // For FHIR types, use ModelProvider to check type compatibility
        let value_type = Self::extract_fhir_type(value);
        if let Some(value_type) = value_type {
            // Use ModelProvider for accurate FHIR type checking
            let is_compatible = context
                .model_provider
                .is_type_compatible(&value_type, &normalized_type)
                .await;
            Ok(is_compatible)
        } else {
            // Not a FHIR resource/type
            Ok(false)
        }
    }

    /// Normalize type names to handle various namespace formats per FHIRPath specification
    /// Supports: String, FHIR.String, System.String, `String`, etc.
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

    fn extract_fhir_type(value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::Resource(resource) => resource.resource_type().map(|s| s.to_string()),
            FhirPathValue::JsonValue(json) => json
                .as_inner()
                .get("resourceType")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        }
    }
}

#[async_trait]
impl FhirPathOperation for IsOperation {
    fn identifier(&self) -> &str {
        "is"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(IsOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.is_empty() {
            // If no arguments provided, return empty collection
            return Ok(FhirPathValue::Collection(Collection::from(vec![])));
        }

        // Handle both function-style (1 arg) and binary-style (2 args) calls
        let (value_to_check, type_name) = if args.len() == 1 {
            // Function-style: value.is(Type) - use context.input as the value
            let type_name = context
                .model_provider
                .extract_type_name(&args[0])
                .map_err(|e| FhirPathError::TypeError {
                    message: format!("is operator {e}"),
                })?;
            (&context.input, type_name)
        } else if args.len() == 2 {
            // Binary-style: value is Type - use first arg as value, second as type
            // Check if the value is empty - if so, just return false
            match &args[0] {
                FhirPathValue::Empty => {
                    return Ok(FhirPathValue::Boolean(false));
                }
                FhirPathValue::Collection(c) if c.is_empty() => {
                    return Ok(FhirPathValue::Boolean(false));
                }
                _ => {}
            }

            let type_name = context
                .model_provider
                .extract_type_name(&args[1])
                .map_err(|e| FhirPathError::TypeError {
                    message: format!("is operator {e}"),
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
            FhirPathValue::Empty => {
                // Empty values are not instances of any type, so return false
                false
            }
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    false
                } else if c.len() == 1 {
                    let value = c.first().unwrap();
                    Self::check_type_with_provider(value, &type_name, context).await?
                } else {
                    type_name.to_lowercase() == "collection"
                }
            }
            single_value => {
                Self::check_type_with_provider(single_value, &type_name, context).await?
            }
        };

        if result {
            Ok(FhirPathValue::Collection(Collection::from(vec![
                FhirPathValue::Boolean(true),
            ])))
        } else {
            Ok(FhirPathValue::Collection(Collection::from(vec![])))
        }
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
        if args.len() != 2 {
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
