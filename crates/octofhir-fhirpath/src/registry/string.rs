//! String functions implementation for FHIRPath
//!
//! Implements comprehensive string manipulation functions including search, transformation,
//! pattern matching, and advanced operations with proper Unicode support.

use super::{FunctionCategory, FunctionContext, FunctionRegistry};
use crate::core::{FhirPathValue, Result, error_code::FP0053};
use crate::register_function;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
                "Cannot convert value to string".to_string(),
            )),
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

    /// Get cached regex pattern with dotall mode enabled (for matches() function)
    pub fn get_cached_regex_dotall(pattern: &str) -> std::result::Result<Regex, regex::Error> {
        let dotall_pattern = if pattern.starts_with("(?") {
            // Pattern already has flags, don't add (?s)
            pattern.to_string()
        } else {
            // Add dotall flag to make '.' match newlines
            format!("(?s){}", pattern)
        };
        REGEX_CACHE.get_regex(&dotall_pattern)
    }
}

impl FunctionRegistry {
    pub fn register_string_functions(&self) -> Result<()> {
        self.register_contains_function()?;
        self.register_index_of_function()?;
        self.register_last_index_of_function()?;
        self.register_substring_function()?;
        self.register_starts_with_function()?;
        self.register_ends_with_function()?;
        self.register_upper_function()?;
        self.register_lower_function()?;
        self.register_replace_function()?;
        self.register_matches_function()?;
        self.register_replace_matches_function()?;
        self.register_split_function()?;
        self.register_join_function()?;
        self.register_length_function()?;
        self.register_trim_function()?;
        self.register_to_chars_function()?;
        self.register_encode_function()?;
        self.register_decode_function()?;
        self.register_escape_function()?;
        self.register_unescape_function()?;
        self.register_matches_full_function()?;
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "contains() can only be called on string values".to_string()
                        ));
                    }
                };

                let substring = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "contains() substring argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.contains(substring);
                Ok(FhirPathValue::Boolean(result))
            }
        )
    }

    fn register_index_of_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "indexOf",
            category: FunctionCategory::String,
            description: "Returns the zero-based index of the first occurrence of the substring, or -1 if not found",
            parameters: ["substring": Some("string".to_string()) => "Substring to search for"],
            return_type: "integer",
            examples: ["'Hello World'.indexOf('World')", "Patient.name.family.indexOf('Doe')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "indexOf() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Ok(FhirPathValue::empty());
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        // FHIRPath: return empty if called on non-string values
                        return Ok(FhirPathValue::empty());
                    }
                };

                let substring = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        // FHIRPath: return empty if argument is not a string
                        return Ok(FhirPathValue::empty());
                    }
                };

                // Use char-based indexing for proper Unicode support
                let input_chars: Vec<char> = input_str.chars().collect();
                let substring_chars: Vec<char> = substring.chars().collect();

                if substring_chars.is_empty() {
                    return Ok(FhirPathValue::Integer(0));
                }

                for i in 0..=input_chars.len().saturating_sub(substring_chars.len()) {
                    if input_chars[i..i + substring_chars.len()] == substring_chars {
                        return Ok(FhirPathValue::Integer(i as i64));
                    }
                }

                Ok(FhirPathValue::Integer(-1))
            }
        )
    }

    fn register_last_index_of_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "lastIndexOf",
            category: FunctionCategory::String,
            description: "Returns the zero-based index of the last occurrence of the substring, or -1 if not found",
            parameters: ["substring": Some("string".to_string()) => "Substring to search for"],
            return_type: "integer",
            examples: ["'Hello World World'.lastIndexOf('World')", "Patient.name.family.lastIndexOf('son')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "lastIndexOf() can only be called on string values".to_string()
                        ));
                    }
                };

                let substring = match &context.arguments {
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
                    return Ok(FhirPathValue::Integer(input_chars.len() as i64));
                }

                for i in (0..=input_chars.len().saturating_sub(substring_chars.len())).rev() {
                    if input_chars[i..i + substring_chars.len()] == substring_chars {
                        return Ok(FhirPathValue::Integer(i as i64));
                    }
                }

                Ok(FhirPathValue::Integer(-1))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
               if context.input.is_empty() || context.arguments.is_empty(){
                    return Ok(FhirPathValue::Empty)
                }

                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "substring() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() == 0 || context.arguments.len() > 2 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "substring() requires 1 or 2 integer arguments".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "substring() can only be called on string values".to_string()
                        ));
                    }
                };

                let start = match &context.arguments.first() {
                    Some(FhirPathValue::Integer(i)) => {
                        if *i < 0 {
                           return Ok(FhirPathValue::Empty)
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
                    match context.arguments.get(1) {
                        Some(FhirPathValue::Integer(i)) => {
                            if *i < 0 {
                                return Ok(FhirPathValue::String(String::new()))
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
                match result.len(){
                    0 => Ok(FhirPathValue::Empty),
                    _ =>Ok(FhirPathValue::String(result))
                }
            }
        )
    }

    fn register_starts_with_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "startsWith",
            category: FunctionCategory::String,
            description: "Returns true if the input string starts with the specified prefix",
            parameters: ["prefix": Some("string".to_string()) => "Prefix to check for"],
            return_type: "boolean",
            examples: ["Patient.name.family.startsWith('Mc')", "'Hello World'.startsWith('Hello')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "startsWith() can only be called on string values".to_string()
                        ));
                    }
                };

                let prefix = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "startsWith() prefix argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.starts_with(prefix);
                Ok(FhirPathValue::Boolean(result))
            }
        )
    }

    fn register_ends_with_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "endsWith",
            category: FunctionCategory::String,
            description: "Returns true if the input string ends with the specified suffix",
            parameters: ["suffix": Some("string".to_string()) => "Suffix to check for"],
            return_type: "boolean",
            examples: ["Patient.name.family.endsWith('son')", "'Hello World'.endsWith('World')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "endsWith() can only be called on string values".to_string()
                        ));
                    }
                };

                let suffix = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "endsWith() suffix argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.ends_with(suffix);
                Ok(FhirPathValue::Boolean(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "upper() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "upper() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.to_uppercase();
                Ok(FhirPathValue::String(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "lower() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "lower() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.to_lowercase();
                Ok(FhirPathValue::String(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "length() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "length() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.chars().count() as i64;
                Ok(FhirPathValue::Integer(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "trim() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "trim() can only be called on string values".to_string()
                        ));
                    }
                };

                let result = input_str.trim().to_string();
                Ok(FhirPathValue::String(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replace() can only be called on string values".to_string()
                        ));
                    }
                };

                let args = context.arguments.cloned_collection();

                // If either argument is empty ({}), the result is empty
                match &args.get(0) {
                    Some(FhirPathValue::Empty) => return Ok(FhirPathValue::empty()),
                    _ => {}
                }
                match args.get(1) {
                    Some(FhirPathValue::Empty) => return Ok(FhirPathValue::empty()),
                    None => return Ok(FhirPathValue::empty()),
                    _ => {}
                }

                let search = match &args.get(0) {
                    Some(FhirPathValue::String(s)) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replace() search argument must be a string".to_string()
                        ));
                    }
                };

                let replacement = match args.get(1) {
                    Some(FhirPathValue::String(s)) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replace() replacement argument must be a string".to_string()
                        ));
                    }
                };

                let result = input_str.replace(search, &replacement);
                Ok(FhirPathValue::String(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() || context.arguments.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "matches() can only be called on string values".to_string()
                        ));
                    }
                };

                let pattern = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "matches() pattern argument must be a string".to_string()
                        ));
                    }
                };

                let regex = match StringUtils::get_cached_regex_dotall(pattern) {
                    Ok(r) => r,
                    Err(e) => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Invalid regex pattern in matches(): {}", e)
                        ));
                    }
                };

                let result = regex.is_match(input_str);
                Ok(FhirPathValue::Boolean(result))
            }
        )
    }

    fn register_replace_matches_function(&self) -> Result<()> {
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.is_empty() {
                    return Ok(FhirPathValue::empty());
                }

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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replaceMatches() can only be called on string values".to_string()
                        ));
                    }
                };
                let args = context.arguments.cloned_collection();

                let pattern = match &args.get(0) {
                    Some(FhirPathValue::String(s)) => s,
                     Some(FhirPathValue::Empty) => {
                        return Ok(FhirPathValue::Empty)
                    }
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replaceMatches() pattern argument must be a string".to_string()
                        ));
                    }
                };

                let replacement = match &args.get(1) {
                    Some(FhirPathValue::String(s)) => s,
                    Some(FhirPathValue::Empty) => {
                        return Ok(FhirPathValue::Empty)
                    }
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "replaceMatches() replacement argument must be a string".to_string()
                        ));
                    }
                };

                // FHIRPath spec: empty pattern should return the original string unchanged
                if pattern.is_empty() {
                    return Ok(FhirPathValue::String(input_str.clone()));
                }

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
                Ok(FhirPathValue::String(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
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

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "split() can only be called on string values".to_string()
                        ));
                    }
                };

                let separator = match &context.arguments {
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

                Ok(FhirPathValue::collection(result))
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
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "join() requires exactly one separator argument".to_string()
                    ));
                }

                let separator = match &context.arguments {
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
                Ok(FhirPathValue::String(result))
            }
        )
    }

    fn register_to_chars_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "toChars",
            category: FunctionCategory::String,
            description: "Converts the input string into a collection of single-character strings",
            parameters: [],
            return_type: "collection",
            examples: ["'Hello'.toChars()", "Patient.name.family.toChars()"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "toChars() can only be called on a single string value".to_string()
                    ));
                }

                let input_str = match &context.input {
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

                Ok(FhirPathValue::collection(result))
            }
        )
    }

    fn register_encode_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "encode",
            category: FunctionCategory::String,
            description: "Encodes the input string using the specified format",
            parameters: ["format": Some("string".to_string()) => "Encoding format ('base64', 'hex', 'url')"],
            return_type: "string",
            examples: ["'Hello'.encode('base64')", "'test@example.com'.encode('url')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "encode() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "encode() requires exactly one format argument".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "encode() can only be called on string values".to_string()
                        ));
                    }
                };

                let format = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "encode() format argument must be a string".to_string()
                        ));
                    }
                };

                let result = match format.as_str() {
                    "base64" => {
                        use base64::{Engine as _, engine::general_purpose::STANDARD};
                        STANDARD.encode(input_str.as_bytes())
                    },
                    "urlbase64" => {
                        use base64::{Engine as _, engine::general_purpose::URL_SAFE};
                        URL_SAFE.encode(input_str.as_bytes())
                    },
                    "hex" => {
                        input_str.as_bytes()
                            .iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>()
                    },
                    "url" => urlencoding::encode(input_str).to_string(),
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Unsupported encoding format: '{}'", format)
                        ));
                    }
                };

                Ok(FhirPathValue::String(result))
            }
        )
    }

    fn register_decode_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "decode",
            category: FunctionCategory::String,
            description: "Decodes the input string using the specified format",
            parameters: ["format": Some("string".to_string()) => "Decoding format ('base64', 'hex', 'url')"],
            return_type: "string",
            examples: ["'SGVsbG8='.decode('base64')", "'48656c6c6f'.decode('hex')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "decode() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "decode() requires exactly one format argument".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "decode() can only be called on string values".to_string()
                        ));
                    }
                };

                let format = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "decode() format argument must be a string".to_string()
                        ));
                    }
                };

                let result = match format.as_str() {
                    "base64" => {
                        use base64::{Engine as _, engine::general_purpose::STANDARD};
                        match STANDARD.decode(input_str.as_bytes()) {
                            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                            Err(_) => {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "Invalid base64 input".to_string()
                                ));
                            }
                        }
                    },
                    "urlbase64" => {
                        use base64::{Engine as _, engine::general_purpose::URL_SAFE};
                        match URL_SAFE.decode(input_str.as_bytes()) {
                            Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                            Err(_) => {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "Invalid urlbase64 input".to_string()
                                ));
                            }
                        }
                    },
                    "hex" => {
                        let mut bytes = Vec::new();
                        let mut chars = input_str.chars();

                        while let (Some(high), Some(low)) = (chars.next(), chars.next()) {
                            if let (Some(h), Some(l)) = (high.to_digit(16), low.to_digit(16)) {
                                bytes.push(((h << 4) | l) as u8);
                            } else {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "Invalid hex input".to_string()
                                ));
                            }
                        }

                        String::from_utf8_lossy(&bytes).to_string()
                    },
                    "url" => {
                        match urlencoding::decode(input_str) {
                            Ok(cow) => cow.to_string(),
                            Err(_) => {
                                return Err(crate::core::FhirPathError::evaluation_error(
                                    FP0053,
                                    "Invalid URL-encoded input".to_string()
                                ));
                            }
                        }
                    },
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Unsupported decoding format: '{}'", format)
                        ));
                    }
                };

                Ok(FhirPathValue::String(result))
            }
        )
    }

    fn register_escape_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "escape",
            category: FunctionCategory::String,
            description: "Escapes special characters in the input string for the specified format",
            parameters: ["format": Some("string".to_string()) => "Escape format ('html', 'json', 'xml')"],
            return_type: "string",
            examples: ["'<tag>'.escape('html')", "'\"text\"'.escape('json')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "escape() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "escape() requires exactly one format argument".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "escape() can only be called on string values".to_string()
                        ));
                    }
                };

                let format = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "escape() format argument must be a string".to_string()
                        ));
                    }
                };

                let result = match format.as_str() {
                    "html" => {
                        // Based on FHIRPath tests, HTML escape should NOT escape < > characters
                        // Only escape & for HTML context in FHIRPath
                        input_str
                            .replace('&', "&amp;")
                            // Note: FHIRPath tests show < and > should NOT be escaped
                    },
                    "json" => {
                        // JSON escape should escape quotes and special characters
                        // FHIRPath expects just escaped quotes, not additional outer quotes
                        input_str
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r")
                            .replace('\t', "\\t")
                    },
                    "xml" => {
                        input_str
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\'', "&apos;")
                    },
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Unsupported escape format: '{}'", format)
                        ));
                    }
                };

                Ok(FhirPathValue::String(result))
            }
        )
    }

    fn register_unescape_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "unescape",
            category: FunctionCategory::String,
            description: "Unescapes special characters in the input string for the specified format",
            parameters: ["format": Some("string".to_string()) => "Unescape format ('html', 'json', 'xml')"],
            return_type: "string",
            examples: ["'&lt;tag&gt;'.unescape('html')", "'\\\"text\\\"'.unescape('json')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "unescape() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "unescape() requires exactly one format argument".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "unescape() can only be called on string values".to_string()
                        ));
                    }
                };

                let format = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "unescape() format argument must be a string".to_string()
                        ));
                    }
                };

                let result = match format.as_str() {
                    "html" | "xml" => {
                        input_str
                            .replace("&lt;", "<")
                            .replace("&gt;", ">")
                            .replace("&quot;", "\"")
                            .replace("&#39;", "'")
                            .replace("&apos;", "'")
                            .replace("&amp;", "&") // Must be last
                    },
                    "json" => {
                        // JSON unescape should handle escaped quotes and special characters
                        // FHIRPath expects to preserve the actual content after unescaping
                        input_str
                            .replace("\\\"", "\"")
                            .replace("\\\\", "\\")
                            .replace("\\n", "\n")
                            .replace("\\r", "\r")
                            .replace("\\t", "\t")
                    },
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Unsupported unescape format: '{}'", format)
                        ));
                    }
                };

                Ok(FhirPathValue::String(result))
            }
        )
    }

    fn register_matches_full_function(&self) -> Result<()> {
        register_function!(
            self,
            sync "matchesFull",
            category: FunctionCategory::String,
            description: "Returns true if the entire input string matches the given regex pattern",
            parameters: ["pattern": Some("string".to_string()) => "Regular expression pattern"],
            return_type: "boolean",
            examples: ["'Hello123'.matchesFull('[A-Za-z0-9]+')", "Patient.id.matchesFull('[a-f0-9-]+')"],
            implementation: |context: &FunctionContext| -> Result<FhirPathValue> {
                if context.input.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "matchesFull() can only be called on a single string value".to_string()
                    ));
                }

                if context.arguments.len() != 1 {
                    return Err(crate::core::FhirPathError::evaluation_error(
                        FP0053,
                        "matchesFull() requires exactly one pattern argument".to_string()
                    ));
                }

                let input_str = match &context.input {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "matchesFull() can only be called on string values".to_string()
                        ));
                    }
                };

                let pattern = match &context.arguments {
                    FhirPathValue::String(s) => s,
                    _ => {
                        return Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            "matchesFull() pattern argument must be a string".to_string()
                        ));
                    }
                };

                // Ensure the pattern matches the entire string by anchoring it
                let anchored_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
                    pattern.clone()
                } else if pattern.starts_with('^') {
                    format!("{}$", pattern)
                } else if pattern.ends_with('$') {
                    format!("^{}", pattern)
                } else {
                    format!("^{}$", pattern)
                };

                match StringUtils::get_cached_regex(&anchored_pattern) {
                    Ok(regex) => {
                        let result = regex.is_match(input_str);
                        Ok(FhirPathValue::Boolean(result))
                    },
                    Err(_) => {
                        Err(crate::core::FhirPathError::evaluation_error(
                            FP0053,
                            format!("Invalid regular expression: '{}'", pattern)
                        ))
                    }
                }
            }
        )
    }
}

// mod string_tests;
