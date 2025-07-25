# Task 6: Fix Type System and Conversion Functions

## Overview
Implement and fix type system functions and conversion functions that are failing in the FHIRPath test suite.

## Current Issues from TEST_COVERAGE.md
- **to-string.json** - 0.0% (0/5 tests) - Missing
- **to-integer.json** - 40.0% (2/5 tests) - Issues  
- **to-decimal.json** - 20.0% (1/5 tests) - Issues
- **types.json** - 29.3% (29/99 tests) - Issues
- **type.json** - 3.3% (1/30 tests) - Issues

## Subtasks

### 6.1 Fix toString() Function
- [ ] Investigate why toString() tests are failing
- [ ] Fix toString() to handle all FHIRPath types properly
- [ ] Handle Collection wrapping correctly

### 6.2 Fix toInteger() Function  
- [ ] Analyze failing toInteger() tests
- [ ] Fix string to integer conversion
- [ ] Handle edge cases (empty strings, non-numeric strings)

### 6.3 Fix toDecimal() Function
- [ ] Analyze failing toDecimal() tests
- [ ] Fix string to decimal conversion
- [ ] Handle precision and rounding correctly

### 6.4 Implement Type Checking Functions
- [ ] Implement `is` operator for type checking
- [ ] Add support for type names (String, Integer, Decimal, Boolean, etc.)
- [ ] Fix type() function to return proper type information

### 6.5 Fix Type System Issues
- [ ] Review TypeInfo enum and ensure all FHIRPath types are covered
- [ ] Fix type coercion in operators and functions
- [ ] Ensure consistent type handling across the codebase

## Expected Outcomes
- toString() tests: 0% → 80%+
- toInteger() tests: 40% → 80%+ 
- toDecimal() tests: 20% → 80%+
- types.json tests: 29.3% → 60%+
- type.json tests: 3.3% → 50%+
- Overall test coverage improvement: +2-4%

## Related Files
- `/fhirpath-registry/src/function.rs` - Conversion functions
- `/fhirpath-model/src/types.rs` - Type definitions
- `/fhirpath-model/src/value.rs` - Value conversions
- `/fhirpath-parser/src/parser.rs` - Type parsing