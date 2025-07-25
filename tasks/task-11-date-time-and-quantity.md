# Task 11: Date/Time and Quantity Functions

## Overview
Implement date/time operations and quantity handling for complete FHIRPath temporal and measurement support.

## Current Issues from TEST_COVERAGE.md
- **now.json** - 0.0% (0/2 tests) - Missing
- **today.json** - 50.0% (1/2 tests) - Issues
- **quantity.json** - 9.1% (1/11 tests) - Issues
- **high-boundary.json** - 0.0% (0/24 tests) - Missing
- **low-boundary.json** - 0.0% (0/28 tests) - Missing
- **precision.json** - 0.0% (0/6 tests) - Missing

## Subtasks

### 11.1 Complete DateTime Functions
- [ ] Verify now() function is working (should be implemented)
- [ ] Debug and fix today() function (currently 50%)
- [ ] Ensure proper timezone handling
- [ ] Add date arithmetic support
- **Target**: now.json 0% → 100%, today.json 50% → 100%

### 11.2 Implement Quantity Operations
- [ ] Fix quantity parsing and conversion
- [ ] Implement quantity arithmetic operations
- [ ] Add unit conversion support (via ucum-rs)
- [ ] Handle quantity comparisons
- **Target**: quantity.json 9.1% → 70%+

### 11.3 Implement Boundary Functions
- [ ] Implement high boundary operations for dates/quantities
- [ ] Implement low boundary operations for dates/quantities
- [ ] Handle precision boundaries properly
- [ ] Support range operations
- **Target**: high-boundary.json 0% → 60%+, low-boundary.json 0% → 60%+

### 11.4 Implement Precision Functions
- [ ] Add precision handling for dates/times
- [ ] Implement precision operations for quantities
- [ ] Support precision arithmetic
- [ ] Handle precision comparisons
- **Target**: precision.json 0% → 70%+

## Expected Outcomes
- now.json: 0% → 100%
- today.json: 50% → 100%
- quantity.json: 9.1% → 70%+
- high-boundary.json: 0% → 60%+
- low-boundary.json: 0% → 60%+
- precision.json: 0% → 70%+
- Overall test coverage improvement: +2-3%

## Files to Modify
- `/fhirpath-registry/src/function.rs` - DateTime and quantity functions
- `/fhirpath-model/src/quantity.rs` - Quantity operations
- `/fhirpath-model/src/value.rs` - DateTime value handling
- Integration with `/ucum-rs` library for unit conversions