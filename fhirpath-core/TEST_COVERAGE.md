# FHIRPath Test Coverage Report

Generated on: 2025-07-29
Implementation: fhirpath-rs (fhirpath-core)

## Executive Summary

This report provides a comprehensive analysis of the current FHIRPath implementation's compliance with the official FHIRPath test suites.

### Overall Statistics

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Test Suites** | 98 | 100% |
| **Total Individual Tests** | 1005 | 100% |
| **Passing Tests** | 663 | 66.0% |
| **Failing Tests** | 132 | 13.1% |
| **Error Tests** | 210 | 20.9% |

## Test Results by Suite

### ✅ Fully Passing (100%)

- **abs.json** - 4/4 tests
- **all.json** - 4/4 tests
- **basics.json** - 7/7 tests
- **boolean-implies.json** - 9/9 tests
- **boolean-logic-and.json** - 9/9 tests
- **boolean-logic-or.json** - 9/9 tests
- **boolean-logic-x-or.json** - 9/9 tests
- **case.json** - 4/4 tests
- **ceiling.json** - 4/4 tests
- **comparable.json** - 3/3 tests
- **contains-collection.json** - 9/9 tests
- **contains-string.json** - 11/11 tests
- **count.json** - 4/4 tests
- **encode-decode.json** - 8/8 tests
- **ends-with.json** - 11/11 tests
- **escape-unescape.json** - 4/4 tests
- **exclude.json** - 4/4 tests
- **exp.json** - 3/3 tests
- **first-last.json** - 2/2 tests
- **floor.json** - 4/4 tests
- **in.json** - 8/8 tests
- **index-of.json** - 6/6 tests
- **indexer.json** - 2/2 tests
- **join.json** - 1/1 tests
- **length.json** - 6/6 tests
- **ln.json** - 3/3 tests
- **multiply.json** - 6/6 tests
- **replace-matches.json** - 7/7 tests
- **replace.json** - 6/6 tests
- **round.json** - 3/3 tests
- **select.json** - 3/3 tests
- **skip.json** - 4/4 tests
- **split.json** - 4/4 tests
- **sqrt.json** - 3/3 tests
- **starts-with.json** - 13/13 tests
- **substring.json** - 11/11 tests
- **tail.json** - 2/2 tests
- **take.json** - 7/7 tests
- **to-chars.json** - 1/1 tests
- **to-decimal.json** - 5/5 tests
- **to-integer.json** - 5/5 tests
- **to-string.json** - 5/5 tests
- **today.json** - 2/2 tests
- **trim.json** - 6/6 tests
- **truncate.json** - 4/4 tests

### 🟡 Well Implemented (70%+)

- **matches.json** - 93.8% (15/16 tests)
- **div.json** - 87.5% (7/8 tests)
- **mod.json** - 87.5% (7/8 tests)
- **not-equivalent.json** - 86.4% (19/22 tests)
- **n-equality.json** - 83.3% (20/24 tests)
- **types.json** - 79.8% (79/99 tests)
- **equality.json** - 78.6% (22/28 tests)
- **greater-than.json** - 76.7% (23/30 tests)
- **greator-or-equal.json** - 76.7% (23/30 tests)
- **less-or-equal.json** - 76.7% (23/30 tests)
- **less-than.json** - 76.7% (23/30 tests)
- **concatenate.json** - 75.0% (3/4 tests)
- **equivalent.json** - 75.0% (18/24 tests)
- **intersect.json** - 75.0% (3/4 tests)

### 🟠 Partially Implemented (30-70%)

- **comments.json** - 66.7% (6/9 tests)
- **iif.json** - 63.6% (7/11 tests)
- **dollar.json** - 60.0% (3/5 tests)
- **exists.json** - 60.0% (3/5 tests)
- **log.json** - 60.0% (3/5 tests)
- **literals.json** - 56.1% (46/82 tests)
- **divide.json** - 55.6% (5/9 tests)
- **minus.json** - 54.5% (6/11 tests)
- **distinct.json** - 50.0% (3/6 tests)
- **from--zulip.json** - 50.0% (1/2 tests)
- **now.json** - 50.0% (1/2 tests)
- **power.json** - 50.0% (3/6 tests)
- **single.json** - 50.0% (1/2 tests)
- **trace.json** - 50.0% (1/2 tests)
- **type.json** - 46.7% (14/30 tests)
- **collection-boolean.json** - 33.3% (2/6 tests)
- **combine.json** - 33.3% (1/3 tests)
- **conforms-to.json** - 33.3% (1/3 tests)
- **miscellaneous-accessor-tests.json** - 33.3% (1/3 tests)
- **precedence.json** - 33.3% (2/6 tests)
- **precision.json** - 33.3% (2/6 tests)

### 🔴 Major Issues (0-30%)

- **quantity.json** - 27.3% (3/11 tests) - Issues
- **plus.json** - 23.5% (8/34 tests) - Issues
- **observations.json** - 20.0% (2/10 tests) - Issues
- **define-variable.json** - 19.0% (4/21 tests) - Issues
- **inheritance.json** - 12.5% (3/24 tests) - Issues
- **sort.json** - 10.0% (1/10 tests) - Issues
- **aggregate.json** - 0.0% (0/4 tests) - Missing
- **cda-tests.json** - 0.0% (0/3 tests) - Missing
- **extension.json** - 0.0% (0/3 tests) - Missing
- **high-boundary.json** - 0.0% (0/24 tests) - Missing
- **index-part.json** - 0.0% (0/1 tests) - Missing
- **low-boundary.json** - 0.0% (0/28 tests) - Missing
- **misc-engine-tests.json** - 0.0% (0/2 tests) - Missing
- **period.json** - 0.0% (0/2 tests) - Missing
- **polymorphics.json** - 0.0% (0/2 tests) - Missing
- **repeat.json** - 0.0% (0/5 tests) - Missing
- **sub-set-of.json** - 0.0% (0/3 tests) - Missing
- **super-set-of.json** - 0.0% (0/2 tests) - Missing

## Summary

The fhirpath-rs implementation currently passes approximately **66.0% of all FHIRPath tests**.

### Key Statistics
- **Test Suites**: 98
- **Total Tests**: 1005
- **Pass Rate**: 66.0%

---

*Report generated on: 2025-07-29 17:34:00*  
*Command: `cargo test run_coverage_report -- --ignored --nocapture`*
