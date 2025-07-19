---
title: FHIRPath Test Compliance
description: Current status of official FHIRPath test suite compliance showing supported features.
---

# FHIRPath Test Compliance

This page shows the current compliance status with the official FHIRPath test suite, indicating which features are currently supported by the OctoFHIR FHIRPath implementation.

## Test Results Summary

Based on the latest test run of the official FHIRPath test suite:

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ **Passed** | 512 | 72.0% |
| ❌ **Failed** | 199 | 28.0% |
| ⏭️ **Skipped** | 0 | 0.0% |
| **Total** | **711** | **100%** |

## Implementation Status

### ✅ Currently Supported Features

The following FHIRPath features are currently working and pass the official tests:

- **Basic Path Navigation**: Simple property access and navigation
- **Collection Operations**: Basic collection handling and iteration
- **String Operations**: String manipulation and comparison functions
- **Numeric Operations**: Basic arithmetic and numeric comparisons
- **Boolean Logic**: Basic boolean operations and comparisons
- **Type Checking**: Basic type validation and conversion
- **Telecom Access**: Patient telecom data access and filtering

### ❌ Known Limitations

The following features are not yet implemented or have known issues:

- **Advanced Functions**: Many built-in functions like `conformsTo()` are not implemented
- **Complex Operators**: Some advanced operators and syntax patterns
- **Date/Time Operations**: Advanced date and time manipulation
- **Regular Expressions**: Pattern matching and regex operations
- **Advanced Type Conversions**: Complex type casting and conversion
- **Aggregate Functions**: Advanced collection aggregation operations

### ⏭️ Skipped Tests

Some tests are currently skipped due to:
- Missing input data files
- XML to JSON conversion requirements
- Test environment setup issues

## Test Groups Status

The official FHIRPath test suite is organized into several test groups. Here's the status of major test groups:

### Core Functionality
- **testBasics**: Partially supported - basic path navigation works
- **testMiscellaneousAccessorTests**: Mixed results - some accessor patterns work
- **testConformsTo**: Not supported - `conformsTo()` function not implemented

### Data Types
- **String Operations**: Partially supported
- **Numeric Operations**: Partially supported  
- **Boolean Operations**: Partially supported
- **Date/Time Operations**: Limited support

### Advanced Features
- **Function Library**: Many functions not yet implemented
- **Complex Expressions**: Limited support for complex syntax
- **Error Handling**: Basic error handling implemented

## Detailed Test Failures

The following sections provide specific details about which tests are failing and why, organized by error type and test group.

### Lexer Errors

These tests fail due to unsupported syntax or characters in the lexer:

#### Date/Time Literal Syntax (@)
- **testLiteralDateYear**: `@2014` - Unexpected character '@' at line 1, column 1
- **testLiteralDateMonth**: `@2014-01` - Unexpected character '@' at line 1, column 1
- **testLiteralDateDay**: `@2014-01-20` - Unexpected character '@' at line 1, column 1
- **testLiteralDateTimeYear**: `@2014T` - Unexpected character '@' at line 1, column 1
- **testDateEqual**: Date comparison expressions - Unexpected character '@' at line 1, column 21

#### Dollar Sign Variables ($)
- **testDollarThis1**: `Bundle.entry.select(resource as Patient).name.given.where($this.length() > 3)` - Unexpected character '$' at line 1, column 36
- **testDollarThis2**: Similar expression with '$this' variable - Unexpected character '$' at line 1, column 36

#### Backtick Identifiers (`)
- **testEscapedIdentifier**: Expression with backtick identifiers - Unexpected character '`' at line 1, column 6
- **testSimpleBackTick1**: Expression starting with backtick - Unexpected character '`' at line 1, column 1

#### Collection Syntax ({})
- **testCollectionNotEqualEmpty**: `{} != {}` - Unexpected character '{' at line 1, column 23

#### Pipe Operator (|)
- **testExpressions**: Expression with pipe operator - Unexpected character '|' at line 1, column 27

#### Decimal Point Parsing
- **testLiteralInteger1**: `1.` - Expected digit after decimal point at line 1, column 3
- **testLiteralInteger0**: `0.` - Expected digit after decimal point at line 1, column 3
- **testLiteralIntegerMax**: Large integer - Expected digit after decimal point at line 1, column 12

### Missing Function Implementations

These tests fail because required FHIRPath functions are not yet implemented:

#### Type Conversion Functions
- **convertsToInteger()**: 15+ tests fail (testStringLiteralConvertsToInteger, testBooleanLiteralConvertsToInteger, etc.)
- **convertsToBoolean()**: 10+ tests fail (testLiteralBooleanTrue, testStringTrueLiteralConvertsToBoolean, etc.)
- **convertsToDecimal()**: 10+ tests fail (testLiteralDecimal10, testDecimalLiteralConvertsToDecimal, etc.)
- **convertsToDate()**: 3 tests fail (testStringYearConvertsToDate, testStringMonthConvertsToDate, etc.)
- **convertsToDateTime()**: 7 tests fail (testStringYearConvertsToDateTime, testStringHourConvertsToDateTime, etc.)
- **convertsToTime()**: 4 tests fail (testStringHourConvertsToTime, testStringMinuteConvertsToTime, etc.)
- **convertsToQuantity()**: 10+ tests fail (testDecimalLiteralConvertsToQuantity, testStringQuantityLiteralConvertsToQuantity, etc.)

#### Type Casting Functions
- **toInteger()**: 5+ tests fail (testStringIntegerLiteralToInteger, testDecimalLiteralToInteger, etc.)
- **toDecimal()**: 4 tests fail (testDecimalLiteralToDecimal, testBooleanLiteralToDecimal, etc.)
- **toQuantity()**: 7 tests fail (testDecimalLiteralToQuantity, testStringQuantityLiteralToQuantity, etc.)

#### Logical Functions
- **not()**: 6 tests fail (testCollectionNotEmpty, testLiteralNotTrue, testLiteralNotFalse, etc.)

#### Date/Time Functions
- **today()**: 1 test fails (testDateNotEqualToday)
- **now()**: 1 test fails (testDateTimeGreaterThanDate)

#### Type Checking Functions
- **is()**: 10+ tests fail (testStringLiteralIsNotInteger, testBooleanLiteralIsNotInteger, etc.)

### Parser Errors

These tests fail due to issues in expression parsing:

#### Token Parsing
- **testLiteralIntegerLessThanPolarityTrue**: `+1 < 2` - Expected expression, got Token { token_type: Plus, lexeme: "+", position: 0, line: 1, column: 1 }

### Type Errors

These tests fail due to type compatibility issues:

#### Arithmetic Operations
- **testPolarityPrecedence**: Negation requires numeric operand
- **testLiteralIntegerGreaterThan**: Comparison requires compatible operands

### Test Groups Summary

#### testBasics (Mixed Results)
- ✅ **Passed**: testSimple, testSimpleWithContext, testDollarOrderAllowed, testDollarOrderNotAllowed
- ❌ **Failed**: testSimpleNone, testEscapedIdentifier, testSimpleBackTick1, testSimpleFail, testSimpleWithWrongContext

#### testLiterals (Many Failures)
- ✅ **Passed**: testLiteralTrue, testLiteralFalse, testLiteralString, testLiteralUnicode, testExpressionsEqual
- ❌ **Failed**: Most date/time literals, type conversion tests, numeric parsing issues

#### testTypes (Mostly Failing)
- ❌ **Failed**: Almost all tests due to missing type conversion and checking functions

#### testMiscellaneousAccessorTests (Mixed Results)
- ✅ **Passed**: testPatientTelecomTypes
- ❌ **Failed**: testExtractBirthDate, testPatientHasBirthDate

#### testObservations (All Skipped)
- ⏭️ **Skipped**: All tests due to XML to JSON conversion not implemented

## How to Interpret Results

- **✅ Passed**: The feature works correctly and matches expected FHIRPath behavior
- **❌ Failed**: The feature is not implemented or has bugs that cause incorrect results
- **⏭️ Skipped**: The test couldn't run due to missing dependencies or setup issues

## Continuous Improvement

This implementation is actively being developed. The test compliance will improve over time as more features are implemented. Key areas of focus include:

1. **Function Library Expansion**: Implementing missing built-in functions
2. **Operator Support**: Adding support for advanced operators
3. **Error Handling**: Improving error messages and edge case handling
4. **Performance**: Optimizing evaluation performance for complex expressions

## Running Tests Locally

To run the official FHIRPath tests locally:

```bash
cd fhirpath-core
cargo test run_official_fhirpath_tests -- --nocapture
```

This will show detailed results for each test, including which expressions are being evaluated and their results.

---

*Last updated: July 19, 2025*
*Test suite version: Official FHIRPath R4 tests*
