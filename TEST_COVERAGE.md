# FHIRPath Test Coverage Report

Generated on: 2025-09-06
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 112 | 100% |
| **Total Individual Tests** | 1090 | 100% |
| **Passing Tests** | 849 | 77.9% |
| **Failing Tests** | 230 | 21.1% |
| **Error Tests** | 11 | 1.0% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **Precision.json** - 6/6 tests
- **day-of.json** - 7/7 tests
- **hour-of.json** - 5/5 tests
- **index-part.json** - 1/1 tests
- **minimal.json** - 1/1 tests
- **minute-of.json** - 5/5 tests
- **month-of.json** - 7/7 tests
- **resolve.json** - 2/2 tests
- **second-of.json** - 5/5 tests
- **testAbs.json** - 4/4 tests
- **testAggregate.json** - 4/4 tests
- **testAll.json** - 4/4 tests
- **testBasics.json** - 7/7 tests
- **testBooleanLogicXOr.json** - 9/9 tests
- **testCase.json** - 4/4 tests
- **testCeiling.json** - 4/4 tests
- **testCombine--.json** - 3/3 tests
- **testContainsString.json** - 12/12 tests
- **testCount.json** - 4/4 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testEndsWith.json** - 12/12 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testExp.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testFloor.json** - 4/4 tests
- **testIif.json** - 12/12 tests
- **testIndexOf.json** - 6/6 tests
- **testIndexer.json** - 2/2 tests
- **testIntersect.json** - 4/4 tests
- **testJoin.json** - 1/1 tests
- **testLength.json** - 6/6 tests
- **testLn.json** - 3/3 tests
- **testLog.json** - 5/5 tests
- **testMatches.json** - 16/16 tests
- **testMinus.json** - 11/11 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests
- **testNow.json** - 2/2 tests
- **testPower.json** - 6/6 tests
- **testRound.json** - 3/3 tests
- **testSkip.json** - 4/4 tests
- **testSplit.json** - 4/4 tests
- **testSqrt.json** - 3/3 tests
- **testStartsWith.json** - 14/14 tests
- **testSuperSetOf.json** - 2/2 tests
- **testTail.json** - 2/2 tests
- **testTake.json** - 7/7 tests
- **testToChars.json** - 1/1 tests
- **testToDecimal.json** - 5/5 tests
- **testToInteger.json** - 5/5 tests
- **testToString.json** - 5/5 tests
- **testToday.json** - 2/2 tests
- **testTrace.json** - 2/2 tests
- **testTrim.json** - 6/6 tests
- **testTruncate.json** - 4/4 tests
- **testWhere.json** - 4/4 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testPlus.json** - 97.1% (33/34 tests)
- **testGreaterThan.json** - 86.7% (26/30 tests)
- **testLessOrEqual.json** - 86.7% (26/30 tests)
- **testCollectionBoolean.json** - 83.3% (5/6 tests)
- **testGreatorOrEqual.json** - 83.3% (25/30 tests)
- **testLessThan.json** - 83.3% (25/30 tests)
- **testNotEquivalent.json** - 81.8% (18/22 tests)
- **testDollar.json** - 80.0% (4/5 tests)
- **testExists.json** - 80.0% (4/5 tests)
- **testEquivalent.json** - 79.2% (19/24 tests)
- **testNEquality.json** - 79.2% (19/24 tests)
- **testBooleanImplies.json** - 77.8% (7/9 tests)
- **testBooleanLogicAnd.json** - 77.8% (7/9 tests)
- **testBooleanLogicOr.json** - 77.8% (7/9 tests)
- **testLiterals.json** - 75.6% (62/82 tests)
- **testEncodeDecode.json** - 75.0% (6/8 tests)
- **testEquality.json** - 75.0% (21/28 tests)
- **testIn.json** - 75.0% (6/8 tests)
- **testUnion.json** - 75.0% (9/12 tests)
- **testVariables.json** - 75.0% (3/4 tests)
- **testTypes.json** - 74.7% (74/99 tests)

### 🟠 Partially Implemented (30-70%)

- **testDistinct.json** - 66.7% (4/6 tests)
- **testMiscellaneousAccessorTests.json** - 66.7% (2/3 tests)
- **testPrecedence.json** - 66.7% (4/6 tests)
- **testSelect.json** - 66.7% (2/3 tests)
- **testSubSetOf.json** - 66.7% (2/3 tests)
- **LowBoundary.json** - 60.7% (17/28 tests)
- **testRepeat.json** - 60.0% (3/5 tests)
- **testSort.json** - 60.0% (6/10 tests)
- **HighBoundary.json** - 58.3% (14/24 tests)
- **comments.json** - 55.6% (5/9 tests)
- **testContainsCollection.json** - 55.6% (5/9 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **testObservations.json** - 50.0% (5/10 tests)
- **testReplace.json** - 50.0% (3/6 tests)
- **testSingle.json** - 50.0% (1/2 tests)
- **testSubstring.json** - 50.0% (6/12 tests)
- **testInheritance.json** - 45.8% (11/24 tests)
- **testReplaceMatches.json** - 42.9% (3/7 tests)
- **testQuantity.json** - 36.4% (4/11 tests)
- **Comparable.json** - 33.3% (1/3 tests)
- **cdaTests.json** - 33.3% (1/3 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testExtension.json** - 33.3% (1/3 tests)
- **testType.json** - 30.0% (9/30 tests)

### 🔴 Major Issues (0-30%)

- **testConcatenate.json** - 25.0% (1/4 tests) - Issues
- **defineVariable.json** - 19.0% (4/21 tests) - Issues
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **from-Zulip.json** - 0.0% (0/2 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **77.9% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 112
- **Total Tests**: 1090
- **Pass Rate**: 77.9%

---

*Report generated on: 2025-09-06 20:34:02*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
