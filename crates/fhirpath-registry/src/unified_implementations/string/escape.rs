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

//! Unified escape() function implementation for FHIRPath

use crate::{
    unified_function::{ExecutionMode, UnifiedFhirPathFunction},
    enhanced_metadata::{EnhancedFunctionMetadata, TypePattern},
    metadata_builder::MetadataBuilder,
    function::{EvaluationContext, FunctionError, FunctionResult},
};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;

/// Unified escape() function implementation
/// 
/// Performs URL/percent encoding on string values
pub struct UnifiedEscapeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedEscapeFunction {
    pub fn new() -> Self {
        use crate::signature::{FunctionSignature, ParameterInfo};
        use octofhir_fhirpath_model::types::TypeInfo;

        // Create proper signature with 1 optional string parameter for format
        let signature = FunctionSignature::new(
            "escape",
            vec![ParameterInfo::optional("format", TypeInfo::String)],
            TypeInfo::String,
        );

        let metadata = MetadataBuilder::string_function("escape")
            .display_name("Escape")
            .description("Escapes the string using specified format (url, html, json)")
            .example("'hello world'.escape()")
            .example("'<test>'.escape('html')")
            .example("'\"quoted\"'.escape('json')")
            .signature(signature)
            .output_type(TypePattern::Exact(TypeInfo::String))
            .lsp_snippet("escape(${1:'format'})")
            .keywords(vec!["escape", "encode", "url", "html", "json", "string"])
            .usage_pattern(
                "Escape string for specific format",
                "value.escape(format)",
                "Preparing strings for different formats"
            )
            .related_function("unescape")
            .build();

        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedEscapeFunction {
    fn name(&self) -> &str {
        "escape"
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
                let escaped = self.escape_string(s.as_ref(), format)?;
                FhirPathValue::collection(vec![FhirPathValue::String(escaped.into())])
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    if let FhirPathValue::String(s) = item {
                        let escaped = self.escape_string(s.as_ref(), format)?;
                        results.push(FhirPathValue::String(escaped.into()));
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

impl UnifiedEscapeFunction {
    /// Escape a string based on the specified format
    fn escape_string(&self, input: &str, format: &str) -> FunctionResult<String> {
        match format {
            "url" => Ok(self.url_encode(input)),
            "html" => Ok(self.html_encode(input)),
            "json" => Ok(self.json_encode(input)),
            _ => Err(FunctionError::EvaluationError {
                name: "escape".to_string(),
                message: format!("Unsupported escape format: {}", format),
            }),
        }
    }

    /// URL encode a string using percent encoding
    fn url_encode(&self, input: &str) -> String {
        let mut result = String::new();
        
        for byte in input.bytes() {
            match byte {
                // Unreserved characters (RFC 3986)
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                    result.push(byte as char);
                }
                // Everything else gets percent-encoded
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        
        result
    }

    /// HTML encode a string
    fn html_encode(&self, input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    /// JSON encode a string
    fn json_encode(&self, input: &str) -> String {
        let mut result = String::new();
        for ch in input.chars() {
            match ch {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\u{08}' => result.push_str("\\b"),
                '\u{0C}' => result.push_str("\\f"),
                c if c.is_control() => {
                    result.push_str(&format!("\\u{:04X}", c as u32));
                }
                c => result.push(c),
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::EvaluationContext;

    #[tokio::test]
    async fn test_unified_escape_function() {
        let escape_func = UnifiedEscapeFunction::new();
        
        // Test basic escaping
        let context = EvaluationContext::new(FhirPathValue::String("hello world".into()));
        let result = escape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("hello%20world".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test special characters
        let context = EvaluationContext::new(FhirPathValue::String("a+b=c".into()));
        let result = escape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("a%2Bb%3Dc".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test empty string
        let context = EvaluationContext::new(FhirPathValue::String("".into()));
        let result = escape_func.evaluate_sync(&[], &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 1);
            assert_eq!(items.get(0), Some(&FhirPathValue::String("".into())));
        } else {
            panic!("Expected collection result");
        }
        
        // Test metadata
        assert_eq!(escape_func.name(), "escape");
        assert_eq!(escape_func.execution_mode(), ExecutionMode::Sync);
    }
}