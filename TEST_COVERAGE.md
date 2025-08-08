# FHIRPath Test Coverage Report

Generated on: 2025-08-08
Implementation: fhirpath-rs (fhirpath-core)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 98 | 100% |
| **Total Individual Tests** | 1005 | 100% |
| **Passing Tests** | 881 | 87.7% |
| **Failing Tests** | 111 | 11.0% |
| **Error Tests** | 13 | 1.3% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **abs.json** - 4/4 tests
- **aggregate.json** - 4/4 tests
- **all.json** - 4/4 tests
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
- **escape-unescape.json** - 4/4 tests
- **exclude.json** - 4/4 tests
- **exists.json** - 5/5 tests
- **exp.json** - 3/3 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
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

- **n-equality.json** - 95.8% (23/24 tests)
- **equality.json** - 92.9% (26/28 tests)
- **literals.json** - 92.7% (76/82 tests)
- **types.json** - 90.9% (90/99 tests)
- **comments.json** - 88.9% (8/9 tests)
- **greater-than.json** - 86.7% (26/30 tests)
- **greator-or-equal.json** - 86.7% (26/30 tests)
- **less-or-equal.json** - 86.7% (26/30 tests)
- **less-than.json** - 86.7% (26/30 tests)
- **not-equivalent.json** - 86.4% (19/22 tests)
- **basics.json** - 85.7% (6/7 tests)
- **plus.json** - 85.3% (29/34 tests)
- **distinct.json** - 83.3% (5/6 tests)
- **power.json** - 83.3% (5/6 tests)
- **dollar.json** - 80.0% (4/5 tests)
- **define-variable.json** - 76.2% (16/21 tests)
- **concatenate.json** - 75.0% (3/4 tests)
- **equivalent.json** - 75.0% (18/24 tests)
- **iif.json** - 72.7% (8/11 tests)

### 🟠 Partially Implemented (30-70%)

- **low-boundary.json** - 67.9% (19/28 tests)
- **conforms-to.json** - 66.7% (2/3 tests)
- **high-boundary.json** - 66.7% (16/24 tests)
- **precedence.json** - 66.7% (4/6 tests)
- **type.json** - 66.7% (20/30 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **inheritance.json** - 50.0% (12/24 tests)
- **misc-engine-tests.json** - 50.0% (1/2 tests)
- **observations.json** - 50.0% (5/10 tests)
- **period.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **precision.json** - 50.0% (3/6 tests)
- **single.json** - 50.0% (1/2 tests)
- **repeat.json** - 40.0% (2/5 tests)
- **extension.json** - 33.3% (1/3 tests)
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests)

### 🔴 Major Issues (0-30%)

None currently.

## Summary

The fhirpath-rs implementation currently passes approximately **87.7% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 98
- **Total Tests**: 1005
- **Pass Rate**: 87.7%

---

*Report generated on: 2025-08-08 19:27:42*
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
