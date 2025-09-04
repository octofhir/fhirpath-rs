//! String functions implementation for FHIRPath
//!
//! Implements comprehensive string manipulation functions including search, transformation,
//! pattern matching, and advanced operations with proper Unicode support.

use super::{FunctionRegistry, FunctionCategory, FunctionContext};
use crate::core::{FhirPathValue, Result, error_code::{FP0053}};
use crate::{register_function};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Thread-safe regex cache for performance optimization
pub struct RegexCache {
    cache: Arc<Mutex<HashMap<String, Regex>>>,
    max_size: usize,
}

impl RegexCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_size,
        }
    }

    pub fn get_regex(&self, pattern: &str) -> std::result::Result<Regex, regex::Error> {
        let mut cache = self.cache.lock().unwrap();
        
        if let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }

        let regex = Regex::new(pattern)?;
        
        // Simple cache eviction: clear cache when it gets too large
        if cache.len() >= self.max_size {
            cache.clear();
        }
        
        cache.insert(pattern.to_string(), regex.clone());
        Ok(regex)
    }
}

impl Default for RegexCache {
    fn default() -> Self {
        Self::new(100) // Cache up to 100 regex patterns
    }
}

// Global regex cache instance
static REGEX_CACHE: Lazy<RegexCache> = Lazy::new(|| RegexCache::default());

/// String utility functions for FHIRPath operations
pub struct StringUtils;

impl StringUtils {
    /// Convert a FhirPathValue to string if possible
    pub fn to_string_value(value: &FhirPathValue) -> Result<String> {
        match value {
            FhirPathValue::String(s) => Ok(s.clone()),
            FhirPathValue::Integer(i) => Ok(i.to_string()),
            FhirPathValue::Decimal(d) => Ok(d.to_string()),
            FhirPathValue::Boolean(b) => Ok(b.to_string()),
            FhirPathValue::Date(d) => Ok(d.to_string()),
            FhirPathValue::DateTime(dt) => Ok(dt.to_string()),
            FhirPathValue::Time(t) => Ok(t.to_string()),
            _ => Err(crate::core::FhirPathError::evaluation_error(
                FP0053,
                "Cannot convert value to string".to_string()
            ))
        }
    }

    /// Perform safe substring operation respecting Unicode boundaries
    pub fn safe_substring(input: &str, start: usize, length: Option<usize>) -> String {
        let chars: Vec<char> = input.chars().collect();
        
        if start >= chars.len() {
            return String::new();
        }

        match length {
            Some(len) => chars.iter().skip(start).take(len).collect(),
            None => chars.iter().skip(start).collect(),
        }
    }

    /// Get cached regex pattern
    pub fn get_cached_regex(pattern: &str) -> std::result::Result<Regex, regex::Error> {
        REGEX_CACHE.get_regex(pattern)
    }
}

impl FunctionRegistry {
    pub fn register_string_functions(&self) -> Result<()> {
        self.register_contains_function()?;
        self.register_indexOf_function()?;
        self.register_lastIndexOf_function()?;
        self.register_substring_function()?;
        self.register_startsWith_function()?;
        self.register_endsWith_function()?;
        self.register_upper_function()?;
        self.register_lower_function()?;
        self.register_replace_function()?;
        self.register_matches_function()?;
        self.register_replaceMatches_function()?;
        self.register_split_function()?;
        self.register_join_function()?;
        self.register_length_function()?;
        self.register_trim_function()?;
        self.register_toChars_function()?;
        Ok(())
    }

    fn register_contains_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "contains",
            category: FunctionCategory::String,
            description: "Returns true if the input string contains the specified substring",
            parameters: ["substring": Some("string".to_string()) => "Substring to search for"],
            return_type: "boolean",
            examples: ["Patient.name.family.contains('Doe')", "'Hello World'.contains('World')"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "contains() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "contains() requires exactly one substring argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "contains() can only be called on string values".to_string()
                        ));
                    }
                };

                let substring = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "contains() substring argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.contains(substring);
                Ok(vec![FhirPathValue::Boolean(result)])
            }
        )
    }

    fn register_indexOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "indexOf",
            category: FunctionCategory::String,
            description: "Returns the zero-based index of the first occurrence of the substring, or -1 if not found",
            parameters: ["substring": Some("string".to_string()) => "Substring to search for"],
            return_type: "integer",
            examples: ["'Hello World'.indexOf('World')", "Patient.name.family.indexOf('Doe')"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "indexOf() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "indexOf() requires exactly one substring argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "indexOf() can only be called on string values".to_string()
                        ));
                    }
                };

                let substring = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "indexOf() substring argument must be a string".to_string()
                        ));
                    }
                };

                // Use char-based indexing for proper Unicode support
                let input_chars: Vec<char> = input_str.chars().collect();
                let substring_chars: Vec<char> = substring.chars().collect();
                
                if substring_chars.is_empty() {
                    return Ok(vec![FhirPathValue::Integer(0)]);
                }

                for i in 0..=input_chars.len().saturating_sub(substring_chars.len()) {
                    if input_chars[i..i + substring_chars.len()] == substring_chars {
                        return Ok(vec![FhirPathValue::Integer(i as i64)]);
                    }
                }

                Ok(vec![FhirPathValue::Integer(-1)])
            }
        )
    }

    fn register_lastIndexOf_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "lastIndexOf",
            category: FunctionCategory::String,
            description: "Returns the zero-based index of the last occurrence of the substring, or -1 if not found",
            parameters: ["substring": Some("string".to_string()) => "Substring to search for"],
            return_type: "integer",
            examples: ["'Hello World World'.lastIndexOf('World')", "Patient.name.family.lastIndexOf('son')"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "lastIndexOf() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "lastIndexOf() requires exactly one substring argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "lastIndexOf() can only be called on string values".to_string()
                        ));
                    }
                };

                let substring = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "lastIndexOf() substring argument must be a string".to_string()
                        ));
                    }
                };

                // Use char-based indexing for proper Unicode support
                let input_chars: Vec<char> = input_str.chars().collect();
                let substring_chars: Vec<char> = substring.chars().collect();
                
                if substring_chars.is_empty() {
                    return Ok(vec![FhirPathValue::Integer(input_chars.len() as i64)]);
                }

                for i in (0..=input_chars.len().saturating_sub(substring_chars.len())).rev() {
                    if input_chars[i..i + substring_chars.len()] == substring_chars {
                        return Ok(vec![FhirPathValue::Integer(i as i64)]);
                    }
                }

                Ok(vec![FhirPathValue::Integer(-1)])
            }
        )
    }

    fn register_substring_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "substring",
            category: FunctionCategory::String,
            description: "Returns a substring starting at the specified index, optionally with specified length",
            parameters: [
                "start": Some("integer".to_string()) => "Zero-based starting index",
                "length": Some("integer".to_string()) => "Length of substring (optional)"
            ],
            return_type: "string",
            examples: ["'Hello World'.substring(6)", "'Hello World'.substring(0, 5)", "Patient.name.family.substring(1, 3)"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "substring() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.is_empty() || context.arguments.len() > 2 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "substring() requires 1 or 2 integer arguments".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "substring() can only be called on string values".to_string()
                        ));
                    }
                };

                let start = match &context.arguments[0] {
                    FhirPathValue::Integer(i) => {
                        if *i < 0 {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "substring() start index must be non-negative".to_string()
                            ));
                        }
                        *i as usize
                    }
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "substring() start argument must be an integer".to_string()
                        ));
                    }
                };

                let length = if context.arguments.len() == 2 {
                    match &context.arguments[1] {
                        FhirPathValue::Integer(i) => {
                            if *i < 0 {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "substring() length must be non-negative".to_string()
                                ));
                            }
                            Some(*i as usize)
                        }
                        _ => {
                            return Err(crate::core::FhirPathError::evaluation_error(
                                FP0053,
                                "substring() length argument must be an integer".to_string()
                            ));
                        }
                    }
                } else {
                    None
                };

                let result = StringUtils::safe_substring(input_str, start, length);
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_startsWith_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "startsWith",
            category: FunctionCategory::String,
            description: "Returns true if the input string starts with the specified prefix",
            parameters: ["prefix": Some("string".to_string()) => "Prefix to check for"],
            return_type: "boolean",
            examples: ["Patient.name.family.startsWith('Mc')", "'Hello World'.startsWith('Hello')"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "startsWith() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "startsWith() requires exactly one prefix argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "startsWith() can only be called on string values".to_string()
                        ));
                    }
                };

                let prefix = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "startsWith() prefix argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.starts_with(prefix);
                Ok(vec![FhirPathValue::Boolean(result)])
            }
        )
    }

    fn register_endsWith_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "endsWith",
            category: FunctionCategory::String,
            description: "Returns true if the input string ends with the specified suffix",
            parameters: ["suffix": Some("string".to_string()) => "Suffix to check for"],
            return_type: "boolean",
            examples: ["Patient.name.family.endsWith('son')", "'Hello World'.endsWith('World')"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "endsWith() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "endsWith() requires exactly one suffix argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "endsWith() can only be called on string values".to_string()
                        ));
                    }
                };

                let suffix = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "endsWith() suffix argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.ends_with(suffix);
                Ok(vec![FhirPathValue::Boolean(result)])
            }
        )
    }

    fn register_upper_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "upper",
            category: FunctionCategory::String,
            description: "Returns the input string converted to uppercase",
            parameters: [],
            return_type: "string",
            examples: ["Patient.name.family.upper()", "'hello world'.upper()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "upper() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "upper() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.to_uppercase();
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_lower_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "lower",
            category: FunctionCategory::String,
            description: "Returns the input string converted to lowercase",
            parameters: [],
            return_type: "string",
            examples: ["Patient.name.family.lower()", "'HELLO WORLD'.lower()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "lower() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "lower() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.to_lowercase();
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_length_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "length",
            category: FunctionCategory::String,
            description: "Returns the length of the input string in characters (Unicode-aware)",
            parameters: [],
            return_type: "integer",
            examples: ["Patient.name.family.length()", "'Hello World'.length()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "length() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "length() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.chars().count() as i64;
                Ok(vec![FhirPathValue::Integer(result)])
            }
        )
    }

    fn register_trim_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "trim",
            category: FunctionCategory::String,
            description: "Returns the input string with leading and trailing whitespace removed",
            parameters: [],
            return_type: "string",
            examples: ["'  hello world  '.trim()", "Patient.name.family.trim()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "trim() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "trim() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.trim().to_string();
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_replace_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "replace",
            category: FunctionCategory::String,
            description: "Returns the input string with all occurrences of the search string replaced with the replacement string",
            parameters: [
                "search": Some("string".to_string()) => "String to search for",
                "replace": Some("string".to_string()) => "Replacement string"
            ],
            return_type: "string",
            examples: [
                "'Hello World'.replace('World', 'Universe')",
                "Patient.name.family.replace('Mc', 'Mac')"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "replace() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 2 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "replace() requires exactly two string arguments".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replace() can only be called on string values".to_string()
                        ));
                    }
                };

                let search = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replace() search argument must be a string".to_string()
                        ));
                    }
                };

                let replacement = match &context.arguments[1] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replace() replacement argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.replace(search, replacement);
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_matches_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "matches",
            category: FunctionCategory::String,
            description: "Returns true if the input string matches the specified regular expression",
            parameters: ["regex": Some("string".to_string()) => "Regular expression pattern"],
            return_type: "boolean",
            examples: [
                "'hello@example.com'.matches('[a-z]+@[a-z]+\\.[a-z]+')",
                "Patient.telecom.value.matches('^\\+1-[0-9]{3}-[0-9]{3}-[0-9]{4}$')"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "matches() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "matches() requires exactly one regex pattern argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "matches() can only be called on string values".to_string()
                        ));
                    }
                };

                let pattern = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "matches() pattern argument must be a string".to_string()
                        ));
                    }
                };

                let regex = match StringUtils::get_cached_regex(pattern) {
                    Ok(r) => r,
                    Err(e) => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Invalid regex pattern in matches(): {}", e)
                        ));
                    }
                };

                let result = regex.is_match(input_str);
                Ok(vec![FhirPathValue::Boolean(result)])
            }
        )
    }

    fn register_replaceMatches_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "replaceMatches",
            category: FunctionCategory::String,
            description: "Returns the input string with all matches of the regex pattern replaced with the replacement string",
            parameters: [
                "regex": Some("string".to_string()) => "Regular expression pattern to match",
                "replacement": Some("string".to_string()) => "Replacement string"
            ],
            return_type: "string",
            examples: [
                "'Hello 123 World 456'.replaceMatches('[0-9]+', 'XXX')",
                "Patient.name.text.replaceMatches('\\s+', ' ')"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "replaceMatches() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 2 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "replaceMatches() requires exactly two arguments: regex pattern and replacement".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replaceMatches() can only be called on string values".to_string()
                        ));
                    }
                };

                let pattern = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replaceMatches() pattern argument must be a string".to_string()
                        ));
                    }
                };

                let replacement = match &context.arguments[1] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replaceMatches() replacement argument must be a string".to_string()
                        ));
                    }
                };

                let regex = match StringUtils::get_cached_regex(pattern) {
                    Ok(r) => r,
                    Err(e) => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Invalid regex pattern in replaceMatches(): {}", e)
                        ));
                    }
                };

                let result = regex.replace_all(input_str, replacement).to_string();
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_split_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "split",
            category: FunctionCategory::String,
            description: "Splits the input string by the specified separator and returns a collection of strings",
            parameters: ["separator": Some("string".to_string()) => "String to split by"],
            return_type: "collection",
            examples: [
                "'a,b,c'.split(',')",
                "Patient.name.text.split(' ')"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "split() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "split() requires exactly one separator argument".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "split() can only be called on string values".to_string()
                        ));
                    }
                };

                let separator = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "split() separator argument must be a string".to_string()
                        ));
                    }
                };

                let result: Vec<FhirPathValue> = if separator.is_empty() {
                    // Split into individual characters
                    input_str.chars().map(|c| FhirPathValue::String(c.to_string())).collect()
                } else {
                    input_str.split(separator)
                        .map(|s| FhirPathValue::String(s.to_string()))
                        .collect()
                };

                Ok(result)
            }
        )
    }

    fn register_join_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "join",
            category: FunctionCategory::String,
            description: "Joins a collection of strings using the specified separator",
            parameters: ["separator": Some("string".to_string()) => "String to join with"],
            return_type: "string",
            examples: [
                "Patient.name.given.join(' ')",
                "('a', 'b', 'c').join(',')"
            ],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "join() requires exactly one separator argument".to_string()
                    ));
                }

                let separator = match &context.arguments[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "join() separator argument must be a string".to_string()
                        ));
                    }
                };

                // Convert all input values to strings
                let string_values: Result<Vec<String>> = context.input
                    .iter()
                    .map(|v| StringUtils::to_string_value(v))
                    .collect();

                let strings = string_values?;
                let result = strings.join(separator);
                Ok(vec![FhirPathValue::String(result)])
            }
        )
    }

    fn register_toChars_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toChars",
            category: FunctionCategory::String,
            description: "Converts the input string into a collection of single-character strings",
            parameters: [],
            return_type: "collection",
            examples: ["'Hello'.toChars()", "Patient.name.family.toChars()"],
            implementation: |context: &FunctionContext| -> Result<Vec<FhirPathValue>> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "toChars() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input[0] {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "toChars() can only be called on string values".to_string()
                        ));
                    }
                };

                let result: Vec<FhirPathValue> = input_str
                    .chars()
                    .map(|c| FhirPathValue::String(c.to_string()))
                    .collect();

                Ok(result)
            }
        )
    }
}

// mod string_tests;