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
| **Passing Tests** | 251 | 23.0% |
| **Failing Tests** | 70 | 6.4% |
| **Error Tests** | 769 | 70.6% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **minimal.json** - 1/1 tests
- **testCount.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **testGreaterThan.json** - 86.7% (26/30 tests)
- **testLessOrEqual.json** - 86.7% (26/30 tests)
- **testBasics.json** - 85.7% (6/7 tests)
- **testGreatorOrEqual.json** - 80.0% (24/30 tests)
- **testLessThan.json** - 80.0% (24/30 tests)
- **testVariables.json** - 75.0% (3/4 tests)

### 🟠 Partially Implemented (30-70%)

- **testMiscellaneousAccessorTests.json** - 66.7% (2/3 tests)
- **testNEquality.json** - 62.5% (15/24 tests)
- **testExists.json** - 60.0% (3/5 tests)
- **testEquality.json** - 57.1% (16/28 tests)
- **comments.json** - 55.6% (5/9 tests)
- **from-Zulip.json** - 50.0% (1/2 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **testIndexer.json** - 50.0% (1/2 tests)
- **testMod.json** - 50.0% (4/8 tests)
- **testMultiply.json** - 50.0% (3/6 tests)
- **testSingle.json** - 50.0% (1/2 tests)
- **testMinus.json** - 45.5% (5/11 tests)
- **testContainsCollection.json** - 44.4% (4/9 tests)
- **testDivide.json** - 44.4% (4/9 tests)
- **testLiterals.json** - 36.6% (30/82 tests)
- **defineVariable.json** - 33.3% (7/21 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)
- **testPrecedence.json** - 33.3% (2/6 tests)

### 🔴 Major Issues (0-30%)

- **testConcatenate.json** - 25.0% (1/4 tests) - Issues
- **testIn.json** - 25.0% (2/8 tests) - Issues
- **testWhere.json** - 25.0% (1/4 tests) - Issues
- **testPlus.json** - 20.6% (7/34 tests) - Issues
- **testDollar.json** - 20.0% (1/5 tests) - Issues
- **testObservations.json** - 20.0% (2/10 tests) - Issues
- **timezone-offset-of.json** - 20.0% (1/5 tests) - Issues
- **testCollectionBoolean.json** - 16.7% (1/6 tests) - Issues
- **testContainsString.json** - 16.7% (2/12 tests) - Issues
- **testEndsWith.json** - 16.7% (2/12 tests) - Issues
- **testIif.json** - 16.7% (2/12 tests) - Issues
- **testLength.json** - 16.7% (1/6 tests) - Issues
- **testStartsWith.json** - 14.3% (2/14 tests) - Issues
- **testInheritance.json** - 12.5% (3/24 tests) - Issues
- **year-of.json** - 11.1% (1/9 tests) - Issues
- **testQuantity.json** - 9.1% (1/11 tests) - Issues
- **testSubstring.json** - 8.3% (1/12 tests) - Issues
- **Comparable.json** - 0.0% (0/3 tests) - Missing
- **HighBoundary.json** - 0.0% (0/24 tests) - Missing
- **LowBoundary.json** - 0.0% (0/28 tests) - Missing
- **Precision.json** - 0.0% (0/6 tests) - Missing
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **cdaTests.json** - 0.0% (0/3 tests) - Missing
- **day-of.json** - 0.0% (0/7 tests) - Missing
- **hour-of.json** - 0.0% (0/5 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **minute-of.json** - 0.0% (0/5 tests) - Missing
- **month-of.json** - 0.0% (0/7 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **resolve.json** - 0.0% (0/2 tests) - Missing
- **second-of.json** - 0.0% (0/5 tests) - Missing
- **testAbs.json** - 0.0% (0/4 tests) - Missing
- **testAggregate.json** - 0.0% (0/4 tests) - Missing
- **testAll.json** - 0.0% (0/4 tests) - Missing
- **testBooleanImplies.json** - 0.0% (0/9 tests) - Missing
- **testBooleanLogicAnd.json** - 0.0% (0/9 tests) - Missing
- **testBooleanLogicOr.json** - 0.0% (0/9 tests) - Missing
- **testBooleanLogicXOr.json** - 0.0% (0/9 tests) - Missing
- **testCase.json** - 0.0% (0/4 tests) - Missing
- **testCeiling.json** - 0.0% (0/4 tests) - Missing
- **testCombine--.json** - 0.0% (0/3 tests) - Missing
- **testDistinct.json** - 0.0% (0/6 tests) - Missing
- **testDiv.json** - 0.0% (0/8 tests) - Missing
- **testEncodeDecode.json** - 0.0% (0/8 tests) - Missing
- **testEquivalent.json** - 0.0% (0/24 tests) - Missing
- **testEscapeUnescape.json** - 0.0% (0/4 tests) - Missing
- **testExclude.json** - 0.0% (0/4 tests) - Missing
- **testExp.json** - 0.0% (0/3 tests) - Missing
- **testExtension.json** - 0.0% (0/3 tests) - Missing
- **testFirstLast.json** - 0.0% (0/2 tests) - Missing
- **testFloor.json** - 0.0% (0/4 tests) - Missing
- **testIndexOf.json** - 0.0% (0/6 tests) - Missing
- **testIntersect.json** - 0.0% (0/4 tests) - Missing
- **testJoin.json** - 0.0% (0/1 tests) - Missing
- **testLn.json** - 0.0% (0/3 tests) - Missing
- **testLog.json** - 0.0% (0/5 tests) - Missing
- **testMatches.json** - 0.0% (0/16 tests) - Missing
- **testNotEquivalent.json** - 0.0% (0/22 tests) - Missing
- **testNow.json** - 0.0% (0/2 tests) - Missing
- **testPower.json** - 0.0% (0/6 tests) - Missing
- **testRepeat.json** - 0.0% (0/5 tests) - Missing
- **testReplace.json** - 0.0% (0/6 tests) - Missing
- **testReplaceMatches.json** - 0.0% (0/7 tests) - Missing
- **testRound.json** - 0.0% (0/3 tests) - Missing
- **testSelect.json** - 0.0% (0/3 tests) - Missing
- **testSkip.json** - 0.0% (0/4 tests) - Missing
- **testSort.json** - 0.0% (0/10 tests) - Missing
- **testSplit.json** - 0.0% (0/4 tests) - Missing
- **testSqrt.json** - 0.0% (0/3 tests) - Missing
- **testSubSetOf.json** - 0.0% (0/3 tests) - Missing
- **testSuperSetOf.json** - 0.0% (0/2 tests) - Missing
- **testTail.json** - 0.0% (0/2 tests) - Missing
- **testTake.json** - 0.0% (0/7 tests) - Missing
- **testToChars.json** - 0.0% (0/1 tests) - Missing
- **testToDecimal.json** - 0.0% (0/5 tests) - Missing
- **testToInteger.json** - 0.0% (0/5 tests) - Missing
- **testToString.json** - 0.0% (0/5 tests) - Missing
- **testToday.json** - 0.0% (0/2 tests) - Missing
- **testTrace.json** - 0.0% (0/2 tests) - Missing
- **testTrim.json** - 0.0% (0/6 tests) - Missing
- **testTruncate.json** - 0.0% (0/4 tests) - Missing
- **testType.json** - 0.0% (0/30 tests) - Missing
- **testTypes.json** - 0.0% (0/99 tests) - Missing
- **testUnion.json** - 0.0% (0/12 tests) - Missing
- **to-date.json** - 0.0% (0/11 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **23.0% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 112
- **Total Tests**: 1090
- **Pass Rate**: 23.0%

---

*Report generated on: 2025-09-08 00:36:33*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
