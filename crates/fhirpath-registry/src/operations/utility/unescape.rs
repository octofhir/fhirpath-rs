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

//! Unescape function implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity, FhirPathType
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Unescape function - unescapes special characters in a string
#[derive(Debug, Clone)]
pub struct UnescapeFunction;

impl UnescapeFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("unescape", OperationType::Function)
            .description("Unescapes special characters in a string using standard escape sequences")
            .example("'Hello\\\\nWorld'.unescape()")
            .example("'Tab\\\\tSeparated'.unescape()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Linear, true)
            .build()
    }

    fn unescape_string(&self, input: &str) -> Result<String> {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'n' => {
                            chars.next(); // consume the 'n'
                            result.push('\n');
                        }
                        'r' => {
                            chars.next(); // consume the 'r'
                            result.push('\r');
                        }
                        't' => {
                            chars.next(); // consume the 't'
                            result.push('\t');
                        }
                        '\\' => {
                            chars.next(); // consume the second '\'
                            result.push('\\');
                        }
                        '"' => {
                            chars.next(); // consume the '"'
                            result.push('"');
                        }
                        '\'' => {
                            chars.next(); // consume the single quote
                            result.push('\'');
                        }
                        'b' => {
                            chars.next(); // consume the 'b'
                            result.push('\u{08}'); // Backspace
                        }
                        'f' => {
                            chars.next(); // consume the 'f'
                            result.push('\u{0C}'); // Form feed
                        }
                        '0' => {
                            chars.next(); // consume the '0'
                            result.push('\u{00}'); // Null
                        }
                        'u' => {
                            chars.next(); // consume the 'u'
                            if let Some(&'{') = chars.peek() {
                                chars.next(); // consume the '{'
                                let mut hex_digits = String::new();

                                // Read hex digits until '}'
                                while let Some(&hex_ch) = chars.peek() {
                                    if hex_ch == '}' {
                                        chars.next(); // consume the '}'
                                        break;
                                    } else if hex_ch.is_ascii_hexdigit() {
                                        hex_digits.push(hex_ch);
                                        chars.next();
                                    } else {
                                        return Err(FhirPathError::EvaluationError { message: 
                                            "Invalid unicode escape sequence".to_string()
                                        });
                                    }
                                }

                                if hex_digits.is_empty() || hex_digits.len() > 6 {
                                    return Err(FhirPathError::EvaluationError { message: 
                                        "Invalid unicode escape sequence length".to_string()
                                    });
                                }

                                match u32::from_str_radix(&hex_digits, 16) {
                                    Ok(code_point) => {
                                        match char::from_u32(code_point) {
                                            Some(unicode_char) => result.push(unicode_char),
                                            None => return Err(FhirPathError::EvaluationError { message: 
                                                "Invalid unicode code point".to_string()
                                            }),
                                        }
                                    }
                                    Err(_) => return Err(FhirPathError::EvaluationError { message: 
                                        "Invalid unicode escape sequence".to_string()
                                    }),
                                }
                            } else {
                                // Simple \u without braces - not supported, treat as literal
                                result.push('\\');
                                result.push('u');
                            }
                        }
                        _ => {
                            // Unknown escape sequence, treat as literal
                            result.push('\\');
                            result.push(next_ch);
                            chars.next();
                        }
                    }
                } else {
                    // Trailing backslash, treat as literal
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl FhirPathOperation for UnescapeFunction {
    fn identifier(&self) -> &str {
        "unescape"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            UnescapeFunction::create_metadata()
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
                    match &items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => return Err(FhirPathError::EvaluationError { message: 
                            "unescape() requires a string input".to_string()
                        }),
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Collection(Collection::new()));
                } else {
                    return Err(FhirPathError::EvaluationError { message: 
                        "unescape() requires a single string value".to_string()
                    });
                }
            }
            _ => return Err(FhirPathError::EvaluationError { message: 
                "unescape() requires a string input".to_string()
            }),
        };

        // Unescape the string
        let unescaped = self.unescape_string(&input_string)?;
        Ok(FhirPathValue::String(unescaped.into()))
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
                    match &items.get(0).unwrap() {
                        FhirPathValue::String(s) => s.clone(),
                        _ => return Some(Err(FhirPathError::EvaluationError { message: 
                            "unescape() requires a string input".to_string()
                        })),
                    }
                } else if items.is_empty() {
                    return Some(Ok(FhirPathValue::Collection(Collection::new())));
                } else {
                    return Some(Err(FhirPathError::EvaluationError { message: 
                        "unescape() requires a single string value".to_string()
                    }));
                }
            }
            _ => return Some(Err(FhirPathError::EvaluationError { message: 
                "unescape() requires a string input".to_string()
            })),
        };

        // Unescape the string
        match self.unescape_string(&input_string) {
            Ok(unescaped) => Some(Ok(FhirPathValue::String(unescaped.into()))),
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
    use octofhir_fhirpath_model::{FhirPathValue, Collection};

    #[tokio::test]
    async fn test_unescape_newline() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hello\\nWorld".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Hello\nWorld");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_tab() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Tab\\tSeparated".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Tab\tSeparated");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_quotes() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("He said \\\"Hello\\\" to me".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "He said \"Hello\" to me");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_backslash() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Path\\\\to\\\\file".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Path\\to\\file");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_carriage_return() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Line1\\rLine2".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Line1\rLine2");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_control_characters() -> Result<()> {
        let function = UnescapeFunction::new();

        // Test backspace and form feed
        let context = EvaluationContext::new(FhirPathValue::String("Text\\bwith\\fcontrol".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Text\u{08}with\u{0C}control");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_null_character() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Text\\0with null".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Text\u{00}with null");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_unicode() -> Result<()> {
        let function = UnescapeFunction::new();

        // Test unicode escape sequences
        let context = EvaluationContext::new(FhirPathValue::String("Text\\u{0001}control\\u{001f}chars".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Text\u{0001}control\u{001f}chars");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_unicode_emoji() -> Result<()> {
        let function = UnescapeFunction::new();

        // Test unicode emoji
        let context = EvaluationContext::new(FhirPathValue::String("Hello\\u{1f44d}World".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Hello👍World");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_mixed_characters() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hello\\nWorld\\t\\\"Test\\\"\\r\\n\\\\Path".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Hello\nWorld\t\"Test\"\r\n\\Path");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_empty_string() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_normal_string() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Normal text with no escapes".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Normal text with no escapes");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_trailing_backslash() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Text\\".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Text\\");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_unknown_escape() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Text\\x41".into()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                // Unknown escape sequences are treated literally
                assert_eq!(unescaped, "Text\\x41");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_unescape_invalid_unicode() -> () {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Text\\u{invalid}".into()));
        let result = function.evaluate(&[], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::EvaluationError { message: msg }) = result {
            assert!(msg.contains("Invalid unicode"));
        } else {
            panic!("Expected InvalidOperation error");
        }
    }

    #[tokio::test]
    async fn test_unescape_collection() -> Result<()> {
        let function = UnescapeFunction::new();

        // Single item collection
        let context = EvaluationContext::new(FhirPathValue::Collection(vec![
            FhirPathValue::String("Hello\\nWorld".into())
        ]));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Hello\nWorld");
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
    async fn test_unescape_roundtrip() -> Result<()> {
        let function = UnescapeFunction::new();

        // Test that escape/unescape are inverse operations
        let original = "Hello\nWorld\t\"Test\"\r\n\\Path";

        // First escape the string
        use super::super::escape::EscapeFunction;
        let escape_fn = EscapeFunction::new();
        let context = EvaluationContext::new(FhirPathValue::String(original.into()));
        let escaped = escape_fn.evaluate(&[], &context).await?;

        // Then unescape it back
        let escaped_context = EvaluationContext::new(escaped);
        let unescaped = function.evaluate(&[], &escaped_context).await?;

        match unescaped {
            FhirPathValue::String(s) => {
                assert_eq!(s, original);
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }

    #[test]
    fn test_unescape_sync() -> Result<()> {
        let function = UnescapeFunction::new();

        let context = EvaluationContext::new(FhirPathValue::String("Hello\\nWorld".into()));
        let result = function.try_evaluate_sync(&[], &context)
            .unwrap()?;

        match result {
            FhirPathValue::String(unescaped) => {
                assert_eq!(unescaped, "Hello\nWorld");
            }
            _ => panic!("Expected String value"),
        }

        Ok(())
    }
}
