# Task 10: Advanced Language Features

## Overview
Implement advanced FHIRPath language features for complete specification compliance.

## Current Issues from TEST_COVERAGE.md
- **define-variable.json** - 0.0% (0/21 tests) - Missing
- **case.json** - 0.0% (0/4 tests) - Missing
- **iif.json** - 27.3% (3/11 tests) - Issues
- **where.json** - Partially implemented in lambda functions
- **select.json** - 33.3% (1/3 tests) - Issues

## Subtasks

### 10.1 Implement Variable Definition
- [ ] Add support for `%variable` syntax
- [ ] Implement `define` statement parsing
- [ ] Add variable scoping and resolution
- [ ] Support variable assignment and retrieval
- **Target**: define-variable.json 0% → 70%+

### 10.2 Implement Case Expressions
- [ ] Add `case` expression parsing
- [ ] Implement pattern matching logic
- [ ] Support multiple case branches
- [ ] Handle default case scenarios
- **Target**: case.json 0% → 80%+

### 10.3 Fix Conditional Functions
- [ ] Debug and fix iif() function (currently 27.3%)
- [ ] Ensure proper boolean evaluation
- [ ] Handle edge cases with empty collections
- [ ] Fix collection handling in conditional logic
- **Target**: iif.json 27.3% → 90%+

### 10.4 Improve Selection and Projection
- [ ] Fix where() function edge cases
- [ ] Improve select() function (currently 33.3%)
- [ ] Handle complex projection scenarios
- [ ] Ensure proper lambda evaluation
- **Target**: select.json 33.3% → 80%+

## Expected Outcomes
- define-variable.json: 0% → 70%+
- case.json: 0% → 80%+
- iif.json: 27.3% → 90%+
- select.json: 33.3% → 80%+
- Overall test coverage improvement: +2-3%

## Files to Modify
- `/fhirpath-parser/src/parser.rs` - Variable and case parsing
- `/fhirpath-ast/src/lib.rs` - AST nodes for variables/case
- `/fhirpath-evaluator/src/engine.rs` - Variable evaluation
- `/fhirpath-registry/src/function.rs` - Conditional functions