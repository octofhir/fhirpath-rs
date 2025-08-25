# FHIRPath Test Coverage Report

Generated on: 2025-08-25
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 111 | 100% |
| **Total Individual Tests** | 1104 | 100% |
| **Passing Tests** | 979 | 88.7% |
| **Failing Tests** | 95 | 8.6% |
| **Error Tests** | 30 | 2.7% |

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
- **comments.json** - 9/9 tests
- **comparable.json** - 3/3 tests
- **concatenate.json** - 4/4 tests
- **conforms-to.json** - 3/3 tests
- **contains-collection.json** - 9/9 tests
- **contains-string.json** - 11/11 tests
- **count.json** - 4/4 tests
- **day-of.json** - 7/7 tests
- **distinct.json** - 6/6 tests
- **div.json** - 8/8 tests
- **divide.json** - 9/9 tests
- **encode-decode.json** - 8/8 tests
- **ends-with.json** - 11/11 tests
- **equivalent.json** - 24/24 tests
- **escape-unescape.json** - 4/4 tests
- **exclude.json** - 4/4 tests
- **exp.json** - 3/3 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
- **hour-of.json** - 5/5 tests
- **index-of.json** - 6/6 tests
- **index-part.json** - 1/1 tests
- **indexer.json** - 2/2 tests
- **intersect.json** - 4/4 tests
- **join.json** - 1/1 tests
- **length.json** - 6/6 tests
- **literals.json** - 82/82 tests
- **ln.json** - 3/3 tests
- **log.json** - 5/5 tests
- **millisecond-of.json** - 5/5 tests
- **minus.json** - 11/11 tests
- **minute-of.json** - 5/5 tests
- **misc-engine-tests.json** - 2/2 tests
- **mod.json** - 8/8 tests
- **month-of.json** - 7/7 tests
- **multiply.json** - 6/6 tests
- **not-equivalent.json** - 22/22 tests
- **period.json** - 2/2 tests
- **quantity.json** - 11/11 tests
- **repeat.json** - 5/5 tests
- **replace-matches.json** - 7/7 tests
- **resolve.json** - 2/2 tests
- **round.json** - 3/3 tests
- **second-of.json** - 5/5 tests
- **select.json** - 3/3 tests
- **skip.json** - 4/4 tests
- **sort.json** - 10/10 tests
- **split.json** - 4/4 tests
- **sqrt.json** - 3/3 tests
- **sub-set-of.json** - 3/3 tests
- **super-set-of.json** - 2/2 tests
- **tail.json** - 2/2 tests
- **take.json** - 7/7 tests
- **timezone-offset-of.json** - 5/5 tests
- **to-chars.json** - 1/1 tests
- **trace.json** - 2/2 tests
- **trim.json** - 6/6 tests
- **truncate.json** - 4/4 tests
- **year-of.json** - 9/9 tests

### 🟡 Well Implemented (70%+)

- **equality.json** - 96.4% (27/28 tests)
- **converts-to-long.json** - 93.8% (15/16 tests)
- **greater-than.json** - 93.3% (28/30 tests)
- **greator-or-equal.json** - 93.3% (28/30 tests)
- **less-or-equal.json** - 93.3% (28/30 tests)
- **less-than.json** - 93.3% (28/30 tests)
- **last-index-of.json** - 92.3% (12/13 tests)
- **starts-with.json** - 92.3% (12/13 tests)
- **n-equality.json** - 91.7% (22/24 tests)
- **to-long.json** - 90.0% (9/10 tests)
- **in.json** - 87.5% (7/8 tests)
- **matches.json** - 87.5% (14/16 tests)
- **plus.json** - 85.3% (29/34 tests)
- **precedence.json** - 83.3% (5/6 tests)
- **types.json** - 82.8% (82/99 tests)
- **iif.json** - 81.8% (9/11 tests)
- **dollar.json** - 80.0% (4/5 tests)
- **exists.json** - 80.0% (4/5 tests)
- **to-decimal.json** - 80.0% (4/5 tests)
- **to-string.json** - 80.0% (4/5 tests)
- **define-variable.json** - 76.2% (16/21 tests)

### 🟠 Partially Implemented (30-70%)

- **extension.json** - 66.7% (2/3 tests)
- **miscellaneous-accessor-tests.json** - 66.7% (2/3 tests)
- **replace.json** - 66.7% (4/6 tests)
- **substring.json** - 63.6% (7/11 tests)
- **type.json** - 63.3% (19/30 tests)
- **high-boundary.json** - 62.5% (15/24 tests)
- **low-boundary.json** - 60.7% (17/28 tests)
- **to-integer.json** - 60.0% (3/5 tests)
- **inheritance.json** - 58.3% (14/24 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **now.json** - 50.0% (1/2 tests)
- **observations.json** - 50.0% (5/10 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **power.json** - 50.0% (3/6 tests)
- **single.json** - 50.0% (1/2 tests)
- **to-date.json** - 50.0% (5/10 tests)
- **today.json** - 50.0% (1/2 tests)
- **precision.json** - 33.3% (2/6 tests)

### 🔴 Major Issues (0-30%)

None currently.

## Summary

The fhirpath-rs implementation currently passes approximately **88.7% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 111
- **Total Tests**: 1104
- **Pass Rate**: 88.7%

---

*Report generated on: 2025-08-25 17:22:19*
*Command: `just test-coverage` or `cargo run --package fhirpath-tools --bin test-coverage`*
