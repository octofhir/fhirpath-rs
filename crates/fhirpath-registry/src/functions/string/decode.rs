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

//! decode() function - decodes URL encoded string

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
/// decode() function - decodes URL encoded string
pub struct DecodeFunction;

#[async_trait]
impl AsyncFhirPathFunction for DecodeFunction {
    fn name(&self) -> &str {
        "decode"
    }
    fn human_friendly_name(&self) -> &str {
        "Decode"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "decode",
                vec![ParameterInfo::required("format", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // decode() is a pure string function
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
                        // URL percent decoding
                        let mut decoded = String::new();
                        let mut chars = s.chars().peekable();
                        while let Some(c) = chars.next() {
                            if c == '%' {
                                // Try to decode percent-encoded character
                                let hex1 = chars.next();
                                let hex2 = chars.next();
                                if let (Some(h1), Some(h2)) = (hex1, hex2) {
                                    if let Ok(byte) = u8::from_str_radix(&format!("{h1}{h2}"), 16) {
                                        if let Ok(decoded_char) = std::str::from_utf8(&[byte]) {
                                            decoded.push_str(decoded_char);
                                        } else {
                                            // Invalid UTF-8, keep original
                                            decoded.push('%');
                                            decoded.push(h1);
                                            decoded.push(h2);
                                        }
                                    } else {
                                        // Invalid hex, keep original
                                        decoded.push('%');
                                        decoded.push(h1);
                                        decoded.push(h2);
                                    }
                                } else {
                                    // Incomplete percent encoding, keep original
                                    decoded.push(c);
                                }
                            } else {
                                decoded.push(c);
                            }
                        }
                        Ok(FhirPathValue::String(decoded.into()))
                    }
                    "html" => {
                        // HTML entity decoding
                        let mut decoded = String::new();
                        let mut chars = s.chars().peekable();
                        while let Some(c) = chars.next() {
                            if c == '&' {
                                // Try to decode HTML entity
                                let mut entity = String::new();
                                let mut found_semicolon = false;
                                while let Some(&next_char) = chars.peek() {
                                    if next_char == ';' {
                                        chars.next(); // consume semicolon
                                        found_semicolon = true;
                                        break;
                                    } else if entity.len() < 10 {
                                        // reasonable limit
                                        entity.push(chars.next().unwrap());
                                    } else {
                                        break;
                                    }
                                }

                                if found_semicolon {
                                    match entity.as_str() {
                                        "lt" => decoded.push('<'),
                                        "gt" => decoded.push('>'),
                                        "amp" => decoded.push('&'),
                                        "quot" => decoded.push('"'),
                                        "#39" => decoded.push('\''),
                                        _ => {
                                            // Unknown entity, keep original
                                            decoded.push('&');
                                            decoded.push_str(&entity);
                                            decoded.push(';');
                                        }
                                    }
                                } else {
                                    // No semicolon found, keep original
                                    decoded.push('&');
                                    decoded.push_str(&entity);
                                }
                            } else {
                                decoded.push(c);
                            }
                        }
                        Ok(FhirPathValue::String(decoded.into()))
                    }
                    "base64" => {
                        // Base64 decoding
                        let chars =
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                        let mut char_map = std::collections::HashMap::new();
                        for (i, c) in chars.chars().enumerate() {
                            char_map.insert(c, i as u8);
                        }

                        let clean_input: String = s
                            .chars()
                            .filter(|c| chars.contains(*c) || *c == '=')
                            .collect();
                        let mut result = Vec::new();

                        for chunk in clean_input.chars().collect::<Vec<_>>().chunks(4) {
                            if chunk.len() < 4 {
                                break;
                            }

                            let b1 = char_map.get(&chunk[0]).copied().unwrap_or(0);
                            let b2 = char_map.get(&chunk[1]).copied().unwrap_or(0);
                            let b3 = if chunk[2] == '=' {
                                0
                            } else {
                                char_map.get(&chunk[2]).copied().unwrap_or(0)
                            };
                            let b4 = if chunk[3] == '=' {
                                0
                            } else {
                                char_map.get(&chunk[3]).copied().unwrap_or(0)
                            };

                            result.push((b1 << 2) | (b2 >> 4));
                            if chunk[2] != '=' {
                                result.push(((b2 & 0x0f) << 4) | (b3 >> 2));
                            }
                            if chunk[3] != '=' {
                                result.push(((b3 & 0x03) << 6) | b4);
                            }
                        }

                        match String::from_utf8(result) {
                            Ok(decoded) => Ok(FhirPathValue::String(decoded.into())),
                            Err(_) => Err(FunctionError::EvaluationError {
                                name: self.name().to_string(),
                                message: "Invalid base64 encoding".to_string(),
                            }),
                        }
                    }
                    "hex" => {
                        // Hexadecimal decoding
                        let clean_input: String =
                            s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
                        if clean_input.len() % 2 != 0 {
                            return Err(FunctionError::EvaluationError {
                                name: self.name().to_string(),
                                message: "Invalid hex string length".to_string(),
                            });
                        }

                        let mut result = Vec::new();
                        for chunk in clean_input.chars().collect::<Vec<_>>().chunks(2) {
                            if let Ok(byte) =
                                u8::from_str_radix(&format!("{}{}", chunk[0], chunk[1]), 16)
                            {
                                result.push(byte);
                            } else {
                                return Err(FunctionError::EvaluationError {
                                    name: self.name().to_string(),
                                    message: "Invalid hex characters".to_string(),
                                });
                            }
                        }

                        match String::from_utf8(result) {
                            Ok(decoded) => Ok(FhirPathValue::String(decoded.into())),
                            Err(_) => Err(FunctionError::EvaluationError {
                                name: self.name().to_string(),
                                message: "Invalid UTF-8 in hex decoded data".to_string(),
                            }),
                        }
                    }
                    "urlbase64" => {
                        // URL-safe Base64 decoding (RFC 4648)
                        let chars =
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                        let mut char_map = std::collections::HashMap::new();
                        for (i, c) in chars.chars().enumerate() {
                            char_map.insert(c, i as u8);
                        }

                        // Add padding if needed for URL-safe base64
                        let mut padded_input = s.to_string();
                        while padded_input.len() % 4 != 0 {
                            padded_input.push('=');
                        }

                        let mut result = Vec::new();
                        for chunk in padded_input.chars().collect::<Vec<_>>().chunks(4) {
                            if chunk.len() < 4 {
                                break;
                            }

                            let b1 = char_map.get(&chunk[0]).copied().unwrap_or(0);
                            let b2 = char_map.get(&chunk[1]).copied().unwrap_or(0);
                            let b3 = if chunk[2] == '=' {
                                0
                            } else {
                                char_map.get(&chunk[2]).copied().unwrap_or(0)
                            };
                            let b4 = if chunk[3] == '=' {
                                0
                            } else {
                                char_map.get(&chunk[3]).copied().unwrap_or(0)
                            };

                            result.push((b1 << 2) | (b2 >> 4));
                            if chunk[2] != '=' {
                                result.push(((b2 & 0x0f) << 4) | (b3 >> 2));
                            }
                            if chunk[3] != '=' {
                                result.push(((b3 & 0x03) << 6) | b4);
                            }
                        }

                        match String::from_utf8(result) {
                            Ok(decoded) => Ok(FhirPathValue::String(decoded.into())),
                            Err(_) => Err(FunctionError::EvaluationError {
                                name: self.name().to_string(),
                                message: "Invalid urlbase64 encoding".to_string(),
                            }),
                        }
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported decoding format: {format}"),
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
