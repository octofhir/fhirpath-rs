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

//! Encode function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose, engine::general_purpose::URL_SAFE};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use url::form_urlencoded;

/// Encode function - encodes a string using the specified encoding
#[derive(Debug, Clone)]
pub struct EncodeFunction;

impl Default for EncodeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl EncodeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("encode", OperationType::Function)
            .description("Encodes a string using the specified encoding")
            .example("'Hello World'.encode('base64')")
            .example("'hello world'.encode('url')")
            .parameter(
                "encoding",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn encode_string(&self, input: &str, encoding: &str) -> Result<String> {
        match encoding.to_lowercase().as_str() {
            "base64" => Ok(general_purpose::STANDARD.encode(input.as_bytes())),
            "url" => Ok(form_urlencoded::byte_serialize(input.as_bytes()).collect()),
            "urlbase64" => Ok(URL_SAFE.encode(input.as_bytes())),
            "hex" => Ok(input
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()),
            _ => Err(FhirPathError::function_error(
                "encode",
                format!(
                    "Unsupported encoding type: '{encoding}'. Supported types are: 'base64', 'url', 'hex'"
                ),
            )),
        }
    }
}

#[async_trait]
impl FhirPathOperation for EncodeFunction {
    fn identifier(&self) -> &str {
        "encode"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(EncodeFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let input = &context.input;
        let encoding_arg = &args[0];

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
                                message: "encode() requires a string input".to_string(),
                            });
                        }
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                } else {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "encode() requires a single string value".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "encode() requires a string input".to_string(),
                });
            }
        };

        // Get encoding type
        let encoding = match encoding_arg {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(cols) if cols.len() == 1 => {
                match cols.iter().next().unwrap() {
                    FhirPathValue::String(s) => s.clone(),
                    _ => {
                        return Err(FhirPathError::InvalidArguments {
                            message: "take() argument must be an integer".to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "encode() encoding parameter must be a string".to_string(),
                });
            }
        };

        // Encode the string
        let encoded = self.encode_string(&input_string, &encoding)?;
        Ok(FhirPathValue::String(encoded.into()))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len(),
            }));
        }

        let input = &context.input;
        let encoding_arg = &args[0];

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
                                message: "encode() requires a string input".to_string(),
                            }));
                        }
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "encode() requires a single string value".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "encode() requires a string input".to_string(),
                }));
            }
        };

        // Get encoding type
        let encoding = match encoding_arg {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(cols) if cols.len() == 1 => {
                match cols.iter().next().unwrap() {
                    FhirPathValue::String(s) => s.clone(),
                    _ => {
                        return Some(Err(FhirPathError::InvalidArguments {
                            message: "take() argument must be an integer".to_string(),
                        }));
                    }
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "encode() encoding parameter must be a string".to_string(),
                }));
            }
        };

        // Encode the string
        match self.encode_string(&input_string, &encoding) {
            Ok(encoded) => Some(Ok(FhirPathValue::String(encoded.into()))),
            Err(e) => Some(Err(e)),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
