# FHIRPath Key Concepts

## Core Language Concepts

### FHIRPath Language
- [§01-title-and-introduction.md](./§01-title-and-introduction.md)
- A path-based navigation and extraction language for hierarchical data structures, similar to XPath but specifically designed for healthcare data. It enables traversal, selection, and filtering of complex nested data while maintaining simplicity and expressiveness.

### Navigation Model
- [§03-navigation-model.md](./§03-navigation-model.md)
- FHIRPath operates on a tree-based data model where data is represented as a directed acyclic graph with labeled nodes. The context (root) is the instance against which expressions are evaluated, and nodes can contain primitive values and child nodes with support for repeating elements.

### Path Selection
- [§04-path-selection.md](./§04-path-selection.md)
- The fundamental mechanism for traversing data structures using dot notation (e.g., `name.given`). Path selection always returns collections (ordered, non-unique, indexed, countable) and supports type filtering through operators like `ofType()`, `is`, and `as` for handling polymorphic data.

### Collections
- [§04-path-selection.md](./§04-path-selection.md), [§05-expressions.md](./§05-expressions.md)
- The foundational data structure in FHIRPath where all expressions return collections rather than single values. Collections are ordered, non-unique, support indexing, and can be empty (`{}`) to represent missing values instead of using null concepts.

## Expression Components

### Expressions
- [§05-expressions.md](./§05-expressions.md)
- The building blocks of FHIRPath that combine literals, operators, and function invocations. Expressions support various literal types (Boolean, String, Integer, Decimal, Date, DateTime, Time, Quantity) and can be chained using fluent-style dot notation.

### Literals
- [§05-expressions.md](./§05-expressions.md), [§09-lexical-elements.md](./§09-lexical-elements.md)
- Concrete values in FHIRPath including Boolean (`true`/`false`), String (single-quoted), Integer, Decimal, Date (`@YYYY-MM-DD`), DateTime, Time, and Quantity (number with unit). These form the basic value types that expressions can evaluate to.

### Operators
- [§07-operations.md](./§07-operations.md)
- Symbols that perform operations on values including mathematical (`+`, `-`, `*`, `/`), boolean (`and`, `or`, `not`), comparison (`>`, `<`, `=`), type checking (`is`, `as`), and collection operations (`|`, `in`, `contains`). Operators follow a 13-level precedence hierarchy.

### Functions
- [§06-functions.md](./§06-functions.md), [§06.0-functions-introduction.md](./§06.0-functions-introduction.md)
- Pre-defined operations that can be invoked using dot notation for tasks like existence checking, filtering, projection, subsetting, type conversion, string manipulation, math operations, tree navigation, and utility operations. Functions enable complex data transformations and queries.

## Type System

### Types
- [§11-types-and-reflection.md](./§11-types-and-reflection.md)
- The classification system for values in FHIRPath, organized into models with namespaces. Types include primitives (SimpleTypeInfo), complex types (ClassInfo), collections (ListTypeInfo), and anonymous types (TupleTypeInfo), with support for runtime type inspection via reflection.

### Type Safety
- [§12-type-safety-and-strict-evaluation.md](./§12-type-safety-and-strict-evaluation.md)
- The framework allowing different implementation strategies from dynamic to static type checking. It defines unsafe operations (functions expecting single items on collections), type inference limitations, and explicit type handling mechanisms to balance flexibility with safety.

### Type Operators
- [§07-operations.md](./§07-operations.md), [§04-path-selection.md](./§04-path-selection.md)
- Special operators for type checking and conversion: `is` (type test), `as` (type assertion), and `ofType()` (filtering by type). These enable safe navigation of polymorphic data structures and runtime type verification.

## Advanced Features

### Aggregates
- [§08-aggregates.md](./§08-aggregates.md)
- A general-purpose aggregation mechanism using the `aggregate()` function with a `$total` accumulator variable. This enables calculations like sum, min, max, and average across collections, marked as Standard for Trial Use (STU).

### Environment Variables
- [§10-environment-variables.md](./§10-environment-variables.md)
- External value injection mechanism using tokens prefixed with `%` (e.g., `%ucum`, `%context`). These provide a formal extension point for implementations to pass external data into FHIRPath expressions.

### Lexical Elements
- [§09-lexical-elements.md](./§09-lexical-elements.md)
- The fundamental syntactic components including whitespace, comments (`//` and `/* */`), symbols, keywords (`and`, `or`, `is`, `as`), and identifiers (simple and delimited with backticks). The language is case-sensitive throughout.

## Function Categories

### Existence Functions
- [§06.1-existence.md](./§06.1-existence.md)
- Functions that test for the presence or absence of values: `empty()`, `exists()`, `all()`, `allTrue()`, `anyTrue()`, `allFalse()`, `anyFalse()`, `subsetOf()`, `supersetOf()`, `count()`, `distinct()`, `isDistinct()`.

### Filtering and Projection
- [§06.2-filtering-and-projection.md](./§06.2-filtering-and-projection.md)
- Functions for selecting and transforming data: `where()`, `select()`, `repeat()`, `ofType()`. These enable complex queries and data transformations while maintaining the collection-based paradigm.

### Subsetting Functions
- [§06.3-subsetting.md](./§06.3-subsetting.md)
- Functions for extracting portions of collections: `single()`, `first()`, `last()`, `tail()`, `skip()`, `take()`, `intersect()`, `exclude()`. These provide precise control over collection contents.

### Combining Functions
- [§06.4-combining.md](./§06.4-combining.md)
- Functions for merging collections: `union()` (using `|` operator) and `combine()`. These enable construction of new collections from existing ones while preserving FHIRPath semantics.

### Conversion Functions
- [§06.5-conversion.md](./§06.5-conversion.md)
- Functions for type conversion: `iif()`, `toBoolean()`, `toInteger()`, `toDecimal()`, `toString()`, `toDate()`, `toDateTime()`, `toTime()`, `toQuantity()`. These provide safe, explicit type conversions with defined error handling.

### String Manipulation
- [§06.6-string-manipulation.md](./§06.6-string-manipulation.md)
- Functions for text processing: `indexOf()`, `substring()`, `startsWith()`, `endsWith()`, `contains()`, `upper()`, `lower()`, `replace()`, `matches()`, `replaceMatches()`, `length()`, `toChars()`.

### Math Functions
- [§06.7-math.md](./§06.7-math.md)
- Mathematical operations: `abs()`, `ceiling()`, `exp()`, `floor()`, `ln()`, `log()`, `power()`, `round()`, `sqrt()`, `truncate()`. These operate on numeric types with defined precision and overflow behavior.

### Tree Navigation
- [§06.8-tree-navigation.md](./§06.8-tree-navigation.md)
- Functions for traversing hierarchical structures: `children()`, `descendants()`. These enable navigation beyond simple path selection for complex tree structures.

### Utility Functions
- [§06.9-utility-functions.md](./§06.9-utility-functions.md)
- General-purpose functions: `trace()` for debugging, `check()` for assertions, `encode()`/`decode()` for data transformation, and `resolve()` for reference resolution. These support various implementation and debugging needs.

## Operator Categories

### Equality Operators
- [§07-operations.md](./§07-operations.md)
- Operators for value comparison: `=` (equals), `~` (equivalent), `!=` (not equals), `!~` (not equivalent). Equivalence (`~`) provides value-based comparison ignoring type differences.

### Comparison Operators
- [§07-operations.md](./§07-operations.md)
- Operators for ordering: `>`, `<`, `>=`, `<=`. These work on comparable types (numbers, strings, dates) with defined semantics for partial ordering and type compatibility.

### Boolean Operators
- [§07-operations.md](./§07-operations.md)
- Logical operators: `and`, `or`, `not()`, `xor`, `implies`. These implement three-valued logic accounting for empty collections representing unknown values.

### Math Operators
- [§07-operations.md](./§07-operations.md)
- Arithmetic operators: `+` (addition/string concatenation), `-` (subtraction), `*` (multiplication), `/` (division), `div` (integer division), `mod` (modulo), `&` (string concatenation).

### Date/Time Arithmetic
- [§07-operations.md](./§07-operations.md)
- Special arithmetic for temporal types allowing addition and subtraction of time-valued quantities (e.g., `@2023-01-01 + 1 month`). These maintain calendar semantics and handle edge cases like month boundaries.

## Implementation Concepts

### Singleton Evaluation
- [§05-expressions.md](./§05-expressions.md)
- Rules for converting collections to single values when required by operators or functions. This includes automatic unwrapping of single-element collections and error handling for multi-element collections.

### Operator Precedence
- [§07-operations.md](./§07-operations.md)
- The 13-level hierarchy determining evaluation order: from highest (path selection/function invocation) to lowest (implies). This ensures consistent expression evaluation across implementations.

### Formal Specifications
- [§13-formal-specifications.md](./§13-formal-specifications.md)
- The grammar definition (fhirpath.g4) and formal semantics providing precise implementation guidance. This enables consistent behavior across different FHIRPath implementations.

### Implementation Flexibility
- [§12-type-safety-and-strict-evaluation.md](./§12-type-safety-and-strict-evaluation.md), [§15-fhirpath-tooling-and-implementation.md](./§15-fhirpath-tooling-and-implementation.md)
- Design principles allowing implementations to choose between dynamic and static evaluation strategies while maintaining semantic compatibility. This enables FHIRPath to work in various runtime environments.