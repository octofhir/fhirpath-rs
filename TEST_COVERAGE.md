# FHIRPath Test Coverage Report

Generated on: 2025-09-15
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 113 | 100% |
| **Total Individual Tests** | 1110 | 100% |
| **Passing Tests** | 936 | 84.3% |
| **Failing Tests** | 168 | 15.1% |
| **Error Tests** | 6 | 0.5% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **Comparable.json** - 3/3 tests
- **Precision.json** - 6/6 tests
- **TerminologyTests.json** - 3/3 tests
- **comments.json** - 9/9 tests
- **day-of.json** - 7/7 tests
- **from-Zulip.json** - 2/2 tests
- **hour-of.json** - 5/5 tests
- **index-part.json** - 1/1 tests
- **minimal.json** - 1/1 tests
- **minute-of.json** - 5/5 tests
- **month-of.json** - 7/7 tests
- **polymorphics.json** - 2/2 tests
- **second-of.json** - 5/5 tests
- **testAggregate.json** - 4/4 tests
- **testAll.json** - 4/4 tests
- **testBasics.json** - 7/7 tests
- **testBooleanImplies.json** - 9/9 tests
- **testBooleanLogicAnd.json** - 9/9 tests
- **testBooleanLogicOr.json** - 9/9 tests
- **testBooleanLogicXOr.json** - 9/9 tests
- **testCase.json** - 4/4 tests
- **testContainsCollection.json** - 9/9 tests
- **testContainsString.json** - 12/12 tests
- **testCount.json** - 4/4 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testDollar.json** - 5/5 tests
- **testEncodeDecode.json** - 8/8 tests
- **testEndsWith.json** - 12/12 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testExists.json** - 5/5 tests
- **testExtension.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testGreaterThan.json** - 30/30 tests
- **testGreatorOrEqual.json** - 30/30 tests
- **testIn.json** - 8/8 tests
- **testIndexOf.json** - 6/6 tests
- **testIndexer.json** - 2/2 tests
- **testIntersect.json** - 4/4 tests
- **testJoin.json** - 1/1 tests
- **testLength.json** - 6/6 tests
- **testLessOrEqual.json** - 30/30 tests
- **testLessThan.json** - 30/30 tests
- **testLn.json** - 3/3 tests
- **testLog.json** - 5/5 tests
- **testMatches.json** - 16/16 tests
- **testMiscellaneousAccessorTests.json** - 3/3 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests
- **testPower.json** - 6/6 tests
- **testReplace.json** - 6/6 tests
- **testReplaceMatches.json** - 7/7 tests
- **testRound.json** - 3/3 tests
- **testSelect.json** - 3/3 tests
- **testSingle.json** - 2/2 tests
- **testSkip.json** - 4/4 tests
- **testSplit.json** - 4/4 tests
- **testSqrt.json** - 3/3 tests
- **testStartsWith.json** - 14/14 tests
- **testSubstring.json** - 12/12 tests
- **testTail.json** - 2/2 tests
- **testTake.json** - 7/7 tests
- **testToChars.json** - 1/1 tests
- **testTrim.json** - 6/6 tests
- **testVariables.json** - 4/4 tests
- **testWhere.json** - 4/4 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testTypes.json** - 93.9% (93/99 tests)
- **testSort.json** - 90.0% (9/10 tests)
- **testCollectionBoolean.json** - 83.3% (5/6 tests)
- **testNEquality.json** - 83.3% (20/24 tests)
- **testUnion.json** - 83.3% (10/12 tests)
- **testEquality.json** - 82.1% (23/28 tests)
- **testNotEquivalent.json** - 81.8% (18/22 tests)
- **testToDecimal.json** - 80.0% (4/5 tests)
- **testToInteger.json** - 80.0% (4/5 tests)
- **testToString.json** - 80.0% (4/5 tests)
- **testEquivalent.json** - 79.2% (19/24 tests)
- **testLiterals.json** - 78.0% (64/82 tests)
- **HighBoundary.json** - 75.0% (18/24 tests)
- **LowBoundary.json** - 75.0% (21/28 tests)
- **testCeiling.json** - 75.0% (3/4 tests)
- **testFloor.json** - 75.0% (3/4 tests)
- **testIif.json** - 75.0% (9/12 tests)
- **testTruncate.json** - 75.0% (3/4 tests)
- **testMinus.json** - 72.7% (8/11 tests)
- **testObservations.json** - 70.0% (7/10 tests)

### 🟠 Partially Implemented (30-70%)

- **testExp.json** - 66.7% (2/3 tests)
- **testPrecedence.json** - 66.7% (4/6 tests)
- **testSubSetOf.json** - 66.7% (2/3 tests)
- **testType.json** - 63.3% (19/30 tests)
- **testPlus.json** - 52.9% (18/34 tests)
- **repeat-all.json** - 52.6% (10/19 tests)
- **defineVariable.json** - 52.4% (11/21 tests)
- **period.json** - 50.0% (1/2 tests)
- **testDistinct.json** - 50.0% (3/6 tests)
- **testNow.json** - 50.0% (1/2 tests)
- **testSuperSetOf.json** - 50.0% (1/2 tests)
- **testToday.json** - 50.0% (1/2 tests)
- **testTrace.json** - 50.0% (1/2 tests)
- **testQuantity.json** - 45.5% (5/11 tests)
- **testInheritance.json** - 41.7% (10/24 tests)
- **testCombine--.json** - 33.3% (1/3 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **testAbs.json** - 25.0% (1/4 tests) - Issues
- **testConcatenate.json** - 25.0% (1/4 tests) - Issues
- **testRepeat.json** - 20.0% (1/5 tests) - Issues
- **cdaTests.json** - 0.0% (0/3 tests) - Missing
- **miscEngineTests.json** - 0.0% (0/2 tests) - Missing
- **resolve.json** - 0.0% (0/3 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **84.3% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 113
- **Total Tests**: 1110
- **Pass Rate**: 84.3%

---

*Report generated on: 2025-09-15 07:54:51*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
