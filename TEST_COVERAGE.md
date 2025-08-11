# FHIRPath Test Coverage Report

Generated on: 2025-08-11
Implementation: fhirpath-rs (fhirpath-core)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 98 | 100% |
| **Total Individual Tests** | 1005 | 100% |
| **Passing Tests** | 895 | 89.1% |
| **Failing Tests** | 110 | 10.9% |
| **Error Tests** | 0 | 0.0% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **abs.json** - 4/4 tests
- **aggregate.json** - 4/4 tests
- **all.json** - 4/4 tests
- **basics.json** - 7/7 tests
- **boolean-implies.json** - 9/9 tests
- **boolean-logic-and.json** - 9/9 tests
- **boolean-logic-or.json** - 9/9 tests
- **boolean-logic-x-or.json** - 9/9 tests
- **case.json** - 4/4 tests
- **cda-tests.json** - 3/3 tests
- **ceiling.json** - 4/4 tests
- **collection-boolean.json** - 6/6 tests
- **combine.json** - 3/3 tests
- **comparable.json** - 3/3 tests
- **contains-collection.json** - 9/9 tests
- **contains-string.json** - 11/11 tests
- **count.json** - 4/4 tests
- **div.json** - 8/8 tests
- **divide.json** - 9/9 tests
- **encode-decode.json** - 8/8 tests
- **ends-with.json** - 11/11 tests
- **equivalent.json** - 24/24 tests
- **escape-unescape.json** - 4/4 tests
- **exclude.json** - 4/4 tests
- **exists.json** - 5/5 tests
- **exp.json** - 3/3 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
- **from--zulip.json** - 2/2 tests
- **in.json** - 8/8 tests
- **index-of.json** - 6/6 tests
- **index-part.json** - 1/1 tests
- **indexer.json** - 2/2 tests
- **intersect.json** - 4/4 tests
- **join.json** - 1/1 tests
- **length.json** - 6/6 tests
- **ln.json** - 3/3 tests
- **log.json** - 5/5 tests
- **matches.json** - 16/16 tests
- **minus.json** - 11/11 tests
- **mod.json** - 8/8 tests
- **multiply.json** - 6/6 tests
- **now.json** - 2/2 tests
- **quantity.json** - 11/11 tests
- **replace-matches.json** - 7/7 tests
- **replace.json** - 6/6 tests
- **round.json** - 3/3 tests
- **select.json** - 3/3 tests
- **skip.json** - 4/4 tests
- **sort.json** - 10/10 tests
- **split.json** - 4/4 tests
- **sqrt.json** - 3/3 tests
- **starts-with.json** - 13/13 tests
- **sub-set-of.json** - 3/3 tests
- **substring.json** - 11/11 tests
- **super-set-of.json** - 2/2 tests
- **tail.json** - 2/2 tests
- **take.json** - 7/7 tests
- **to-chars.json** - 1/1 tests
- **to-decimal.json** - 5/5 tests
- **to-integer.json** - 5/5 tests
- **to-string.json** - 5/5 tests
- **today.json** - 2/2 tests
- **trace.json** - 2/2 tests
- **trim.json** - 6/6 tests
- **truncate.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **equality.json** - 96.4% (27/28 tests)
- **n-equality.json** - 95.8% (23/24 tests)
- **literals.json** - 95.1% (78/82 tests)
- **types.json** - 90.9% (90/99 tests)
- **greater-than.json** - 90.0% (27/30 tests)
- **greator-or-equal.json** - 90.0% (27/30 tests)
- **less-or-equal.json** - 90.0% (27/30 tests)
- **less-than.json** - 90.0% (27/30 tests)
- **comments.json** - 88.9% (8/9 tests)
- **not-equivalent.json** - 86.4% (19/22 tests)
- **plus.json** - 85.3% (29/34 tests)
- **distinct.json** - 83.3% (5/6 tests)
- **power.json** - 83.3% (5/6 tests)
- **precedence.json** - 83.3% (5/6 tests)
- **dollar.json** - 80.0% (4/5 tests)
- **define-variable.json** - 76.2% (16/21 tests)
- **concatenate.json** - 75.0% (3/4 tests)
- **iif.json** - 72.7% (8/11 tests)

### 🟠 Partially Implemented (30-70%)

- **low-boundary.json** - 67.9% (19/28 tests)
- **conforms-to.json** - 66.7% (2/3 tests)
- **extension.json** - 66.7% (2/3 tests)
- **high-boundary.json** - 66.7% (16/24 tests)
- **type.json** - 60.0% (18/30 tests)
- **inheritance.json** - 50.0% (12/24 tests)
- **misc-engine-tests.json** - 50.0% (1/2 tests)
- **period.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **precision.json** - 50.0% (3/6 tests)
- **single.json** - 50.0% (1/2 tests)
- **observations.json** - 40.0% (4/10 tests)
- **repeat.json** - 40.0% (2/5 tests)
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

None currently.

## Summary

The fhirpath-rs implementation currently passes approximately **89.1% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 98
- **Total Tests**: 1005
- **Pass Rate**: 89.1%

---

*Report generated on: 2025-08-11 17:05:30*
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
