# FHIRPath Test Requirements Analysis

## Overview
The specs/fhirpath/tests directory contains comprehensive JSON test definitions that we must pass. These tests are structured as follows:

## Test Format
Each test file is a JSON object with:
- `name`: Test suite name
- `description`: Description of the test suite
- `source`: Source of the tests
- `tests`: Array of test cases

Each test case contains:
- `name`: Test name
- `expression`: FHIRPath expression to evaluate
- `input`: Direct input data (or null if using inputfile)
- `inputfile`: Reference to a file in the input/ directory
- `expected`: Expected result (array of values)
- `tags`: Tags for categorization

## Test Categories
Based on the files in specs/fhirpath/tests:

### Core Operations
- **basics.json**: Basic path navigation (name.given, name.suffix)
- **literals.json**: Literal values (numbers, strings, dates)
- **types.json**: Type checking and conversion
- **precedence.json**: Operator precedence

### Collections
- **collection-boolean.json**: Boolean operations on collections
- **count.json**: Collection counting
- **distinct.json**: Removing duplicates
- **first-last.json**: First/last element access
- **single.json**: Single element extraction
- **skip.json**: Skipping elements
- **take.json**: Taking elements
- **tail.json**: All but first element
- **combine.json**: Combining collections
- **exclude.json**: Excluding elements
- **intersect.json**: Set intersection
- **union.json**: Set union (implicit via | operator)

### Functions
- **abs.json**: Absolute value
- **aggregate.json**: Aggregation functions
- **all.json**: All elements match predicate
- **ceiling.json**: Ceiling function
- **contains-collection.json**: Collection containment
- **contains-string.json**: String containment
- **ends-with.json**: String ends with
- **exists.json**: Existence checks
- **exp.json**: Exponential function
- **floor.json**: Floor function
- **iif.json**: If-then-else
- **index-of.json**: Find index
- **join.json**: String joining
- **length.json**: String/collection length
- **ln.json**: Natural logarithm
- **log.json**: Logarithm
- **matches.json**: Regular expression matching
- **now.json**: Current datetime
- **power.json**: Power function
- **repeat.json**: Repeat operation
- **replace.json**: String replacement
- **replace-matches.json**: Regex replacement
- **round.json**: Rounding
- **select.json**: Projection/mapping
- **sort.json**: Sorting
- **split.json**: String splitting
- **sqrt.json**: Square root
- **starts-with.json**: String starts with
- **substring.json**: Substring extraction
- **to-chars.json**: Convert to characters
- **to-decimal.json**: Convert to decimal
- **to-integer.json**: Convert to integer
- **to-string.json**: Convert to string
- **today.json**: Current date
- **trace.json**: Debugging trace
- **trim.json**: String trimming
- **truncate.json**: Truncation

### Operators
- **boolean-implies.json**: Implies operator
- **boolean-logic-and.json**: And operator
- **boolean-logic-or.json**: Or operator
- **boolean-logic-x-or.json**: Xor operator
- **concatenate.json**: String concatenation (&)
- **div.json**: Integer division
- **divide.json**: Decimal division
- **equality.json**: Equality (=, !=)
- **equivalent.json**: Equivalence (~, !~)
- **greater-than.json**: Greater than (>)
- **greator-or-equal.json**: Greater or equal (>=)
- **in.json**: Membership test
- **less-or-equal.json**: Less or equal (<=)
- **less-than.json**: Less than (<)
- **minus.json**: Subtraction
- **mod.json**: Modulo
- **multiply.json**: Multiplication
- **n-equality.json**: Not equal
- **not-equivalent.json**: Not equivalent
- **plus.json**: Addition

### Special Features
- **dollar.json**: Variables ($this, $index, $total)
- **define-variable.json**: Variable definitions
- **extension.json**: FHIR extension navigation
- **polymorphics.json**: Polymorphic property access
- **indexer.json**: Array indexing
- **index-part.json**: Index-based access
- **case.json**: Case sensitivity
- **comments.json**: Comment handling
- **escape-unescape.json**: String escaping
- **encode-decode.json**: Base64 encoding/decoding

### Type System
- **comparable.json**: Type comparability
- **conforms-to.json**: Type conformance
- **inheritance.json**: Type inheritance
- **type.json**: Type checking functions

### Date/Time Operations
- **period.json**: Period handling
- **high-boundary.json**: Period boundaries
- **low-boundary.json**: Period boundaries
- **precision.json**: Date/time precision

### Quantity Operations
- **quantity.json**: Quantity arithmetic and comparison

### Advanced Features
- **from--zulip.json**: Complex real-world examples
- **misc-engine-tests.json**: Edge cases
- **miscellaneous-accessor-tests.json**: Property access edge cases
- **observations.json**: FHIR Observation-specific tests
- **cda-tests.json**: CDA-specific tests

## Test Input Files
Located in specs/fhirpath/tests/input/:
- **patient-example.json**: Sample Patient resource
- **observation-example.json**: Sample Observation resource
- **questionnaire-example.json**: Sample Questionnaire resource
- **valueset-example-expansion.json**: Sample ValueSet with expansion

## Implementation Priority

### Phase 1: Core Features (Must Have)
1. Basic path navigation (., [])
2. Literals (numbers, strings, booleans)
3. Basic operators (+, -, *, /, =, !=, <, >, <=, >=)
4. Collection operations (first, last, count, empty)
5. Basic functions (where, select, exists)

### Phase 2: Essential Features
1. Type system (is, as, ofType)
2. String functions (substring, contains, startsWith, endsWith)
3. Boolean logic (and, or, not, implies, xor)
4. Advanced collection operations (distinct, union, intersect)
5. Variables ($this, $index, $total)

### Phase 3: Advanced Features
1. Date/time operations
2. Quantity operations with UCUM
3. Regular expressions
4. Aggregation functions
5. Advanced type checking

### Phase 4: Complete Compliance
1. All remaining functions
2. Edge cases and error handling
3. Performance optimizations
4. Full spec compliance

## Test Runner Requirements
The test runner must:
1. Load JSON test definitions
2. Parse input files when referenced
3. Execute FHIRPath expressions
4. Compare results with expected values
5. Handle collection ordering appropriately
6. Report detailed test failures
7. Support filtering by tags
8. Generate compliance reports

## Success Criteria
- Pass 100% of tests in specs/fhirpath/tests
- Performance comparable to reference implementations
- Clear error messages for failures
- Comprehensive test coverage reports