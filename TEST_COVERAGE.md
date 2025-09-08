# FHIRPath Test Coverage Report

Generated on: 2025-09-08
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 112 | 100% |
| **Total Individual Tests** | 1090 | 100% |
| **Passing Tests** | 655 | 60.1% |
| **Failing Tests** | 186 | 17.1% |
| **Error Tests** | 249 | 22.8% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **minimal.json** - 1/1 tests
- **testCase.json** - 4/4 tests
- **testCount.json** - 4/4 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testFirstLast.json** - 2/2 tests
- **testLn.json** - 3/3 tests
- **testSingle.json** - 2/2 tests
- **testSkip.json** - 4/4 tests
- **testSplit.json** - 4/4 tests
- **testTail.json** - 2/2 tests
- **testTake.json** - 7/7 tests
- **testToChars.json** - 1/1 tests
- **testToString.json** - 5/5 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testContainsCollection.json** - 88.9% (8/9 tests)
- **testGreaterThan.json** - 86.7% (26/30 tests)
- **testLessOrEqual.json** - 86.7% (26/30 tests)
- **testNotEquivalent.json** - 86.4% (19/22 tests)
- **day-of.json** - 85.7% (6/7 tests)
- **month-of.json** - 85.7% (6/7 tests)
- **Precision.json** - 83.3% (5/6 tests)
- **testLength.json** - 83.3% (5/6 tests)
- **testReplace.json** - 83.3% (5/6 tests)
- **testTrim.json** - 83.3% (5/6 tests)
- **testMatches.json** - 81.2% (13/16 tests)
- **hour-of.json** - 80.0% (4/5 tests)
- **minute-of.json** - 80.0% (4/5 tests)
- **second-of.json** - 80.0% (4/5 tests)
- **testExists.json** - 80.0% (4/5 tests)
- **testGreatorOrEqual.json** - 80.0% (24/30 tests)
- **testLessThan.json** - 80.0% (24/30 tests)
- **testToDecimal.json** - 80.0% (4/5 tests)
- **testAbs.json** - 75.0% (3/4 tests)
- **testCeiling.json** - 75.0% (3/4 tests)
- **testContainsString.json** - 75.0% (9/12 tests)
- **testEncodeDecode.json** - 75.0% (6/8 tests)
- **testEndsWith.json** - 75.0% (9/12 tests)
- **testEquivalent.json** - 75.0% (18/24 tests)
- **testFloor.json** - 75.0% (3/4 tests)
- **testIntersect.json** - 75.0% (3/4 tests)
- **testNEquality.json** - 75.0% (18/24 tests)
- **testTruncate.json** - 75.0% (3/4 tests)
- **testVariables.json** - 75.0% (3/4 tests)
- **testTypes.json** - 71.7% (71/99 tests)
- **testEquality.json** - 71.4% (20/28 tests)
- **testStartsWith.json** - 71.4% (10/14 tests)

### 🟠 Partially Implemented (30-70%)

- **testBooleanLogicAnd.json** - 66.7% (6/9 tests)
- **testBooleanLogicOr.json** - 66.7% (6/9 tests)
- **testExp.json** - 66.7% (2/3 tests)
- **testPrecedence.json** - 66.7% (4/6 tests)
- **testRound.json** - 66.7% (2/3 tests)
- **testUnion.json** - 66.7% (8/12 tests)
- **testIn.json** - 62.5% (5/8 tests)
- **testLiterals.json** - 62.2% (51/82 tests)
- **LowBoundary.json** - 60.7% (17/28 tests)
- **testToInteger.json** - 60.0% (3/5 tests)
- **HighBoundary.json** - 58.3% (14/24 tests)
- **comments.json** - 55.6% (5/9 tests)
- **testDivide.json** - 55.6% (5/9 tests)
- **from-Zulip.json** - 50.0% (1/2 tests)
- **testCollectionBoolean.json** - 50.0% (3/6 tests)
- **testDiv.json** - 50.0% (4/8 tests)
- **testIif.json** - 50.0% (6/12 tests)
- **testIndexOf.json** - 50.0% (3/6 tests)
- **testMod.json** - 50.0% (4/8 tests)
- **testMultiply.json** - 50.0% (3/6 tests)
- **testNow.json** - 50.0% (1/2 tests)
- **testToday.json** - 50.0% (1/2 tests)
- **testMinus.json** - 45.5% (5/11 tests)
- **testBooleanImplies.json** - 44.4% (4/9 tests)
- **testBooleanLogicXOr.json** - 44.4% (4/9 tests)
- **testBasics.json** - 42.9% (3/7 tests)
- **testReplaceMatches.json** - 42.9% (3/7 tests)
- **testDollar.json** - 40.0% (2/5 tests)
- **testLog.json** - 40.0% (2/5 tests)
- **Comparable.json** - 33.3% (1/3 tests)
- **cdaTests.json** - 33.3% (1/3 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testDistinct.json** - 33.3% (2/6 tests)
- **testExtension.json** - 33.3% (1/3 tests)
- **testPower.json** - 33.3% (2/6 tests)
- **testSqrt.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **testType.json** - 26.7% (8/30 tests) - Issues
- **testInheritance.json** - 25.0% (6/24 tests) - Issues
- **testWhere.json** - 25.0% (1/4 tests) - Issues
- **testPlus.json** - 20.6% (7/34 tests) - Issues
- **testSubstring.json** - 16.7% (2/12 tests) - Issues
- **testQuantity.json** - 9.1% (1/11 tests) - Issues
- **defineVariable.json** - 4.8% (1/21 tests) - Issues
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **miscEngineTests.json** - 0.0% (0/2 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **resolve.json** - 0.0% (0/2 tests) - Missing
- **testAggregate.json** - 0.0% (0/4 tests) - Missing
- **testAll.json** - 0.0% (0/4 tests) - Missing
- **testCombine--.json** - 0.0% (0/3 tests) - Missing
- **testConcatenate.json** - 0.0% (0/4 tests) - Missing
- **testIndexer.json** - 0.0% (0/2 tests) - Missing
- **testJoin.json** - 0.0% (0/1 tests) - Missing
- **testMiscellaneousAccessorTests.json** - 0.0% (0/3 tests) - Missing
- **testObservations.json** - 0.0% (0/10 tests) - Missing
- **testRepeat.json** - 0.0% (0/5 tests) - Missing
- **testSelect.json** - 0.0% (0/3 tests) - Missing
- **testSort.json** - 0.0% (0/10 tests) - Missing
- **testSubSetOf.json** - 0.0% (0/3 tests) - Missing
- **testSuperSetOf.json** - 0.0% (0/2 tests) - Missing
- **testTrace.json** - 0.0% (0/2 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **60.1% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 112
- **Total Tests**: 1090
- **Pass Rate**: 60.1%

---

*Report generated on: 2025-09-08 09:06:31*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
