# Phase 3 Task 01: Literals Parsing Fix

**Task ID**: phase3-01  
**Priority**: MEDIUM  
**Status**: 🔴 TODO  
**Estimated Time**: 4-5 days  
**Dependencies**: phase1-04 (Type Conversion Functions)  

## Overview

Fix the literals parsing implementation which currently has a 22% pass rate (18/82 tests). This represents the largest single test suite with major issues and will significantly improve test coverage when resolved.

## Current Status

| Test Suite | Current Pass Rate | Tests | Impact |
|------------|------------------|-------|---------|
| literals.json | 22.0% (18/82) | 82 | All literal value parsing |

**Total Impact**: 82 tests, currently 22% passing  
**Expected Coverage Increase**: ~6-7% of total test suite

## Problem Analysis

Based on the low pass rate, the main issues appear to be:
1. **String literal parsing** - Escape sequences, quotes, unicode
2. **Numeric literal parsing** - Integers, decimals, scientific notation
3. **Boolean literal parsing** - true/false values
4. **Date/time literal parsing** - ISO 8601 formats
5. **Quantity literal parsing** - Numbers with units
6. **Collection literal parsing** - Arrays and sets
7. **Null/empty literal handling** - {} and null values

## FHIRPath Literal Types

### String Literals
- Single quotes: `'hello world'`
- Escape sequences: `'can\'t'`, `'line\nbreak'`
- Unicode: `'unicode \u0041'`

### Numeric Literals
- Integers: `42`, `-17`
- Decimals: `3.14`, `-0.5`
- Scientific notation: `1.5e-3`

### Boolean Literals
- `true`, `false`

### Date/Time Literals
- Date: `@2023-12-25`
- DateTime: `@2023-12-25T10:30:00`
- Time: `@T10:30:00`

### Quantity Literals
- Simple: `5 'mg'`
- Complex: `10.5 'kg/m2'`

### Collection Literals
- Empty: `{}`
- Single: `{42}`
- Multiple: `{1, 2, 3}`

## Implementation Tasks

### 1. String Literal Improvements (Days 1-2)
- [ ] Fix escape sequence parsing (`\'`, `\"`, `\\`, `\n`, `\r`, `\t`)
- [ ] Add unicode escape support (`\uXXXX`)
- [ ] Handle nested quotes correctly
- [ ] Fix empty string parsing
- [ ] Add proper error messages for malformed strings

### 2. Numeric Literal Fixes (Day 2)
- [ ] Fix integer parsing edge cases (leading zeros, signs)
- [ ] Improve decimal parsing (trailing zeros, precision)
- [ ] Add scientific notation support
- [ ] Handle numeric overflow/underflow
- [ ] Fix negative number parsing

### 3. Date/Time Literal Parsing (Day 3)
- [ ] Implement ISO 8601 date parsing (`@2023-12-25`)
- [ ] Add datetime parsing (`@2023-12-25T10:30:00`)
- [ ] Support time-only parsing (`@T10:30:00`)
- [ ] Handle timezone information
- [ ] Add date/time validation

### 4. Quantity and Collection Literals (Days 4-5)
- [ ] Fix quantity literal parsing (`5 'mg'`)
- [ ] Handle complex units (`'kg/m2'`)
- [ ] Improve collection literal parsing (`{1, 2, 3}`)
- [ ] Fix empty collection handling (`{}`)
- [ ] Add nested collection support
- [ ] Final testing and optimization

## Acceptance Criteria

### Functional Requirements
- [ ] All literals.json tests pass (82/82)
- [ ] String literals with all escape sequences work
- [ ] Numeric literals in all formats parse correctly
- [ ] Date/time literals follow ISO 8601 standard
- [ ] Quantity literals integrate with unit system
- [ ] Collection literals support nesting

### Technical Requirements
- [ ] Follow FHIRPath specification for literal syntax
- [ ] Maintain parsing performance
- [ ] Add comprehensive error messages
- [ ] Handle malformed input gracefully
- [ ] Support all required literal types

### Quality Requirements
- [ ] Add unit tests for each literal type
- [ ] Update parser documentation
- [ ] Follow nom parser combinator patterns
- [ ] Ensure memory efficiency

## Implementation Strategy

### Phase 1: String and Numeric Fixes (Days 1-2)
1. Analyze failing string literal tests
2. Fix escape sequence parsing in nom parser
3. Improve numeric literal parsing
4. Test against string and numeric literal subsets

### Phase 2: Date/Time Literals (Day 3)
1. Implement ISO 8601 date parsing
2. Add datetime and time parsing
3. Handle timezone and validation
4. Test against date/time literal tests

### Phase 3: Complex Literals (Days 4-5)
1. Fix quantity literal parsing
2. Improve collection literal handling
3. Add nested collection support
4. Final testing and optimization

## Files to Modify

### Core Implementation
- `fhirpath-parser/src/parser.rs` - Main parser logic
- `fhirpath-parser/src/literals.rs` - Literal parsing functions
- `fhirpath-model/src/value.rs` - Value construction from literals

### Testing
- Add comprehensive literal parsing tests
- Update parser test framework
- Add edge case tests

## Testing Strategy

### Unit Tests
- Test each literal type individually
- Test escape sequences and edge cases
- Test malformed input handling
- Test parser performance

### Integration Tests
- Run literals.json test suite continuously
- Verify no regressions in other parsing
- Test complex expressions with literals

### Validation
- Run `./scripts/update-test-coverage.sh` after each phase
- Verify coverage increase from current level
- Ensure parsing performance maintained

## Success Metrics

- **Primary**: Increase overall test coverage by ~6-7%
- **Secondary**: All 82 literal tests passing
- **Performance**: No parsing performance regression
- **Quality**: Clean, maintainable parser code

## Technical Considerations

### Parser Structure (nom combinators)
```rust
// String literal with escape sequences
fn string_literal(input: &str) -> IResult<&str, Value> {
    delimited(
        char('\''),
        escaped_transform(
            none_of("\\\'"),
            '\\',
            alt((
                value("\\", char('\\')),
                value("\'", char('\'')),
                value("\n", char('n')),
                value("\r", char('r')),
                value("\t", char('t')),
                unicode_escape,
            ))
        ),
        char('\'')
    )(input)
}
```

### Date/Time Parsing
- Use chrono crate for ISO 8601 parsing
- Handle partial dates and times
- Validate date/time values

### Quantity Integration
- Integrate with ucum-rs for unit parsing
- Handle unit validation
- Support complex unit expressions

## Risks and Mitigation

### High Risk
- **Complex escape sequences**: Test thoroughly with edge cases
- **Date/time parsing**: Use proven chrono library patterns

### Medium Risk
- **Quantity unit parsing**: May need ucum-rs integration improvements
- **Performance impact**: Profile parser before/after changes

### Low Risk
- **Basic numeric parsing**: Well-understood problem domain

## Dependencies

### Blocking Dependencies
- **phase1-04**: Type conversion functions for proper value creation

### Related Libraries
- **nom**: Parser combinator library (already in use)
- **chrono**: Date/time parsing (may need integration)
- **ucum-rs**: Unit parsing for quantities

## Next Steps After Completion

1. Update task status to 🟢 COMPLETED
2. Run comprehensive test coverage report
3. Update phase progress in task index
4. Verify parsing performance maintained
5. Begin next Phase 3 task

---

*Created: 2025-07-27*  
*Last Updated: 2025-07-27*
