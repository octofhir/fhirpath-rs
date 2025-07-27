# UCUM Functions Plan for FHIRPath Quantity Operations

## Overview
FHIRPath requires quantity arithmetic operations (multiplication and division) that involve UCUM unit operations. We need to implement these functions in the octofhir-ucum-core package.

## Required Functions

### 1. `multiply_units(unit1: &str, unit2: &str) -> Result<String, UcumError>`

**Purpose**: Multiply two UCUM units and return the resulting unit expression.

**Examples**:
- `multiply_units("cm", "m")` → `"cm.m"` or `"0.01.m2"` (normalized)
- `multiply_units("kg", "m/s2")` → `"kg.m/s2"` or `"N"` (if Newton is defined)
- `multiply_units("1", "m")` → `"m"`
- `multiply_units("m", "1")` → `"m"`

**Test Cases**:
```rust
#[test]
fn test_multiply_units() {
    // Basic multiplication
    assert_eq!(multiply_units("m", "m").unwrap(), "m2");
    assert_eq!(multiply_units("cm", "m").unwrap(), "cm.m");
    
    // With dimensionless units
    assert_eq!(multiply_units("1", "m").unwrap(), "m");
    assert_eq!(multiply_units("m", "1").unwrap(), "m");
    assert_eq!(multiply_units("1", "1").unwrap(), "1");
    
    // Complex units
    assert_eq!(multiply_units("kg.m/s2", "m").unwrap(), "kg.m2/s2");
    assert_eq!(multiply_units("mol/L", "L").unwrap(), "mol");
    
    // Error cases
    assert!(multiply_units("invalid", "m").is_err());
    assert!(multiply_units("m", "invalid").is_err());
}
```

### 2. `divide_units(unit1: &str, unit2: &str) -> Result<String, UcumError>`

**Purpose**: Divide two UCUM units and return the resulting unit expression.

**Examples**:
- `divide_units("m", "s")` → `"m/s"`
- `divide_units("m", "m")` → `"1"` (dimensionless)
- `divide_units("kg.m2/s2", "m")` → `"kg.m/s2"`
- `divide_units("mol", "L")` → `"mol/L"`

**Test Cases**:
```rust
#[test]
fn test_divide_units() {
    // Basic division
    assert_eq!(divide_units("m", "s").unwrap(), "m/s");
    assert_eq!(divide_units("m2", "m").unwrap(), "m");
    
    // Same units cancel out
    assert_eq!(divide_units("m", "m").unwrap(), "1");
    assert_eq!(divide_units("kg", "kg").unwrap(), "1");
    
    // Division by dimensionless
    assert_eq!(divide_units("m", "1").unwrap(), "m");
    assert_eq!(divide_units("1", "m").unwrap(), "1/m");
    
    // Complex units
    assert_eq!(divide_units("kg.m/s2", "m").unwrap(), "kg/s2");
    assert_eq!(divide_units("mol", "mol/L").unwrap(), "L");
    
    // Error cases
    assert!(divide_units("invalid", "m").is_err());
    assert!(divide_units("m", "invalid").is_err());
}
```

### 3. `normalize_unit(unit: &str) -> Result<String, UcumError>` (Optional Enhancement)

**Purpose**: Normalize a unit expression to its canonical form.

**Examples**:
- `normalize_unit("cm.m")` → `"0.01.m2"`
- `normalize_unit("kg.m/s2")` → `"N"` (if Newton is defined)
- `normalize_unit("1000.g")` → `"kg"`

## Implementation Requirements

1. **Parse Input Units**: Both functions should parse the input unit strings using the existing `parse_expression` function.

2. **Unit Algebra**: Implement the mathematical operations on unit expressions:
   - Multiplication: Combine units with multiplication operator (`.`)
   - Division: Combine units with division operator (`/`)
   - Simplification: Cancel out common units where possible

3. **Error Handling**: Return appropriate errors for:
   - Invalid unit expressions
   - Parse errors
   - Mathematical inconsistencies

4. **Canonical Form**: Return results in canonical UCUM form where possible.

## Integration Points

These functions will be used in:
- `fhirpath-registry/src/operator.rs` - MultiplyOperator::multiply_quantities()
- `fhirpath-registry/src/operator.rs` - DivideOperator::divide_quantities()

## FHIRPath Test Cases Expected

Based on the FHIRPath quantity tests:
- `2.0 'cm' * 2.0 'm' = 0.040 'm2'` (unit multiplication with conversion)
- `4.0 'g' / 2.0 'm' = 2 'g/m'` (unit division)
- `1.0 'm' / 1.0 'm' = 1 '1'` (unit cancellation)

## Priority

**High** - These functions are required for FHIRPath quantity arithmetic operations to pass the official test suite.