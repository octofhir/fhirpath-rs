# Task 9: Complete Arithmetic and Logic Operations

## Overview
Fix remaining arithmetic operations and implement missing boolean logic functions.

## Current Issues from TEST_COVERAGE.md
- **plus.json** - 23.5% (8/34 tests) - Issues
- **divide.json** - 33.3% (3/9 tests) - Issues
- **boolean-logic-and.json** - 0.0% (0/9 tests) - Missing
- **boolean-logic-or.json** - 44.4% (4/9 tests) - Issues
- **boolean-logic-x-or.json** - 44.4% (4/9 tests) - Issues
- **boolean-implies.json** - 44.4% (4/9 tests) - Issues
- **power.json** - 0.0% (0/6 tests) - Missing

## Subtasks

### 9.1 Fix Addition Operations
- [ ] Debug plus operator for string concatenation
- [ ] Fix plus operator for quantity operations
- [ ] Handle date/time arithmetic properly
- [ ] Fix collection handling in plus operations
- **Target**: plus.json 23.5% → 70%+

### 9.2 Fix Division Operations  
- [ ] Debug divide operator edge cases
- [ ] Handle division by zero properly
- [ ] Fix decimal precision in division
- [ ] Ensure proper type coercion
- **Target**: divide.json 33.3% → 80%+

### 9.3 Implement Missing Boolean Logic
- [ ] Implement AND operator (boolean-logic-and)
- [ ] Fix OR operator issues (boolean-logic-or)
- [ ] Fix XOR operator issues (boolean-logic-x-or)
- [ ] Fix IMPLIES operator issues (boolean-implies)
- **Target**: AND 0% → 90%+, OR 44.4% → 90%+, XOR 44.4% → 90%+, IMPLIES 44.4% → 90%+

### 9.4 Implement Power Operations
- [ ] Implement power operator registration
- [ ] Add power operator to parser if missing
- [ ] Handle edge cases (0^0, negative bases, etc.)
- [ ] Support both integer and decimal operations
- **Target**: power.json 0% → 80%+

## Expected Outcomes
- plus.json: 23.5% → 70%+
- divide.json: 33.3% → 80%+
- boolean-logic-and.json: 0% → 90%+
- boolean-logic-or.json: 44.4% → 90%+
- boolean-logic-x-or.json: 44.4% → 90%+
- boolean-implies.json: 44.4% → 90%+
- power.json: 0% → 80%+
- Overall test coverage improvement: +3-4%

## Files to Modify
- `/fhirpath-registry/src/operator.rs` - Arithmetic and logic operators
- `/fhirpath-parser/src/parser.rs` - Operator parsing
- `/fhirpath-parser/src/tokenizer.rs` - Token definitions