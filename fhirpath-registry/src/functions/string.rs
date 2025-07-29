//! String manipulation functions

use std::io::Empty;

use crate::function::{EvaluationContext, FhirPathFunction, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use regex::{Regex, RegexBuilder};

/// substring() function - extracts a substring
pub struct SubstringFunction;

impl FhirPathFunction for SubstringFunction {
    fn name(&self) -> &str {
        "substring"
    }
    fn human_friendly_name(&self) -> &str {
        "Substring"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty collections in arguments
        if let Some(FhirPathValue::Collection(items)) = args.get(0) {
            if items.is_empty() {
                return Ok(FhirPathValue::Empty);
            }
        }
        if let Some(FhirPathValue::Collection(items)) = args.get(1) {
            if items.is_empty() {
                return Ok(FhirPathValue::Empty);
            }
        }

        let input_string = match &context.input {
            FhirPathValue::String(s) => s.clone(),
            FhirPathValue::Resource(r) => {
                // Try to extract string value from FhirResource
                match r.as_json() {
                    serde_json::Value::String(s) => s.clone(),
                    _ => return Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) if items.is_empty() => {
                return Ok(FhirPathValue::Empty);
            }
            _ => return Ok(FhirPathValue::Empty),
        };

        let start_int = match &args[0] {
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Handle negative indices and out of bounds - return empty string
        if start_int < 0 {
            return Ok(FhirPathValue::Empty);
        }

        let start = start_int as usize;
        let chars: Vec<char> = input_string.chars().collect();

        if start >= chars.len() {
            return Ok(FhirPathValue::Empty);
        }

        let result = if let Some(length_arg) = args.get(1) {
            match length_arg {
                FhirPathValue::Integer(len_int) => {
                    if *len_int < 0 {
                        return Ok(FhirPathValue::Empty);
                    }
                    let len = *len_int as usize;
                    chars.iter().skip(start).take(len).collect()
                }
                FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
                _ => {
                    return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 1,
                        expected: "Integer".to_string(),
                        actual: format!("{:?}", length_arg),
                    });
                }
            }
        } else {
            chars.iter().skip(start).collect()
        };

        Ok(FhirPathValue::String(result))
    }
}

/// startsWith() function - checks if string starts with prefix
pub struct StartsWithFunction;

impl FhirPathFunction for StartsWithFunction {
    fn name(&self) -> &str {
        "startsWith"
    }
    fn human_friendly_name(&self) -> &str {
        "Starts With"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(prefix)) => {
                Ok(FhirPathValue::Boolean(s.starts_with(prefix)))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// endsWith() function - checks if string ends with suffix
pub struct EndsWithFunction;

impl FhirPathFunction for EndsWithFunction {
    fn name(&self) -> &str {
        "endsWith"
    }
    fn human_friendly_name(&self) -> &str {
        "Ends With"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(suffix)) => {
                Ok(FhirPathValue::Boolean(s.ends_with(suffix)))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// contains() function - checks if string contains substring
pub struct ContainsFunction;

impl FhirPathFunction for ContainsFunction {
    fn name(&self) -> &str {
        "contains"
    }
    fn human_friendly_name(&self) -> &str {
        "Contains"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                Ok(FhirPathValue::Boolean(s.contains(substring)))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// matches() function - regex match
pub struct MatchesFunction;

impl FhirPathFunction for MatchesFunction {
    fn name(&self) -> &str {
        "matches"
    }
    fn human_friendly_name(&self) -> &str {
        "Matches"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(pattern)) => {
                // Use RegexBuilder with single-line mode (dot matches newlines)
                match RegexBuilder::new(pattern)
                    .dot_matches_new_line(true)
                    .build()
                {
                    Ok(re) => Ok(FhirPathValue::Boolean(re.is_match(s))),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
                    }),
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
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
    fn name(&self) -> &str {
        "matchesFull"
    }
    fn human_friendly_name(&self) -> &str {
        "Matches Full"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
    fn name(&self) -> &str {
        "replace"
    }
    fn human_friendly_name(&self) -> &str {
        "Replace"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match (&context.input, &args[0], &args[1]) {
            (
                FhirPathValue::String(s),
                FhirPathValue::String(pattern),
                FhirPathValue::String(substitution),
            ) => {
                // Handle empty pattern case: 'abc'.replace('', 'x') should return 'xaxbxcx'
                if pattern.is_empty() {
                    let mut result = String::new();
                    result.push_str(substitution);
                    for ch in s.chars() {
                        result.push(ch);
                        result.push_str(substitution);
                    }
                    Ok(FhirPathValue::String(result))
                } else {
                    Ok(FhirPathValue::String(s.replace(pattern, substitution)))
                }
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, _, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _, _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, FhirPathValue::Collection(items), _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, _, FhirPathValue::Collection(items)) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
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
    fn name(&self) -> &str {
        "replaceMatches"
    }
    fn human_friendly_name(&self) -> &str {
        "Replace Matches"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match (&context.input, &args[0], &args[1]) {
            (
                FhirPathValue::String(s),
                FhirPathValue::String(pattern),
                FhirPathValue::String(substitution),
            ) => {
                // Handle empty pattern - return original string unchanged
                if pattern.is_empty() {
                    return Ok(FhirPathValue::String(s.clone()));
                }

                match Regex::new(pattern) {
                    Ok(re) => Ok(FhirPathValue::String(
                        re.replace_all(s, substitution).to_string(),
                    )),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {}", e),
                    }),
                }
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _, _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, _, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items), _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, _, FhirPathValue::Collection(items)) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
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
    fn name(&self) -> &str {
        "split"
    }
    fn human_friendly_name(&self) -> &str {
        "Split"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(separator)) => {
                let parts: Vec<FhirPathValue> = s
                    .split(separator)
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
    fn name(&self) -> &str {
        "join"
    }
    fn human_friendly_name(&self) -> &str {
        "Join"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let separator = match args.get(0) {
            Some(FhirPathValue::String(s)) => s.as_str(),
            Some(_) => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
            None => "",
        };

        let items = context.input.clone().to_collection();
        let strings: Vec<String> = items
            .into_iter()
            .map(|item| match item {
                FhirPathValue::String(s) => s.clone(),
                FhirPathValue::Integer(i) => i.to_string(),
                FhirPathValue::Decimal(d) => d.to_string(),
                FhirPathValue::Boolean(b) => b.to_string(),
                FhirPathValue::Date(d) => d.to_string(),
                FhirPathValue::DateTime(dt) => dt.to_string(),
                FhirPathValue::Time(t) => t.to_string(),
                FhirPathValue::Quantity(q) => q.to_string(),
                FhirPathValue::Resource(r) => {
                    // Try to extract string value from FhirResource
                    match r.as_json() {
                        serde_json::Value::String(s) => s.clone(),
                        _ => format!("{:?}", r),
                    }
                }
                FhirPathValue::Empty => String::new(),
                FhirPathValue::Collection(_) => String::new(), // Empty collections become empty strings
                FhirPathValue::TypeInfoObject { namespace, name } => {
                    format!("{}::{}", namespace, name)
                }
            })
            .collect();

        Ok(FhirPathValue::String(strings.join(separator)))
    }
}

/// trim() function - removes whitespace from both ends
pub struct TrimFunction;

impl FhirPathFunction for TrimFunction {
    fn name(&self) -> &str {
        "trim"
    }
    fn human_friendly_name(&self) -> &str {
        "Trim"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("trim", vec![], TypeInfo::String));
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
    fn name(&self) -> &str {
        "toChars"
    }
    fn human_friendly_name(&self) -> &str {
        "To Chars"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toChars", vec![], TypeInfo::collection(TypeInfo::String))
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<FhirPathValue> = s
                    .chars()
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
    fn name(&self) -> &str {
        "indexOf"
    }
    fn human_friendly_name(&self) -> &str {
        "Index Of"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                match s.find(substring) {
                    Some(index) => Ok(FhirPathValue::Integer(index as i64)),
                    None => Ok(FhirPathValue::Integer(-1)),
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// upper() function - converts to uppercase
pub struct UpperFunction;

impl FhirPathFunction for UpperFunction {
    fn name(&self) -> &str {
        "upper"
    }
    fn human_friendly_name(&self) -> &str {
        "Upper"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("upper", vec![], TypeInfo::String));
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
    fn name(&self) -> &str {
        "lower"
    }
    fn human_friendly_name(&self) -> &str {
        "Lower"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("lower", vec![], TypeInfo::String));
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
    fn name(&self) -> &str {
        "encode"
    }
    fn human_friendly_name(&self) -> &str {
        "Encode"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(format)) => {
                match format.as_str() {
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
                        Ok(FhirPathValue::String(encoded))
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
                        Ok(FhirPathValue::String(encoded))
                    }
                    "base64" => {
                        // Base64 encoding
                        let chars =
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
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

                        Ok(FhirPathValue::String(result))
                    }
                    "hex" => {
                        // Hexadecimal encoding
                        let encoded = s
                            .as_bytes()
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<String>();
                        Ok(FhirPathValue::String(encoded))
                    }
                    "urlbase64" => {
                        // URL-safe Base64 encoding (RFC 4648) with padding
                        let chars =
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
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

                        Ok(FhirPathValue::String(result))
                    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(format)) => {
                match format.as_str() {
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
                                    if let Ok(byte) =
                                        u8::from_str_radix(&format!("{}{}", h1, h2), 16)
                                    {
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
                        Ok(FhirPathValue::String(decoded))
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
                        Ok(FhirPathValue::String(decoded))
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
                            Ok(decoded) => Ok(FhirPathValue::String(decoded)),
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
                            Ok(decoded) => Ok(FhirPathValue::String(decoded)),
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
                            Ok(decoded) => Ok(FhirPathValue::String(decoded)),
                            Err(_) => Err(FunctionError::EvaluationError {
                                name: self.name().to_string(),
                                message: "Invalid urlbase64 encoding".to_string(),
                            }),
                        }
                    }
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
    fn name(&self) -> &str {
        "escape"
    }
    fn human_friendly_name(&self) -> &str {
        "Escape"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(escape_type)) => {
                match escape_type.as_str() {
                    "json" => {
                        let escaped = s
                            .chars()
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
                    "html" => {
                        let escaped = s
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
    fn evaluate(
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
