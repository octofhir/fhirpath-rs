# Task 13: Implement Missing String Functions

## Status: Pending
**Priority:** High  
**Estimated Time:** 4-5 days  
**Dependencies:** None

## Overview

Based on the test coverage analysis, critical string manipulation functions are completely missing or have significant issues:

### ❌ **Completely Missing (0% pass rate):**
- **contains()**: String contains check (0% - 0/11 tests)
- **matches()**: Regex matching (0% - 0/16 tests)
- **replace()**: String replacement (0% - 0/6 tests)
- **replace-matches()**: Regex replacement (0% - 0/7 tests)
- **split()**: String splitting (0% - 0/4 tests)
- **trim()**: String trimming (0% - 0/6 tests)
- **to-chars()**: Character conversion (0% - 0/1 tests)
- **to-string()**: String conversion (0% - 0/5 tests)

### 🔧 **Partially Working:**
- **length()**: 16.7% pass rate (1/6 tests) - needs fixes
- **starts-with()**: 23.1% pass rate (3/13 tests) - needs fixes  
- **ends-with()**: 27.3% pass rate (3/11 tests) - needs fixes
- **substring()**: 45.5% pass rate (5/11 tests) - needs improvement

## Test Files Affected

- `contains-string.json` - 0% (0/11 tests)
- `matches.json` - 0% (0/16 tests)
- `replace.json` - 0% (0/6 tests)
- `replace-matches.json` - 0% (0/7 tests)
- `split.json` - 0% (0/4 tests)
- `trim.json` - 0% (0/6 tests)
- `to-chars.json` - 0% (0/1 tests)
- `to-string.json` - 0% (0/5 tests)
- `length.json` - 16.7% (1/6 tests)
- `starts-with.json` - 23.1% (3/13 tests)
- `ends-with.json` - 27.3% (3/11 tests)
- `substring.json` - 45.5% (5/11 tests)

## Current Implementation Status

Based on analysis of `/fhirpath-registry/src/function.rs`:

✅ **Already Implemented:**
- `substring()` - working but needs improvement
- `startsWith()` - working but has issues
- `endsWith()` - working but has issues
- `contains()` - implemented but failing due to parser issues
- `length()` - implemented but has issues

❌ **Missing Functions:**
- `matches()`, `replace()`, `replaceMatches()`, `split()`, `trim()`, `toChars()`, `toString()`

## Analysis of Parser Issues

The test coverage shows many string functions are failing due to **parser errors**:
```
Error: Parser error in ''12345'.contains('6') = false': Unexpected token 'Contains' at position 8
```

This indicates that the **tokenizer/parser doesn't recognize `contains` as a function call** but treats it as a separate token.

## Implementation Plan

### Phase 1: Fix Parser Issues (Day 1)

1. **Investigate Parser/Tokenizer Issues**
   - Check `/fhirpath-parser/src/tokenizer.rs` for missing keywords
   - Ensure `contains`, `matches`, `replace`, etc. are properly tokenized
   - Fix function call parsing for string methods

2. **Fix Existing Function Implementations**
   - Debug why `length()`, `startsWith()`, `endsWith()` are failing
   - Likely issues with collection handling or type checking

### Phase 2: Implement Missing Core Functions (Days 2-3)

3. **Implement Contains Function (Fix)**
   ```rust
   // Already implemented - likely parser issue
   // May need to rename or fix registration
   ```

4. **Implement Replace Function**
   ```rust
   struct ReplaceFunction;
   // replace(old, new) - replaces all occurrences of old with new
   ```

5. **Implement Split Function**
   ```rust
   struct SplitFunction;
   // split(separator) - splits string by separator into collection
   ```

6. **Implement Trim Function**
   ```rust
   struct TrimFunction;
   // trim() - removes leading and trailing whitespace
   ```

7. **Implement ToChars Function**
   ```rust
   struct ToCharsFunction;
   // toChars() - converts string to collection of single-character strings
   ```

### Phase 3: Advanced String Functions (Days 4-5)

8. **Implement Regex Functions**
   ```rust
   struct MatchesFunction;
   // matches(regex) - returns true if string matches regex pattern
   
   struct ReplaceMatchesFunction;
   // replaceMatches(regex, replacement) - replaces regex matches
   ```

9. **Fix ToString Function**
   ```rust
   // Already implemented but failing - investigate issues
   ```

## Implementation Details

### String Function Template
```rust
struct [FunctionName]Function;

impl FhirPathFunction for [FunctionName]Function {
    fn name(&self) -> &str { "[function_name]" }
    
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "[function_name]",
                vec![
                    // Parameters specific to each function
                    ParameterInfo::required("param1", TypeInfo::String),
                    ParameterInfo::optional("param2", TypeInfo::String),
                ],
                TypeInfo::String, // or appropriate return type
            )
        });
        &SIG
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        match &context.input {
            FhirPathValue::String(s) => {
                // Perform string operation
                let result = /* string manipulation */;
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(result)]))
            }
            FhirPathValue::Collection(items) => {
                // Handle collection of strings
                let mut results = Vec::new();
                for item in items {
                    if let FhirPathValue::String(s) = item {
                        let result = /* string manipulation */;
                        results.push(FhirPathValue::String(result));
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::collection(vec![])),
        }
    }
}
```

### Specific Implementation Details

#### 1. Replace Function
```rust
fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
    if let (FhirPathValue::String(s), Some(FhirPathValue::String(old)), Some(FhirPathValue::String(new))) 
        = (&context.input, args.get(0), args.get(1)) {
        let result = s.replace(old, new);
        Ok(FhirPathValue::collection(vec![FhirPathValue::String(result)]))
    } else {
        Ok(FhirPathValue::collection(vec![]))
    }
}
```

#### 2. Split Function
```rust
fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
    if let (FhirPathValue::String(s), Some(FhirPathValue::String(separator))) = (&context.input, args.get(0)) {
        let parts: Vec<FhirPathValue> = s.split(separator)
            .map(|part| FhirPathValue::String(part.to_string()))
            .collect();
        Ok(FhirPathValue::collection(parts))
    } else {
        Ok(FhirPathValue::collection(vec![]))
    }
}
```

#### 3. Matches Function (Regex)
```rust
use regex::Regex;

fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
    if let (FhirPathValue::String(s), Some(FhirPathValue::String(pattern))) = (&context.input, args.get(0)) {
        match Regex::new(pattern) {
            Ok(regex) => {
                let matches = regex.is_match(s);
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(matches)]))
            }
            Err(_) => {
                // Invalid regex pattern
                Ok(FhirPathValue::collection(vec![]))
            }
        }
    } else {
        Ok(FhirPathValue::collection(vec![]))
    }
}
```

## Parser Investigation Priority

The first step should be investigating why string function calls are being parsed incorrectly:

1. **Check Tokenizer Keywords**: Ensure all string function names are in the token list
2. **Function Call Parsing**: Verify that `string.functionName(args)` syntax is handled
3. **Method vs Function**: Determine if these should be methods on strings vs standalone functions

## Dependencies

- **Regex Support**: Will need to add `regex` crate to dependencies for `matches()` and `replaceMatches()`
- **Parser Fixes**: May require changes to tokenizer and parser modules

## Expected Outcomes

After completion, the following test pass rates should be achieved:
- `contains-string.json`: 90%+ (10+/11 tests)
- `length.json`: 95%+ (5+/6 tests)
- `starts-with.json`: 90%+ (11+/13 tests)
- `ends-with.json`: 90%+ (10+/11 tests)
- `substring.json`: 90%+ (10+/11 tests)
- `split.json`: 90%+ (3+/4 tests)
- `trim.json`: 90%+ (5+/6 tests)
- `replace.json`: 85%+ (5+/6 tests)
- `matches.json`: 80%+ (12+/16 tests)
- `replace-matches.json`: 75%+ (5+/7 tests)
- `to-chars.json`: 100% (1/1 tests)
- `to-string.json`: 90%+ (4+/5 tests)

## Files to Modify

- `/fhirpath-parser/src/tokenizer.rs` - Add missing string function tokens
- `/fhirpath-parser/src/parser.rs` - Fix function call parsing
- `/fhirpath-registry/src/function.rs` - Implement missing functions and fix existing ones
- `Cargo.toml` - Add regex dependency if needed

## Testing Strategy

1. **Parser Testing**: Verify that string method calls are correctly parsed
2. **Function Testing**: Unit tests for each string function with various inputs
3. **Edge Case Testing**: Empty strings, null inputs, special characters
4. **Regex Testing**: Valid and invalid patterns, complex regex expressions
5. **Official Test Suite**: Run string-related test files

## Notes

- String functions must handle empty collections according to FHIRPath spec
- Regex functions need careful error handling for invalid patterns
- Unicode support should be considered for all string operations
- Performance is important for string functions as they may be used frequently