# FHIRPath Test Coverage Report

Generated on: 2025-09-10
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 113 | 100% |
| **Total Individual Tests** | 1110 | 100% |
| **Passing Tests** | 1009 | 90.9% |
| **Failing Tests** | 92 | 8.3% |
| **Error Tests** | 9 | 0.8% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **LowBoundary.json** - 28/28 tests
- **Precision.json** - 6/6 tests
- **comments.json** - 9/9 tests
- **day-of.json** - 7/7 tests
- **from-Zulip.json** - 2/2 tests
- **hour-of.json** - 5/5 tests
- **index-part.json** - 1/1 tests
- **minimal.json** - 1/1 tests
- **minute-of.json** - 5/5 tests
- **miscEngineTests.json** - 2/2 tests
- **month-of.json** - 7/7 tests
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
- **testCombine--.json** - 3/3 tests
- **testConcatenate.json** - 4/4 tests
- **testContainsCollection.json** - 9/9 tests
- **testContainsString.json** - 12/12 tests
- **testCount.json** - 4/4 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testEncodeDecode.json** - 8/8 tests
- **testEndsWith.json** - 12/12 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testExists.json** - 5/5 tests
- **testExp.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testFloor.json** - 4/4 tests
- **testIn.json** - 8/8 tests
- **testIndexOf.json** - 6/6 tests
- **testIndexer.json** - 2/2 tests
- **testIntersect.json** - 4/4 tests
- **testJoin.json** - 1/1 tests
- **testLength.json** - 6/6 tests
- **testLn.json** - 3/3 tests
- **testLog.json** - 5/5 tests
- **testMatches.json** - 16/16 tests
- **testMinus.json** - 11/11 tests
- **testMiscellaneousAccessorTests.json** - 3/3 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests
- **testNotEquivalent.json** - 22/22 tests
- **testNow.json** - 2/2 tests
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
- **testSubSetOf.json** - 3/3 tests
- **testSubstring.json** - 12/12 tests
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
- **testUnion.json** - 12/12 tests
- **testVariables.json** - 4/4 tests
- **testWhere.json** - 4/4 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-date.json** - 11/11 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **testGreaterThan.json** - 96.7% (29/30 tests)
- **testGreatorOrEqual.json** - 96.7% (29/30 tests)
- **testLessOrEqual.json** - 96.7% (29/30 tests)
- **testLessThan.json** - 96.7% (29/30 tests)
- **testEquality.json** - 96.4% (27/28 tests)
- **testEquivalent.json** - 95.8% (23/24 tests)
- **testNEquality.json** - 95.8% (23/24 tests)
- **testTypes.json** - 92.9% (92/99 tests)
- **testLiterals.json** - 91.5% (75/82 tests)
- **testDollar.json** - 80.0% (4/5 tests)
- **repeat-all.json** - 73.7% (14/19 tests)

### 🟠 Partially Implemented (30-70%)

- **testDistinct.json** - 66.7% (4/6 tests)
- **testRepeat.json** - 60.0% (3/5 tests)
- **HighBoundary.json** - 54.2% (13/24 tests)
- **testInheritance.json** - 54.2% (13/24 tests)
- **period.json** - 50.0% (1/2 tests)
- **testIif.json** - 50.0% (6/12 tests)
- **testObservations.json** - 50.0% (5/10 tests)
- **testType.json** - 50.0% (15/30 tests)
- **defineVariable.json** - 47.6% (10/21 tests)
- **Comparable.json** - 33.3% (1/3 tests)
- **TerminologyTests.json** - 33.3% (1/3 tests)
- **cdaTests.json** - 33.3% (1/3 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testExtension.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

None currently.

## Summary

The fhirpath-rs implementation currently passes approximately **90.9% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 113
- **Total Tests**: 1110
- **Pass Rate**: 90.9%

---

*Report generated on: 2025-09-10 22:39:16*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
