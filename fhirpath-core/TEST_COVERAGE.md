# FHIRPath Test Coverage Report

Generated on: 2025-07-28
Implementation: fhirpath-rs (fhirpath-core)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 98 | 100% |
| **Total Individual Tests** | 1005 | 100% |
| **Passing Tests** | 504 | 50.1% |
| **Failing Tests** | 138 | 13.7% |
| **Error Tests** | 363 | 36.1% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **abs.json** - 4/4 tests
- **basics.json** - 7/7 tests
- **boolean-implies.json** - 9/9 tests
- **boolean-logic-and.json** - 9/9 tests
- **boolean-logic-or.json** - 9/9 tests
- **boolean-logic-x-or.json** - 9/9 tests
- **case.json** - 4/4 tests
- **ceiling.json** - 4/4 tests
- **contains-collection.json** - 9/9 tests
- **exclude.json** - 4/4 tests
- **exp.json** - 3/3 tests
- **floor.json** - 4/4 tests
- **in.json** - 8/8 tests
- **length.json** - 6/6 tests
- **ln.json** - 3/3 tests
- **multiply.json** - 6/6 tests
- **round.json** - 3/3 tests
- **select.json** - 3/3 tests
- **to-chars.json** - 1/1 tests
- **to-decimal.json** - 5/5 tests
- **to-integer.json** - 5/5 tests
- **to-string.json** - 5/5 tests
- **trim.json** - 6/6 tests
- **truncate.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **not-equivalent.json** - 90.9% (20/22 tests)
- **div.json** - 87.5% (7/8 tests)
- **mod.json** - 87.5% (7/8 tests)
- **starts-with.json** - 84.6% (11/13 tests)
- **n-equality.json** - 83.3% (20/24 tests)
- **ends-with.json** - 81.8% (9/11 tests)
- **equivalent.json** - 79.2% (19/24 tests)
- **equality.json** - 78.6% (22/28 tests)
- **greater-than.json** - 76.7% (23/30 tests)
- **greator-or-equal.json** - 76.7% (23/30 tests)
- **less-or-equal.json** - 76.7% (23/30 tests)
- **less-than.json** - 76.7% (23/30 tests)
- **concatenate.json** - 75.0% (3/4 tests)
- **intersect.json** - 75.0% (3/4 tests)

### 🟠 Partially Implemented (30-70%)

- **comments.json** - 66.7% (6/9 tests)
- **sqrt.json** - 66.7% (2/3 tests)
- **substring.json** - 63.6% (7/11 tests)
- **exists.json** - 60.0% (3/5 tests)
- **log.json** - 60.0% (3/5 tests)
- **divide.json** - 55.6% (5/9 tests)
- **minus.json** - 54.5% (6/11 tests)
- **all.json** - 50.0% (2/4 tests)
- **count.json** - 50.0% (2/4 tests)
- **distinct.json** - 50.0% (3/6 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **power.json** - 50.0% (3/6 tests)
- **split.json** - 50.0% (2/4 tests)
- **today.json** - 50.0% (1/2 tests)
- **trace.json** - 50.0% (1/2 tests)
- **types.json** - 44.4% (44/99 tests)
- **literals.json** - 43.9% (36/82 tests)
- **take.json** - 42.9% (3/7 tests)
- **contains-string.json** - 36.4% (4/11 tests)
- **combine.json** - 33.3% (1/3 tests)
- **index-of.json** - 33.3% (2/6 tests)
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests)
- **precedence.json** - 33.3% (2/6 tests)

### 🔴 Major Issues (0-30%)

- **quantity.json** - 27.3% (3/11 tests) - Issues
- **skip.json** - 25.0% (1/4 tests) - Issues
- **plus.json** - 23.5% (8/34 tests) - Issues
- **dollar.json** - 20.0% (1/5 tests) - Issues
- **iif.json** - 18.2% (2/11 tests) - Issues
- **precision.json** - 16.7% (1/6 tests) - Issues
- **replace.json** - 16.7% (1/6 tests) - Issues
- **replace-matches.json** - 14.3% (1/7 tests) - Issues
- **matches.json** - 12.5% (2/16 tests) - Issues
- **sort.json** - 10.0% (1/10 tests) - Issues
- **aggregate.json** - 0.0% (0/4 tests) - Missing
- **cda-tests.json** - 0.0% (0/3 tests) - Missing
- **collection-boolean.json** - 0.0% (0/6 tests) - Missing
- **comparable.json** - 0.0% (0/3 tests) - Missing
- **conforms-to.json** - 0.0% (0/3 tests) - Missing
- **define-variable.json** - 0.0% (0/21 tests) - Missing
- **encode-decode.json** - 0.0% (0/8 tests) - Missing
- **escape-unescape.json** - 0.0% (0/4 tests) - Missing
- **extension.json** - 0.0% (0/3 tests) - Missing
- **first-last.json** - 0.0% (0/2 tests) - Missing
- **high-boundary.json** - 0.0% (0/24 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **indexer.json** - 0.0% (0/2 tests) - Missing
- **inheritance.json** - 0.0% (0/24 tests) - Missing
- **join.json** - 0.0% (0/1 tests) - Missing
- **low-boundary.json** - 0.0% (0/28 tests) - Missing
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing
- **now.json** - 0.0% (0/2 tests) - Missing
- **observations.json** - 0.0% (0/10 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **repeat.json** - 0.0% (0/5 tests) - Missing
- **single.json** - 0.0% (0/2 tests) - Missing
- **sub-set-of.json** - 0.0% (0/3 tests) - Missing
- **super-set-of.json** - 0.0% (0/2 tests) - Missing
- **tail.json** - 0.0% (0/2 tests) - Missing
- **type.json** - 0.0% (0/30 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **50.1% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 98
- **Total Tests**: 1005
- **Pass Rate**: 50.1%

---

*Report generated on: 2025-07-28 17:20:22*  
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
