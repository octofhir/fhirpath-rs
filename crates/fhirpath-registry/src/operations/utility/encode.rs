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

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity,
    FhirPathType
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use url::form_urlencoded;

/// Encode function - encodes a string using the specified encoding
#[derive(Debug, Clone)]
pub struct EncodeFunction;

impl EncodeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("encode", OperationType::Function)
            .description("Encodes a string using the specified encoding")
            .example("'Hello World'.encode('base64')")
            .example("'hello world'.encode('url')")
            .parameter("encoding", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn encode_string(&self, input: &str, encoding: &str) -> Result<String> {
        match encoding.to_lowercase().as_str() {
            "base64" => {
                Ok(general_purpose::STANDARD.encode(input.as_bytes()))
            }
            "url" => {
                Ok(form_urlencoded::byte_serialize(input.as_bytes()).collect())
            }
            "hex" => {
                Ok(input.as_bytes().iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>())
            }
            _ => Err(FhirPathError::function_error("encode",
                format!("Unsupported encoding type: '{}'. Supported types are: 'base64', 'url', 'hex'", encoding)
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
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            EncodeFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len()
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
                        _ => return Err(FhirPathError::EvaluationError { message: 
                            "encode() requires a string input".to_string()
                        }),
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                } else {
                    return Err(FhirPathError::EvaluationError { message: 
                        "encode() requires a single string value".to_string()
                    });
                }
            }
            _ => return Err(FhirPathError::EvaluationError { message: 
                "encode() requires a string input".to_string()
            }),
        };

        // Get encoding type
        let encoding = match encoding_arg {
            FhirPathValue::String(s) => s.clone(),
            _ => return Err(FhirPathError::EvaluationError { message: 
                "encode() encoding parameter must be a string".to_string()
            }),
        };

        // Encode the string
        let encoded = self.encode_string(&input_string, &encoding)?;
        Ok(FhirPathValue::String(encoded.into()))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Validate exactly one argument
        if args.len() != 1 {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 1,
                actual: args.len()
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
                        _ => return Some(Err(FhirPathError::EvaluationError { message: 
                            "encode() requires a string input".to_string()
                        })),
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError { message: 
                        "encode() requires a single string value".to_string()
                    }));
                }
            }
            _ => return Some(Err(FhirPathError::EvaluationError { message: 
                "encode() requires a string input".to_string()
            })),
        };

        // Get encoding type
        let encoding = match encoding_arg {
            FhirPathValue::String(s) => s.clone(),
            _ => return Some(Err(FhirPathError::EvaluationError { message: 
                "encode() encoding parameter must be a string".to_string()
            })),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_encode_base64() -> Result<()> {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hello World".into()));
        let result = function.evaluate(&[FhirPathValue::String("base64".into())], &context).await?;

        match result {
            FhirPathValue::String(encoded) => {
                // "Hello World" in base64 is "SGVsbG8gV29ybGQ="
                assert_eq!(encoded, "SGVsbG8gV29ybGQ=");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encode_url() -> Result<()> {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("hello world".into()));
        let result = function.evaluate(&[FhirPathValue::String("url".into())], &context).await?;

        match result {
            FhirPathValue::String(encoded) => {
                assert_eq!(encoded, "hello%20world");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encode_hex() -> Result<()> {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hi".into()));
        let result = function.evaluate(&[FhirPathValue::String("hex".into())], &context).await?;

        match result {
            FhirPathValue::String(encoded) => {
                // "Hi" in hex is "4869" (H=0x48, i=0x69)
                assert_eq!(encoded, "4869");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encode_case_insensitive() -> Result<()> {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("test".into()));
        let result = function.evaluate(&[FhirPathValue::String("BASE64".into())], &context).await?;

        match result {
            FhirPathValue::String(encoded) => {
                // "test" in base64 is "dGVzdA=="
                assert_eq!(encoded, "dGVzdA==");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encode_empty_string() -> Result<()> {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("".into()));
        let result = function.evaluate(&[FhirPathValue::String("base64".into())], &context).await?;

        match result {
            FhirPathValue::String(encoded) => {
                assert_eq!(encoded, ""); // Empty string encodes to empty string
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encode_collection() -> Result<()> {
        let function = EncodeFunction::new();

        // Single item collection
        let context = EvaluationContext::new(FhirPathValue::Collection(vec![
            FhirPathValue::String("test".into())
        ]));
        let result = function.evaluate(&[FhirPathValue::String("base64".into())], &context).await?;

        match result {
            FhirPathValue::String(encoded) => {
                assert_eq!(encoded, "dGVzdA==");
            }
            _ => panic!("Expected String value"),
        }

        // Empty collection
        let context = EvaluationContext::new(FhirPathValue::Collection(Collection::new()));
        let result = function.evaluate(&[FhirPathValue::String("base64".into())], &context).await?;

        match result {
            FhirPathValue::Collection(items) => {
                assert!(items.is_empty());
            }
            _ => panic!("Expected empty collection"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_encode_invalid_encoding() -> () {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("test".into()));
        let result = function.evaluate(&[FhirPathValue::String("invalid".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::EvaluationError { message: msg }) = result {
            assert!(msg.contains("Unsupported encoding type"));
        } else {
            panic!("Expected InvalidOperation error");
        }
    }

    #[tokio::test]
    async fn test_encode_invalid_input() -> () {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = function.evaluate(&[FhirPathValue::String("base64".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::EvaluationError { message: msg }) = result {
            assert!(msg.contains("requires a string input"));
        } else {
            panic!("Expected InvalidOperation error");
        }
    }

    #[tokio::test]
    async fn test_encode_invalid_args() -> () {
        let function = EncodeFunction::new();
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));

        // No arguments
        let result = function.evaluate(&[], &context).await;
        assert!(result.is_err());

        // Too many arguments
        let result = function.evaluate(&[
            FhirPathValue::String("base64".into()),
            FhirPathValue::String("extra".into())
        ], &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_sync() -> Result<()> {
        let function = EncodeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("test".into()));
        let result = function.try_evaluate_sync(&[FhirPathValue::String("base64".into())], &context)
            .unwrap()?;

        match result {
            FhirPathValue::String(encoded) => {
                assert_eq!(encoded, "dGVzdA==");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }
}
