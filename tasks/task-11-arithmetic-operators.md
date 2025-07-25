
# Task 11: Fix Missing Arithmetic Operators

## Status: Pending
**Priority:** High  
**Estimated Time:** 2-3 days  
**Dependencies:** None

## Overview

Based on the test coverage analysis, several critical arithmetic operators are missing or broken in the current implementation:

- **Subtraction (`-`)**: Currently failing (0% pass rate)
- **Multiplication (`*`)**: Currently failing (0% pass rate) 
- **Modulo (`mod`)**: Not implemented (0% pass rate)
- **Power operator**: Not implemented (0% pass rate)

## Test Files Affected

- `minus.json` - 0% (0/11 tests)
- `multiply.json` - 0% (0/6 tests) 
- `mod.json` - 0% (0/8 tests)
- `power.json` - 0% (0/6 tests)
- `plus.json` - 23.5% (8/34 tests) - needs improvement

## Current Implementation Status

Based on analysis of `/fhirpath-registry/src/operator.rs`:

✅ **Already Implemented:**
- Addition (`+`) - basic implementation exists but needs fixes
- Division (`/`) - implemented with proper division by zero handling
- Integer division (`div`) - implemented

❌ **Missing/Broken:**
- Subtraction (`-`) - implementation exists but not working properly
- Multiplication (`*`) - implementation exists but not working properly  
- Modulo (`mod`) - implementation exists but not working properly
- Power operator - completely missing

## Implementation Plan

### Phase 1: Fix Existing Operators (Day 1)

1. **Debug Subtraction Operator**
   - Review `SubtractOperator` implementation in `operator.rs:453-528`
   - Fix issues with empty operand handling
   - Ensure proper type coercion between Integer/Decimal
   - Test with date/time arithmetic

2. **Debug Multiplication Operator**
   - Review `MultiplyOperator` implementation in `operator.rs:530-589`
   - Fix quantity multiplication logic
   - Ensure proper overflow handling

3. **Debug Modulo Operator**
   - Review `ModuloOperator` implementation in `operator.rs:704-738`
   - The implementation looks correct - investigate why tests are failing
   - May be registration issue or evaluation engine problem

### Phase 2: Implement Missing Operators (Day 2)

4. **Implement Power Operator**
   - Create `PowerOperator` struct
   - Support Integer^Integer -> Integer for small powers
   - Support Decimal^Integer -> Decimal
   - Handle edge cases (0^0, negative bases)
   - Add appropriate signatures

5. **Improve Addition Operator**
   - Fix edge cases causing test failures
   - Improve string concatenation handling
   - Fix date/time arithmetic edge cases

### Phase 3: Integration and Testing (Day 3)

6. **Registry Integration**
   - Ensure all operators are registered in `register_builtin_operators()`
   - Verify precedence and associativity settings
   - Test operator resolution

7. **Comprehensive Testing**
   - Run all arithmetic operator tests
   - Fix any remaining edge cases
   - Ensure proper error handling

## Implementation Details

### Power Operator Signature
```rust
struct PowerOperator;

impl FhirPathOperator for PowerOperator {
    fn symbol(&self) -> &str { "**" }  // or "power"
    fn precedence(&self) -> u8 { 8 }   // Higher than multiplication
    fn associativity(&self) -> Associativity { Associativity::Right }
    
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![
                OperatorSignature::binary("**", TypeInfo::Integer, TypeInfo::Integer, TypeInfo::Integer),
                OperatorSignature::binary("**", TypeInfo::Decimal, TypeInfo::Integer, TypeInfo::Decimal),
                OperatorSignature::binary("**", TypeInfo::Integer, TypeInfo::Decimal, TypeInfo::Decimal),
                OperatorSignature::binary("**", TypeInfo::Decimal, TypeInfo::Decimal, TypeInfo::Decimal),
            ]
        });
        &SIGS
    }
}
```

## Expected Outcomes

After completion, the following test pass rates should be achieved:
- `minus.json`: 90%+ (9+/11 tests)
- `multiply.json`: 90%+ (5+/6 tests)
- `mod.json`: 90%+ (7+/8 tests)
- `power.json`: 80%+ (5+/6 tests)
- `plus.json`: 80%+ (27+/34 tests)

## Files to Modify

- `/fhirpath-registry/src/operator.rs`
  - Fix existing operator implementations
  - Add PowerOperator implementation
  - Update `register_builtin_operators()` if needed

## Testing Strategy

1. Create unit tests for each operator with edge cases
2. Run official FHIRPath test suites for arithmetic operators
3. Test with various numeric types (Integer, Decimal, Quantity)
4. Verify error handling for division by zero, overflow, etc.

## Notes

- The existing implementations in `operator.rs` appear mostly correct syntactically
- The failures may be due to integration issues with the evaluation engine
- Pay special attention to FHIRPath's special handling of empty collections
- Ensure proper type coercion between Integer and Decimal types
