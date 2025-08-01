# FHIRPath Examples

This directory contains comprehensive examples demonstrating how to use the `octofhir-fhirpath` library for parsing and evaluating FHIRPath expressions against FHIR data.

## Overview

FHIRPath is a path-based navigation and extraction language for FHIR resources. These examples show you how to:

- Parse and evaluate basic FHIRPath expressions
- Handle complex data transformations
- Work with advanced features like lambda expressions
- Implement error handling strategies
- Understand the library's API patterns

## Examples

### 1. Basic Usage (`basic_usage.rs`)

**Purpose**: Introduction to fundamental FHIRPath operations

**Features Demonstrated**:
- Simple property access (`Patient.id`)
- Array filtering (`Patient.name.where(use = 'official')`)
- Nested property navigation (`Patient.name.family`)
- Collection operations (`Patient.name.count()`)
- Boolean expressions (`Patient.active = true`)
- String concatenation
- Type checking (`Patient.birthDate is System.String`)
- Mathematical operations
- Existence checking
- Complex expressions with `select()`

**Run Example**:
```bash
cargo run --example basic_usage
```

### 2. Advanced Evaluation (`advanced_evaluation.rs`)

**Purpose**: Explore sophisticated FHIRPath features and patterns

**Features Demonstrated**:
- Lambda expressions with higher-order functions
- Complex filtering across Bundle resources
- Aggregation operations (`count()`, mathematical operations)
- Type operations and conversions
- Advanced string manipulation
- Date handling
- Complex path navigation through nested structures
- Union operations (`|`)
- Conditional expressions (`iif()`)
- Advanced aggregation with `distinct()`

**Run Example**:
```bash
cargo run --example advanced_evaluation
```

### 3. Custom Functions (`custom_functions.rs`)

**Purpose**: Understanding built-in functions and extension patterns

**Features Demonstrated**:
- Built-in string functions (`upper()`, `lower()`, `contains()`)
- Mathematical functions and operations
- Date and time functions
- Type conversion functions (`toString()`)
- Aggregate functions (`sum()`, `avg()`)
- Advanced collection functions (`distinct()`)
- Conceptual overview of custom function implementation

**Run Example**:
```bash
cargo run --example custom_functions
```

### 4. Error Handling (`error_handling.rs`)

**Purpose**: Robust error handling and defensive programming

**Features Demonstrated**:
- Parse error handling (syntax errors, malformed expressions)
- Evaluation error management (type mismatches, unknown functions)
- Working with missing or incomplete data
- Defensive programming with existence checks
- Error recovery strategies
- Input validation patterns

**Run Example**:
```bash
cargo run --example error_handling
```

## Getting Started

### Prerequisites

- Rust 1.70+ installed
- Basic understanding of FHIR resources
- Familiarity with FHIRPath specification (helpful but not required)

### Running Examples

1. Clone the repository:
```bash
git clone <repository-url>
cd fhirpath-rs
```

2. Run any example:
```bash
cargo run --example basic_usage
cargo run --example advanced_evaluation
cargo run --example custom_functions
cargo run --example error_handling
```

3. Or run all examples:
```bash
cargo run --example basic_usage && \
cargo run --example advanced_evaluation && \
cargo run --example custom_functions && \
cargo run --example error_handling
```

## Key API Patterns

### Engine Creation
```rust
use octofhir_fhirpath::engine::FhirPathEngine;

let mut engine = FhirPathEngine::new();
```

### Expression Evaluation
```rust
use serde_json::json;

let patient = json!({
    "resourceType": "Patient",
    "id": "example",
    "name": [{"family": "Smith", "given": ["John"]}]
});

let result = engine.evaluate("Patient.name.family", patient)?;
```

### Error Handling
```rust
match engine.evaluate("invalid.expression", data) {
    Ok(result) => println!("Success: {:?}", result),
    Err(error) => println!("Error: {}", error),
}
```

## Common Expression Patterns

### Property Access
```fhirpath
Patient.id                          // Simple property
Patient.name.family                 // Nested property
Patient.name[0].family              // Array indexing
```

### Filtering
```fhirpath
Patient.name.where(use = 'official')           // Filter by value
Patient.telecom.where(system = 'email')        // Filter array
Bundle.entry.resource.where(resourceType = 'Patient')  // Filter resources
```

### Collection Operations
```fhirpath
Patient.name.count()                // Count items
Patient.name.exists()               // Check existence
Patient.name.empty()                // Check if empty
Patient.name.first()                // Get first item
Patient.name.last()                 // Get last item
```

### Transformations
```fhirpath
Patient.name.select(given + ' ' + family)      // Transform each item
Patient.name.given.distinct()                  // Remove duplicates
Patient.contact.telecom.value.join(', ')       // Join strings
```

### Boolean Logic
```fhirpath
Patient.active = true                           // Equality
Patient.birthDate > @1990-01-01               // Date comparison
Patient.name.exists() and Patient.id.exists() // Logical AND
```

## Performance Tips

1. **Expression Caching**: The engine automatically caches parsed expressions for better performance
2. **Efficient Filtering**: Use `where()` early in expressions to reduce data processing
3. **Avoid Deep Nesting**: Break complex expressions into simpler parts when possible
4. **Use Exists Checks**: Validate data existence before accessing nested properties

## Troubleshooting

### Common Issues

1. **Parse Errors**: Check expression syntax, especially parentheses and quotes
2. **Empty Results**: Verify data structure and property names
3. **Type Errors**: Ensure operations are compatible with data types
4. **Performance**: Consider expression complexity and data size

### Debug Tips

1. Test expressions step by step: `Patient` → `Patient.name` → `Patient.name.family`
2. Use `exists()` to verify data presence
3. Check the FHIRPath specification for correct syntax
4. Examine error messages for specific guidance

## Additional Resources

- [FHIRPath Specification](http://hl7.org/fhirpath/)
- [FHIR Resource Documentation](http://hl7.org/fhir/)
- [Library Documentation](../README.md)
- [Official Test Suite](../specs/fhirpath/tests/)

## Contributing

Found an issue with the examples or have suggestions for improvements? Please:

1. Check existing issues in the repository
2. Create a new issue with detailed information
3. Submit a pull request with fixes or enhancements

## License

These examples are provided under the same license as the main library. See [LICENSE.md](../LICENSE.md) for details.