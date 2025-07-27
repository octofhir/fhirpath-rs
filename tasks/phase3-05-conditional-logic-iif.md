# Phase 3 Task 05: Conditional Logic (iif)

**Task ID**: phase3-05  
**Priority**: MEDIUM  
**Status**: 🔴 TODO  
**Estimated Time**: 2-3 days  
**Dependencies**: phase1-06 (Boolean Logic Edge Cases)  

## Overview

Implement the conditional logic function `iif()` (if-then-else) that currently has major issues (18.2% pass rate). This function is essential for conditional expressions and decision-making logic in FHIRPath expressions.

## Current Status

| Test Suite | Current Pass Rate | Tests | Category |
|------------|------------------|-------|----------|
| iif.json | 18.2% (2/11) | 11 | Major Issues |

**Total Impact**: 11 tests, currently 18.2% passing  
**Expected Coverage Increase**: ~0.9% of total test suite  
**Strategic Importance**: Essential for conditional expressions and decision logic

## Problem Analysis

Based on the low pass rate, the main issues appear to be:
1. **Conditional evaluation** - Proper if-then-else logic implementation
2. **Boolean condition handling** - Evaluating conditions correctly
3. **Lazy evaluation** - Only evaluating the chosen branch
4. **Type handling** - Proper handling of different return types
5. **Null/empty handling** - Edge cases with null conditions and values

## Implementation Tasks

### 1. Core iif Function Implementation (Days 1-2)
- [ ] Implement `iif(condition, true_result, false_result)` function
- [ ] Add proper boolean condition evaluation
- [ ] Implement lazy evaluation (only evaluate chosen branch)
- [ ] Handle null and empty collection conditions
- [ ] Add proper type handling for return values

### 2. Advanced Conditional Logic (Days 2-3)
- [ ] Handle complex boolean expressions as conditions
- [ ] Add support for collection-based conditions
- [ ] Implement proper short-circuit evaluation
- [ ] Handle edge cases with mixed return types
- [ ] Add comprehensive error handling

### 3. Integration and Testing (Days 2-3)
- [ ] Integrate with boolean logic system
- [ ] Add comprehensive error handling
- [ ] Test with various condition types
- [ ] Optimize for performance
- [ ] Final testing and validation

## Acceptance Criteria

### Functional Requirements
- [ ] All iif tests pass (11/11)
- [ ] Conditional logic works correctly
- [ ] Lazy evaluation implemented properly
- [ ] Boolean conditions evaluated correctly
- [ ] Mixed return types handled properly

### Technical Requirements
- [ ] Follow FHIRPath specification for iif semantics
- [ ] Implement proper lazy evaluation
- [ ] Add comprehensive error handling
- [ ] Support all FHIRPath data types
- [ ] Handle null/empty conditions correctly

### Quality Requirements
- [ ] Add comprehensive unit tests
- [ ] Update documentation for conditional logic
- [ ] Follow Rust conditional handling best practices
- [ ] Ensure memory efficiency

## Implementation Strategy

### Phase 1: Core Implementation (Days 1-2)
1. Analyze failing iif tests
2. Implement basic iif function structure
3. Add boolean condition evaluation
4. Implement lazy evaluation logic
5. Test against basic iif scenarios

### Phase 2: Advanced Features (Days 2-3)
1. Handle complex conditions
2. Add collection-based condition support
3. Implement proper error handling
4. Test against full iif test suite

### Phase 3: Integration (Days 2-3)
1. Integrate with boolean logic system
2. Add comprehensive error handling
3. Optimize for performance
4. Final testing and validation

## Files to Modify

### Core Implementation
- `fhirpath-evaluator/src/engine.rs` - Conditional function evaluation
- `fhirpath-evaluator/src/functions/conditional.rs` - New conditional functions module
- `fhirpath-model/src/value.rs` - Conditional value operations

### Testing
- Add comprehensive conditional logic tests
- Update integration tests
- Add lazy evaluation tests

## Testing Strategy

### Unit Tests
- Test iif function with various conditions
- Test lazy evaluation behavior
- Test with different return types
- Test null/empty condition handling
- Test error cases

### Integration Tests
- Run iif test suite continuously
- Test complex conditional expressions
- Verify no regressions in other areas

### Validation
- Run `./scripts/update-test-coverage.sh` after completion
- Verify coverage increase from current level
- Test with various conditional scenarios

## Success Metrics

- **Primary**: Increase overall test coverage by ~0.9%
- **Secondary**: All 11 iif tests passing
- **Performance**: Efficient conditional evaluation with lazy evaluation
- **Quality**: Robust conditional logic with proper error handling

## Technical Considerations

### Lazy Evaluation Implementation
- Only evaluate the condition first
- Evaluate true_result only if condition is true
- Evaluate false_result only if condition is false
- Avoid unnecessary computation

### Boolean Condition Handling
- Handle boolean values correctly
- Convert collections to boolean (empty = false, non-empty = true)
- Handle null conditions appropriately
- Support complex boolean expressions

### Type System Integration
- Support mixed return types from branches
- Proper type coercion if needed
- Handle collection vs single value returns
- Integration with existing type system

## Risks and Mitigation

### High Risk
- **Lazy evaluation complexity**: Design carefully, test thoroughly
- **Type handling**: Follow FHIRPath specification strictly

### Medium Risk
- **Performance impact**: Profile conditional operations
- **Boolean conversion**: Test with various condition types

### Low Risk
- **Basic conditional logic**: Well-understood programming concept

## Dependencies

### Blocking Dependencies
- **phase1-06**: Boolean Logic Edge Cases must be complete
- **Boolean evaluation**: Requires stable boolean logic system

### Enables Future Tasks
- **Complex expressions**: Conditional logic enables sophisticated expressions
- **Decision trees**: Foundation for complex decision-making logic
- **Data validation**: Conditional logic for validation rules

## Next Steps After Completion

1. Update task status to 🟢 COMPLETED
2. Run comprehensive test coverage report
3. Update phase progress in task index
4. Begin phase3-06 (Sort and Ordering)
5. Validate conditional logic with complex expressions

---

*Created: 2025-07-27*  
*Last Updated: 2025-07-27*
