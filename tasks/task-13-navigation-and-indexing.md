# Task 13: Navigation and Indexing Functions

## Overview
Implement navigation, indexing, and accessor functions for complete FHIRPath data traversal.

## Current Issues from TEST_COVERAGE.md
- **indexer.json** - 0.0% (0/2 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests) - Issues
- **observations.json** - 10.0% (1/10 tests) - Issues
- **dollar.json** - 40.0% (2/5 tests) - Issues
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **inheritance.json** - 0.0% (0/24 tests) - Missing

## Subtasks

### 13.1 Implement Indexing Functions
- [ ] Implement array indexing with [n] syntax
- [ ] Implement index-part operations for complex paths
- [ ] Support negative indexing
- [ ] Handle out-of-bounds indexing gracefully
- **Target**: indexer.json 0% → 90%+, index-part.json 0% → 100%

### 13.2 Fix Accessor Functions
- [ ] Debug miscellaneous accessor issues (currently 33.3%)
- [ ] Fix complex path navigation
- [ ] Improve property access patterns
- [ ] Handle nested object navigation
- **Target**: miscellaneous-accessor-tests.json 33.3% → 80%+

### 13.3 Improve Context Functions
- [ ] Fix dollar ($) context variable issues (currently 40%)
- [ ] Ensure proper context preservation in nested evaluations
- [ ] Support multiple context levels
- [ ] Handle context switching properly
- **Target**: dollar.json 40% → 90%+

### 13.4 Implement FHIR Type System Features
- [ ] Implement polymorphic type handling
- [ ] Add inheritance-based navigation
- [ ] Support FHIR resource type hierarchy
- [ ] Handle abstract type scenarios
- **Target**: polymorphics.json 0% → 70%+, inheritance.json 0% → 50%+

### 13.5 Fix Real-world Data Tests
- [ ] Debug observations test failures (currently 10%)
- [ ] Ensure proper handling of complex FHIR resources
- [ ] Fix resource property navigation
- [ ] Handle array and object mixed scenarios
- **Target**: observations.json 10% → 60%+

## Expected Outcomes
- indexer.json: 0% → 90%+
- index-part.json: 0% → 100%
- miscellaneous-accessor-tests.json: 33.3% → 80%+
- dollar.json: 40% → 90%+
- polymorphics.json: 0% → 70%+
- inheritance.json: 0% → 50%+
- observations.json: 10% → 60%+
- Overall test coverage improvement: +2-3%

## Files to Modify
- `/fhirpath-parser/src/parser.rs` - Indexing and navigation parsing
- `/fhirpath-evaluator/src/engine.rs` - Context and navigation evaluation
- `/fhirpath-model/src/resource.rs` - FHIR resource navigation
- `/fhirpath-model/src/types.rs` - Type system and inheritance