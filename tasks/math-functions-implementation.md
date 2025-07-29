# Math Functions Implementation and Fixes Task

## Overview

This task addresses the implementation and fixing of remaining math functions in the fhirpath-rs implementation to improve test coverage from the current 60.1% overall pass rate.

## Current State Analysis

Based on the TEST_COVERAGE.md report generated on 2025-07-29, the following math-related functions need attention:

### Critical Issues (Major Problems - 0-30% pass rate)
- **plus.json** - 23.5% (8/34 tests) - Major arithmetic and date/time issues
- **quantity.json** - 27.3% (3/11 tests) - UCUM unit conversion and quantity operations

### Moderate Issues (Partially Implemented - 30-70% pass rate)
- **sqrt.json** - 66.7% (2/3 tests) - Square root edge cases
- **log.json** - 60.0% (3/5 tests) - Logarithm edge cases
- **divide.json** - 55.6% (5/9 tests) - Division operations
- **minus.json** - 54.5% (6/11 tests) - Subtraction operations
- **power.json** - 50.0% (3/6 tests) - Power operations

### Well Implemented (87%+ pass rate)
- **div.json** - 87.5% (7/8 tests) - Integer division
- **mod.json** - 87.5% (7/8 tests) - Modulo operations

### Fully Working (100% pass rate)
- abs.json, ceiling.json, floor.json, round.json, truncate.json, exp.json, ln.json, multiply.json

## Root Cause Analysis

### 1. AddOperator (Plus Operations) Issues
**Location**: `fhirpath-registry/src/operator.rs` lines 307-538

**Problems Identified**:
- Complex date/time arithmetic with UCUM units
- String concatenation edge cases
- Quantity addition with unit conversion
- Fractional time unit handling (e.g., 7.7 days)
- Time zone preservation in date arithmetic

**Test Cases Failing**:
- Basic arithmetic: `1 + 1 = 2`, `1.2 + 1.8 = 3.0`
- String concatenation: `'a'+'b' = 'ab'`
- Empty collection handling: `'a'+{}`
- Date arithmetic: `@1973-12-25 + 7 days`, `@1973-12-25 + 7.7 days`
- DateTime with timezone: `@1973-12-25T00:00:00.000+10:00 + 7 days`
- Various time units: seconds, milliseconds, minutes, hours, days, months, weeks

### 2. Quantity Operations Issues
**Location**: `fhirpath-model/src/quantity.rs` lines 12-158

**Problems Identified**:
- UCUM unit conversion accuracy
- Quantity comparison operations (=, ~, !=, <, >)
- Unit derivation in multiplication/division
- Dimensionless result handling

**Test Cases Failing**:
- Unit conversions: `4.0 'g' = 4000.0 'mg'`
- Equivalence: `4 'g' ~ 4000 'mg'`
- Time units: `7 days = 1 week`, `7 days = 1 'wk'`
- Comparisons: `6 days < 1 week`, `8 days > 1 week`
- Unit derivation: `2.0 'cm' * 2.0 'm' = 0.040 'm2'`
- Division: `4.0 'g' / 2.0 'm' = 2 'g/m'`
- Dimensionless: `1.0 'm' / 1.0 'm' = 1 '1'`

### 3. Math Function Edge Cases
**Locations**: `fhirpath-registry/src/functions/math.rs`

**Problems Identified**:
- Error handling for invalid inputs (negative sqrt, log of zero/negative)
- Precision handling in decimal operations
- Overflow/underflow detection
- NaN and infinity handling

## Implementation Plan

### Phase 1: Fix AddOperator (Plus Operations) - Priority: Critical
**Estimated Effort**: 3-4 days

#### Tasks:
1. **Enhance date/time arithmetic** (lines 410-537 in operator.rs)
   - Fix fractional time unit handling
   - Improve UCUM time unit classification
   - Ensure timezone preservation
   - Add support for all time units (milliseconds, seconds, minutes, hours, days, weeks, months, years)

2. **Fix basic arithmetic operations**
   - Ensure proper decimal precision handling
   - Fix integer + decimal operations
   - Handle overflow/underflow cases

3. **Improve string concatenation**
   - Handle empty collection cases properly
   - Ensure proper type coercion

4. **Add comprehensive error handling**
   - Invalid operand type combinations
   - Arithmetic overflow detection
   - Proper empty result handling

#### Acceptance Criteria:
- All 34 plus.json tests pass
- No regression in existing functionality
- Proper error messages for invalid operations

### Phase 2: Fix Quantity Operations - Priority: Critical
**Estimated Effort**: 4-5 days

#### Tasks:
1. **Enhance UCUM integration** (quantity.rs lines 44-89)
   - Improve unit parsing and validation
   - Fix unit conversion accuracy
   - Add support for complex unit expressions

2. **Implement proper quantity comparison**
   - Fix equality (=) operations with unit conversion
   - Implement equivalence (~) operations
   - Fix inequality operations (<, >, <=, >=, !=)

3. **Fix quantity arithmetic**
   - Improve multiplication with unit derivation
   - Fix division with unit derivation
   - Handle dimensionless results properly

4. **Add quantity validation**
   - Validate unit compatibility
   - Proper error handling for incompatible operations

#### Acceptance Criteria:
- All 11 quantity.json tests pass
- Accurate UCUM unit conversions
- Proper unit derivation in arithmetic operations

### Phase 3: Fix Math Function Edge Cases - Priority: Medium
**Estimated Effort**: 2-3 days

#### Tasks:
1. **Fix SqrtFunction** (math.rs lines 208-256)
   - Handle negative input validation
   - Improve precision for edge cases
   - Add proper error handling

2. **Fix LogFunction** (math.rs lines 378-444)
   - Handle zero and negative input validation
   - Fix base validation
   - Improve precision handling

3. **Fix PowerFunction** (math.rs lines 446-498)
   - Handle overflow/underflow cases
   - Fix fractional exponent handling
   - Add proper error handling for invalid operations

4. **Fix DivideOperator** (operator.rs lines 726-847)
   - Improve quantity division
   - Fix division by zero handling
   - Handle precision issues

5. **Fix SubtractOperator** (operator.rs lines 540-621)
   - Fix date/time subtraction
   - Improve quantity subtraction
   - Handle edge cases properly

#### Acceptance Criteria:
- sqrt.json: 3/3 tests pass (currently 2/3)
- log.json: 5/5 tests pass (currently 3/5)
- power.json: 6/6 tests pass (currently 3/6)
- divide.json: 9/9 tests pass (currently 5/9)
- minus.json: 11/11 tests pass (currently 6/11)

## Technical Implementation Details

### Key Files to Modify:
1. `fhirpath-registry/src/operator.rs` - Arithmetic operators
2. `fhirpath-model/src/quantity.rs` - Quantity operations
3. `fhirpath-registry/src/functions/math.rs` - Math functions
4. Integration with `octofhir_ucum_core` for unit conversions

### Performance Considerations:
- Use efficient decimal arithmetic with `rust_decimal`
- Cache UCUM unit expressions where possible
- Minimize allocations in hot paths
- Use const generics for compile-time optimizations

### Error Handling Strategy:
- Use `Result<T, E>` for all fallible operations
- Provide descriptive error messages
- Follow FHIRPath specification for error behavior
- Maintain backward compatibility

## Testing Strategy

### Unit Tests:
- Add comprehensive unit tests for each fixed function
- Test edge cases and error conditions
- Performance regression tests

### Integration Tests:
- Run full official FHIRPath test suite
- Verify no regressions in passing tests
- Target 90%+ pass rate for math operations

### Validation:
- Run `./scripts/update-test-coverage.sh` after each phase
- Compare results with baseline
- Document improvements in TEST_COVERAGE.md

## Success Metrics

### Target Improvements:
- **plus.json**: 23.5% → 100% (34/34 tests)
- **quantity.json**: 27.3% → 100% (11/11 tests)
- **sqrt.json**: 66.7% → 100% (3/3 tests)
- **log.json**: 60.0% → 100% (5/5 tests)
- **power.json**: 50.0% → 100% (6/6 tests)
- **divide.json**: 55.6% → 100% (9/9 tests)
- **minus.json**: 54.5% → 100% (11/11 tests)

### Overall Impact:
- Current overall pass rate: 60.1% (604/1005 tests)
- Target overall pass rate: 70%+ (700+/1005 tests)
- Math operations should achieve 95%+ pass rate

## Risk Assessment

### High Risk:
- UCUM unit conversion complexity
- Date/time arithmetic edge cases
- Backward compatibility with existing code

### Medium Risk:
- Performance impact of enhanced error handling
- Complex quantity operations
- Precision handling in decimal arithmetic

### Mitigation Strategies:
- Incremental implementation with continuous testing
- Comprehensive unit test coverage
- Performance benchmarking
- Code review for each phase

## Dependencies

### External Libraries:
- `octofhir_ucum_core` - UCUM unit conversions
- `rust_decimal` - Decimal arithmetic
- `chrono` - Date/time operations

### Internal Dependencies:
- `fhirpath-model` - Value types and structures
- `fhirpath-registry` - Function and operator registry
- `fhirpath-evaluator` - Expression evaluation engine

## Timeline

- **Phase 1** (Plus Operations): Days 1-4
- **Phase 2** (Quantity Operations): Days 5-9
- **Phase 3** (Math Function Edge Cases): Days 10-12
- **Testing and Validation**: Days 13-14
- **Documentation and Cleanup**: Day 15

**Total Estimated Duration**: 15 working days

## Status

- [ ] Phase 1: Fix AddOperator (Plus Operations)
- [ ] Phase 2: Fix Quantity Operations  
- [ ] Phase 3: Fix Math Function Edge Cases
- [ ] Testing and Validation
- [ ] Documentation Update

**Current Status**: Planning Complete - Ready for Implementation
**Last Updated**: 2025-07-29
**Assigned To**: TBD
