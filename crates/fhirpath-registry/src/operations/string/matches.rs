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

//! Matches function implementation for FHIRPath

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

/// Matches function: returns true when the value matches the given regular expression
pub struct MatchesFunction;

impl Default for MatchesFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchesFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("matches", OperationType::Function)
            .description("Returns true when the value matches the given regular expression")
            .example("'123'.matches('\\\\d+')")
            .example("Patient.name.family.matches('[A-Z][a-z]+')")
            .parameter(
                "regex",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MatchesFunction {
    fn identifier(&self) -> &str {
        "matches"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(MatchesFunction::create_metadata);
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

        self.evaluate_matches(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_matches(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MatchesFunction {
    fn evaluate_matches(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "matches() requires exactly one argument (regex)".to_string(),
            });
        }

        // Handle empty collections and Empty values first - they should return empty
        match &args[0] {
            FhirPathValue::Collection(items) if items.is_empty() => {
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Collection(items)
                if items.len() == 1 && matches!(items.get(0).unwrap(), FhirPathValue::Empty) =>
            {
                return Ok(FhirPathValue::Empty);
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {}
        }

        // Get regex pattern parameter - handle both direct strings and collections containing strings
        let pattern = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(s) => s,
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "matches() regex parameter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "matches() regex parameter must be a string".to_string(),
                });
            }
        };

        // If pattern is empty string, return empty per spec
        if pattern.as_ref().is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Compile regex with single-line mode (per FHIRPath spec - dot matches newlines)
        let pattern_with_flags =
            if pattern.as_ref().contains("(?") && pattern.as_ref().contains('s') {
                // Pattern already has single-line flag set
                pattern.as_ref().to_string()
            } else {
                // Add single-line flag to enable . to match newlines
                format!("(?s){}", pattern.as_ref())
            };

        let regex =
            Regex::new(&pattern_with_flags).map_err(|e| FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: format!("Invalid regex pattern '{}': {}", pattern.as_ref(), e),
            })?;

        // Handle different input types
        match &context.input {
            FhirPathValue::String(s) => {
                let matches = regex.is_match(s.as_ref());
                Ok(FhirPathValue::Boolean(matches))
            }
            FhirPathValue::Collection(collection) => {
                let mut results = Vec::new();
                for value in collection.iter() {
                    match value {
                        FhirPathValue::String(s) => {
                            let matches = regex.is_match(s.as_ref());
                            results.push(FhirPathValue::Boolean(matches));
                        }
                        FhirPathValue::Empty => {
                            // Empty values are skipped in collections
                        }
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                                expression: None,
                                location: None,
                                message: "matches() can only be applied to strings".to_string(),
                            });
                        }
                    }
                }
                Ok(FhirPathValue::normalize_collection_result(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message:
                    "matches() can only be applied to strings or collections containing strings"
                        .to_string(),
            }),
        }
    }
}
