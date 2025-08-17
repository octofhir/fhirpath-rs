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

//! Escape function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// Escape function - escapes special characters in a string
#[derive(Debug, Clone)]
pub struct EscapeFunction;

impl Default for EscapeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl EscapeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("escape", OperationType::Function)
            .description("Escapes special characters in a string for a given target format")
            .parameter(
                "target",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .example("'\"1<2\"'.escape('html')")
            .example("'\"1<2\"'.escape('json')")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn escape_string(&self, input: &str, target: &str) -> String {
        match target {
            "html" => {
                let mut result = String::with_capacity(input.len() * 2);
                for ch in input.chars() {
                    match ch {
                        '<' => result.push_str("&lt;"),
                        '>' => result.push_str("&gt;"),
                        '&' => result.push_str("&amp;"),
                        '"' => result.push_str("&quot;"),
                        '\'' => result.push_str("&#39;"),
                        ch => result.push(ch),
                    }
                }
                result
            }
            "json" => {
                let mut result = String::with_capacity(input.len() * 2);
                for ch in input.chars() {
                    match ch {
                        '"' => result.push_str("\\\""),
                        '\\' => result.push_str("\\\\"),
                        '\n' => result.push_str("\\n"),
                        '\r' => result.push_str("\\r"),
                        '\t' => result.push_str("\\t"),
                        '\u{08}' => result.push_str("\\b"), // Backspace
                        '\u{0C}' => result.push_str("\\f"), // Form feed
                        ch if ch.is_control() => {
                            result.push_str(&format!("\\u{:04x}", ch as u32));
                        }
                        ch => result.push(ch),
                    }
                }
                result
            }
            _ => input.to_string(), // Unknown target, return as-is
        }
    }
}

#[async_trait]
impl FhirPathOperation for EscapeFunction {
    fn identifier(&self) -> &str {
        "escape"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(EscapeFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument (target format)
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        // Get target format
        let target = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.as_ref(),
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                                message: "escape() target parameter must be a string".to_string(),
                            });
                        }
                    }
                } else {
                    return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "escape() target parameter must be a single string".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: format!(
                        "escape() target parameter must be a string, got: {:?}",
                        args[0]
                    ),
                });
            }
        };

        let input = &context.input;

        // Get input string
        let input_string = match input {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                                message: "escape() requires a string input".to_string(),
                            });
                        }
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                } else {
                    return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "escape() requires a single string value".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "escape() requires a string input".to_string(),
                });
            }
        };

        // Escape the string
        let escaped = self.escape_string(&input_string, target);
        Ok(FhirPathValue::String(escaped.into()))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Validate exactly one argument (target format)
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        // Get target format
        let target = match &args[0] {
            FhirPathValue::String(s) => s.as_ref(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.as_ref(),
                        _ => {
                            return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                                message: "escape() target parameter must be a string".to_string(),
                            }));
                        }
                    }
                } else {
                    return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "escape() target parameter must be a single string".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "escape() target parameter must be a string".to_string(),
                }));
            }
        };

        let input = &context.input;

        // Get input string
        let input_string = match input {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => {
                            return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                                message: "escape() requires a string input".to_string(),
                            }));
                        }
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                        message: "escape() requires a single string value".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "escape() requires a string input".to_string(),
                }));
            }
        };

        // Escape the string
        let escaped = self.escape_string(&input_string, target);
        Some(Ok(FhirPathValue::String(escaped.into())))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
