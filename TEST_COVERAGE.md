# FHIRPath Test Coverage Report

Generated on: 2025-09-08
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 113 | 100% |
| **Total Individual Tests** | 1110 | 100% |
| **Passing Tests** | 886 | 79.8% |
| **Failing Tests** | 163 | 14.7% |
| **Error Tests** | 61 | 5.5% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **Precision.json** - 6/6 tests
- **day-of.json** - 7/7 tests
- **from-Zulip.json** - 2/2 tests
- **hour-of.json** - 5/5 tests
- **minimal.json** - 1/1 tests
- **minute-of.json** - 5/5 tests
- **month-of.json** - 7/7 tests
- **resolve.json** - 3/3 tests
- **second-of.json** - 5/5 tests
- **testAbs.json** - 4/4 tests
- **testBooleanImplies.json** - 9/9 tests
- **testBooleanLogicAnd.json** - 9/9 tests
- **testBooleanLogicOr.json** - 9/9 tests
- **testBooleanLogicXOr.json** - 9/9 tests
- **testCase.json** - 4/4 tests
- **testCeiling.json** - 4/4 tests
- **testConcatenate.json** - 4/4 tests
- **testContainsCollection.json** - 9/9 tests
- **testContainsString.json** - 12/12 tests
- **testCount.json** - 4/4 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testEndsWith.json** - 12/12 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testExists.json** - 5/5 tests
- **testExp.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testFloor.json** - 4/4 tests
- **testJoin.json** - 1/1 tests
- **testLength.json** - 6/6 tests
- **testLn.json** - 3/3 tests
- **testMatches.json** - 16/16 tests
- **testMinus.json** - 11/11 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests
- **testPlus.json** - 34/34 tests
- **testReplace.json** - 6/6 tests
- **testRound.json** - 3/3 tests
- **testSingle.json** - 2/2 tests
- **testSkip.json** - 4/4 tests
- **testSplit.json** - 4/4 tests
- **testStartsWith.json** - 14/14 tests
- **testSubstring.json** - 12/12 tests
- **testSuperSetOf.json** - 2/2 tests
- **testTail.json** - 2/2 tests
- **testTake.json** - 7/7 tests
- **testToChars.json** - 1/1 tests
- **testToDecimal.json** - 5/5 tests
- **testToInteger.json** - 5/5 tests
- **testToString.json** - 5/5 tests
- **testToday.json** - 2/2 tests
- **testTrim.json** - 6/6 tests
- **testTruncate.json** - 4/4 tests
- **testWhere.json** - 4/4 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testGreaterThan.json** - 96.7% (29/30 tests)
- **testLessThan.json** - 96.7% (29/30 tests)
- **testUnion.json** - 91.7% (11/12 tests)
- **testGreatorOrEqual.json** - 90.0% (27/30 tests)
- **testLessOrEqual.json** - 90.0% (27/30 tests)
- **comments.json** - 88.9% (8/9 tests)
- **testIn.json** - 87.5% (7/8 tests)
- **testBasics.json** - 85.7% (6/7 tests)
- **testLiterals.json** - 84.1% (69/82 tests)
- **testTypes.json** - 83.8% (83/99 tests)
- **testCollectionBoolean.json** - 83.3% (5/6 tests)
- **testIndexOf.json** - 83.3% (5/6 tests)
- **testNotEquivalent.json** - 81.8% (18/22 tests)
- **testDollar.json** - 80.0% (4/5 tests)
- **testLog.json** - 80.0% (4/5 tests)
- **testSort.json** - 80.0% (8/10 tests)
- **testEncodeDecode.json** - 75.0% (6/8 tests)
- **testIntersect.json** - 75.0% (3/4 tests)
- **testNEquality.json** - 75.0% (18/24 tests)
- **testVariables.json** - 75.0% (3/4 tests)
- **repeat-all.json** - 73.7% (14/19 tests)
- **testEquality.json** - 71.4% (20/28 tests)
- **testEquivalent.json** - 70.8% (17/24 tests)

### 🟠 Partially Implemented (30-70%)

- **testCombine--.json** - 66.7% (2/3 tests)
- **testMiscellaneousAccessorTests.json** - 66.7% (2/3 tests)
- **testPower.json** - 66.7% (4/6 tests)
- **testPrecedence.json** - 66.7% (4/6 tests)
- **testSelect.json** - 66.7% (2/3 tests)
- **testSqrt.json** - 66.7% (2/3 tests)
- **testSubSetOf.json** - 66.7% (2/3 tests)
- **LowBoundary.json** - 60.7% (17/28 tests)
- **HighBoundary.json** - 58.3% (14/24 tests)
- **testReplaceMatches.json** - 57.1% (4/7 tests)
- **testQuantity.json** - 54.5% (6/11 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **testAll.json** - 50.0% (2/4 tests)
- **testIif.json** - 50.0% (6/12 tests)
- **testNow.json** - 50.0% (1/2 tests)
- **testTrace.json** - 50.0% (1/2 tests)
- **testInheritance.json** - 37.5% (9/24 tests)
- **testType.json** - 36.7% (11/30 tests)
- **Comparable.json** - 33.3% (1/3 tests)
- **cdaTests.json** - 33.3% (1/3 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testDistinct.json** - 33.3% (2/6 tests)
- **testExtension.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **testAggregate.json** - 25.0% (1/4 tests) - Issues
- **defineVariable.json** - 4.8% (1/21 tests) - Issues
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **testIndexer.json** - 0.0% (0/2 tests) - Missing
- **testObservations.json** - 0.0% (0/10 tests) - Missing
- **testRepeat.json** - 0.0% (0/5 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **79.8% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 113
- **Total Tests**: 1110
- **Pass Rate**: 79.8%

---

*Report generated on: 2025-09-08 21:39:13*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
