# Phase 2 Task 03: Collection Indexing

**Task ID**: phase2-03  
**Priority**: HIGH  
**Status**: 🔴 TODO  
**Estimated Time**: 3-4 days  
**Dependencies**: phase1-05 (Collection Operations Polish)  

## Overview

Implement collection indexing functionality that is currently 0% implemented. This includes the indexer operator `[]` and related collection access patterns that are fundamental to FHIRPath collection manipulation.

## Current Status

| Test Suite | Current Pass Rate | Tests | Category |
|------------|------------------|-------|----------|
| indexer.json | 0.0% (0/2) | 2 | Missing |
| index-part.json | 0.0% (0/1) | 1 | Missing |

**Total Impact**: 3 tests, currently 0% passing  
**Expected Coverage Increase**: ~0.3% of total test suite  
**Strategic Importance**: Foundation for collection access and manipulation

## Problem Analysis

Collection indexing requires implementing:
1. **Indexer operator syntax** - `collection[index]` syntax
2. **Numeric indexing** - Access by integer index (0-based)
3. **Range indexing** - Access by index ranges
4. **Negative indexing** - Access from end of collection
5. **Error handling** - Out-of-bounds and invalid index handling

## Implementation Tasks

### 1. Parser Integration (Days 1-2)
- [ ] Add indexer operator syntax to parser
- [ ] Implement `collection[index]` parsing
- [ ] Add support for numeric and range indices
- [ ] Handle negative index syntax
- [ ] Add proper error handling for syntax errors

### 2. Indexer Evaluation (Days 2-3)
- [ ] Implement indexer operator evaluation
- [ ] Add numeric index access (0-based)
- [ ] Implement negative index access (from end)
- [ ] Add range index support if required
- [ ] Handle out-of-bounds access gracefully

### 3. Error Handling and Edge Cases (Days 3-4)
- [ ] Add comprehensive bounds checking
- [ ] Handle empty collection indexing
- [ ] Implement proper error messages
- [ ] Add null/invalid index handling
- [ ] Final testing and optimization

## Acceptance Criteria

### Functional Requirements
- [ ] All indexer tests pass (2/2)
- [ ] All index-part tests pass (1/1)
- [ ] Numeric indexing works correctly (0-based)
- [ ] Negative indexing works (from end)
- [ ] Out-of-bounds access handled properly

### Technical Requirements
- [ ] Follow FHIRPath specification for indexing semantics
- [ ] Maintain performance for indexing operations
- [ ] Add comprehensive error handling
- [ ] Support all collection types
- [ ] Handle edge cases gracefully

### Quality Requirements
- [ ] Add comprehensive unit tests
- [ ] Update documentation for indexing
- [ ] Follow Rust indexing best practices
- [ ] Ensure memory safety

## Implementation Strategy

### Phase 1: Parser Integration (Days 1-2)
1. Analyze indexer test requirements
2. Add indexer syntax to parser
3. Implement index expression parsing
4. Add basic validation and error handling

### Phase 2: Indexer Evaluation (Days 2-3)
1. Implement indexer operator in evaluator
2. Add numeric and negative index support
3. Handle different collection types
4. Test against indexer test suite

### Phase 3: Error Handling (Days 3-4)
1. Add comprehensive bounds checking
2. Implement graceful error handling
3. Add detailed error messages
4. Final testing and optimization

## Files to Modify

### Core Implementation
- `fhirpath-parser/src/parser.rs` - Indexer syntax parsing
- `fhirpath-parser/src/ast.rs` - AST nodes for indexer
- `fhirpath-evaluator/src/engine.rs` - Indexer evaluation
- `fhirpath-evaluator/src/operators/indexer.rs` - New indexer operator module

### Testing
- Add comprehensive indexer tests
- Update integration tests
- Add performance benchmarks

## Testing Strategy

### Unit Tests
- Test indexer parsing
- Test numeric indexing (positive/negative)
- Test bounds checking
- Test error handling
- Test with different collection types

### Integration Tests
- Run indexer test suites continuously
- Test complex expressions with indexing
- Verify no regressions in other areas

### Validation
- Run `./scripts/update-test-coverage.sh` after completion
- Verify coverage increase from current level
- Ensure indexing works in complex scenarios

## Success Metrics

- **Primary**: Increase overall test coverage by ~0.3%
- **Secondary**: All 3 indexing tests passing
- **Performance**: Fast indexing operations
- **Quality**: Clean, maintainable indexing code

## Technical Considerations

### Indexing Semantics
- 0-based indexing following FHIRPath specification
- Negative indices count from end (-1 = last element)
- Out-of-bounds access returns empty collection
- Support for all collection types

### Parser Integration
- Extend existing parser with indexer syntax
- Handle precedence correctly with other operators
- Proper error messages for syntax errors
- Integration with existing AST structure

### Performance Considerations
- Efficient indexing for different collection types
- Avoid unnecessary collection copying
- Optimize for common indexing patterns

## Risks and Mitigation

### High Risk
- **Parser precedence**: Ensure indexer precedence is correct
- **Performance impact**: Profile indexing operations

### Medium Risk
- **Edge case handling**: Test thoroughly with various inputs
- **Collection type support**: Ensure all collection types work

### Low Risk
- **Basic indexing**: Well-understood problem domain

## Dependencies

### Blocking Dependencies
- **phase1-05**: Collection Operations Polish must be complete
- **Parser foundation**: Requires stable parser infrastructure

### Enables Future Tasks
- **Advanced collection operations**: Indexing enables complex collection access
- **Data navigation**: Foundation for navigating complex data structures
- **Query optimization**: Efficient data access patterns

## Next Steps After Completion

1. Update task status to 🟢 COMPLETED
2. Run comprehensive test coverage report
3. Update phase progress in task index
4. Begin phase2-04 (Date/Time Functions)
5. Validate indexing works with other collection operations

---

*Created: 2025-07-27*  
*Last Updated: 2025-07-27*
