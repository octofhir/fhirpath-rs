# Examples and Usage

This document provides comprehensive examples of using octofhir-fhirpath in various scenarios.

## Table of Contents

- [Basic Usage](#basic-usage)
- [FHIRPath Engine](#fhirpath-engine)
- [Value System](#value-system)
- [Expression Parsing](#expression-parsing)  
- [Reference Resolution](#reference-resolution)
- [Environment Variables](#environment-variables)
- [Error Handling](#error-handling)
- [Advanced Usage](#advanced-usage)

## Basic Usage

### Simple Example

The easiest way to get started:

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with mock provider (good for testing)
    let engine = FhirPathEngine::with_mock_provider();
    
    // Simple FHIR Patient
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["Alice"], "family": "Smith"}]
    });
    
    // Extract the first name
    let result = engine.evaluate("Patient.name.given", patient).await?;
    println!("First name: {:?}", result); // Outputs: ["Alice"]
    
    Ok(())
}
```

### Complete Example

For more advanced usage:

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with mock provider (easiest approach)
    let engine = FhirPathEngine::with_mock_provider();
    
    // Or create with custom model provider:
    // use octofhir_fhirpath::MockModelProvider;
    // let model_provider = Arc::new(MockModelProvider::new());
    // let engine = FhirPathEngine::with_model_provider(model_provider);
    
    // Sample FHIR Patient resource
    let patient = json!({
        "resourceType": "Patient",
        "name": [{
            "use": "official",
            "given": ["John"],
            "family": "Doe"
        }],
        "telecom": [{
            "system": "phone",
            "value": "+1-555-123-4567"
        }]
    });
    
    // Evaluate FHIRPath expressions
    let result = engine.evaluate("Patient.name.given", patient.clone()).await?;
    println!("Given names: {:?}", result);
    
    let phone = engine.evaluate("Patient.telecom.where(system='phone').value", patient).await?;
    println!("Phone: {:?}", phone);
    
    Ok(())
}
```

## FHIRPath Engine

### Engine Creation

The `FhirPathEngine` is the main entry point for evaluating FHIRPath expressions. **As of v0.3.0, a model provider is required:**

```rust
use octofhir_fhirpath::FhirPathEngine;

// Create with model provider (unified engine approach)
let engine = FhirPathEngine::with_mock_provider();
let result = engine.evaluate("Patient.name.family", fhir_resource).await?;
```

### Custom Model Provider

```rust
use octofhir_fhirpath::{FhirPathEngine, MockModelProvider};
use std::sync::Arc;

let model_provider = Arc::new(MockModelProvider::new());
let engine = FhirPathEngine::with_model_provider(model_provider);
```

## Value System

FHIRPath expressions return `FhirPathValue` which represents various FHIR data types:

```rust
use octofhir_fhirpath::FhirPathValue;

match result {
    FhirPathValue::String(s) => println!("String: {}", s),
    FhirPathValue::Integer(i) => println!("Integer: {}", i),
    FhirPathValue::Boolean(b) => println!("Boolean: {}", b),
    FhirPathValue::Collection(items) => println!("Collection with {} items", items.len()),
    FhirPathValue::Empty => println!("No result"),
}
```

### Working with Collections

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "name": [
            {"given": ["John"], "family": "Doe"},
            {"given": ["Jane"], "family": "Smith"}
        ]
    });
    
    // Get all family names
    let families = engine.evaluate("Patient.name.family", patient).await?;
    if let FhirPathValue::Collection(items) = families {
        for item in items {
            if let FhirPathValue::String(name) = item {
                println!("Family name: {}", name);
            }
        }
    }
    
    Ok(())
}
```

## Expression Parsing

Parse and analyze FHIRPath expressions:

```rust
use octofhir_fhirpath::parser::parse;

let expression = parse("Patient.name.where(use = 'official').given")?;
println!("Parsed AST: {:#?}", expression);
```

### Syntax Validation

```rust
use octofhir_fhirpath::parser::parse;

fn validate_expression(expr: &str) -> bool {
    match parse(expr) {
        Ok(_) => {
            println!("✅ Valid expression: {}", expr);
            true
        }
        Err(e) => {
            println!("❌ Invalid expression: {} - Error: {}", expr, e);
            false
        }
    }
}

// Example usage
validate_expression("Patient.name.given");      // ✅ Valid
validate_expression("Patient.name.invalid(");   // ❌ Invalid
```

## Reference Resolution

Advanced reference resolution with full Bundle support:

### Bundle with Cross-References

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    // Bundle with references between entries
    let bundle = json!({
        "resourceType": "Bundle",
        "type": "searchset", 
        "entry": [
            {
                "fullUrl": "http://example.com/Patient/123",
                "resource": {
                    "resourceType": "Patient",
                    "id": "123",
                    "name": [{"family": "Doe", "given": ["Jane"]}]
                }
            },
            {
                "fullUrl": "http://example.com/Observation/456", 
                "resource": {
                    "resourceType": "Observation",
                    "id": "456",
                    "subject": {"reference": "Patient/123"},
                    "valueQuantity": {"value": 98.6, "unit": "F"}
                }
            }
        ]
    });
    
    // Resolve references within Bundle context
    let result = engine.evaluate(
        "Bundle.entry[1].resource.subject.resolve().name.family",
        bundle
    ).await?;
    
    println!("Patient family name: {:?}", result); // "Doe"
    Ok(())
}
```

### Contained Resources

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "id": "patient-1",
        "generalPractitioner": [
            {"reference": "#practitioner-1"}
        ],
        "contained": [
            {
                "resourceType": "Practitioner",
                "id": "practitioner-1",
                "name": [{"family": "Smith", "given": ["Dr. John"]}]
            }
        ]
    });
    
    // Resolve contained resource reference
    let result = engine.evaluate(
        "Patient.generalPractitioner.resolve().name.family",
        patient
    ).await?;
    
    println!("Practitioner family name: {:?}", result); // "Smith"
    Ok(())
}
```

### Reference Resolution Features

- **Contained Resources**: Resolves `#id` references to contained resources
- **Bundle Entry Resolution**: Resolves references between Bundle entries using `fullUrl`
- **Relative References**: Handles `ResourceType/id` patterns within Bundle context  
- **Absolute URL References**: Supports full URLs and URN references
- **Multiple References**: Handles collections of references efficiently

## Environment Variables

FHIRPath supports environment variables for dynamic expressions:

### Standard Environment Variables

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["Alice"]}]
    });
    
    // Use %resource to get the containing resource
    let result = engine.evaluate("%resource.resourceType", patient).await?;
    println!("Resource type: {:?}", result); // "Patient"
    
    Ok(())
}
```

### Custom Environment Variables

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "birthDate": "1990-05-15"
    });
    
    // Set up custom variables
    let mut variables = HashMap::new();
    variables.insert("minAge".to_string(), FhirPathValue::Integer(18));
    variables.insert("today".to_string(), FhirPathValue::String("2023-05-15".into()));
    
    // Use custom variables in expression
    let result = engine.evaluate_with_variables(
        "Patient.birthDate <= %today", 
        patient, 
        variables
    ).await?;
    
    println!("Born before today: {:?}", result);
    
    Ok(())
}
```

### Complex Variable Examples

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let bundle = json!({
        "resourceType": "Bundle",
        "entry": [
            {"resource": {"resourceType": "Patient", "active": true}},
            {"resource": {"resourceType": "Patient", "active": false}},
            {"resource": {"resourceType": "Observation", "status": "final"}}
        ]
    });
    
    // Complex filtering with variables
    let mut variables = HashMap::new();
    variables.insert("resourceType".to_string(), FhirPathValue::String("Patient".into()));
    variables.insert("activeOnly".to_string(), FhirPathValue::Boolean(true));
    
    let result = engine.evaluate_with_variables(
        "Bundle.entry.resource.where(resourceType = %resourceType and active = %activeOnly)",
        bundle,
        variables
    ).await?;
    
    println!("Active patients: {:?}", result);
    Ok(())
}
```

## Error Handling

### Basic Error Handling

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathError};
use serde_json::json;

#[tokio::main]
async fn main() {
    let engine = FhirPathEngine::with_mock_provider();
    let patient = json!({"resourceType": "Patient"});
    
    match engine.evaluate("Patient.name.invalidFunction()", patient).await {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => {
            println!("Error: {}", e);
            // Error includes line/column information and suggestions
        }
    }
}
```

### Rich Error Information

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathError};

fn handle_fhirpath_error(error: FhirPathError) {
    println!("Error Type: {:?}", error.kind());
    println!("Message: {}", error);
    
    if let Some(location) = error.location() {
        println!("Location: line {}, column {}", location.line, location.column);
    }
    
    for suggestion in error.suggestions() {
        println!("Suggestion: {}", suggestion);
    }
}
```

### Error Recovery

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    let patient = json!({"resourceType": "Patient"});
    
    // Multiple expressions with error handling
    let expressions = vec![
        "Patient.name.family",           // Valid
        "Patient.name.invalidMethod()",  // Invalid - will error
        "Patient.resourceType",          // Valid fallback
    ];
    
    for expr in expressions {
        match engine.evaluate(expr, patient.clone()).await {
            Ok(result) => println!("✅ {}: {:?}", expr, result),
            Err(e) => println!("❌ {}: {}", expr, e),
        }
    }
    
    Ok(())
}
```

## Advanced Usage

### Conditional Logic

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "gender": "female",
        "birthDate": "1985-03-20"
    });
    
    // Conditional expressions using iif()
    let result = engine.evaluate(
        "Patient.gender = 'female'.iif('Ms.', 'Mr.')",
        patient.clone()
    ).await?;
    println!("Title: {:?}", result);
    
    // Complex conditional logic
    let age_category = engine.evaluate(
        "today().toString().substring(0,4).toInteger() - Patient.birthDate.substring(0,4).toInteger() > 65.iif('Senior', 'Adult')",
        patient
    ).await?;
    println!("Age category: {:?}", age_category);
    
    Ok(())
}
```

### Collection Operations

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let bundle = json!({
        "resourceType": "Bundle",
        "entry": [
            {"resource": {"resourceType": "Patient", "active": true, "gender": "male"}},
            {"resource": {"resourceType": "Patient", "active": false, "gender": "female"}},
            {"resource": {"resourceType": "Patient", "active": true, "gender": "female"}},
            {"resource": {"resourceType": "Observation", "status": "final"}}
        ]
    });
    
    // Count patients by gender
    let male_count = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient' and gender = 'male').count()",
        bundle.clone()
    ).await?;
    println!("Male patients: {:?}", male_count);
    
    // Get distinct resource types
    let resource_types = engine.evaluate(
        "Bundle.entry.resource.resourceType.distinct()",
        bundle.clone()
    ).await?;
    println!("Resource types: {:?}", resource_types);
    
    // Complex filtering and aggregation
    let active_female_patients = engine.evaluate(
        "Bundle.entry.resource.where(resourceType = 'Patient' and active = true and gender = 'female').count()",
        bundle
    ).await?;
    println!("Active female patients: {:?}", active_female_patients);
    
    Ok(())
}
```

### String Manipulation

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "name": [{
            "given": ["John", "Michael"],
            "family": "Doe"
        }],
        "telecom": [{
            "value": "john.doe@example.com"
        }]
    });
    
    // String operations
    let full_name = engine.evaluate(
        "(Patient.name.given.join(' ') + ' ' + Patient.name.family).trim()",
        patient.clone()
    ).await?;
    println!("Full name: {:?}", full_name);
    
    // Email domain extraction
    let email_domain = engine.evaluate(
        "Patient.telecom.value.where(contains('@')).substring(indexOf('@') + 1)",
        patient.clone()
    ).await?;
    println!("Email domain: {:?}", email_domain);
    
    // Pattern matching
    let has_valid_email = engine.evaluate(
        "Patient.telecom.value.matches('[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\\\.[a-zA-Z]{2,}')",
        patient
    ).await?;
    println!("Has valid email: {:?}", has_valid_email);
    
    Ok(())
}
```

### Mathematical Operations

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let observation = json!({
        "resourceType": "Observation",
        "valueQuantity": {
            "value": 98.6,
            "unit": "°F"
        }
    });
    
    // Temperature conversion (Fahrenheit to Celsius)
    let celsius = engine.evaluate(
        "(Observation.valueQuantity.value - 32) * 5 / 9",
        observation.clone()
    ).await?;
    println!("Temperature in Celsius: {:?}", celsius);
    
    // Mathematical functions
    let rounded_temp = engine.evaluate(
        "((Observation.valueQuantity.value - 32) * 5 / 9).round()",
        observation.clone()
    ).await?;
    println!("Rounded temperature: {:?}", rounded_temp);
    
    // Statistical operations on collections
    let values = json!({
        "values": [1, 2, 3, 4, 5]
    });
    
    let average = engine.evaluate(
        "values.sum() / values.count()",
        values
    ).await?;
    println!("Average: {:?}", average);
    
    Ok(())
}
```

### Date and Time Operations

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "birthDate": "1990-05-15"
    });
    
    // Current date operations
    let is_birthday_today = engine.evaluate(
        "Patient.birthDate.substring(5) = today().toString().substring(5)",
        patient.clone()
    ).await?;
    println!("Is birthday today: {:?}", is_birthday_today);
    
    // Age calculation (simplified)
    let birth_year = engine.evaluate(
        "Patient.birthDate.substring(0, 4).toInteger()",
        patient.clone()
    ).await?;
    println!("Birth year: {:?}", birth_year);
    
    // Date boundaries
    let birth_date_boundaries = engine.evaluate(
        "Patient.birthDate.toDate().lowBoundary()",
        patient
    ).await?;
    println!("Birth date low boundary: {:?}", birth_date_boundaries);
    
    Ok(())
}
```

This comprehensive guide covers most common usage patterns for octofhir-fhirpath. For more advanced scenarios, refer to the [Architecture Guide](ARCHITECTURE.md) and the official [FHIRPath specification](http://hl7.org/fhirpath/).