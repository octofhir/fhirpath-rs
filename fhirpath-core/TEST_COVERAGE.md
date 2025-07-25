# FHIRPath Test Coverage Report

Generated on: 2025-07-25
Implementation: fhirpath-rs (fhirpath-core)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 98 | 100% |
| **Total Individual Tests** | 1005 | 100% |
| **Passing Tests** | 431 | 42.9% |
| **Failing Tests** | 166 | 16.5% |
| **Error Tests** | 408 | 40.6% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **abs.json** - 4/4 tests
- **basics.json** - 7/7 tests
- **ceiling.json** - 4/4 tests
- **contains-collection.json** - 9/9 tests
- **count.json** - 4/4 tests
- **exists.json** - 5/5 tests
- **floor.json** - 4/4 tests
- **in.json** - 8/8 tests
- **ln.json** - 3/3 tests
- **log.json** - 5/5 tests
- **multiply.json** - 6/6 tests
- **round.json** - 3/3 tests
- **sqrt.json** - 3/3 tests
- **to-integer.json** - 5/5 tests
- **to-string.json** - 5/5 tests
- **truncate.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **contains-string.json** - 90.9% (10/11 tests)
- **div.json** - 87.5% (7/8 tests)
- **mod.json** - 87.5% (7/8 tests)
- **not-equivalent.json** - 81.8% (18/22 tests)
- **greater-than.json** - 76.7% (23/30 tests)
- **greator-or-equal.json** - 76.7% (23/30 tests)
- **less-or-equal.json** - 76.7% (23/30 tests)
- **less-than.json** - 76.7% (23/30 tests)
- **equality.json** - 75.0% (21/28 tests)
- **equivalent.json** - 70.8% (17/24 tests)
- **n-equality.json** - 70.8% (17/24 tests)

### 🟠 Partially Implemented (30-70%)

- **to-decimal.json** - 60.0% (3/5 tests)
- **take.json** - 57.1% (4/7 tests)
- **minus.json** - 54.5% (6/11 tests)
- **all.json** - 50.0% (2/4 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **intersect.json** - 50.0% (2/4 tests)
- **single.json** - 50.0% (1/2 tests)
- **skip.json** - 50.0% (2/4 tests)
- **today.json** - 50.0% (1/2 tests)
- **literals.json** - 48.8% (40/82 tests)
- **substring.json** - 45.5% (5/11 tests)
- **boolean-implies.json** - 44.4% (4/9 tests)
- **boolean-logic-or.json** - 44.4% (4/9 tests)
- **boolean-logic-x-or.json** - 44.4% (4/9 tests)
- **types.json** - 41.4% (41/99 tests)
- **dollar.json** - 40.0% (2/5 tests)
- **distinct.json** - 33.3% (2/6 tests)
- **divide.json** - 33.3% (3/9 tests)
- **exp.json** - 33.3% (1/3 tests)
- **length.json** - 33.3% (2/6 tests)
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests)
- **select.json** - 33.3% (1/3 tests)
- **type.json** - 30.0% (9/30 tests)

### 🔴 Major Issues (0-30%)

- **ends-with.json** - 27.3% (3/11 tests) - Issues
- **iif.json** - 27.3% (3/11 tests) - Issues
- **plus.json** - 23.5% (8/34 tests) - Issues
- **starts-with.json** - 23.1% (3/13 tests) - Issues
- **sort.json** - 20.0% (2/10 tests) - Issues
- **precedence.json** - 16.7% (1/6 tests) - Issues
- **observations.json** - 10.0% (1/10 tests) - Issues
- **quantity.json** - 9.1% (1/11 tests) - Issues
- **aggregate.json** - 0.0% (0/4 tests) - Missing
- **boolean-logic-and.json** - 0.0% (0/9 tests) - Missing
- **case.json** - 0.0% (0/4 tests) - Missing
- **cda-tests.json** - 0.0% (0/3 tests) - Missing
- **collection-boolean.json** - 0.0% (0/6 tests) - Missing
- **combine.json** - 0.0% (0/3 tests) - Missing
- **comments.json** - 0.0% (0/9 tests) - Missing
- **comparable.json** - 0.0% (0/3 tests) - Missing
- **concatenate.json** - 0.0% (0/4 tests) - Missing
- **conforms-to.json** - 0.0% (0/3 tests) - Missing
- **define-variable.json** - 0.0% (0/21 tests) - Missing
- **encode-decode.json** - 0.0% (0/8 tests) - Missing
- **escape-unescape.json** - 0.0% (0/4 tests) - Missing
- **exclude.json** - 0.0% (0/4 tests) - Missing
- **extension.json** - 0.0% (0/3 tests) - Missing
- **first-last.json** - 0.0% (0/2 tests) - Missing
- **high-boundary.json** - 0.0% (0/24 tests) - Missing
- **index-of.json** - 0.0% (0/6 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **indexer.json** - 0.0% (0/2 tests) - Missing
- **inheritance.json** - 0.0% (0/24 tests) - Missing
- **join.json** - 0.0% (0/1 tests) - Missing
- **low-boundary.json** - 0.0% (0/28 tests) - Missing
- **matches.json** - 0.0% (0/16 tests) - Missing
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing
- **now.json** - 0.0% (0/2 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **power.json** - 0.0% (0/6 tests) - Missing
- **precision.json** - 0.0% (0/6 tests) - Missing
- **repeat.json** - 0.0% (0/5 tests) - Missing
- **replace-matches.json** - 0.0% (0/7 tests) - Missing
- **replace.json** - 0.0% (0/6 tests) - Missing
- **split.json** - 0.0% (0/4 tests) - Missing
- **sub-set-of.json** - 0.0% (0/3 tests) - Missing
- **super-set-of.json** - 0.0% (0/2 tests) - Missing
- **tail.json** - 0.0% (0/2 tests) - Missing
- **to-chars.json** - 0.0% (0/1 tests) - Missing
- **trace.json** - 0.0% (0/2 tests) - Missing
- **trim.json** - 0.0% (0/6 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **42.9% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 98
- **Total Tests**: 1005
- **Pass Rate**: 42.9%

---

*Report generated on: 2025-07-25 23:36:13*  
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
