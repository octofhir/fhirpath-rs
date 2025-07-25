# Task 14: Edge Cases and Performance Optimization

## Overview
Address remaining edge cases, improve error handling, and optimize performance for production use.

## Current Issues from TEST_COVERAGE.md
- **precedence.json** - 16.7% (1/6 tests) - Issues
- **literals.json** - 48.8% (40/82 tests) - Issues
- **comments.json** - 0.0% (0/9 tests) - Missing
- **comparable.json** - 0.0% (0/3 tests) - Missing
- Various edge cases across multiple test suites

## Subtasks

### 14.1 Fix Operator Precedence
- [ ] Debug operator precedence issues (currently 16.7%)
- [ ] Ensure proper parsing of complex expressions
- [ ] Fix associativity rules
- [ ] Test nested operator scenarios
- **Target**: precedence.json 16.7% → 90%+

### 14.2 Improve Literal Handling
- [ ] Debug literal parsing issues (currently 48.8%)
- [ ] Fix numeric literal edge cases
- [ ] Improve string literal handling
- [ ] Handle special characters and escaping
- **Target**: literals.json 48.8% → 80%+

### 14.3 Implement Missing Parser Features
- [ ] Add comment support in FHIRPath expressions
- [ ] Implement comparable operations
- [ ] Add proper whitespace handling
- [ ] Support multiline expressions
- **Target**: comments.json 0% → 100%, comparable.json 0% → 90%+

### 14.4 Performance Optimization
- [ ] Profile and optimize hot paths
- [ ] Implement expression caching where appropriate
- [ ] Optimize collection operations
- [ ] Reduce memory allocations

### 14.5 Error Handling Improvements
- [ ] Add better error messages for common mistakes
- [ ] Implement proper error recovery
- [ ] Add position information to errors
- [ ] Improve debugging information

## Expected Outcomes
- precedence.json: 16.7% → 90%+
- literals.json: 48.8% → 80%+
- comments.json: 0% → 100%
- comparable.json: 0% → 90%+
- Improved overall performance and error handling
- Overall test coverage improvement: +2-3%

## Files to Modify
- `/fhirpath-parser/src/parser.rs` - Precedence and literal parsing
- `/fhirpath-parser/src/tokenizer.rs` - Comment and literal tokenization
- `/fhirpath-evaluator/src/engine.rs` - Performance optimizations
- `/fhirpath-core/src/error.rs` - Error handling improvements