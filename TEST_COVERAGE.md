# FHIRPath Test Coverage Report

Generated on: 2025-08-14
Implementation: fhirpath-rs (octofhir-fhirpath)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 99 | 100% |
| **Total Individual Tests** | 1015 | 100% |
| **Passing Tests** | 259 | 25.5% |
| **Failing Tests** | 121 | 11.9% |
| **Error Tests** | 635 | 62.6% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **div.json** - 8/8 tests
- **mod.json** - 8/8 tests

### 🟡 Well Implemented (70%+)

- **comments.json** - 88.9% (8/9 tests)
- **concatenate.json** - 75.0% (3/4 tests)
- **in.json** - 75.0% (6/8 tests)
- **basics.json** - 71.4% (5/7 tests)

### 🟠 Partially Implemented (30-70%)

- **miscellaneous-accessor-tests.json** - 66.7% (2/3 tests)
- **precedence.json** - 66.7% (4/6 tests)
- **dollar.json** - 60.0% (3/5 tests)
- **to-string.json** - 60.0% (3/5 tests)
- **boolean-implies.json** - 55.6% (5/9 tests)
- **boolean-logic-or.json** - 55.6% (5/9 tests)
- **contains-collection.json** - 55.6% (5/9 tests)
- **not-equivalent.json** - 54.5% (12/22 tests)
- **quantity.json** - 54.5% (6/11 tests)
- **equality.json** - 53.6% (15/28 tests)
- **equivalent.json** - 50.0% (12/24 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **multiply.json** - 50.0% (3/6 tests)
- **polymorphics.json** - 50.0% (1/2 tests)
- **single.json** - 50.0% (1/2 tests)
- **minus.json** - 45.5% (5/11 tests)
- **divide.json** - 44.4% (4/9 tests)
- **exists.json** - 40.0% (2/5 tests)
- **greater-than.json** - 40.0% (12/30 tests)
- **greator-or-equal.json** - 40.0% (12/30 tests)
- **inheritance.json** - 37.5% (9/24 tests)
- **less-or-equal.json** - 36.7% (11/30 tests)
- **less-than.json** - 36.7% (11/30 tests)
- **plus.json** - 35.3% (12/34 tests)
- **cda-tests.json** - 33.3% (1/3 tests)
- **conforms-to.json** - 33.3% (1/3 tests)
- **define-variable.json** - 33.3% (7/21 tests)
- **sqrt.json** - 33.3% (1/3 tests)
- **to-date.json** - 30.0% (3/10 tests)

### 🔴 Major Issues (0-30%)

- **n-equality.json** - 25.0% (6/24 tests) - Issues
- **boolean-logic-x-or.json** - 22.2% (2/9 tests) - Issues
- **literals.json** - 22.0% (18/82 tests) - Issues
- **observations.json** - 20.0% (2/10 tests) - Issues
- **iif.json** - 18.2% (2/11 tests) - Issues
- **low-boundary.json** - 17.9% (5/28 tests) - Issues
- **collection-boolean.json** - 16.7% (1/6 tests) - Issues
- **power.json** - 16.7% (1/6 tests) - Issues
- **boolean-logic-and.json** - 11.1% (1/9 tests) - Issues
- **contains-string.json** - 9.1% (1/11 tests) - Issues
- **ends-with.json** - 9.1% (1/11 tests) - Issues
- **types.json** - 9.1% (9/99 tests) - Issues
- **high-boundary.json** - 8.3% (2/24 tests) - Issues
- **starts-with.json** - 7.7% (1/13 tests) - Issues
- **abs.json** - 0.0% (0/4 tests) - Missing
- **aggregate.json** - 0.0% (0/4 tests) - Missing
- **all.json** - 0.0% (0/4 tests) - Missing
- **case.json** - 0.0% (0/4 tests) - Missing
- **ceiling.json** - 0.0% (0/4 tests) - Missing
- **combine.json** - 0.0% (0/3 tests) - Missing
- **comparable.json** - 0.0% (0/3 tests) - Missing
- **count.json** - 0.0% (0/4 tests) - Missing
- **distinct.json** - 0.0% (0/6 tests) - Missing
- **encode-decode.json** - 0.0% (0/8 tests) - Missing
- **escape-unescape.json** - 0.0% (0/4 tests) - Missing
- **exclude.json** - 0.0% (0/4 tests) - Missing
- **exp.json** - 0.0% (0/3 tests) - Missing
- **extension.json** - 0.0% (0/3 tests) - Missing
- **first-last.json** - 0.0% (0/2 tests) - Missing
- **floor.json** - 0.0% (0/4 tests) - Missing
- **index-of.json** - 0.0% (0/6 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **indexer.json** - 0.0% (0/2 tests) - Missing
- **intersect.json** - 0.0% (0/4 tests) - Missing
- **join.json** - 0.0% (0/1 tests) - Missing
- **length.json** - 0.0% (0/6 tests) - Missing
- **ln.json** - 0.0% (0/3 tests) - Missing
- **log.json** - 0.0% (0/5 tests) - Missing
- **matches.json** - 0.0% (0/16 tests) - Missing
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing
- **now.json** - 0.0% (0/2 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **precision.json** - 0.0% (0/6 tests) - Missing
- **repeat.json** - 0.0% (0/5 tests) - Missing
- **replace-matches.json** - 0.0% (0/7 tests) - Missing
- **replace.json** - 0.0% (0/6 tests) - Missing
- **round.json** - 0.0% (0/3 tests) - Missing
- **select.json** - 0.0% (0/3 tests) - Missing
- **skip.json** - 0.0% (0/4 tests) - Missing
- **sort.json** - 0.0% (0/10 tests) - Missing
- **split.json** - 0.0% (0/4 tests) - Missing
- **sub-set-of.json** - 0.0% (0/3 tests) - Missing
- **substring.json** - 0.0% (0/11 tests) - Missing
- **super-set-of.json** - 0.0% (0/2 tests) - Missing
- **tail.json** - 0.0% (0/2 tests) - Missing
- **take.json** - 0.0% (0/7 tests) - Missing
- **to-chars.json** - 0.0% (0/1 tests) - Missing
- **to-decimal.json** - 0.0% (0/5 tests) - Missing
- **to-integer.json** - 0.0% (0/5 tests) - Missing
- **today.json** - 0.0% (0/2 tests) - Missing
- **trace.json** - 0.0% (0/2 tests) - Missing
- **trim.json** - 0.0% (0/6 tests) - Missing
- **truncate.json** - 0.0% (0/4 tests) - Missing
- **type.json** - 0.0% (0/30 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **25.5% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 99
- **Total Tests**: 1015
- **Pass Rate**: 25.5%

---

*Report generated on: 2025-08-14 15:54:19*
*Command: `just test-coverage` or `cargo run --package fhirpath-tools --bin test-coverage`*
