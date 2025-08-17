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

//! Unescape function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// Unescape function - unescapes special characters in a string
#[derive(Debug, Clone)]
pub struct UnescapeFunction;

impl Default for UnescapeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl UnescapeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("unescape", OperationType::Function)
            .description("Unescapes special characters in a string for a given target format")
            .parameter(
                "target",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .example("'&quot;1&lt;2&quot;'.unescape('html')")
            .example("'\\\"1<2\\\"'.unescape('json')")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn unescape_string(&self, input: &str, target: &str) -> Result<String> {
        match target {
            "html" => {
                let mut result = String::with_capacity(input.len());
                let mut chars = input.chars().peekable();

                while let Some(ch) = chars.next() {
                    if ch == '&' {
                        let mut entity = String::new();
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch == ';' {
                                chars.next(); // consume ';'
                                break;
                            } else {
                                entity.push(next_ch);
                                chars.next();
                            }
                        }

                        match entity.as_str() {
                            "lt" => result.push('<'),
                            "gt" => result.push('>'),
                            "amp" => result.push('&'),
                            "quot" => result.push('"'),
                            "#39" => result.push('\''),
                            _ => {
                                // Unknown entity, keep as-is
                                result.push('&');
                                result.push_str(&entity);
                                result.push(';');
                            }
                        }
                    } else {
                        result.push(ch);
                    }
                }
                Ok(result)
            }
            "json" => {
                let mut result = String::with_capacity(input.len());
                let mut chars = input.chars().peekable();

                while let Some(ch) = chars.next() {
                    if ch == '\\' {
                        if let Some(&next_ch) = chars.peek() {
                            match next_ch {
                                'n' => {
                                    chars.next();
                                    result.push('\n');
                                }
                                'r' => {
                                    chars.next();
                                    result.push('\r');
                                }
                                't' => {
                                    chars.next();
                                    result.push('\t');
                                }
                                '\\' => {
                                    chars.next();
                                    result.push('\\');
                                }
                                '"' => {
                                    chars.next();
                                    result.push('"');
                                }
                                'b' => {
                                    chars.next();
                                    result.push('\u{08}'); // Backspace
                                }
                                'f' => {
                                    chars.next();
                                    result.push('\u{0C}'); // Form feed
                                }
                                'u' => {
                                    chars.next(); // consume 'u'
                                    let mut hex_digits = String::new();
                                    for _ in 0..4 {
                                        if let Some(&hex_ch) = chars.peek() {
                                            if hex_ch.is_ascii_hexdigit() {
                                                hex_digits.push(hex_ch);
                                                chars.next();
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }

                                    if hex_digits.len() == 4 {
                                        if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16)
                                        {
                                            if let Some(unicode_char) = char::from_u32(code_point) {
                                                result.push(unicode_char);
                                            } else {
                                                return Err(FhirPathError::EvaluationError {
                                                    expression: None,
                                                    location: None,
                                                    message: "Invalid unicode code point"
                                                        .to_string(),
                                                });
                                            }
                                        } else {
                                            return Err(FhirPathError::EvaluationError {
                                                expression: None,
                                                location: None,
                                                message: "Invalid unicode escape sequence"
                                                    .to_string(),
                                            });
                                        }
                                    } else {
                                        // Invalid unicode sequence, treat as literal
                                        result.push('\\');
                                        result.push('u');
                                        result.push_str(&hex_digits);
                                    }
                                }
                                _ => {
                                    // Unknown escape sequence, treat as literal
                                    result.push('\\');
                                    result.push(next_ch);
                                    chars.next();
                                }
                            }
                        } else {
                            // Trailing backslash, treat as literal
                            result.push('\\');
                        }
                    } else {
                        result.push(ch);
                    }
                }
                Ok(result)
            }
            _ => Ok(input.to_string()), // Unknown target, return as-is
        }
    }
}

#[async_trait]
impl FhirPathOperation for UnescapeFunction {
    fn identifier(&self) -> &str {
        "unescape"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(UnescapeFunction::create_metadata);
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
                                message: "unescape() target parameter must be a string".to_string(),
                            });
                        }
                    }
                } else {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "unescape() target parameter must be a single string".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: format!(
                        "unescape() target parameter must be a string, got: {:?}",
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
                    match &items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                                expression: None,
                                location: None,
                                message: "unescape() requires a string input".to_string(),
                            });
                        }
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                } else {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "unescape() requires a single string value".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "unescape() requires a string input".to_string(),
                });
            }
        };

        // Unescape the string
        let unescaped = self.unescape_string(&input_string, target)?;
        Ok(FhirPathValue::String(unescaped.into()))
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
                                message: "unescape() target parameter must be a string".to_string(),
                            }));
                        }
                    }
                } else {
                    return Some(Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "unescape() target parameter must be a single string".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "unescape() target parameter must be a string".to_string(),
                }));
            }
        };

        let input = &context.input;

        // Get input string
        let input_string = match input {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match &items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => {
                            return Some(Err(FhirPathError::EvaluationError {
                                expression: None,
                                location: None,
                                message: "unescape() requires a string input".to_string(),
                            }));
                        }
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "unescape() requires a single string value".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "unescape() requires a string input".to_string(),
                }));
            }
        };

        // Unescape the string
        let unescaped = match self.unescape_string(&input_string, target) {
            Ok(result) => result,
            Err(e) => return Some(Err(e)),
        };
        Some(Ok(FhirPathValue::String(unescaped.into())))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
