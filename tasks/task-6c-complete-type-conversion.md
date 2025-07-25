# Task 6c: Complete Type Conversion Functions

## Overview
Complete the remaining type conversion functions to achieve full FHIRPath specification compliance.

## Current Status from TEST_COVERAGE.md
- **to-decimal.json** - 40.0% (2/5 tests) - Issues
- **types.json** - 37.4% (37/99 tests) - Issues  
- **type.json** - 3.3% (1/30 tests) - Issues

## Subtasks

### 6c.1 Fix toDecimal() Function
- [ ] Apply same Collection handling pattern as toString/toInteger
- [ ] Handle string to decimal conversion properly
- [ ] Support integer to decimal conversion
- [ ] Handle empty collections and edge cases
- **Target**: 40% → 80%+

### 6c.2 Implement Type Checking Functions
- [ ] Implement `is` operator for type checking
- [ ] Add support for type names (String, Integer, Decimal, Boolean, etc.)
- [ ] Fix type() function to return proper type information
- [ ] Handle collection type checking
- **Target**: types.json 37.4% → 60%+

### 6c.3 Fix Type System Issues
- [ ] Review TypeInfo enum coverage for all FHIRPath types
- [ ] Fix type coercion in operators and functions
- [ ] Ensure consistent type handling across codebase
- [ ] Add missing type validation
- **Target**: type.json 3.3% → 30%+

## Expected Outcomes
- to-decimal.json: 40% → 80%+ 
- types.json: 37.4% → 60%+
- type.json: 3.3% → 30%+
- Overall test coverage improvement: +1-2%

## Files to Modify
- `/fhirpath-registry/src/function.rs` - ToDecimalFunction
- `/fhirpath-model/src/types.rs` - Type definitions
- `/fhirpath-parser/src/parser.rs` - Type parsing
- `/fhirpath-registry/src/operator.rs` - Type operators