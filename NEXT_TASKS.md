# Next Development Tasks - FHIRPath Rust Implementation

## Current Status Summary
- ✅ Modular architecture complete (7/7 crates implemented)  
- ✅ JSON test runner infrastructure ready
- ✅ Basic parsing and evaluation working
- ❌ Compilation errors need fixing
- ❌ Missing operator implementations

## Completed High Priority Tasks ✅

### 1. Fix Compilation Errors [COMPLETED]
- ✅ Fixed all compilation errors in fhirpath-core
- ✅ Resolved Collection access patterns
- ✅ Fixed type mismatches and trait implementations

### 2. Fix Architectural Duplication [COMPLETED]
- ✅ Removed duplicate `fhirpath-core/src/registry/` folder
- ✅ Consolidated all implementations in `fhirpath-registry`
- ✅ Updated `fhirpath-core` to use `fhirpath-evaluator`
- ✅ Established single source of truth per ADR-002

### 3. Implement Core Operators [COMPLETED]
**Files**: `fhirpath-registry/src/operator.rs`
- ✅ Equal operator (`=`) with proper FHIRPath semantics
- ✅ NotEqual operator (`!=`)
- ✅ LessThan operator (`<`)
- ✅ GreaterThan operator (`>`)
- ✅ LessThanOrEqual operator (`<=`)
- ✅ GreaterThanOrEqual operator (`>=`)

### 4. Implement Arithmetic Operators [COMPLETED]
**Files**: `fhirpath-registry/src/operator.rs`
- ✅ Plus operator (`+`)
- ✅ Minus operator (`-`)
- ✅ Multiply operator (`*`)
- ✅ Divide operator (`/`)
- ✅ Modulo operator (`mod`)
- ✅ DivideInteger operator (`div`)

## Current Status Update ✅

### **Architecture Fixed Successfully!**
- ✅ All compilation errors resolved
- ✅ Modular architecture per ADR-002 working
- ✅ Test runner functional and running basic tests
- ✅ 2/7 basic tests passing, 1 failing, 4 with errors

### **Test Results from basics.json:**
```
Total:   7
✅ Passed:  2 (28.6%) - testSimpleNone, testSimpleFail
❌ Failed:  1 (14.3%) - testSimple (collection flattening issue)
⚠️  Errors:  4 (57.1%) - parser and evaluation issues
```

## Current Priority Tasks (Week 1-2)

## Secondary Tasks (Week 2)

### 4. Run and Fix Basic Tests [MEDIUM PRIORITY]
**Target**: `specs/fhirpath/tests/basics.json`
- Run test suite after operators are implemented
- Fix any failing test cases
- Document test coverage

### 5. Implement String Functions [MEDIUM PRIORITY]
**Files**: `fhirpath-registry/src/functions/string.rs`
- [ ] `contains()` function
- [ ] `startsWith()` function
- [ ] `endsWith()` function
- [ ] `indexOf()` function
- [ ] `substring()` function
- [ ] `replace()` function

### 6. Implement Collection Functions [MEDIUM PRIORITY]
**Files**: `fhirpath-registry/src/functions/collection.rs`
- [ ] `first()` function
- [ ] `last()` function
- [ ] `take()` function
- [ ] `skip()` function
- [ ] `tail()` function
- [ ] `where()` function
- [ ] `select()` function

## Test Suite Progress Tracking

### Phase 1: Core Functionality Tests
- [ ] `basics.json` - Basic FHIRPath operations
- [ ] `literals.json` - Literal value parsing
- [ ] `comparable.json` - Comparison operations
- [ ] `boolean-logic-*.json` - Boolean logic tests

### Phase 2: String and Collection Tests  
- [ ] `contains-string.json`
- [ ] `concatenate.json`
- [ ] `collection-boolean.json`
- [ ] `count.json`

### Phase 3: Advanced Function Tests
- [ ] `aggregate.json`
- [ ] `all.json`
- [ ] `any.json`
- [ ] `exists.json`

## Development Workflow

1. **Fix compilation first** - No progress possible until code compiles
2. **Implement operators** - Required by most test cases
3. **Run basics.json** - Verify core functionality
4. **Iterate on failures** - Fix issues one test suite at a time
5. **Track progress** - Update this file after each milestone

## Success Metrics

- [ ] All crates compile without errors
- [ ] Zero compiler warnings
- [ ] `basics.json` test suite passes 100%
- [ ] At least 50% of official test suites passing
- [ ] Performance benchmarks established

## Commands for Development

```bash
# Fix compilation errors
cargo check --workspace

# Run specific test suite
cargo run --bin fhirpath-test-runner -- specs/fhirpath/tests/basics.json

# Run all tests with verbose output
cargo run --bin fhirpath-test-runner -- --verbose specs/fhirpath/tests/

# Run benchmarks (after compilation fixed)
cargo bench --workspace

# Fix warnings
cargo clippy --workspace --fix
```

## Architecture Notes

Per ADR-002, maintain separation of concerns:
- Parser changes → `fhirpath-parser` crate
- Operator implementations → `fhirpath-registry` crate  
- Value type changes → `fhirpath-model` crate
- Evaluation logic → `fhirpath-evaluator` crate

## Next Review Date
Review and update this document after completing Phase 1 operators implementation.