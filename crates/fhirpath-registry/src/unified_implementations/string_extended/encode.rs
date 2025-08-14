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

//! Unified encode() function implementation

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

/// Unified encode() function implementation
/// 
/// Encodes strings using various encoding formats including URI, HTML, Base64, Hex, and URL-safe Base64.
/// Syntax: encode(format)
pub struct UnifiedEncodeFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedEncodeFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "encode",
            vec![ParameterInfo::required("format", TypeInfo::String)],
            TypeInfo::String,
        );
        
        let metadata = MetadataBuilder::new("encode", FunctionCategory::StringOperations)
            .display_name("Encode")
            .description("Encodes strings using various formats (uri, html, base64, hex, urlbase64)")
            .example("'hello world'.encode('uri')")
            .example("'<tag>content</tag>'.encode('html')")
            .example("'Hello'.encode('base64')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::StringLike)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("encode(${1:'format'})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["encode", "uri", "html", "base64", "hex", "urlbase64"])
            .usage_pattern(
                "String encoding",
                "string.encode(format)",
                "Encoding strings for various formats"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedEncodeFunction {
    fn name(&self) -> &str {
        "encode"
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
                            let encoded = self.encode_string(s, &format)?;
                            results.push(FhirPathValue::String(encoded.into()));
                        }
                        _ => {
                            // Non-string items are converted to string first
                            let s = self.value_to_string(item);
                            let encoded = self.encode_string(&s, &format)?;
                            results.push(FhirPathValue::String(encoded.into()));
                        }
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::String(s) => {
                let encoded = self.encode_string(s, &format)?;
                Ok(FhirPathValue::String(encoded.into()))
            }
            single_value => {
                // Convert to string and encode
                let s = self.value_to_string(single_value);
                let encoded = self.encode_string(&s, &format)?;
                Ok(FhirPathValue::String(encoded.into()))
            }
        }
    }
}

impl UnifiedEncodeFunction {
    /// Encode a string using the specified format
    fn encode_string(&self, s: &str, format: &str) -> FunctionResult<String> {
        match format {
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
                Ok(encoded)
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
                Ok(encoded)
            }
            "base64" => {
                // Base64 encoding
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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

                Ok(result)
            }
            "hex" => {
                // Hexadecimal encoding
                let encoded = s
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<String>();
                Ok(encoded)
            }
            "urlbase64" => {
                // URL-safe Base64 encoding (RFC 4648)
                let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
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

                Ok(result)
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: format!("Unsupported encoding format: {format}"),
            }),
        }
    }
    
    /// Convert FhirPathValue to string for encoding operations
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
    async fn test_encode_uri() {
        let func = UnifiedEncodeFunction::new();
        let context = create_test_context(FhirPathValue::String("hello world".into()));
        
        let args = vec![FhirPathValue::String("uri".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("hello%20world".into()));
    }
    
    #[tokio::test]
    async fn test_encode_html() {
        let func = UnifiedEncodeFunction::new();
        let context = create_test_context(FhirPathValue::String("<tag>content</tag>".into()));
        
        let args = vec![FhirPathValue::String("html".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("&lt;tag&gt;content&lt;/tag&gt;".into()));
    }
    
    #[tokio::test]
    async fn test_encode_base64() {
        let func = UnifiedEncodeFunction::new();
        let context = create_test_context(FhirPathValue::String("Hello".into()));
        
        let args = vec![FhirPathValue::String("base64".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("SGVsbG8=".into()));
    }
    
    #[tokio::test]
    async fn test_encode_hex() {
        let func = UnifiedEncodeFunction::new();
        let context = create_test_context(FhirPathValue::String("Hello".into()));
        
        let args = vec![FhirPathValue::String("hex".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("48656C6C6F".into()));
    }
    
    #[tokio::test]
    async fn test_encode_urlbase64() {
        let func = UnifiedEncodeFunction::new();
        let context = create_test_context(FhirPathValue::String("Hello".into()));
        
        let args = vec![FhirPathValue::String("urlbase64".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("SGVsbG8=".into()));
    }
    
    #[tokio::test]
    async fn test_encode_collection() {
        let func = UnifiedEncodeFunction::new();
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
        ]);
        let context = create_test_context(collection);
        
        let args = vec![FhirPathValue::String("base64".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        if let FhirPathValue::Collection(items) = result {
            assert_eq!(items.len(), 2);
            assert_eq!(items.iter().nth(0), Some(&FhirPathValue::String("aGVsbG8=".into())));
            assert_eq!(items.iter().nth(1), Some(&FhirPathValue::String("d29ybGQ=".into())));
        } else {
            panic!("Expected collection result");
        }
    }
    
    #[tokio::test]
    async fn test_encode_unsupported_format() {
        let func = UnifiedEncodeFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        
        let args = vec![FhirPathValue::String("unknown".into())];
        let result = func.evaluate_sync(&args, &context);
        
        assert!(result.is_err());
        if let Err(FunctionError::EvaluationError { message, .. }) = result {
            assert!(message.contains("Unsupported encoding format"));
        } else {
            panic!("Expected EvaluationError");
        }
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedEncodeFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "encode");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}