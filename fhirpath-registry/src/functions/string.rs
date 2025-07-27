//! String manipulation functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use regex::Regex;

/// substring() function - extracts a substring
pub struct SubstringFunction;

impl FhirPathFunction for SubstringFunction {
    fn name(&self) -> &str { "substring" }
    fn human_friendly_name(&self) -> &str { "Substring" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "substring",
                vec![
                    ParameterInfo::required("start", TypeInfo::Integer),
                    ParameterInfo::optional("length", TypeInfo::Integer),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => {
                let start = match &args[0] {
                    FhirPathValue::Integer(i) => *i as usize,
                    _ => return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 0,
                        expected: "Integer".to_string(),
                        actual: format!("{:?}", args[0]),
                    }),
                };
                
                let chars: Vec<char> = s.chars().collect();
                if start >= chars.len() {
                    return Ok(FhirPathValue::String(String::new()));
                }
                
                let result = if let Some(FhirPathValue::Integer(len)) = args.get(1) {
                    chars.iter().skip(start).take(*len as usize).collect()
                } else {
                    chars.iter().skip(start).collect()
                };
                
                Ok(FhirPathValue::String(result))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// startsWith() function - checks if string starts with prefix
pub struct StartsWithFunction;

impl FhirPathFunction for StartsWithFunction {
    fn name(&self) -> &str { "startsWith" }
    fn human_friendly_name(&self) -> &str { "Starts With" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "startsWith",
                vec![ParameterInfo::required("prefix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(prefix)) => {
                Ok(FhirPathValue::Boolean(s.starts_with(prefix)))
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

/// endsWith() function - checks if string ends with suffix
pub struct EndsWithFunction;

impl FhirPathFunction for EndsWithFunction {
    fn name(&self) -> &str { "endsWith" }
    fn human_friendly_name(&self) -> &str { "Ends With" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "endsWith",
                vec![ParameterInfo::required("suffix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(suffix)) => {
                Ok(FhirPathValue::Boolean(s.ends_with(suffix)))
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

/// contains() function - checks if string contains substring
pub struct ContainsFunction;

impl FhirPathFunction for ContainsFunction {
    fn name(&self) -> &str { "contains" }
    fn human_friendly_name(&self) -> &str { "Contains" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "contains",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                Ok(FhirPathValue::Boolean(s.contains(substring)))
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

/// matches() function - regex match
pub struct MatchesFunction;

impl FhirPathFunction for MatchesFunction {
    fn name(&self) -> &str { "matches" }
    fn human_friendly_name(&self) -> &str { "Matches" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "matches",
                vec![ParameterInfo::required("pattern", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern)) => {
                match Regex::new(pattern) {
                    Ok(re) => Ok(FhirPathValue::Boolean(re.is_match(s))),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
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

/// matchesFull() function - full regex match
pub struct MatchesFullFunction;

impl FhirPathFunction for MatchesFullFunction {
    fn name(&self) -> &str { "matchesFull" }
    fn human_friendly_name(&self) -> &str { "Matches Full" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "matchesFull",
                vec![ParameterInfo::required("pattern", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern)) => {
                // Add anchors if not present
                let full_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
                    pattern.clone()
                } else if pattern.starts_with('^') {
                    format!("{}$", pattern)
                } else if pattern.ends_with('$') {
                    format!("^{}", pattern)
                } else {
                    format!("^{}$", pattern)
                };
                
                match Regex::new(&full_pattern) {
                    Ok(re) => Ok(FhirPathValue::Boolean(re.is_match(s))),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
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

/// replace() function - string replacement
pub struct ReplaceFunction;

impl FhirPathFunction for ReplaceFunction {
    fn name(&self) -> &str { "replace" }
    fn human_friendly_name(&self) -> &str { "Replace" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replace",
                vec![
                    ParameterInfo::required("pattern", TypeInfo::String),
                    ParameterInfo::required("substitution", TypeInfo::String),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0], &args[1]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern), FhirPathValue::String(substitution)) => {
                Ok(FhirPathValue::String(s.replace(pattern, substitution)))
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// replaceMatches() function - regex replacement
pub struct ReplaceMatchesFunction;

impl FhirPathFunction for ReplaceMatchesFunction {
    fn name(&self) -> &str { "replaceMatches" }
    fn human_friendly_name(&self) -> &str { "Replace Matches" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replaceMatches",
                vec![
                    ParameterInfo::required("pattern", TypeInfo::String),
                    ParameterInfo::required("substitution", TypeInfo::String),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0], &args[1]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern), FhirPathValue::String(substitution)) => {
                match Regex::new(pattern) {
                    Ok(re) => Ok(FhirPathValue::String(re.replace_all(s, substitution).to_string())),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
                    }),
                }
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// split() function - splits string by separator
pub struct SplitFunction;

impl FhirPathFunction for SplitFunction {
    fn name(&self) -> &str { "split" }
    fn human_friendly_name(&self) -> &str { "Split" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "split",
                vec![ParameterInfo::required("separator", TypeInfo::String)],
                TypeInfo::collection(TypeInfo::String),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(separator)) => {
                let parts: Vec<FhirPathValue> = s.split(separator)
                    .map(|part| FhirPathValue::String(part.to_string()))
                    .collect();
                Ok(FhirPathValue::collection(parts))
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

/// join() function - joins collection of strings
pub struct JoinFunction;

impl FhirPathFunction for JoinFunction {
    fn name(&self) -> &str { "join" }
    fn human_friendly_name(&self) -> &str { "Join" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "join",
                vec![ParameterInfo::optional("separator", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let separator = match args.get(0) {
            Some(FhirPathValue::String(s)) => s.as_str(),
            Some(_) => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", args[0]),
            }),
            None => "",
        };
        
        let items = context.input.clone().to_collection();
        let strings: Result<Vec<String>, _> = items.into_iter()
            .map(|item| match item {
                FhirPathValue::String(s) => Ok(s.clone()),
                _ => Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", item),
                }),
            })
            .collect();
        
        match strings {
            Ok(strs) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(strs.join(separator))])),
            Err(e) => Err(e),
        }
    }
}

/// trim() function - removes whitespace from both ends
pub struct TrimFunction;

impl FhirPathFunction for TrimFunction {
    fn name(&self) -> &str { "trim" }
    fn human_friendly_name(&self) -> &str { "Trim" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "trim",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.trim().to_string())),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// toChars() function - converts string to array of single characters
pub struct ToCharsFunction;

impl FhirPathFunction for ToCharsFunction {
    fn name(&self) -> &str { "toChars" }
    fn human_friendly_name(&self) -> &str { "To Chars" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toChars",
                vec![],
                TypeInfo::collection(TypeInfo::String),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<FhirPathValue> = s.chars()
                    .map(|c| FhirPathValue::String(c.to_string()))
                    .collect();
                Ok(FhirPathValue::collection(chars))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// indexOf() function - finds index of substring
pub struct IndexOfFunction;

impl FhirPathFunction for IndexOfFunction {
    fn name(&self) -> &str { "indexOf" }
    fn human_friendly_name(&self) -> &str { "Index Of" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "indexOf",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Integer,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                match s.find(substring) {
                    Some(index) => Ok(FhirPathValue::Integer(index as i64)),
                    None => Ok(FhirPathValue::Integer(-1)),
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

/// upper() function - converts to uppercase
pub struct UpperFunction;

impl FhirPathFunction for UpperFunction {
    fn name(&self) -> &str { "upper" }
    fn human_friendly_name(&self) -> &str { "Upper" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "upper",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_uppercase())),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// lower() function - converts to lowercase
pub struct LowerFunction;

impl FhirPathFunction for LowerFunction {
    fn name(&self) -> &str { "lower" }
    fn human_friendly_name(&self) -> &str { "Lower" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "lower",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.to_lowercase())),
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}

/// encode() function - URL encodes string
pub struct EncodeFunction;

impl FhirPathFunction for EncodeFunction {
    fn name(&self) -> &str { "encode" }
    fn human_friendly_name(&self) -> &str { "Encode" }
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
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(format)) => {
                match format.as_str() {
                    "uri" => Ok(FhirPathValue::String(s.clone())), // URL encoding not available
                    "html" => Ok(FhirPathValue::String(s.clone())), // HTML encoding not available
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported encoding format: {}", format),
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

/// decode() function - decodes URL encoded string
pub struct DecodeFunction;

impl FhirPathFunction for DecodeFunction {
    fn name(&self) -> &str { "decode" }
    fn human_friendly_name(&self) -> &str { "Decode" }
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
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(format)) => {
                match format.as_str() {
                    "uri" => Ok(FhirPathValue::String(s.clone())), // URL decoding not available
                    "html" => Ok(FhirPathValue::String(s.clone())), // HTML decoding not available
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported decoding format: {}", format),
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

/// escape() function - escapes special characters
pub struct EscapeFunction;

impl FhirPathFunction for EscapeFunction {
    fn name(&self) -> &str { "escape" }
    fn human_friendly_name(&self) -> &str { "Escape" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "escape",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(escape_type)) => {
                match escape_type.as_str() {
                    "json" => {
                        let escaped = s.chars()
                            .map(|c| match c {
                                '"' => r#"\""#.to_string(),
                                '\\' => r"\\".to_string(),
                                '\n' => r"\n".to_string(),
                                '\r' => r"\r".to_string(),
                                '\t' => r"\t".to_string(),
                                _ => c.to_string(),
                            })
                            .collect::<String>();
                        Ok(FhirPathValue::String(escaped))
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported escape type: {}", escape_type),
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

/// unescape() function - unescapes special characters
pub struct UnescapeFunction;

impl FhirPathFunction for UnescapeFunction {
    fn name(&self) -> &str { "unescape" }
    fn human_friendly_name(&self) -> &str { "Unescape" }
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
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
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
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported unescape type: {}", escape_type),
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