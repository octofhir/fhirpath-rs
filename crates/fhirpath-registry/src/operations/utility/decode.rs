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

//! Decode function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose, engine::general_purpose::URL_SAFE};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use url::form_urlencoded;

/// Decode function - decodes a string using the specified encoding
#[derive(Debug, Clone)]
pub struct DecodeFunction;

impl Default for DecodeFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DecodeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("decode", OperationType::Function)
            .description("Decodes a string using the specified encoding")
            .example("'SGVsbG8gV29ybGQ='.decode('base64')")
            .example("'hello%20world'.decode('url')")
            .parameter(
                "encoding",
                TypeConstraint::Specific(FhirPathType::String),
                false,
            )
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn decode_string(&self, input: &str, encoding: &str) -> Result<String> {
        match encoding.to_lowercase().as_str() {
            "base64" => match general_purpose::STANDARD.decode(input.as_bytes()) {
                Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                    Ok(decoded_string) => Ok(decoded_string),
                    Err(_) => Err(FhirPathError::function_error(
                        "decode",
                        "Invalid UTF-8 sequence in base64 decoded data",
                    )),
                },
                Err(_) => Err(FhirPathError::EvaluationError {
                    message: "Invalid base64 input".to_string(),
                }),
            },
            "urlbase64" => match URL_SAFE.decode(input.as_bytes()) {
                Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
                    Ok(decoded_string) => Ok(decoded_string),
                    Err(_) => Err(FhirPathError::function_error(
                        "decode",
                        "Invalid UTF-8 sequence in base64 decoded data",
                    )),
                },
                Err(_) => Err(FhirPathError::EvaluationError {
                    message: "Invalid base64 input".to_string(),
                }),
            },
            "url" => {
                let decoded: String = form_urlencoded::parse(input.as_bytes())
                    .map(|(key, val)| {
                        if val.is_empty() {
                            key
                        } else {
                            format!("{key}={val}").into()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("&");

                // If no '=' found in input, it's simple URL decoding
                if !input.contains('=') {
                    match percent_encoding::percent_decode(input.as_bytes()).decode_utf8() {
                        Ok(decoded) => Ok(decoded.to_string()),
                        Err(_) => Err(FhirPathError::EvaluationError {
                            message: "Invalid URL encoded input".to_string(),
                        }),
                    }
                } else {
                    Ok(decoded)
                }
            }
            "hex" => {
                if input.len() % 2 != 0 {
                    return Err(FhirPathError::EvaluationError {
                        message: "Hex string must have even length".to_string(),
                    });
                }

                let mut decoded_bytes = Vec::new();
                for chunk in input.as_bytes().chunks(2) {
                    let hex_str =
                        std::str::from_utf8(chunk).map_err(|_| FhirPathError::EvaluationError {
                            message: "Invalid hex characters".to_string(),
                        })?;

                    let byte = u8::from_str_radix(hex_str, 16).map_err(|_| {
                        FhirPathError::EvaluationError {
                            message: "Invalid hex characters".to_string(),
                        }
                    })?;

                    decoded_bytes.push(byte);
                }

                match String::from_utf8(decoded_bytes) {
                    Ok(decoded_string) => Ok(decoded_string),
                    Err(_) => Err(FhirPathError::EvaluationError {
                        message: "Invalid UTF-8 sequence in hex decoded data".to_string(),
                    }),
                }
            }
            _ => Err(FhirPathError::EvaluationError {
                message: format!(
                    "Unsupported encoding type: '{encoding}'. Supported types are: 'base64', 'url', 'hex'"
                ),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for DecodeFunction {
    fn identifier(&self) -> &str {
        "decode"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(DecodeFunction::create_metadata);
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
                    match items.iter().next() {
                        Some(FhirPathValue::String(s)) => s.clone(),
                        _ => {
                            return Err(FhirPathError::EvaluationError {
                                message: "decode() requires a string input".to_string(),
                            });
                        }
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(vec![].into()));
                } else {
                    return Err(FhirPathError::EvaluationError {
                        message: "decode() requires a single string value".to_string(),
                    });
                }
            }
            _ => {
                return Err(FhirPathError::EvaluationError {
                    message: "decode() requires a string input".to_string(),
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
                    message: "decode() encoding parameter must be a string".to_string(),
                });
            }
        };

        // Decode the string
        let decoded = self.decode_string(&input_string, &encoding)?;
        Ok(FhirPathValue::String(decoded.into()))
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
                    match items.iter().next() {
                        Some(FhirPathValue::String(s)) => s.clone(),
                        _ => {
                            return Some(Err(FhirPathError::EvaluationError {
                                message: "decode() requires a string input".to_string(),
                            }));
                        }
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(vec![].into())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError {
                        message: "decode() requires a single string value".to_string(),
                    }));
                }
            }
            _ => {
                return Some(Err(FhirPathError::EvaluationError {
                    message: "decode() requires a string input".to_string(),
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
                    message: "decode() encoding parameter must be a string".to_string(),
                }));
            }
        };

        // Decode the string
        match self.decode_string(&input_string, &encoding) {
            Ok(decoded) => Some(Ok(FhirPathValue::String(decoded.into()))),
            Err(e) => Some(Err(e)),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
