//! unescape() function - unescapes special characters

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
/// unescape() function - unescapes special characters
pub struct UnescapeFunction;

#[async_trait]
impl AsyncFhirPathFunction for UnescapeFunction {
    fn name(&self) -> &str {
        "unescape"
    }
    fn human_friendly_name(&self) -> &str {
        "Unescape"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "unescape",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // unescape() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(escape_type)) => {
                match escape_type.as_str() {
                    "json" => {
                        let mut result = String::new();
                        let mut chars = s.chars();
                        while let Some(c) = chars.next() {
                            if c == '\\' {
                                match chars.next() {
                                    Some('"') => result.push('"'),
                                    Some('\\') => result.push('\\'),
                                    Some('n') => result.push('\n'),
                                    Some('r') => result.push('\r'),
                                    Some('t') => result.push('\t'),
                                    Some(other) => {
                                        result.push('\\');
                                        result.push(other);
                                    }
                                    None => result.push('\\'),
                                }
                            } else {
                                result.push(c);
                            }
                        }
                        Ok(FhirPathValue::String(result))
                    }
                    "html" => {
                        let mut result = String::new();
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
                                        "lt" => result.push('<'),
                                        "gt" => result.push('>'),
                                        "amp" => result.push('&'),
                                        "quot" => result.push('"'),
                                        "#39" => result.push('\''),
                                        _ => {
                                            // Unknown entity, keep original
                                            result.push('&');
                                            result.push_str(&entity);
                                            result.push(';');
                                        }
                                    }
                                } else {
                                    // No semicolon found, keep original
                                    result.push('&');
                                    result.push_str(&entity);
                                }
                            } else {
                                result.push(c);
                            }
                        }
                        Ok(FhirPathValue::String(result))
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported unescape type: {escape_type}"),
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
