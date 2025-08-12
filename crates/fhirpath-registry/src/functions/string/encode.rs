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

//! encode() function - URL encodes string

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
/// encode() function - URL encodes string
pub struct EncodeFunction;

#[async_trait]
impl AsyncFhirPathFunction for EncodeFunction {
    fn name(&self) -> &str {
        "encode"
    }
    fn human_friendly_name(&self) -> &str {
        "Encode"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "encode",
                vec![ParameterInfo::required("format", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // encode() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(format)) => {
                match format.as_ref() {
                    "uri" => {
                        // URL percent encoding
                        let encoded = s
                            .chars()
                            .map(|c| match c {
                                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                                    c.to_string()
                                }
                                ' ' => "%20".to_string(),
                                _ => format!("%{:02X}", c as u32),
                            })
                            .collect::<String>();
                        Ok(FhirPathValue::String(encoded.into()))
                    }
                    "html" => {
                        // HTML entity encoding
                        let encoded = s
                            .chars()
                            .map(|c| match c {
                                '<' => "&lt;".to_string(),
                                '>' => "&gt;".to_string(),
                                '&' => "&amp;".to_string(),
                                '"' => "&quot;".to_string(),
                                '\'' => "&#39;".to_string(),
                                _ => c.to_string(),
                            })
                            .collect::<String>();
                        Ok(FhirPathValue::String(encoded.into()))
                    }
                    "base64" => {
                        // Base64 encoding
                        let chars =
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                        let char_vec: Vec<char> = chars.chars().collect();
                        let bytes = s.as_bytes();
                        let mut result = String::new();

                        for chunk in bytes.chunks(3) {
                            let b1 = chunk[0];
                            let b2 = chunk.get(1).copied().unwrap_or(0);
                            let b3 = chunk.get(2).copied().unwrap_or(0);

                            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

                            result.push(char_vec[((n >> 18) & 63) as usize]);
                            result.push(char_vec[((n >> 12) & 63) as usize]);
                            result.push(if chunk.len() > 1 {
                                char_vec[((n >> 6) & 63) as usize]
                            } else {
                                '='
                            });
                            result.push(if chunk.len() > 2 {
                                char_vec[(n & 63) as usize]
                            } else {
                                '='
                            });
                        }

                        Ok(FhirPathValue::String(result.into()))
                    }
                    "hex" => {
                        // Hexadecimal encoding
                        let encoded = s
                            .as_bytes()
                            .iter()
                            .map(|b| format!("{b:02X}"))
                            .collect::<String>();
                        Ok(FhirPathValue::String(encoded.into()))
                    }
                    "urlbase64" => {
                        // URL-safe Base64 encoding (RFC 4648) with padding
                        let chars =
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                        let char_vec: Vec<char> = chars.chars().collect();
                        let bytes = s.as_bytes();
                        let mut result = String::new();

                        for chunk in bytes.chunks(3) {
                            let b1 = chunk[0];
                            let b2 = chunk.get(1).copied().unwrap_or(0);
                            let b3 = chunk.get(2).copied().unwrap_or(0);

                            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

                            result.push(char_vec[((n >> 18) & 63) as usize]);
                            result.push(char_vec[((n >> 12) & 63) as usize]);
                            result.push(if chunk.len() > 1 {
                                char_vec[((n >> 6) & 63) as usize]
                            } else {
                                '='
                            });
                            result.push(if chunk.len() > 2 {
                                char_vec[(n & 63) as usize]
                            } else {
                                '='
                            });
                        }

                        Ok(FhirPathValue::String(result.into()))
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported encoding format: {format}"),
                    }),
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
