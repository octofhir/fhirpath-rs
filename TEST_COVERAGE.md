# FHIRPath Test Coverage Report

Generated on: 2025-09-16
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 113 | 100% |
| **Total Individual Tests** | 1110 | 100% |
| **Passing Tests** | 56 | 5.0% |
| **Failing Tests** | 39 | 3.5% |
| **Error Tests** | 1015 | 91.4% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **minimal.json** - 1/1 tests
- **testBasics.json** - 7/7 tests
- **testMiscellaneousAccessorTests.json** - 3/3 tests

### 🟡 Well Implemented (70%+)

None currently.

### 🟠 Partially Implemented (30-70%)

- **repeat-all.json** - 52.6% (10/19 tests)
- **from-Zulip.json** - 50.0% (1/2 tests)
- **miscEngineTests.json** - 50.0% (1/2 tests)
- **testIndexer.json** - 50.0% (1/2 tests)
- **testSingle.json** - 50.0% (1/2 tests)
- **testConformsTo.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **testConcatenate.json** - 25.0% (1/4 tests) - Issues
- **comments.json** - 22.2% (2/9 tests) - Issues
- **hour-of.json** - 20.0% (1/5 tests) - Issues
- **minute-of.json** - 20.0% (1/5 tests) - Issues
- **second-of.json** - 20.0% (1/5 tests) - Issues
- **timezone-offset-of.json** - 20.0% (1/5 tests) - Issues
- **testMinus.json** - 18.2% (2/11 tests) - Issues
- **testPrecedence.json** - 16.7% (1/6 tests) - Issues
- **day-of.json** - 14.3% (1/7 tests) - Issues
- **month-of.json** - 14.3% (1/7 tests) - Issues
- **testIn.json** - 12.5% (1/8 tests) - Issues
- **testInheritance.json** - 12.5% (3/24 tests) - Issues
- **year-of.json** - 11.1% (1/9 tests) - Issues
- **testObservations.json** - 10.0% (1/10 tests) - Issues
- **testLiterals.json** - 9.8% (8/82 tests) - Issues
- **testIif.json** - 8.3% (1/12 tests) - Issues
- **testPlus.json** - 5.9% (2/34 tests) - Issues
- **testEquality.json** - 3.6% (1/28 tests) - Issues
- **Comparable.json** - 0.0% (0/3 tests) - Missing
- **HighBoundary.json** - 0.0% (0/24 tests) - Missing
- **LowBoundary.json** - 0.0% (0/28 tests) - Missing
- **Precision.json** - 0.0% (0/6 tests) - Missing
- **TerminologyTests.json** - 0.0% (0/3 tests) - Missing
- **cdaTests.json** - 0.0% (0/3 tests) - Missing
- **defineVariable.json** - 0.0% (0/21 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **resolve.json** - 0.0% (0/3 tests) - Missing
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
- **testContainsCollection.json** - 0.0% (0/9 tests) - Missing
- **testContainsString.json** - 0.0% (0/12 tests) - Missing
- **testCount.json** - 0.0% (0/4 tests) - Missing
- **testDistinct.json** - 0.0% (0/6 tests) - Missing
- **testDiv.json** - 0.0% (0/8 tests) - Missing
- **testDivide.json** - 0.0% (0/9 tests) - Missing
- **testDollar.json** - 0.0% (0/5 tests) - Missing
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
- **testGreaterThan.json** - 0.0% (0/30 tests) - Missing
- **testGreatorOrEqual.json** - 0.0% (0/30 tests) - Missing
- **testIndexOf.json** - 0.0% (0/6 tests) - Missing
- **testIntersect.json** - 0.0% (0/4 tests) - Missing
- **testJoin.json** - 0.0% (0/1 tests) - Missing
- **testLength.json** - 0.0% (0/6 tests) - Missing
- **testLessOrEqual.json** - 0.0% (0/30 tests) - Missing
- **testLessThan.json** - 0.0% (0/30 tests) - Missing
- **testLn.json** - 0.0% (0/3 tests) - Missing
- **testLog.json** - 0.0% (0/5 tests) - Missing
- **testMatches.json** - 0.0% (0/16 tests) - Missing
- **testMod.json** - 0.0% (0/8 tests) - Missing
- **testMultiply.json** - 0.0% (0/6 tests) - Missing
- **testNEquality.json** - 0.0% (0/24 tests) - Missing
- **testNotEquivalent.json** - 0.0% (0/22 tests) - Missing
- **testNow.json** - 0.0% (0/2 tests) - Missing
- **testPower.json** - 0.0% (0/6 tests) - Missing
- **testQuantity.json** - 0.0% (0/11 tests) - Missing
- **testRepeat.json** - 0.0% (0/5 tests) - Missing
- **testReplace.json** - 0.0% (0/6 tests) - Missing
- **testReplaceMatches.json** - 0.0% (0/7 tests) - Missing
- **testRound.json** - 0.0% (0/3 tests) - Missing
- **testSelect.json** - 0.0% (0/3 tests) - Missing
- **testSkip.json** - 0.0% (0/4 tests) - Missing
- **testSort.json** - 0.0% (0/10 tests) - Missing
- **testSplit.json** - 0.0% (0/4 tests) - Missing
- **testSqrt.json** - 0.0% (0/3 tests) - Missing
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
- **testTypes.json** - 0.0% (0/99 tests) - Missing
- **testUnion.json** - 0.0% (0/12 tests) - Missing
- **testVariables.json** - 0.0% (0/4 tests) - Missing
- **testWhere.json** - 0.0% (0/4 tests) - Missing
- **to-date.json** - 0.0% (0/11 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **5.0% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 113
- **Total Tests**: 1110
- **Pass Rate**: 5.0%

---

*Report generated on: 2025-09-16 21:55:58*
*Command: `just test-coverage` or `cargo run --package octofhir-fhirpath --bin test-coverage`*
