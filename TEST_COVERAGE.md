# FHIRPath Test Coverage Report

Generated on: 2025-08-15
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 100 | 100% |
| **Total Individual Tests** | 1017 | 100% |
| **Passing Tests** | 698 | 68.6% |
| **Failing Tests** | 117 | 11.5% |
| **Error Tests** | 202 | 19.9% |

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
- **concatenate.json** - 4/4 tests
- **count.json** - 4/4 tests
- **div.json** - 8/8 tests
- **exp.json** - 3/3 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
- **index-of.json** - 6/6 tests
- **indexer.json** - 2/2 tests
- **intersect.json** - 4/4 tests
- **join.json** - 1/1 tests
- **length.json** - 6/6 tests
- **ln.json** - 3/3 tests
- **mod.json** - 8/8 tests
- **multiply.json** - 6/6 tests
- **replace-matches.json** - 7/7 tests
- **replace.json** - 6/6 tests
- **resolve.json** - 2/2 tests
- **select.json** - 3/3 tests
- **single.json** - 2/2 tests
- **skip.json** - 4/4 tests
- **split.json** - 4/4 tests
- **sqrt.json** - 3/3 tests
- **tail.json** - 2/2 tests
- **take.json** - 7/7 tests
- **to-chars.json** - 1/1 tests
- **to-decimal.json** - 5/5 tests
- **to-integer.json** - 5/5 tests
- **trace.json** - 2/2 tests
- **trim.json** - 6/6 tests
- **truncate.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **not-equivalent.json** - 95.5% (21/22 tests)
- **starts-with.json** - 92.3% (12/13 tests)
- **contains-string.json** - 90.9% (10/11 tests)
- **ends-with.json** - 90.9% (10/11 tests)
- **quantity.json** - 90.9% (10/11 tests)
- **substring.json** - 90.9% (10/11 tests)
- **greater-than.json** - 90.0% (27/30 tests)
- **greator-or-equal.json** - 90.0% (27/30 tests)
- **less-or-equal.json** - 90.0% (27/30 tests)
- **less-than.json** - 90.0% (27/30 tests)
- **comments.json** - 88.9% (8/9 tests)
- **contains-collection.json** - 88.9% (8/9 tests)
- **divide.json** - 88.9% (8/9 tests)
- **equivalent.json** - 87.5% (21/24 tests)
- **matches.json** - 87.5% (14/16 tests)
- **equality.json** - 85.7% (24/28 tests)
- **collection-boolean.json** - 83.3% (5/6 tests)
- **dollar.json** - 80.0% (4/5 tests)
- **exists.json** - 80.0% (4/5 tests)
- **to-string.json** - 80.0% (4/5 tests)
- **all.json** - 75.0% (3/4 tests)
- **exclude.json** - 75.0% (3/4 tests)
- **minus.json** - 72.7% (8/11 tests)
- **n-equality.json** - 70.8% (17/24 tests)
- **to-date.json** - 70.0% (7/10 tests)

### 🟠 Partially Implemented (30-70%)

- **literals.json** - 67.1% (55/82 tests)
- **miscellaneous-accessor-tests.json** - 66.7% (2/3 tests)
- **precedence.json** - 66.7% (4/6 tests)
- **round.json** - 66.7% (2/3 tests)
- **sub-set-of.json** - 66.7% (2/3 tests)
- **iif.json** - 63.6% (7/11 tests)
- **types.json** - 63.6% (63/99 tests)
- **sort.json** - 60.0% (6/10 tests)
- **now.json** - 50.0% (1/2 tests)
- **period.json** - 50.0% (1/2 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **power.json** - 50.0% (3/6 tests)
- **super-set-of.json** - 50.0% (1/2 tests)
- **today.json** - 50.0% (1/2 tests)
- **log.json** - 40.0% (2/5 tests)
- **in.json** - 37.5% (3/8 tests)
- **inheritance.json** - 37.5% (9/24 tests)
- **plus.json** - 35.3% (12/34 tests)
- **conforms-to.json** - 33.3% (1/3 tests)
- **distinct.json** - 33.3% (2/6 tests)
- **precision.json** - 33.3% (2/6 tests)

### 🔴 Major Issues (0-30%)

- **type.json** - 26.7% (8/30 tests) - Issues
- **observations.json** - 20.0% (2/10 tests) - Issues
- **low-boundary.json** - 17.9% (5/28 tests) - Issues
- **define-variable.json** - 14.3% (3/21 tests) - Issues
- **high-boundary.json** - 8.3% (2/24 tests) - Issues
- **aggregate.json** - 0.0% (0/4 tests) - Missing
- **cda-tests.json** - 0.0% (0/3 tests) - Missing
- **combine.json** - 0.0% (0/3 tests) - Missing
- **comparable.json** - 0.0% (0/3 tests) - Missing
- **encode-decode.json** - 0.0% (0/8 tests) - Missing
- **escape-unescape.json** - 0.0% (0/4 tests) - Missing
- **extension.json** - 0.0% (0/3 tests) - Missing
- **from--zulip.json** - 0.0% (0/2 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing
- **repeat.json** - 0.0% (0/5 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **68.6% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 100
- **Total Tests**: 1017
- **Pass Rate**: 68.6%

---

*Report generated on: 2025-08-15 22:52:11*
*Command: `just test-coverage` or `cargo run --package fhirpath-tools --bin test-coverage`*
