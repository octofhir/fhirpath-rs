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

//! Unified decode() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use std::collections::HashMap;

/// Unified decode() function implementation
/// 
/// Decodes strings using various encoding formats including URI, HTML, Base64, Hex, and URL-safe Base64.
/// Syntax: decode(format)
pub struct UnifiedDecodeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedDecodeFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "decode",
            vec![ParameterInfo::required("format", TypeInfo::String)],
            TypeInfo::String,
        );
        
        let metadata = MetadataBuilder::new("decode", FunctionCategory::StringOperations)
            .display_name("Decode")
            .description("Decodes strings using various formats (uri, html, base64, hex, urlbase64)")
            .example("'hello%20world'.decode('uri')")
            .example("'&lt;tag&gt;'.decode('html')")
            .example("'SGVsbG8='.decode('base64')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::StringLike)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("decode(${1:'format'})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["decode", "uri", "html", "base64", "hex", "urlbase64"])
            .usage_pattern(
                "String decoding",
                "string.decode(format)",
                "Decoding strings from various formats"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedDecodeFunction {
    fn name(&self) -> &str {
        "decode"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required (format)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let format = match &args[0] {
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.get(0) {
                    Some(FhirPathValue::String(s)) => s.to_string(),
                    _ => return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Format argument must be a string".to_string(),
                    }),
                }
            }
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Format argument must be a string".to_string(),
            }),
        };
        
        // Handle collections and single values
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::String(s) => {
                            let decoded = self.decode_string(s, &format)?;
                            results.push(FhirPathValue::String(decoded.into()));
                        }
                        _ => {
                            // Non-string items are converted to string first
                            let s = self.value_to_string(item);
                            let decoded = self.decode_string(&s, &format)?;
                            results.push(FhirPathValue::String(decoded.into()));
                        }
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::String(s) => {
                let decoded = self.decode_string(s, &format)?;
                Ok(FhirPathValue::String(decoded.into()))
            }
            single_value => {
                // Convert to string and decode
                let s = self.value_to_string(single_value);
                let decoded = self.decode_string(&s, &format)?;
                Ok(FhirPathValue::String(decoded.into()))
            }
        }
    }
}

impl UnifiedDecodeFunction {
    /// Decode a string using the specified format
    fn decode_string(&self, s: &str, format: &str) -> FunctionResult<String> {
        match format {
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
                Ok(decoded)
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
                            } else if entity.len() < 10 { // reasonable limit
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
                Ok(decoded)
            }
            "base64" => {
                // Base64 decoding
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                let mut char_map = HashMap::new();
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
                    Ok(decoded) => Ok(decoded),
                    Err(_) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Invalid base64 encoding".to_string(),
                    }),
                }
            }
            "hex" => {
                // Hexadecimal decoding
                let clean_input: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
                if clean_input.len() % 2 != 0 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Invalid hex string length".to_string(),
                    });
                }

                let mut result = Vec::new();
                for chunk in clean_input.chars().collect::<Vec<_>>().chunks(2) {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", chunk[0], chunk[1]), 16) {
                        result.push(byte);
                    } else {
                        return Err(FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: "Invalid hex characters".to_string(),
                        });
                    }
                }

                match String::from_utf8(result) {
                    Ok(decoded) => Ok(decoded),
                    Err(_) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Invalid UTF-8 in hex decoded data".to_string(),
                    }),
                }
            }
            "urlbase64" => {
                // URL-safe Base64 decoding (RFC 4648)
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
                let mut char_map = HashMap::new();
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
                    Ok(decoded) => Ok(decoded),
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
    
    /// Convert FhirPathValue to string for decoding operations
    fn value_to_string(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s) => s.to_string(),
            FhirPathValue::Integer(i) => i.to_string(),
            FhirPathValue::Decimal(d) => d.to_string(),
            FhirPathValue::Boolean(b) => b.to_string(),
            FhirPathValue::Date(d) => d.to_string(),
            FhirPathValue::DateTime(dt) => dt.to_string(),
            FhirPathValue::Time(t) => t.to_string(),
            FhirPathValue::Empty => String::new(),
            _ => format!("{:?}", value), // Fallback for complex types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_decode_uri() {
        let func = UnifiedDecodeFunction::new();
        let context = create_test_context(FhirPathValue::String("hello%20world".into()));
        
        let args = vec![FhirPathValue::String("uri".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("hello world".into()));
    }
    
    #[tokio::test]
    async fn test_decode_html() {
        let func = UnifiedDecodeFunction::new();
        let context = create_test_context(FhirPathValue::String("&lt;tag&gt;content&lt;/tag&gt;".into()));
        
        let args = vec![FhirPathValue::String("html".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("<tag>content</tag>".into()));
    }
    
    #[tokio::test]
    async fn test_decode_base64() {
        let func = UnifiedDecodeFunction::new();
        let context = create_test_context(FhirPathValue::String("SGVsbG8=".into()));
        
        let args = vec![FhirPathValue::String("base64".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("Hello".into()));
    }
    
    #[tokio::test]
    async fn test_decode_hex() {
        let func = UnifiedDecodeFunction::new();
        let context = create_test_context(FhirPathValue::String("48656C6C6F".into()));
        
        let args = vec![FhirPathValue::String("hex".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("Hello".into()));
    }
    
    #[tokio::test]
    async fn test_decode_urlbase64() {
        let func = UnifiedDecodeFunction::new();
        let context = create_test_context(FhirPathValue::String("SGVsbG8".into())); // without padding
        
        let args = vec![FhirPathValue::String("urlbase64".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("Hello".into()));
    }
    
    #[tokio::test]
    async fn test_decode_collection() {
        let func = UnifiedDecodeFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("aGVsbG8=".into()),
            FhirPathValue::String("d29ybGQ=".into()),
        ]);
        let context = create_test_context(collection);
        
        let args = vec![FhirPathValue::String("base64".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("hello".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("world".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_decode_unsupported_format() {
        let func = UnifiedDecodeFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        
        let args = vec![FhirPathValue::String("unknown".into())];
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
        if let Err(FunctionError::EvaluationError { message, .. }) = result {
            assert!(message.contains("Unsupported decoding format"));
        } else {
            panic!("Expected EvaluationError");
        }
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedDecodeFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "decode");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}