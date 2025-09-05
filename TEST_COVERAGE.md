# FHIRPath Test Coverage Report

Generated on: 2025-09-05
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 111 | 100% |
| **Total Individual Tests** | 1079 | 100% |
| **Passing Tests** | 687 | 63.7% |
| **Failing Tests** | 378 | 35.0% |
| **Error Tests** | 14 | 1.3% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **Precision.json** - 6/6 tests
- **hour-of.json** - 5/5 tests
- **index-part.json** - 1/1 tests
- **minute-of.json** - 5/5 tests
- **second-of.json** - 5/5 tests
- **testAbs.json** - 4/4 tests
- **testAll.json** - 4/4 tests
- **testCase.json** - 4/4 tests
- **testCeiling.json** - 4/4 tests
- **testContainsCollection.json** - 9/9 tests
- **testCount.json** - 4/4 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testExp.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testFloor.json** - 4/4 tests
- **testIndexOf.json** - 6/6 tests
- **testIndexer.json** - 2/2 tests
- **testIntersect.json** - 4/4 tests
- **testJoin.json** - 1/1 tests
- **testLength.json** - 6/6 tests
- **testLn.json** - 3/3 tests
- **testLog.json** - 5/5 tests
- **testMatches.json** - 16/16 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests
- **testNow.json** - 2/2 tests
- **testPower.json** - 6/6 tests
- **testRound.json** - 3/3 tests
- **testSkip.json** - 4/4 tests
- **testSplit.json** - 4/4 tests
- **testSqrt.json** - 3/3 tests
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

### 🟡 Well Implemented (70%+)

- **testIif.json** - 91.7% (11/12 tests)
- **year-of.json** - 88.9% (8/9 tests)
- **testIn.json** - 87.5% (7/8 tests)
- **day-of.json** - 85.7% (6/7 tests)
- **month-of.json** - 85.7% (6/7 tests)
- **testStartsWith.json** - 85.7% (12/14 tests)
- **testCollectionBoolean.json** - 83.3% (5/6 tests)
- **testContainsString.json** - 83.3% (10/12 tests)
- **testEndsWith.json** - 83.3% (10/12 tests)
- **testDollar.json** - 80.0% (4/5 tests)
- **testNEquality.json** - 79.2% (19/24 tests)
- **testGreaterThan.json** - 76.7% (23/30 tests)
- **testGreatorOrEqual.json** - 76.7% (23/30 tests)
- **testLessOrEqual.json** - 76.7% (23/30 tests)
- **testLessThan.json** - 76.7% (23/30 tests)
- **testEncodeDecode.json** - 75.0% (6/8 tests)
- **testEquality.json** - 75.0% (21/28 tests)
- **testTypes.json** - 73.7% (73/99 tests)
- **testBasics.json** - 71.4% (5/7 tests)
- **testLiterals.json** - 70.7% (58/82 tests)

### 🟠 Partially Implemented (30-70%)

- **testDistinct.json** - 66.7% (4/6 tests)
- **testMiscellaneousAccessorTests.json** - 66.7% (2/3 tests)
- **testSelect.json** - 66.7% (2/3 tests)
- **testUnion.json** - 66.7% (8/12 tests)
- **testExists.json** - 60.0% (3/5 tests)
- **testRepeat.json** - 60.0% (3/5 tests)
- **testSort.json** - 60.0% (6/10 tests)
- **testNotEquivalent.json** - 59.1% (13/22 tests)
- **testBooleanLogicXOr.json** - 55.6% (5/9 tests)
- **testMinus.json** - 54.5% (6/11 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **period.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **testEquivalent.json** - 50.0% (12/24 tests)
- **testReplace.json** - 50.0% (3/6 tests)
- **testSingle.json** - 50.0% (1/2 tests)
- **testSubstring.json** - 50.0% (6/12 tests)
- **testVariables.json** - 50.0% (2/4 tests)
- **comments.json** - 44.4% (4/9 tests)
- **testReplaceMatches.json** - 42.9% (3/7 tests)
- **Comparable.json** - 33.3% (1/3 tests)
- **cdaTests.json** - 33.3% (1/3 tests)
- **testBooleanImplies.json** - 33.3% (3/9 tests)
- **testBooleanLogicAnd.json** - 33.3% (3/9 tests)
- **testBooleanLogicOr.json** - 33.3% (3/9 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testExtension.json** - 33.3% (1/3 tests)
- **testPrecedence.json** - 33.3% (2/6 tests)
- **testObservations.json** - 30.0% (3/10 tests)
- **testType.json** - 30.0% (9/30 tests)

### 🔴 Major Issues (0-30%)

- **testEscapeUnescape.json** - 25.0% (1/4 tests) - Issues
- **testExclude.json** - 25.0% (1/4 tests) - Issues
- **testInheritance.json** - 25.0% (6/24 tests) - Issues
- **testPlus.json** - 23.5% (8/34 tests) - Issues
- **testQuantity.json** - 9.1% (1/11 tests) - Issues
- **HighBoundary.json** - 0.0% (0/24 tests) - Missing
- **LowBoundary.json** - 0.0% (0/28 tests) - Missing
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **defineVariable.json** - 0.0% (0/21 tests) - Missing
- **from-Zulip.json** - 0.0% (0/2 tests) - Missing
- **minimal.json** - 0.0% (0/1 tests) - Missing
- **resolve.json** - 0.0% (0/2 tests) - Missing
- **testAggregate.json** - 0.0% (0/4 tests) - Missing
- **testCombine--.json** - 0.0% (0/3 tests) - Missing
- **testConcatenate.json** - 0.0% (0/4 tests) - Missing
- **testSubSetOf.json** - 0.0% (0/3 tests) - Missing
- **testSuperSetOf.json** - 0.0% (0/2 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **63.7% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 111
- **Total Tests**: 1079
- **Pass Rate**: 63.7%

---

*Report generated on: 2025-09-05 14:29:18*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
