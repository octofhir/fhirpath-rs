# FHIRPath Test Coverage Report

Generated on: 2025-09-04
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 110 | 100% |
| **Total Individual Tests** | 1078 | 100% |
| **Passing Tests** | 121 | 11.2% |
| **Failing Tests** | 881 | 81.7% |
| **Error Tests** | 76 | 7.1% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **testDiv.json** - 8/8 tests
- **testMod.json** - 8/8 tests
- **testMultiply.json** - 6/6 tests

### 🟡 Well Implemented (70%+)

- **testDivide.json** - 88.9% (8/9 tests)

### 🟠 Partially Implemented (30-70%)

- **testMinus.json** - 54.5% (6/11 tests)
- **testEquality.json** - 53.6% (15/28 tests)
- **comments.json** - 44.4% (4/9 tests)
- **testDollar.json** - 40.0% (2/5 tests)
- **testSqrt.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **testIn.json** - 25.0% (2/8 tests) - Issues
- **testInheritance.json** - 25.0% (6/24 tests) - Issues
- **testPlus.json** - 23.5% (8/34 tests) - Issues
- **testObservations.json** - 20.0% (2/10 tests) - Issues
- **LowBoundary.json** - 17.9% (5/28 tests) - Issues
- **testGreaterThan.json** - 16.7% (5/30 tests) - Issues
- **testGreatorOrEqual.json** - 16.7% (5/30 tests) - Issues
- **testLessOrEqual.json** - 16.7% (5/30 tests) - Issues
- **testLessThan.json** - 16.7% (5/30 tests) - Issues
- **testPower.json** - 16.7% (1/6 tests) - Issues
- **testPrecedence.json** - 16.7% (1/6 tests) - Issues
- **testBasics.json** - 14.3% (1/7 tests) - Issues
- **testContainsCollection.json** - 11.1% (1/9 tests) - Issues
- **year-of.json** - 11.1% (1/9 tests) - Issues
- **testLiterals.json** - 9.8% (8/82 tests) - Issues
- **HighBoundary.json** - 8.3% (2/24 tests) - Issues
- **testNEquality.json** - 4.2% (1/24 tests) - Issues
- **testTypes.json** - 4.0% (4/99 tests) - Issues
- **Comparable.json** - 0.0% (0/3 tests) - Missing
- **Precision.json** - 0.0% (0/6 tests) - Missing
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **cdaTests.json** - 0.0% (0/3 tests) - Missing
- **day-of.json** - 0.0% (0/7 tests) - Missing
- **defineVariable.json** - 0.0% (0/21 tests) - Missing
- **from-Zulip.json** - 0.0% (0/2 tests) - Missing
- **hour-of.json** - 0.0% (0/5 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **minute-of.json** - 0.0% (0/5 tests) - Missing
- **miscEngineTests.json** - 0.0% (0/2 tests) - Missing
- **month-of.json** - 0.0% (0/7 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
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
- **testCollectionBoolean.json** - 0.0% (0/6 tests) - Missing
- **testCombine--.json** - 0.0% (0/3 tests) - Missing
- **testConcatenate.json** - 0.0% (0/4 tests) - Missing
- **testConformsTo.json** - 0.0% (0/3 tests) - Missing
- **testContainsString.json** - 0.0% (0/12 tests) - Missing
- **testCount.json** - 0.0% (0/4 tests) - Missing
- **testDistinct.json** - 0.0% (0/6 tests) - Missing
- **testEncodeDecode.json** - 0.0% (0/8 tests) - Missing
- **testEndsWith.json** - 0.0% (0/12 tests) - Missing
- **testEquivalent.json** - 0.0% (0/24 tests) - Missing
- **testEscapeUnescape.json** - 0.0% (0/4 tests) - Missing
- **testExclude.json** - 0.0% (0/4 tests) - Missing
- **testExists.json** - 0.0% (0/5 tests) - Missing
- **testExp.json** - 0.0% (0/3 tests) - Missing
- **testExtension.json** - 0.0% (0/3 tests) - Missing
- **testFirstLast.json** - 0.0% (0/2 tests) - Missing
- **testFloor.json** - 0.0% (0/4 tests) - Missing
- **testIif.json** - 0.0% (0/12 tests) - Missing
- **testIndexOf.json** - 0.0% (0/6 tests) - Missing
- **testIndexer.json** - 0.0% (0/2 tests) - Missing
- **testIntersect.json** - 0.0% (0/4 tests) - Missing
- **testJoin.json** - 0.0% (0/1 tests) - Missing
- **testLength.json** - 0.0% (0/6 tests) - Missing
- **testLn.json** - 0.0% (0/3 tests) - Missing
- **testLog.json** - 0.0% (0/5 tests) - Missing
- **testMatches.json** - 0.0% (0/16 tests) - Missing
- **testMiscellaneousAccessorTests.json** - 0.0% (0/3 tests) - Missing
- **testNotEquivalent.json** - 0.0% (0/22 tests) - Missing
- **testNow.json** - 0.0% (0/2 tests) - Missing
- **testQuantity.json** - 0.0% (0/11 tests) - Missing
- **testRepeat.json** - 0.0% (0/5 tests) - Missing
- **testReplace.json** - 0.0% (0/6 tests) - Missing
- **testReplaceMatches.json** - 0.0% (0/7 tests) - Missing
- **testRound.json** - 0.0% (0/3 tests) - Missing
- **testSelect.json** - 0.0% (0/3 tests) - Missing
- **testSingle.json** - 0.0% (0/2 tests) - Missing
- **testSkip.json** - 0.0% (0/4 tests) - Missing
- **testSort.json** - 0.0% (0/10 tests) - Missing
- **testSplit.json** - 0.0% (0/4 tests) - Missing
- **testStartsWith.json** - 0.0% (0/14 tests) - Missing
- **testSubSetOf.json** - 0.0% (0/3 tests) - Missing
- **testSubstring.json** - 0.0% (0/12 tests) - Missing
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
- **testUnion.json** - 0.0% (0/12 tests) - Missing
- **testVariables.json** - 0.0% (0/4 tests) - Missing
- **testWhere.json** - 0.0% (0/4 tests) - Missing
- **timezone-offset-of.json** - 0.0% (0/5 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **11.2% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 110
- **Total Tests**: 1078
- **Pass Rate**: 11.2%

---

*Report generated on: 2025-09-04 23:24:37*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
