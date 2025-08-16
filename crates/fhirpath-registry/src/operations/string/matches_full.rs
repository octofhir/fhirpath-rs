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

//! MatchesFull function implementation for FHIRPath

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

/// MatchesFull function: returns true when the entire value matches the given regular expression
pub struct MatchesFullFunction;

impl Default for MatchesFullFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchesFullFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("matchesFull", OperationType::Function)
            .description("Returns true if the entire input string matches the provided regular expression pattern")
            .example("'123'.matchesFull('\\\\d+')")
            .example("'hello123world'.matchesFull('\\\\d+') // returns false")
            .parameter("regex", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for MatchesFullFunction {
    fn identifier(&self) -> &str {
        "matchesFull"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(MatchesFullFunction::create_metadata);
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

        self.evaluate_matches_full(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_matches_full(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl MatchesFullFunction {
    fn evaluate_matches_full(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::EvaluationError {
                message: "matchesFull() requires exactly one argument (regex)".to_string(),
            });
        }

        // Get regex pattern parameter - handle both direct strings and collections containing strings
        let pattern = match &args[0] {
            FhirPathValue::String(s) => s,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(s) => s,
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        message: "matchesFull() regex parameter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(FhirPathError::EvaluationError {
                    message: "matchesFull() regex parameter must be a string".to_string(),
                });
            }
        };

        // If pattern is empty string, return empty per spec
        if pattern.as_ref().is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Handle different input types according to FHIRPath spec
        match &context.input {
            // Single string - normal case
            FhirPathValue::String(s) => {
                // Ensure the pattern matches the entire string by anchoring it
                let anchored_pattern =
                    if pattern.as_ref().starts_with('^') && pattern.as_ref().ends_with('$') {
                        pattern.as_ref().to_string()
                    } else if pattern.as_ref().starts_with('^') {
                        format!("{}$", pattern.as_ref())
                    } else if pattern.as_ref().ends_with('$') {
                        format!("^{}", pattern.as_ref())
                    } else {
                        format!("^{}$", pattern.as_ref())
                    };

                let regex =
                    Regex::new(&anchored_pattern).map_err(|e| FhirPathError::EvaluationError {
                        message: format!("Invalid regex pattern '{}': {}", pattern.as_ref(), e),
                    })?;

                let matches = regex.is_match(s.as_ref());
                Ok(FhirPathValue::Boolean(matches))
            }
            // Empty input collection - return empty per spec
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            // Collection with items - check spec requirements
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    // Empty collection - return empty per spec
                    Ok(FhirPathValue::Empty)
                } else if collection.len() > 1 {
                    // Multiple items - signal error per spec
                    return Err(FhirPathError::EvaluationError {
                        message: "matchesFull() evaluation ended - input collection contains multiple items".to_string(),
                    });
                } else {
                    // Single item in collection - evaluate it
                    match collection.first().unwrap() {
                        FhirPathValue::String(s) => {
                            let anchored_pattern = if pattern.as_ref().starts_with('^')
                                && pattern.as_ref().ends_with('$')
                            {
                                pattern.as_ref().to_string()
                            } else if pattern.as_ref().starts_with('^') {
                                format!("{}$", pattern.as_ref())
                            } else if pattern.as_ref().ends_with('$') {
                                format!("^{}", pattern.as_ref())
                            } else {
                                format!("^{}$", pattern.as_ref())
                            };

                            let regex = Regex::new(&anchored_pattern).map_err(|e| {
                                FhirPathError::EvaluationError {
                                    message: format!(
                                        "Invalid regex pattern '{}': {}",
                                        pattern.as_ref(),
                                        e
                                    ),
                                }
                            })?;

                            let matches = regex.is_match(s.as_ref());
                            Ok(FhirPathValue::Boolean(matches))
                        }
                        FhirPathValue::Empty => Ok(FhirPathValue::Empty),
                        _ => Err(FhirPathError::EvaluationError {
                            message: "matchesFull() can only be applied to strings".to_string(),
                        }),
                    }
                }
            }
            _ => Err(FhirPathError::EvaluationError {
                message:
                    "matchesFull() can only be applied to strings or collections containing strings"
                        .to_string(),
            }),
        }
    }
}
