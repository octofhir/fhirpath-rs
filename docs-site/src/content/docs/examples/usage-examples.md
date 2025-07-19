---
title: Usage Examples
description: Comprehensive examples of how to use the FHIRPath Rust engine in various scenarios
---

# FHIRPath Engine Usage Examples

This document provides comprehensive examples of how to use the FHIRPath Rust engine in various scenarios.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [CLI Tool Examples](#cli-tool-examples)
3. [Rust API Examples](#rust-api-examples)
4. [Node.js Integration Examples](#nodejs-integration-examples)
5. [Advanced Usage Patterns](#advanced-usage-patterns)

## Basic Usage

### Simple Property Access

```rust
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::json;

let patient = json!({
    "resourceType": "Patient",
    "id": "example",
    "name": [{
        "family": "Smith",
        "given": ["John"]
    }],
    "gender": "male"
});

// Access basic properties
let result = evaluate_expression("resourceType", patient.clone()).unwrap();
println!("Resource Type: {:?}", result);

let result = evaluate_expression("gender", patient.clone()).unwrap();
println!("Gender: {:?}", result);
```

### Array and Path Navigation

```rust
// Access nested properties
let result = evaluate_expression("name.family", patient.clone()).unwrap();
println!("Family Name: {:?}", result);

let result = evaluate_expression("name.given", patient.clone()).unwrap();
println!("Given Names: {:?}", result);

// Array indexing
let result = evaluate_expression("name[0].family", patient.clone()).unwrap();
println!("First Name Family: {:?}", result);
```

## CLI Tool Examples

The FHIRPath CLI tool provides an easy way to test expressions against FHIR resources.

### Basic Evaluation

```bash
# Evaluate a simple expression
octofhir-fhirpath eval "resourceType" --resource patient.json

# Evaluate with pretty formatting
octofhir-fhirpath eval "name.family" --resource patient.json --format pretty

# Evaluate complex expressions
octofhir-fhirpath eval "name.where(use = 'official').family" --resource patient.json
```

### Expression Validation

```bash
# Validate FHIRPath expression syntax
octofhir-fhirpath validate "Patient.name.family"
octofhir-fhirpath validate "invalid..expression"
```

### Working with Different Resource Types

```bash
# Observation resource
octofhir-fhirpath eval "code.coding.system" --resource observation.json

# Bundle resource
octofhir-fhirpath eval "entry.resource.resourceType" --resource bundle.json

# Medication resource
octofhir-fhirpath eval "ingredient.item.display" --resource medication.json
```

## Rust API Examples

### Basic Integration

```rust
use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::model::FhirPathValue;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load FHIR resource from file
    let resource_json = std::fs::read_to_string("patient.json")?;
    let resource: serde_json::Value = serde_json::from_str(&resource_json)?;
    
    // Evaluate FHIRPath expression
    let result = evaluate_expression("Patient.name.family", resource)?;
    
    // Process result
    match result {
        FhirPathValue::Collection(values) => {
            for value in values {
                match value {
                    FhirPathValue::String(s) => println!("Family name: {}", s),
                    _ => println!("Non-string value: {:?}", value),
                }
            }
        }
        _ => println!("Unexpected result type: {:?}", result),
    }
    
    Ok(())
}
```

### Error Handling

```rust
use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::errors::FhirPathError;

fn safe_evaluation(expression: &str, resource: serde_json::Value) {
    match evaluate_expression(expression, resource) {
        Ok(result) => println!("Result: {:?}", result),
        Err(FhirPathError::ParseError(msg)) => {
            eprintln!("Parse error: {}", msg);
        }
        Err(FhirPathError::EvaluationError(msg)) => {
            eprintln!("Evaluation error: {}", msg);
        }
        Err(FhirPathError::TypeError(msg)) => {
            eprintln!("Type error: {}", msg);
        }
        Err(err) => eprintln!("Other error: {:?}", err),
    }
}
```

### Performance Optimization

```rust
use fhirpath_core::evaluator::evaluate_expression_optimized;

// Use optimized evaluation for better performance
let result = evaluate_expression_optimized("complex.expression.here", resource)?;
```

### Streaming Mode for Large Resources

```rust
use fhirpath_core::evaluator::evaluate_expression_streaming;
use std::fs::File;

let file = File::open("large-bundle.json")?;
let result = evaluate_expression_streaming("entry.resource.resourceType", file)?;
```

## Node.js Integration Examples

### Basic Usage

```javascript
const { FhirPathEngine } = require('@octofhir/fhirpath');

const engine = new FhirPathEngine();

const patient = {
    resourceType: "Patient",
    id: "example",
    name: [{
        family: "Smith",
        given: ["John"]
    }],
    gender: "male"
};

// Synchronous evaluation
try {
    const result = engine.evaluate("name.family", JSON.stringify(patient));
    console.log("Family name:", JSON.parse(result));
} catch (error) {
    console.error("Evaluation error:", error.message);
}
```

### Asynchronous Evaluation

```javascript
// Asynchronous evaluation for CPU-intensive operations
async function evaluateAsync() {
    try {
        const result = await engine.evaluateAsync("complex.expression", JSON.stringify(largeResource));
        console.log("Result:", JSON.parse(result));
    } catch (error) {
        console.error("Async evaluation error:", error.message);
    }
}

evaluateAsync();
```

### Expression Validation

```javascript
// Validate expression syntax
const isValid = engine.validate("Patient.name.family");
console.log("Expression is valid:", isValid);

const isInvalid = engine.validate("invalid..expression");
console.log("Invalid expression:", isInvalid);
```

### Utility Functions

```javascript
const { exists } = require('@octofhir/fhirpath');

// Check if expression returns any results
const hasResults = exists("name.where(use = 'official')", JSON.stringify(patient));
console.log("Has official name:", hasResults);
```

## Advanced Usage Patterns

### Conditional Logic

```rust
// Using where() function for filtering
let result = evaluate_expression("name.where(use = 'official').family", resource)?;

// Boolean expressions
let result = evaluate_expression("gender = 'male'", resource)?;

// Existence checks
let result = evaluate_expression("name.exists()", resource)?;
```

### Mathematical Operations

```rust
// Numeric calculations
let result = evaluate_expression("age + 10", resource)?;

// Comparisons
let result = evaluate_expression("age > 18", resource)?;
```

### String Operations

```rust
// String concatenation
let result = evaluate_expression("name.family + ', ' + name.given.first()", resource)?;

// String functions
let result = evaluate_expression("name.family.upper()", resource)?;
```

### Working with Collections

```rust
// Collection operations
let result = evaluate_expression("name.count()", resource)?;

// First and last elements
let result = evaluate_expression("name.first().family", resource)?;
let result = evaluate_expression("telecom.last().value", resource)?;
```

### Complex Nested Expressions

```rust
// Multi-level navigation
let result = evaluate_expression(
    "contact.where(relationship.coding.code = 'N').name.family",
    resource
)?;

// Combining multiple conditions
let result = evaluate_expression(
    "telecom.where(system = 'email' and use = 'work').value",
    resource
)?;
```

## Best Practices

### 1. Error Handling
Always handle potential errors when evaluating expressions:

```rust
match evaluate_expression(expression, resource) {
    Ok(result) => {
        // Process successful result
    }
    Err(error) => {
        // Handle error appropriately
        log::error!("FHIRPath evaluation failed: {}", error);
    }
}
```

### 2. Performance Considerations
- Use `evaluate_expression_optimized` for complex expressions
- Use streaming mode for large resources
- Cache compiled expressions when evaluating the same expression multiple times

### 3. Type Safety
Always check the result type before processing:

```rust
match result {
    FhirPathValue::Collection(values) => {
        for value in values {
            match value {
                FhirPathValue::String(s) => println!("String: {}", s),
                FhirPathValue::Boolean(b) => println!("Boolean: {}", b),
                FhirPathValue::Integer(i) => println!("Integer: {}", i),
                _ => println!("Other type: {:?}", value),
            }
        }
    }
    _ => println!("Not a collection: {:?}", result),
}
```

### 4. Resource Validation
Validate FHIR resources before evaluation:

```rust
// Ensure resource has required structure
if !resource.is_object() {
    return Err("Resource must be a JSON object".into());
}

if resource.get("resourceType").is_none() {
    return Err("Resource must have a resourceType".into());
}
```

## Troubleshooting

### Common Issues

1. **Parse Errors**: Check expression syntax
2. **Type Errors**: Ensure operations are performed on compatible types
3. **Empty Results**: Verify the path exists in the resource
4. **Performance Issues**: Consider using optimized evaluation or streaming mode

### Debug Mode

Enable debug logging to troubleshoot issues:

```rust
env_logger::init();
log::debug!("Evaluating expression: {}", expression);
```

For more detailed examples and API documentation, see the generated rustdoc documentation.
