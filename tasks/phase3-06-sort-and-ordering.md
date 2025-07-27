# Phase 3 Task 06: Sort and Ordering

**Task ID**: phase3-06  
**Priority**: MEDIUM  
**Status**: 🔴 TODO  
**Estimated Time**: 2-3 days  
**Dependencies**: phase1-05 (Collection Operations Polish)  

## Overview

Implement sort and ordering functions that currently have major issues (10% pass rate). This includes collection sorting capabilities that are essential for data organization and presentation in FHIRPath expressions.

## Current Status

| Test Suite | Current Pass Rate | Tests | Category |
|------------|------------------|-------|----------|
| sort.json | 10.0% (1/10) | 10 | Major Issues |

**Total Impact**: 10 tests, currently 10% passing  
**Expected Coverage Increase**: ~0.9% of total test suite  
**Strategic Importance**: Essential for data organization and presentation

## Problem Analysis

Based on the low pass rate, the main issues appear to be:
1. **Collection sorting** - Basic sort functionality implementation
2. **Custom sort criteria** - Sorting by specific fields or expressions
3. **Type-aware sorting** - Proper sorting of different data types
4. **Null/empty handling** - Handling null values in sorting
5. **Stable sorting** - Maintaining relative order for equal elements

## Implementation Tasks

### 1. Basic Sort Implementation (Days 1-2)
- [ ] Implement basic `sort()` function for collections
- [ ] Add support for natural ordering of primitive types
- [ ] Handle string, numeric, and date/time sorting
- [ ] Implement proper null value handling in sorts
- [ ] Add stable sorting algorithm

### 2. Advanced Sort Features (Days 2-3)
- [ ] Implement sort with custom criteria/expressions
- [ ] Add support for multi-field sorting
- [ ] Handle complex object sorting
- [ ] Add ascending/descending sort options
- [ ] Implement case-insensitive string sorting

### 3. Integration and Testing (Days 2-3)
- [ ] Integrate with collection operations
- [ ] Add comprehensive error handling
- [ ] Optimize sorting performance
- [ ] Handle edge cases and empty collections
- [ ] Final testing and validation

## Acceptance Criteria

### Functional Requirements
- [ ] All sort tests pass (10/10)
- [ ] Basic collection sorting works correctly
- [ ] Custom sort criteria supported
- [ ] Multi-type sorting handled properly
- [ ] Null values sorted appropriately

### Technical Requirements
- [ ] Follow FHIRPath specification for sorting semantics
- [ ] Implement stable sorting algorithm
- [ ] Add comprehensive error handling
- [ ] Support all comparable FHIRPath data types
- [ ] Handle large collections efficiently

### Quality Requirements
- [ ] Add comprehensive unit tests
- [ ] Update documentation for sorting functions
- [ ] Follow Rust sorting best practices
- [ ] Ensure memory efficiency

## Implementation Strategy

### Phase 1: Basic Implementation (Days 1-2)
1. Analyze failing sort tests
2. Implement basic sort function
3. Add natural ordering for primitive types
4. Handle null values in sorting
5. Test against basic sort scenarios

### Phase 2: Advanced Features (Days 2-3)
1. Add custom sort criteria support
2. Implement multi-field sorting
3. Handle complex object sorting
4. Test against full sort test suite

### Phase 3: Integration (Days 2-3)
1. Integrate with collection operations
2. Add comprehensive error handling
3. Optimize for performance
4. Final testing and validation

## Files to Modify

### Core Implementation
- `fhirpath-evaluator/src/engine.rs` - Sort function evaluation
- `fhirpath-evaluator/src/functions/sort.rs` - New sorting functions module
- `fhirpath-model/src/value.rs` - Value comparison and ordering

### Testing
- Add comprehensive sorting tests
- Update integration tests
- Add performance tests for large collections

## Testing Strategy

### Unit Tests
- Test sorting with various data types
- Test custom sort criteria
- Test null value handling
- Test empty collection sorting
- Test performance with large collections

### Integration Tests
- Run sort test suite continuously
- Test complex sorting expressions
- Verify no regressions in other areas

### Validation
- Run `./scripts/update-test-coverage.sh` after completion
- Verify coverage increase from current level
- Test with various collection scenarios

## Success Metrics

- **Primary**: Increase overall test coverage by ~0.9%
- **Secondary**: All 10 sort tests passing
- **Performance**: Efficient sorting for large collections
- **Quality**: Stable and reliable sorting implementation

## Technical Considerations

### Sorting Algorithm
- Use stable sorting algorithm (maintains relative order)
- Efficient for large collections (O(n log n))
- Handle various data types correctly
- Proper null value handling

### Type Comparison
- Implement proper ordering for all FHIRPath types
- Handle mixed type collections appropriately
- Support case-insensitive string comparison
- Date/time ordering with timezone considerations

### Performance Optimization
- Use efficient sorting algorithms
- Avoid unnecessary allocations
- Consider parallel sorting for very large collections
- Profile sorting operations

## Risks and Mitigation

### High Risk
- **Performance with large collections**: Use efficient algorithms, profile operations
- **Type comparison complexity**: Follow FHIRPath specification strictly

### Medium Risk
- **Memory usage**: Optimize sorting implementation
- **Null handling**: Test thoroughly with various null scenarios

### Low Risk
- **Basic sorting**: Well-understood algorithmic problem

## Dependencies

### Blocking Dependencies
- **phase1-05**: Collection Operations Polish must be complete
- **Comparison operations**: Requires stable value comparison

### Enables Future Tasks
- **Data presentation**: Sorted data for user interfaces
- **Report generation**: Organized data for reports
- **Advanced queries**: Sorted results for complex queries

## Next Steps After Completion

1. Update task status to 🟢 COMPLETED
2. Run comprehensive test coverage report
3. Update phase progress in task index
4. Complete Phase 3 (all 6 tasks done)
5. Begin Phase 4 with phase4-01 (Collection Contains Polish)

---

*Created: 2025-07-27*  
*Last Updated: 2025-07-27*
