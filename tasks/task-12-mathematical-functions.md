# Task 12: Implement Missing Mathematical Functions

## Status: Pending
**Priority:** High  
**Estimated Time:** 3-4 days  
**Dependencies:** Task 11 (Arithmetic Operators)

## Overview

Based on the test coverage analysis, several critical mathematical functions are completely missing:

- **ceiling()**: Missing (0% pass rate - 0/4 tests)
- **floor()**: Missing (0% pass rate - 0/4 tests)
- **sqrt()**: Missing (0% pass rate - 0/3 tests)
- **power()**: Missing (0% pass rate - 0/6 tests)
- **round()**: Missing (0% pass rate - 0/3 tests)
- **truncate()**: Missing (0% pass rate - 0/4 tests)
- **exp()**: Missing (0% pass rate - 0/3 tests)
- **ln()**: Missing (0% pass rate - 0/3 tests)
- **log()**: Missing (0% pass rate - 0/5 tests)
- **abs()**: Partially working (25% pass rate - 1/4 tests)

## Test Files Affected

- `ceiling.json` - 0% (0/4 tests)
- `floor.json` - 0% (0/4 tests)
- `sqrt.json` - 0% (0/3 tests)
- `power.json` - 0% (0/6 tests)
- `round.json` - 0% (0/3 tests)
- `truncate.json` - 0% (0/4 tests)
- `exp.json` - 0% (0/3 tests)
- `ln.json` - 0% (0/3 tests)
- `log.json` - 0% (0/5 tests)
- `abs.json` - 25% (1/4 tests)

## Current Implementation Status

Based on analysis of `/fhirpath-registry/src/function.rs`:

✅ **Already Implemented:**
- `abs()` - basic implementation exists but needs fixes

❌ **Missing Functions:**
- All other mathematical functions listed above

## Implementation Plan

### Phase 1: Basic Mathematical Functions (Days 1-2)

1. **Implement Ceiling Function**
   ```rust
   struct CeilingFunction;
   // Returns the smallest integer greater than or equal to the input
   ```

2. **Implement Floor Function**
   ```rust
   struct FloorFunction;
   // Returns the largest integer less than or equal to the input
   ```

3. **Implement Round Function**
   ```rust
   struct RoundFunction;
   // Rounds to the nearest integer (with optional precision parameter)
   ```

4. **Implement Truncate Function**
   ```rust
   struct TruncateFunction;
   // Removes the fractional part, keeping the integer part
   ```

5. **Fix Abs Function**
   - Debug why 3/4 tests are failing
   - Likely issues with quantity handling or empty collection behavior

### Phase 2: Advanced Mathematical Functions (Days 3-4)

6. **Implement Sqrt Function**
   ```rust
   struct SqrtFunction;
   // Returns the square root of the input
   ```

7. **Implement Power Function (if not handled by operator)**
   ```rust
   struct PowerFunction;
   // Raises base to the power of exponent
   ```

8. **Implement Exponential Functions**
   ```rust
   struct ExpFunction;   // e^x
   struct LnFunction;    // natural logarithm
   struct LogFunction;   // logarithm with optional base
   ```

## Implementation Details

### Function Template Pattern
```rust
struct [FunctionName]Function;

impl FhirPathFunction for [FunctionName]Function {
    fn name(&self) -> &str { "[function_name]" }
    
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "[function_name]",
                vec![], // No parameters for most math functions
                TypeInfo::Any, // Can return Integer, Decimal, or Quantity
            )
        });
        &SIG
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        
        match &context.input {
            FhirPathValue::Integer(i) => {
                // Handle integer input
                let result = /* apply mathematical operation */;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(result)]))
            }
            FhirPathValue::Decimal(d) => {
                // Handle decimal input
                let result = /* apply mathematical operation */;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(result)]))
            }
            FhirPathValue::Quantity(q) => {
                // Handle quantity input (preserve units)
                let mut result = q.clone();
                result.value = /* apply mathematical operation to q.value */;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(result)]))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::collection(vec![])),
        }
    }
}
```

### Key Implementation Considerations

1. **Type Handling**:
   - Support Integer, Decimal, and Quantity types
   - For quantities, preserve units where mathematically valid
   - Return appropriate empty results for invalid inputs

2. **Error Handling**:
   - Handle domain errors (e.g., sqrt of negative numbers)
   - Handle overflow/underflow conditions
   - Follow FHIRPath spec for empty collection handling

3. **Precision**:
   - Use appropriate precision for decimal operations
   - Consider rounding behavior consistency with FHIRPath spec

4. **Edge Cases**:
   - Handle special values (infinity, NaN equivalent handling)
   - Handle very large/small numbers appropriately
   - Empty collection handling as per FHIRPath spec

## Registration

Add all new functions to `register_builtin_functions()` in `function.rs`:

```rust
pub fn register_builtin_functions(registry: &mut FunctionRegistry) {
    // ... existing functions ...
    
    // Mathematical functions
    registry.register(CeilingFunction);
    registry.register(FloorFunction);
    registry.register(RoundFunction);
    registry.register(TruncateFunction);
    registry.register(SqrtFunction);
    registry.register(ExpFunction);
    registry.register(LnFunction);
    registry.register(LogFunction);
    // abs() already registered - just need to fix implementation
}
```

## Expected Outcomes

After completion, the following test pass rates should be achieved:
- `ceiling.json`: 90%+ (3+/4 tests)
- `floor.json`: 90%+ (3+/4 tests)
- `sqrt.json`: 90%+ (2+/3 tests)
- `round.json`: 90%+ (2+/3 tests)
- `truncate.json`: 90%+ (3+/4 tests)
- `exp.json`: 80%+ (2+/3 tests)
- `ln.json`: 80%+ (2+/3 tests)
- `log.json`: 80%+ (4+/5 tests)
- `abs.json`: 95%+ (4/4 tests)

## Files to Modify

- `/fhirpath-registry/src/function.rs`
  - Add all mathematical function implementations
  - Fix existing `AbsFunction`
  - Update `register_builtin_functions()`

## Testing Strategy

1. **Unit Testing**: Create tests for each function with various input types
2. **Edge Case Testing**: Test with special values, empty collections, quantities
3. **Official Test Suite**: Run the specific test files to validate compliance
4. **Integration Testing**: Ensure functions work in complex expressions

## Dependencies

- Uses `rust_decimal` crate for precise decimal arithmetic
- May need additional math functions from standard library
- Consider using specialized math crates for advanced functions (exp, ln, log)

## Notes

- Mathematical functions should preserve FHIRPath semantics (empty in = empty out)
- Quantity handling requires careful consideration of unit preservation
- Some functions may need special handling for edge cases as defined in FHIRPath spec
- Performance should be considered for frequently used functions like abs(), ceiling(), floor()