# FHIRPath Test Coverage Report

Generated on: 2025-09-18
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 113 | 100% |
| **Total Individual Tests** | 1110 | 100% |
| **Passing Tests** | 881 | 79.4% |
| **Failing Tests** | 136 | 12.3% |
| **Error Tests** | 93 | 8.4% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **Precision.json** - 6/6 tests
- **comments.json** - 9/9 tests
- **index-part.json** - 1/1 tests
- **minimal.json** - 1/1 tests
- **second-of.json** - 5/5 tests
- **testAll.json** - 4/4 tests
- **testBasics.json** - 7/7 tests
- **testBooleanImplies.json** - 9/9 tests
- **testBooleanLogicAnd.json** - 9/9 tests
- **testBooleanLogicOr.json** - 9/9 tests
- **testBooleanLogicXOr.json** - 9/9 tests
- **testCase.json** - 4/4 tests
- **testConcatenate.json** - 4/4 tests
- **testContainsCollection.json** - 9/9 tests
- **testCount.json** - 4/4 tests
- **testDollar.json** - 5/5 tests
- **testEncodeDecode.json** - 8/8 tests
- **testExclude.json** - 4/4 tests
- **testFirstLast.json** - 2/2 tests
- **testIif.json** - 12/12 tests
- **testIndexer.json** - 2/2 tests
- **testIntersect.json** - 4/4 tests
- **testJoin.json** - 1/1 tests
- **testLength.json** - 6/6 tests
- **testMatches.json** - 16/16 tests
- **testMiscellaneousAccessorTests.json** - 3/3 tests
- **testMultiply.json** - 6/6 tests
- **testPlus.json** - 34/34 tests
- **testPrecedence.json** - 6/6 tests
- **testReplace.json** - 6/6 tests
- **testReplaceMatches.json** - 7/7 tests
- **testSelect.json** - 3/3 tests
- **testSingle.json** - 2/2 tests
- **testSkip.json** - 4/4 tests
- **testSort.json** - 10/10 tests
- **testSplit.json** - 4/4 tests
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
- **testVariables.json** - 4/4 tests
- **testWhere.json** - 4/4 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testNEquality.json** - 95.8% (23/24 tests)
- **testEquality.json** - 92.9% (26/28 tests)
- **testStartsWith.json** - 92.9% (13/14 tests)
- **testContainsString.json** - 91.7% (11/12 tests)
- **testEndsWith.json** - 91.7% (11/12 tests)
- **testNotEquivalent.json** - 90.9% (20/22 tests)
- **testDivide.json** - 88.9% (8/9 tests)
- **testDiv.json** - 87.5% (7/8 tests)
- **testIn.json** - 87.5% (7/8 tests)
- **testGreaterThan.json** - 86.7% (26/30 tests)
- **testGreatorOrEqual.json** - 86.7% (26/30 tests)
- **testLessOrEqual.json** - 86.7% (26/30 tests)
- **testLessThan.json** - 86.7% (26/30 tests)
- **testLiterals.json** - 86.6% (71/82 tests)
- **LowBoundary.json** - 85.7% (24/28 tests)
- **day-of.json** - 85.7% (6/7 tests)
- **month-of.json** - 85.7% (6/7 tests)
- **testCollectionBoolean.json** - 83.3% (5/6 tests)
- **testUnion.json** - 83.3% (10/12 tests)
- **hour-of.json** - 80.0% (4/5 tests)
- **minute-of.json** - 80.0% (4/5 tests)
- **testExists.json** - 80.0% (4/5 tests)
- **testEquivalent.json** - 79.2% (19/24 tests)
- **testTypes.json** - 78.8% (78/99 tests)
- **testCeiling.json** - 75.0% (3/4 tests)
- **testFloor.json** - 75.0% (3/4 tests)
- **testMod.json** - 75.0% (6/8 tests)
- **HighBoundary.json** - 70.8% (17/24 tests)

### 🟠 Partially Implemented (30-70%)

- **repeat-all.json** - 68.4% (13/19 tests)
- **testDistinct.json** - 66.7% (4/6 tests)
- **testExp.json** - 66.7% (2/3 tests)
- **testLn.json** - 66.7% (2/3 tests)
- **testRound.json** - 66.7% (2/3 tests)
- **testSubstring.json** - 66.7% (8/12 tests)
- **testMinus.json** - 54.5% (6/11 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **testAbs.json** - 50.0% (2/4 tests)
- **testEscapeUnescape.json** - 50.0% (2/4 tests)
- **testIndexOf.json** - 50.0% (3/6 tests)
- **testNow.json** - 50.0% (1/2 tests)
- **testSuperSetOf.json** - 50.0% (1/2 tests)
- **testLog.json** - 40.0% (2/5 tests)
- **testRepeat.json** - 40.0% (2/5 tests)
- **testType.json** - 40.0% (12/30 tests)
- **testInheritance.json** - 37.5% (9/24 tests)
- **Comparable.json** - 33.3% (1/3 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testPower.json** - 33.3% (2/6 tests)
- **testSqrt.json** - 33.3% (1/3 tests)
- **testSubSetOf.json** - 33.3% (1/3 tests)
- **testObservations.json** - 30.0% (3/10 tests)

### 🔴 Major Issues (0-30%)

- **testQuantity.json** - 18.2% (2/11 tests) - Issues
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **cdaTests.json** - 0.0% (0/3 tests) - Missing
- **defineVariable.json** - 0.0% (0/21 tests) - Missing
- **from-Zulip.json** - 0.0% (0/2 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **resolve.json** - 0.0% (0/3 tests) - Missing
- **testAggregate.json** - 0.0% (0/4 tests) - Missing
- **testCombine--.json** - 0.0% (0/3 tests) - Missing
- **testExtension.json** - 0.0% (0/3 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **79.4% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 113
- **Total Tests**: 1110
- **Pass Rate**: 79.4%

---

*Report generated on: 2025-09-18 07:48:36*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
