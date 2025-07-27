# FHIRPath Test Coverage Report

Generated on: 2025-07-27
Implementation: fhirpath-rs (fhirpath-core)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric                     | Count | Percentage |
|----------------------------|-------|------------|
| **Total Test Suites**      | 98    | 100%       |
| **Total Individual Tests** | 1005  | 100%       |
| **Passing Tests**          | 324   | 32.2%      |
| **Failing Tests**          | 123   | 12.2%      |
| **Error Tests**            | 558   | 55.5%      |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **abs.json** - 4/4 tests
- **all.json** - 4/4 tests
- **basics.json** - 7/7 tests
- **boolean-implies.json** - 9/9 tests
- **boolean-logic-and.json** - 9/9 tests
- **boolean-logic-or.json** - 9/9 tests
- **boolean-logic-x-or.json** - 9/9 tests
- **ceiling.json** - 4/4 tests
- **exclude.json** - 4/4 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
- **in.json** - 8/8 tests
- **intersect.json** - 4/4 tests
- **join.json** - 1/1 tests
- **multiply.json** - 6/6 tests
- **round.json** - 3/3 tests
- **skip.json** - 4/4 tests
- **tail.json** - 2/2 tests
- **take.json** - 7/7 tests

### 🟡 Well Implemented (70%+)

- **contains-collection.json** - 88.9% (8/9 tests)
- **div.json** - 87.5% (7/8 tests)
- **mod.json** - 87.5% (7/8 tests)
- **concatenate.json** - 75.0% (3/4 tests)

### 🟠 Partially Implemented (30-70%)

- **comments.json** - 66.7% (6/9 tests)
- **select.json** - 66.7% (2/3 tests)
- **not-equivalent.json** - 63.6% (14/22 tests)
- **equality.json** - 57.1% (16/28 tests)
- **minus.json** - 54.5% (6/11 tests)
- **equivalent.json** - 54.2% (13/24 tests)
- **count.json** - 50.0% (2/4 tests)
- **distinct.json** - 50.0% (3/6 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **n-equality.json** - 50.0% (12/24 tests)
- **single.json** - 50.0% (1/2 tests)
- **today.json** - 50.0% (1/2 tests)
- **trace.json** - 50.0% (1/2 tests)
- **dollar.json** - 40.0% (2/5 tests)
- **exists.json** - 40.0% (2/5 tests)
- **greater-than.json** - 40.0% (12/30 tests)
- **greator-or-equal.json** - 40.0% (12/30 tests)
- **less-or-equal.json** - 40.0% (12/30 tests)
- **less-than.json** - 40.0% (12/30 tests)
- **to-integer.json** - 40.0% (2/5 tests)
- **contains-string.json** - 36.4% (4/11 tests)
- **combine.json** - 33.3% (1/3 tests)
- **divide.json** - 33.3% (3/9 tests)
- **exp.json** - 33.3% (1/3 tests)
- **index-of.json** - 33.3% (2/6 tests)
- **ln.json** - 33.3% (1/3 tests)
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests)
- **precedence.json** - 33.3% (2/6 tests)
- **sqrt.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

- **truncate.json** - 25.0% (1/4 tests) - Issues
- **plus.json** - 23.5% (8/34 tests) - Issues
- **starts-with.json** - 23.1% (3/13 tests) - Issues
- **literals.json** - 22.0% (18/82 tests) - Issues
- **log.json** - 20.0% (1/5 tests) - Issues
- **to-decimal.json** - 20.0% (1/5 tests) - Issues
- **ends-with.json** - 18.2% (2/11 tests) - Issues
- **iif.json** - 18.2% (2/11 tests) - Issues
- **quantity.json** - 18.2% (2/11 tests) - Issues
- **substring.json** - 18.2% (2/11 tests) - Issues
- **length.json** - 16.7% (1/6 tests) - Issues
- **power.json** - 16.7% (1/6 tests) - Issues
- **precision.json** - 16.7% (1/6 tests) - Issues
- **replace.json** - 16.7% (1/6 tests) - Issues
- **trim.json** - 16.7% (1/6 tests) - Issues
- **replace-matches.json** - 14.3% (1/7 tests) - Issues
- **matches.json** - 12.5% (2/16 tests) - Issues
- **sort.json** - 10.0% (1/10 tests) - Issues
- **types.json** - 2.0% (2/99 tests) - Issues
- **aggregate.json** - 0.0% (0/4 tests) - Missing
- **case.json** - 0.0% (0/4 tests) - Missing
- **cda-tests.json** - 0.0% (0/3 tests) - Missing
- **collection-boolean.json** - 0.0% (0/6 tests) - Missing
- **comparable.json** - 0.0% (0/3 tests) - Missing
- **conforms-to.json** - 0.0% (0/3 tests) - Missing
- **define-variable.json** - 0.0% (0/21 tests) - Missing
- **encode-decode.json** - 0.0% (0/8 tests) - Missing
- **escape-unescape.json** - 0.0% (0/4 tests) - Missing
- **extension.json** - 0.0% (0/3 tests) - Missing
- **high-boundary.json** - 0.0% (0/24 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **indexer.json** - 0.0% (0/2 tests) - Missing
- **inheritance.json** - 0.0% (0/24 tests) - Missing
- **low-boundary.json** - 0.0% (0/28 tests) - Missing
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing
- **now.json** - 0.0% (0/2 tests) - Missing
- **observations.json** - 0.0% (0/10 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **repeat.json** - 0.0% (0/5 tests) - Missing
- **split.json** - 0.0% (0/4 tests) - Missing
- **sub-set-of.json** - 0.0% (0/3 tests) - Missing
- **super-set-of.json** - 0.0% (0/2 tests) - Missing
- **to-chars.json** - 0.0% (0/1 tests) - Missing
- **to-string.json** - 0.0% (0/5 tests) - Missing
- **type.json** - 0.0% (0/30 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **32.2% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 98
- **Total Tests**: 1005
- **Pass Rate**: 32.2%

---

*Report generated on: 2025-07-27 20:39:40*  
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
