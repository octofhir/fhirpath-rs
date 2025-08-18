# FHIRPath Test Coverage Report

Generated on: 2025-08-18
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 100 | 100% |
| **Total Individual Tests** | 1017 | 100% |
| **Passing Tests** | 852 | 83.8% |
| **Failing Tests** | 80 | 7.9% |
| **Error Tests** | 85 | 8.4% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **aggregate.json** - 4/4 tests
- **all.json** - 4/4 tests
- **basics.json** - 7/7 tests
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
- **contains-collection.json** - 9/9 tests
- **contains-string.json** - 11/11 tests
- **count.json** - 4/4 tests
- **distinct.json** - 6/6 tests
- **div.json** - 8/8 tests
- **divide.json** - 9/9 tests
- **encode-decode.json** - 8/8 tests
- **ends-with.json** - 11/11 tests
- **escape-unescape.json** - 4/4 tests
- **exclude.json** - 4/4 tests
- **exp.json** - 3/3 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
- **from--zulip.json** - 2/2 tests
- **high-boundary.json** - 24/24 tests
- **index-of.json** - 6/6 tests
- **index-part.json** - 1/1 tests
- **indexer.json** - 2/2 tests
- **intersect.json** - 4/4 tests
- **join.json** - 1/1 tests
- **length.json** - 6/6 tests
- **ln.json** - 3/3 tests
- **log.json** - 5/5 tests
- **low-boundary.json** - 28/28 tests
- **matches.json** - 16/16 tests
- **minus.json** - 11/11 tests
- **mod.json** - 8/8 tests
- **multiply.json** - 6/6 tests
- **now.json** - 2/2 tests
- **period.json** - 2/2 tests
- **power.json** - 6/6 tests
- **quantity.json** - 11/11 tests
- **repeat.json** - 5/5 tests
- **replace-matches.json** - 7/7 tests
- **replace.json** - 6/6 tests
- **resolve.json** - 2/2 tests
- **round.json** - 3/3 tests
- **select.json** - 3/3 tests
- **single.json** - 2/2 tests
- **skip.json** - 4/4 tests
- **split.json** - 4/4 tests
- **sqrt.json** - 3/3 tests
- **starts-with.json** - 13/13 tests
- **sub-set-of.json** - 3/3 tests
- **substring.json** - 11/11 tests
- **super-set-of.json** - 2/2 tests
- **tail.json** - 2/2 tests
- **take.json** - 7/7 tests
- **to-chars.json** - 1/1 tests
- **to-date.json** - 10/10 tests
- **to-decimal.json** - 5/5 tests
- **to-integer.json** - 5/5 tests
- **trace.json** - 2/2 tests
- **trim.json** - 6/6 tests
- **truncate.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **not-equivalent.json** - 90.9% (20/22 tests)
- **equality.json** - 89.3% (25/28 tests)
- **boolean-implies.json** - 88.9% (8/9 tests)
- **equivalent.json** - 87.5% (21/24 tests)
- **in.json** - 87.5% (7/8 tests)
- **n-equality.json** - 87.5% (21/24 tests)
- **greater-than.json** - 86.7% (26/30 tests)
- **greator-or-equal.json** - 86.7% (26/30 tests)
- **less-or-equal.json** - 86.7% (26/30 tests)
- **less-than.json** - 86.7% (26/30 tests)
- **plus.json** - 85.3% (29/34 tests)
- **dollar.json** - 80.0% (4/5 tests)
- **exists.json** - 80.0% (4/5 tests)
- **sort.json** - 80.0% (8/10 tests)
- **to-string.json** - 80.0% (4/5 tests)
- **types.json** - 79.8% (79/99 tests)
- **abs.json** - 75.0% (3/4 tests)
- **literals.json** - 73.2% (60/82 tests)
- **iif.json** - 72.7% (8/11 tests)

### 🟠 Partially Implemented (30-70%)

- **conforms-to.json** - 66.7% (2/3 tests)
- **miscellaneous-accessor-tests.json** - 66.7% (2/3 tests)
- **precedence.json** - 66.7% (4/6 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **today.json** - 50.0% (1/2 tests)
- **inheritance.json** - 37.5% (9/24 tests)
- **extension.json** - 33.3% (1/3 tests)
- **precision.json** - 33.3% (2/6 tests)
- **observations.json** - 30.0% (3/10 tests)

### 🔴 Major Issues (0-30%)

- **define-variable.json** - 28.6% (6/21 tests) - Issues
- **type.json** - 3.3% (1/30 tests) - Issues
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **83.8% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 100
- **Total Tests**: 1017
- **Pass Rate**: 83.8%

---

*Report generated on: 2025-08-18 14:12:29*
*Command: `just test-coverage` or `cargo run --package fhirpath-tools --bin test-coverage`*
