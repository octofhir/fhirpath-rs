# Phase 1 Task 01: Comparison Operators Completion

**Task ID**: phase1-01  
**Priority**: HIGH  
**Status**: 🟡 IN PROGRESS  
**Estimated Time**: 5-6 days  
**Dependencies**: None

## Overview

Complete the implementation of comparison operators that are currently only 40% passing. This affects 4 major test suites with 120 total tests, making it one of the highest impact tasks for improving test coverage.

## Current Status

| Test Suite | Current Pass Rate | Tests |
|------------|------------------|-------|
| equality.json | 57.1% (16/28) | 28 |
| greater-than.json | 40.0% (12/30) | 30 |
| greator-or-equal.json | 40.0% (12/30) | 30 |
| less-or-equal.json | 40.0% (12/30) | 30 |
| less-than.json | 40.0% (12/30) | 30 |
| not-equivalent.json | 63.6% (14/22) | 22 |
| equivalent.json | 54.2% (13/24) | 24 |
| n-equality.json | 50.0% (12/24) | 24 |

**Total Impact**: 218 tests, currently ~50% passing

## Progress Update (2025-07-27)

### ✅ Completed
- Fixed compilation errors in fhirpath-registry crate
- Added missing `compare_quantities_equal` method for quantity comparisons
- Fixed empty collection handling logic in EqualOperator
- Added `compare_values_equal` method to avoid recursive comparison issues

### 🔄 Current Issues
- **testEquality25** (`name = name`): Still returns `[false]` instead of `[true]` - collection self-comparison issue
- **testEquality26** (`name.take(2) = name.take(2).first() | name.take(2).last()`): Collection vs union comparison fails
- **testEquality28** (`Observation.value = 185 '[lb_av]'`): Now returns `[false]` instead of `[]` (progress), but should return `[true]`

### 🎯 Next Steps
- Debug collection comparison logic further
- Fix quantity comparison with unit handling
- Implement proper FHIRPath collection equality semantics

## Problem Analysis

Based on test coverage, the main issues appear to be:
1. **Type coercion** - Comparisons between different types (string vs number, etc.)
2. **Null/empty handling** - Comparisons involving null or empty collections
3. **Quantity comparisons** - Comparisons involving FHIR quantities with units
4. **Date/time comparisons** - Temporal value comparisons
5. **Collection comparisons** - Comparing collections vs single values

## Implementation Tasks

### 1. Equality Operations (`=`, `!=`)
- [ ] Fix type coercion rules for equality
- [ ] Handle null/empty collection equality
- [ ] Implement quantity equality with unit conversion
- [ ] Fix date/time equality comparisons
- [ ] Add collection vs single value equality

### 2. Ordering Operations (`>`, `>=`, `<`, `<=`)
- [ ] Implement proper type ordering rules
- [ ] Handle null/empty collection ordering
- [ ] Add quantity ordering with unit conversion
- [ ] Implement date/time ordering
- [ ] Fix string vs number ordering edge cases

### 3. Equivalence Operations (`~`, `!~`)
- [ ] Distinguish equivalence from equality
- [ ] Handle case-insensitive string equivalence
- [ ] Implement quantity equivalence
- [ ] Add collection equivalence logic

## Acceptance Criteria

### Functional Requirements
- [ ] All equality tests pass (28/28 in equality.json)
- [ ] All greater-than tests pass (30/30 in greater-than.json)
- [ ] All greater-or-equal tests pass (30/30 in greator-or-equal.json)
- [ ] All less-than tests pass (30/30 in less-than.json)
- [ ] All less-or-equal tests pass (30/30 in less-or-equal.json)
- [ ] All not-equivalent tests pass (22/22 in not-equivalent.json)
- [ ] All equivalent tests pass (24/24 in equivalent.json)
- [ ] All n-equality tests pass (24/24 in n-equality.json)

### Technical Requirements
- [ ] Follow FHIRPath specification for comparison semantics
- [ ] Maintain performance for common comparison operations
- [ ] Add comprehensive error handling
- [ ] Include proper type checking and coercion

### Quality Requirements
- [ ] Add unit tests for edge cases
- [ ] Update documentation for comparison operators
- [ ] Follow Rust performance guidelines
- [ ] Ensure memory safety and efficiency

## Implementation Strategy

### Phase 1: Core Equality (Days 1-2)
1. Analyze failing equality tests
2. Fix type coercion logic in evaluator
3. Handle null/empty cases properly
4. Test against equality.json suite

### Phase 2: Ordering Operations (Days 3-4)
1. Implement ordering for all supported types
2. Add proper null/empty handling for ordering
3. Fix edge cases in comparison logic
4. Test against all ordering test suites

### Phase 3: Equivalence Logic (Days 5-6)
1. Implement equivalence vs equality distinction
2. Add case-insensitive string handling
3. Handle collection equivalence
4. Final testing and optimization

## Files to Modify

### Core Implementation
- `fhirpath-evaluator/src/engine.rs` - Main evaluation logic
- `fhirpath-evaluator/src/operators/` - Comparison operator implementations
- `fhirpath-model/src/value.rs` - Value comparison methods

### Testing
- Add specific test cases for edge cases
- Update integration tests
- Add performance benchmarks

## Testing Strategy

### Unit Tests
- Test each comparison operator individually
- Test type coercion edge cases
- Test null/empty handling
- Test quantity comparisons

### Integration Tests
- Run full test suites after each phase
- Verify no regressions in other areas
- Test performance impact

### Validation
- Run `./scripts/update-test-coverage.sh` after completion
- Verify coverage increase from ~32% to ~40%+
- Ensure no new test failures

## Success Metrics

- **Primary**: Increase overall test coverage by ~8-10%
- **Secondary**: All 218 comparison tests passing
- **Performance**: No significant performance regression
- **Quality**: Clean, maintainable code following project guidelines

## Risks and Mitigation

### High Risk
- **Complex type coercion**: Start with simple cases, add complexity gradually
- **Performance impact**: Profile before/after, optimize hot paths

### Medium Risk
- **Quantity handling**: May need ucum-rs library integration
- **Date/time logic**: May need additional temporal handling

### Low Risk
- **String comparisons**: Well-understood, straightforward implementation

## Next Steps After Completion

1. Update task status to 🟢 COMPLETED
2. Run test coverage report
3. Update phase progress in task index
4. Begin phase1-02 (String Functions Completion)

---

*Created: 2025-07-27*  
*Last Updated: 2025-07-27*
