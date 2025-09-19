# FHIRPath Test Coverage Report

Generated on: 2025-09-19
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 114 | 100% |
| **Total Individual Tests** | 1118 | 100% |
| **Passing Tests** | 994 | 88.9% |
| **Failing Tests** | 91 | 8.1% |
| **Error Tests** | 33 | 3.0% |

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
- **polymorphics.json** - 2/2 tests
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
- **testConcatenate.json** - 4/4 tests
- **testConformsTo.json** - 3/3 tests
- **testContainsCollection.json** - 9/9 tests
- **testContainsString.json** - 12/12 tests
- **testCount.json** - 4/4 tests
- **testDistinct.json** - 6/6 tests
- **testDiv.json** - 8/8 tests
- **testDivide.json** - 9/9 tests
- **testDollar.json** - 5/5 tests
- **testEncodeDecode.json** - 8/8 tests
- **testEndsWith.json** - 12/12 tests
- **testEscapeHtmlCustom.json** - 8/8 tests
- **testEscapeUnescape.json** - 4/4 tests
- **testExclude.json** - 4/4 tests
- **testExists.json** - 5/5 tests
- **testExp.json** - 3/3 tests
- **testFirstLast.json** - 2/2 tests
- **testFloor.json** - 4/4 tests
- **testIif.json** - 12/12 tests
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
- **testNEquality.json** - 24/24 tests
- **testNotEquivalent.json** - 22/22 tests
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
- **testUnion.json** - 91.7% (11/12 tests)
- **testLiterals.json** - 86.6% (71/82 tests)
- **testTypes.json** - 84.8% (84/99 tests)
- **testCollectionBoolean.json** - 83.3% (5/6 tests)
- **LowBoundary.json** - 75.0% (21/28 tests)
- **testVariables.json** - 75.0% (3/4 tests)
- **testObservations.json** - 70.0% (7/10 tests)

### 🟠 Partially Implemented (30-70%)

- **repeat-all.json** - 68.4% (13/19 tests)
- **HighBoundary.json** - 58.3% (14/24 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **testNow.json** - 50.0% (1/2 tests)
- **testSuperSetOf.json** - 50.0% (1/2 tests)
- **testType.json** - 46.7% (14/30 tests)
- **testInheritance.json** - 41.7% (10/24 tests)
- **testRepeat.json** - 40.0% (2/5 tests)
- **defineVariable.json** - 38.1% (8/21 tests)
- **testCombine--.json** - 33.3% (1/3 tests)
- **testExtension.json** - 33.3% (1/3 tests)
- **testSubSetOf.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **resolve.json** - 0.0% (0/3 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **88.9% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 114
- **Total Tests**: 1118
- **Pass Rate**: 88.9%

---

*Report generated on: 2025-09-19 15:19:11*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
