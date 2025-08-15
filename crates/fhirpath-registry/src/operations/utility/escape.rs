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

//! Escape function implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity, FhirPathType
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Escape function - escapes special characters in a string
#[derive(Debug, Clone)]
pub struct EscapeFunction;

impl EscapeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("escape", OperationType::Function)
            .description("Escapes special characters in a string using standard escape sequences")
            .example("'Hello\\nWorld'.escape()")
            .example("'Tab\\tSeparated'.escape()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn escape_string(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());

        for ch in input.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\'' => result.push_str("\\'"),
                '\u{08}' => result.push_str("\\b"), // Backspace
                '\u{0C}' => result.push_str("\\f"), // Form feed
                '\u{00}' => result.push_str("\\0"), // Null
                ch if ch.is_control() => {
                    // Other control characters as unicode escape
                    result.push_str(&format!("\\u{{{:04x}}}", ch as u32));
                }
                ch => result.push(ch),
            }
        }

        result
    }
}

#[async_trait]
impl FhirPathOperation for EscapeFunction {
    fn identifier(&self) -> &str {
        "escape"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            EscapeFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            });
        }

        let input = &context.input;

        // Get input string
        let input_string = match input {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => return Err(FhirPathError::EvaluationError { message: 
                            "escape() requires a string input".to_string()
                        }),
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                } else {
                    return Err(FhirPathError::EvaluationError { message: 
                        "escape() requires a single string value".to_string()
                    });
                }
            }
            _ => return Err(FhirPathError::EvaluationError { message: 
                "escape() requires a string input".to_string()
            }),
        };

        // Escape the string
        let escaped = self.escape_string(&input_string);
        Ok(FhirPathValue::String(escaped.into()))
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            }));
        }

        let input = &context.input;

        // Get input string
        let input_string = match input {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    match items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => return Some(Err(FhirPathError::EvaluationError { message: 
                            "escape() requires a string input".to_string()
                        })),
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError { message: 
                        "escape() requires a single string value".to_string()
                    }));
                }
            }
            _ => return Some(Err(FhirPathError::EvaluationError { message: 
                "escape() requires a string input".to_string()
            })),
        };

        // Escape the string
        let escaped = self.escape_string(&input_string);
        Some(Ok(FhirPathValue::String(escaped.into())))
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
    async fn test_escape_newline() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hello\nWorld".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Hello\\nWorld");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_tab() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Tab\tSeparated".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Tab\\tSeparated");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_quotes() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("He said \"Hello\" to me".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "He said \\\"Hello\\\" to me");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_backslash() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Path\\to\\file".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Path\\\\to\\\\file");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_carriage_return() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Line1\rLine2".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Line1\\rLine2");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_control_characters() -> Result<()> {
        let function = EscapeFunction::new();

        // Test backspace and form feed
        let input = "Text\u{08}with\u{0C}control";
        let context = EvaluationContext::new(FhirPathValue::String(input.into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Text\\bwith\\fcontrol");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_null_character() -> Result<()> {
        let function = EscapeFunction::new();

        let input = "Text\u{00}with null";
        let context = EvaluationContext::new(FhirPathValue::String(input.into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Text\\0with null");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_unicode_control() -> Result<()> {
        let function = EscapeFunction::new();

        // Test other control characters that should be unicode escaped
        let input = "Text\u{0001}control\u{001F}chars";
        let context = EvaluationContext::new(FhirPathValue::String(input.into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Text\\u{0001}control\\u{001f}chars");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_mixed_characters() -> Result<()> {
        let function = EscapeFunction::new();

        let input = "Hello\nWorld\t\"Test\"\r\n\\Path";
        let context = EvaluationContext::new(FhirPathValue::String(input.into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Hello\\nWorld\\t\\\"Test\\\"\\r\\n\\\\Path");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_empty_string() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_normal_string() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Normal text with no special chars".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Normal text with no special chars");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_collection() -> Result<()> {
        let function = EscapeFunction::new();

        // Single item collection
        let context = EvaluationContext::new(FhirPathValue::Collection(vec![
            FhirPathValue::String("Hello\nWorld".into())
        ]));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Hello\\nWorld");
            }
            _ => panic!("Expected String value"),
        }

        // Empty collection
        let context = EvaluationContext::new(FhirPathValue::Collection(Collection::new()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Collection(items) => {
                assert!(items.is_empty());
            }
            _ => panic!("Expected empty collection"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_escape_invalid_input() -> () {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::Integer(42));
        let result = function.evaluate(&[], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::EvaluationError { message: msg }) = result {
            assert!(msg.contains("requires a string input"));
        } else {
            panic!("Expected InvalidOperation error");
        }
    }

    #[tokio::test]
    async fn test_escape_invalid_args() -> () {
        let function = EscapeFunction::new();
        let context = EvaluationContext::new(FhirPathValue::String("test".into()));

        let result = function.evaluate(&[FhirPathValue::String("extra".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = result {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }

    #[test]
    fn test_escape_sync() -> Result<()> {
        let function = EscapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hello\nWorld".into()));
        let result = function.try_evaluate_sync(&[], &context)
            .unwrap()?;

        match result {
            FhirPathValue::String(escaped) => {
                assert_eq!(escaped, "Hello\\nWorld");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }
}
