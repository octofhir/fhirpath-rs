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

//! ReplaceMatches function implementation for FHIRPath

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use regex::Regex;

/// ReplaceMatches function: replaces all instances matching a regex pattern with a substitution string
pub struct ReplaceMatchesFunction;

impl Default for ReplaceMatchesFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplaceMatchesFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("replaceMatches", OperationType::Function)
            .description("Replaces all instances matching a regex pattern with a substitution string, supporting capture groups")
            .example("'hello 123 world 456'.replaceMatches('\\\\d+', 'X')")
            .example("'John Doe'.replaceMatches('(\\\\w+) (\\\\w+)', '$2, $1')")
            .parameter("regex", TypeConstraint::Specific(FhirPathType::String), false)
            .parameter("substitution", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for ReplaceMatchesFunction {
    fn identifier(&self) -> &str {
        "replaceMatches"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ReplaceMatchesFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        self.evaluate_replace_matches(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_replace_matches(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ReplaceMatchesFunction {
    fn evaluate_replace_matches(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 2 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "replaceMatches() requires exactly two arguments (regex, substitution)"
                    .to_string(),
            });
        }

        // Handle collection inputs
        let input = &context.input;
        match input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![]),
                    ));
                }
                if items.len() > 1 {
                    return Ok(FhirPathValue::Collection(
                        octofhir_fhirpath_model::Collection::from(vec![]),
                    ));
                }
                // Single element collection - unwrap and process
                let value = items.first().unwrap();
                self.process_single_value(value, args)
            }
            _ => {
                // Process as single value
                self.process_single_value(input, args)
            }
        }
    }

    fn process_single_value(
        &self,
        value: &FhirPathValue,
        args: &[FhirPathValue],
    ) -> Result<FhirPathValue> {
        // Convert input to string (including numeric values)
        let input_str = match value {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ));
            }
            _ => {
                return Ok(FhirPathValue::Collection(
                    octofhir_fhirpath_model::Collection::from(vec![]),
                ));
            } // Return empty for other non-convertible types
        };

        // Extract and convert pattern parameter to string (handle collections)
        let pattern = self.extract_string_from_value(&args[0])?;
        if pattern.is_none() {
            return Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![]),
            )); // Return empty for non-convertible types
        }
        let pattern = pattern.unwrap();

        // Extract and convert substitution parameter to string (handle collections)
        let substitution = self.extract_string_from_value(&args[1])?;
        if substitution.is_none() {
            return Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![]),
            )); // Return empty for non-convertible types
        }
        let substitution = substitution.unwrap();

        // Special case: empty pattern should return the original string unchanged for replaceMatches
        if pattern.is_empty() {
            return Ok(FhirPathValue::Collection(
                octofhir_fhirpath_model::Collection::from(vec![FhirPathValue::String(
                    input_str.into(),
                )]),
            ));
        }

        // Compile regex
        let regex = Regex::new(&pattern).map_err(|e| FhirPathError::EvaluationError {
            expression: None,
            location: None,
            message: format!("Invalid regex pattern '{pattern}': {e}"),
        })?;

        // Perform regex replacement
        let result = regex.replace_all(&input_str, &substitution);
        Ok(FhirPathValue::Collection(
            octofhir_fhirpath_model::Collection::from(vec![FhirPathValue::String(
                result.to_string().into(),
            )]),
        ))
    }

    /// Extract a string from a FhirPathValue, handling collections and type conversion
    fn extract_string_from_value(&self, value: &FhirPathValue) -> Result<Option<String>> {
        match value {
            FhirPathValue::String(s) => Ok(Some(s.as_ref().to_string())),
            FhirPathValue::Integer(i) => Ok(Some(i.to_string())),
            FhirPathValue::Decimal(d) => Ok(Some(d.to_string())),
            FhirPathValue::Boolean(b) => Ok(Some(b.to_string())),
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    Ok(None)
                } else if items.len() == 1 {
                    // Single element collection - recursively extract
                    self.extract_string_from_value(items.first().unwrap())
                } else {
                    // Multiple elements - can't convert
                    Ok(None)
                }
            }
            _ => Ok(None), // Other types can't be converted
        }
    }
}
