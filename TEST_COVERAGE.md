# FHIRPath Test Coverage Report

Generated on: 2025-09-20
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 114 | 100% |
| **Total Individual Tests** | 1118 | 100% |
| **Passing Tests** | 1021 | 91.3% |
| **Failing Tests** | 89 | 8.0% |
| **Error Tests** | 8 | 0.7% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **Comparable.json** - 3/3 tests
- **Precision.json** - 6/6 tests
- **cdaTests.json** - 3/3 tests
- **comments.json** - 9/9 tests
- **day-of.json** - 7/7 tests
- **from-Zulip.json** - 2/2 tests
- **hour-of.json** - 5/5 tests
- **index-part.json** - 1/1 tests
- **minimal.json** - 1/1 tests
- **minute-of.json** - 5/5 tests
- **month-of.json** - 7/7 tests
- **period.json** - 2/2 tests
- **polymorphics.json** - 2/2 tests
- **resolve.json** - 3/3 tests
- **second-of.json** - 5/5 tests
- **testAbs.json** - 4/4 tests
- **testAggregate.json** - 4/4 tests
- **testAll.json** - 4/4 tests
- **testBasics.json** - 7/7 tests
- **testBooleanImplies.json** - 9/9 tests
- **testBooleanLogicAnd.json** - 9/9 tests
- **testBooleanLogicOr.json** - 9/9 tests
- **testBooleanLogicXOr.json** - 9/9 tests
- **testCase.json** - 4/4 tests
- **testCeiling.json** - 4/4 tests
- **testCollectionBoolean.json** - 6/6 tests
- **testConcatenate.json** - 4/4 tests
- **testConformsTo.json** - 3/3 tests
- **testContainsCollection.json** - 9/9 tests
- **testCount.json** - 4/4 tests
- **testDistinct.json** - 6/6 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testEncodeDecode.json** - 8/8 tests
- **testEquality.json** - 28/28 tests
- **testEquivalent.json** - 24/24 tests
- **testEscapeHtmlCustom.json** - 8/8 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testExists.json** - 5/5 tests
- **testExp.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testFloor.json** - 4/4 tests
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
- **testMinus.json** - 11/11 tests
- **testMiscellaneousAccessorTests.json** - 3/3 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests
- **testNEquality.json** - 24/24 tests
- **testNotEquivalent.json** - 22/22 tests
- **testNow.json** - 2/2 tests
- **testObservations.json** - 10/10 tests
- **testPlus.json** - 34/34 tests
- **testPower.json** - 6/6 tests
- **testPrecedence.json** - 6/6 tests
- **testQuantity.json** - 11/11 tests
- **testReplace.json** - 6/6 tests
- **testReplaceMatches.json** - 7/7 tests
- **testRound.json** - 3/3 tests
- **testSelect.json** - 3/3 tests
- **testSingle.json** - 2/2 tests
- **testSkip.json** - 4/4 tests
- **testSort.json** - 10/10 tests
- **testSplit.json** - 4/4 tests
- **testSqrt.json** - 3/3 tests
- **testStartsWith.json** - 14/14 tests
- **testSubstring.json** - 12/12 tests
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
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testInheritance.json** - 95.8% (23/24 tests)
- **testContainsString.json** - 91.7% (11/12 tests)
- **testEndsWith.json** - 91.7% (11/12 tests)
- **testLiterals.json** - 91.5% (75/82 tests)
- **testTypes.json** - 88.9% (88/99 tests)
- **testIif.json** - 83.3% (10/12 tests)
- **testUnion.json** - 83.3% (10/12 tests)
- **testDollar.json** - 80.0% (4/5 tests)
- **testWhere.json** - 75.0% (3/4 tests)
- **repeat-all.json** - 73.7% (14/19 tests)

### 🟠 Partially Implemented (30-70%)

- **testCombine--.json** - 66.7% (2/3 tests)
- **testSubSetOf.json** - 66.7% (2/3 tests)
- **testRepeat.json** - 60.0% (3/5 tests)
- **testType.json** - 56.7% (17/30 tests)
- **defineVariable.json** - 52.4% (11/21 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **testSuperSetOf.json** - 50.0% (1/2 tests)
- **LowBoundary.json** - 46.4% (13/28 tests)
- **HighBoundary.json** - 33.3% (8/24 tests)
- **testExtension.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **91.3% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 114
- **Total Tests**: 1118
- **Pass Rate**: 91.3%

---

*Report generated on: 2025-09-20 21:28:09*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
