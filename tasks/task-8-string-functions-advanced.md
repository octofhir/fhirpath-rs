# Task 8: Complete String Functions Implementation

## Overview
Implement remaining string manipulation functions to achieve full FHIRPath string processing capabilities.

## Current Issues from TEST_COVERAGE.md
- **length.json** - 16.7% (1/6 tests) - Issues
- **starts-with.json** - 23.1% (3/13 tests) - Issues  
- **ends-with.json** - 27.3% (3/11 tests) - Issues
- **matches.json** - 0.0% (0/16 tests) - Missing
- **replace.json** - 0.0% (0/6 tests) - Missing
- **replace-matches.json** - 0.0% (0/7 tests) - Missing
- **split.json** - 0.0% (0/4 tests) - Missing
- **trim.json** - 0.0% (0/6 tests) - Missing
- **to-chars.json** - 0.0% (0/1 tests) - Missing
- **index-of.json** - 0.0% (0/6 tests) - Missing

## Subtasks

### 8.1 Fix Existing String Functions
- [ ] Debug and fix length() function (currently 16.7%)
- [ ] Fix startsWith() function Collection handling (currently 23.1%)
- [ ] Fix endsWith() function Collection handling (currently 27.3%)
- [ ] Apply same pattern as contains() function
- **Target**: length 16.7% → 90%+, startsWith 23.1% → 90%+, endsWith 27.3% → 90%+

### 8.2 Implement Regular Expression Functions
- [ ] Implement matches() function with regex support
- [ ] Implement replace() function for simple string replacement
- [ ] Implement replaceMatches() function with regex replacement
- [ ] Add proper regex library integration
- **Target**: matches 0% → 80%+, replace 0% → 90%+, replace-matches 0% → 70%+

### 8.3 Implement String Manipulation Functions
- [ ] Implement split() function with delimiter support
- [ ] Implement trim() function for whitespace removal
- [ ] Implement toChars() function to convert string to character collection
- [ ] Implement indexOf() function for substring searching
- **Target**: split 0% → 90%+, trim 0% → 100%, toChars 0% → 100%, indexOf 0% → 90%+

### 8.4 Handle Edge Cases
- [ ] Empty string handling
- [ ] Null/empty collection handling
- [ ] Unicode and special character support
- [ ] Case sensitivity options where applicable

## Expected Outcomes
- length.json: 16.7% → 90%+
- starts-with.json: 23.1% → 90%+ 
- ends-with.json: 27.3% → 90%+
- matches.json: 0% → 80%+
- replace.json: 0% → 90%+
- replace-matches.json: 0% → 70%+
- split.json: 0% → 90%+
- trim.json: 0% → 100%
- to-chars.json: 0% → 100%
- index-of.json: 0% → 90%+
- Overall test coverage improvement: +4-6%

## Files to Modify
- `/fhirpath-registry/src/function.rs` - String functions
- `/Cargo.toml` - Add regex dependency if needed
- `/fhirpath-registry/Cargo.toml` - Add regex dependency