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

//! Unified unescape() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified unescape() function implementation
/// 
/// Performs URL/percent decoding on string values
pub struct UnifiedUnescapeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedUnescapeFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 optional string parameter for format
        let signature = FunctionSignature::new(
            "unescape",
            vec![ParameterInfo::optional("format", TypeInfo::String)],
            TypeInfo::String,
        );

        let metadata = MetadataBuilder::string_function("unescape")
            .display_name("Unescape")
            .description("Unescapes the string using specified format (url, html, json)")
            .example("'hello%20world'.unescape()")
            .example("'&lt;test&gt;'.unescape('html')")
            .example("'\\\"quoted\\\"'.unescape('json')")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::String))
            .lsp_snippet("unescape(${1:'format'})")
            .keywords(vec!["unescape", "decode", "url", "html", "json", "string"])
            .usage_pattern(
                "Unescape string from specific format",
                "encoded_value.unescape(format)",
                "Converting encoded strings back to normal text"
            )
            .related_function("escape")
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedUnescapeFunction {
    fn name(&self) -> &str {
        "unescape"
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
        // Validate arguments - should accept format parameter
        if args.len() > 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(1),
                actual: args.len(),
            });
        }

        // Get format (default to 'url' if not specified)
        let format = if args.is_empty() {
            "url"
        } else {
            match &args[0] {
                FhirPathValue::String(s) => s.as_ref(),
                _ => return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: format!("Expected String format, got {}", args[0].type_name()),
                }),
            }
        };

        let result = match &context.input {
            FhirPathValue::String(s) => {
                match self.unescape_string(s.as_ref(), format) {
                    Ok(unescaped) => FhirPathValue::collection(vec![FhirPathValue::String(unescaped.into())]),
                    Err(e) => return Err(e),
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    if let FhirPathValue::String(s) = item {
                        match self.unescape_string(s.as_ref(), format) {
                            Ok(unescaped) => results.push(FhirPathValue::String(unescaped.into())),
                            Err(e) => return Err(e),
                        }
                    }
                }
                FhirPathValue::collection(results)
            }
            FhirPathValue::Empty => FhirPathValue::Empty,
            _ => {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: format!("Expected String, got {}", context.input.type_name()),
                });
            }
        };

        Ok(result)
    }
}

impl UnifiedUnescapeFunction {
    /// Unescape a string based on the specified format  
    fn unescape_string(&self, input: &str, format: &str) -> FunctionResult<String> {
        match format {
            "url" => match self.url_decode(input) {
                Ok(result) => Ok(result),
                Err(e) => Err(FunctionError::EvaluationError {
                    name: "unescape".to_string(),
                    message: e,
                }),
            },
            "html" => Ok(self.html_decode(input)),
            "json" => match self.json_decode(input) {
                Ok(result) => Ok(result),
                Err(e) => Err(FunctionError::EvaluationError {
                    name: "unescape".to_string(), 
                    message: e,
                }),
            },
            _ => Err(FunctionError::EvaluationError {
                name: "unescape".to_string(),
                message: format!("Unsupported unescape format: {}", format),
            }),
        }
    }

    /// URL decode a string using percent decoding
    fn url_decode(&self, input: &str) -> Result<String, String> {
        let mut result = Vec::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '%' {
                // Expect two hexadecimal digits after %
                let hex1 = chars.next().ok_or("Incomplete percent encoding: missing first hex digit".to_string())?;
                let hex2 = chars.next().ok_or("Incomplete percent encoding: missing second hex digit".to_string())?;
                
                // Convert hex digits to byte value
                let hex_str = format!("{}{}", hex1, hex2);
                let byte_value = u8::from_str_radix(&hex_str, 16)
                    .map_err(|_| format!("Invalid hex digits in percent encoding: {}", hex_str))?;
                
                result.push(byte_value);
            } else {
                // Regular character - convert to UTF-8 bytes
                let mut buf = [0; 4];
                let utf8_bytes = ch.encode_utf8(&mut buf).as_bytes();
                result.extend_from_slice(utf8_bytes);
            }
        }
        
        // Convert bytes back to string
        String::from_utf8(result)
            .map_err(|_| "Invalid UTF-8 sequence in URL-decoded string".to_string())
    }

    /// HTML decode a string
    fn html_decode(&self, input: &str) -> String {
        input
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&") // This should be last to avoid double-decoding
    }

    /// JSON decode a string
    fn json_decode(&self, input: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                match chars.next() {
                    Some('"') => result.push('"'),
                    Some('\\') => result.push('\\'),
                    Some('/') => result.push('/'),
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('b') => result.push('\u{08}'),
                    Some('f') => result.push('\u{0C}'),
                    Some('u') => {
                        // Unicode escape sequence \uXXXX
                        let mut hex_digits = String::new();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(digit) if digit.is_ascii_hexdigit() => hex_digits.push(digit),
                                _ => return Err("Invalid unicode escape sequence".to_string()),
                            }
                        }
                        match u32::from_str_radix(&hex_digits, 16) {
                            Ok(code_point) => {
                                match char::from_u32(code_point) {
                                    Some(unicode_char) => result.push(unicode_char),
                                    None => return Err("Invalid unicode code point".to_string()),
                                }
                            }
                            Err(_) => return Err("Invalid hex digits in unicode escape".to_string()),
                        }
                    }
                    Some(c) => return Err(format!("Invalid escape sequence: \\{}", c)),
                    None => return Err("Incomplete escape sequence".to_string()),
                }
            } else {
                result.push(ch);
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_unescape_function() {
        let unescape_func = UnifiedUnescapeFunction::new();
        
        // Test basic unescaping
        let context = EvaluationContext::new(FhirPathValue::String("hello%20world".into()));
        let result = unescape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("hello world".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test special characters
        let context = EvaluationContext::new(FhirPathValue::String("a%2Bb%3Dc".into()));
        let result = unescape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("a+b=c".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test no percent encoding
        let context = EvaluationContext::new(FhirPathValue::String("hello".into()));
        let result = unescape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("hello".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test empty string
        let context = EvaluationContext::new(FhirPathValue::String("".into()));
        let result = unescape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(unescape_func.name(), "unescape");
        assert_eq!(unescape_func.execution_mode(), ExecutionMode::Sync);
    }
    
    #[tokio::test] 
    async fn test_unescape_error_handling() {
        let unescape_func = UnifiedUnescapeFunction::new();
        
        // Test incomplete percent encoding
        let context = EvaluationContext::new(FhirPathValue::String("hello%2".into()));
        let result = unescape_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
        
        // Test invalid hex digits
        let context = EvaluationContext::new(FhirPathValue::String("hello%ZZ".into()));
        let result = unescape_func.evaluate_sync(&[], &context);
        assert!(result.is_err());
    }
}