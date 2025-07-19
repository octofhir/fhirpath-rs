---
title: Rust Library Usage
description: Complete guide to using OctoFHIR FHIRPath in Rust applications
---

The OctoFHIR FHIRPath Rust library provides a high-performance FHIRPath engine that can be integrated into Rust applications.

## Installation

Add the library to your `Cargo.toml`:

```toml
[dependencies]
fhirpath-core = "0.1.0"
serde_json = "1.0"
```

## Basic Usage

### Simple Expression Evaluation

```rust
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fhir_resource: Value = serde_json::from_str(r#"
    {
        "resourceType": "Patient",
        "name": [
            {
                "given": ["John", "Doe"]
            }
        ]
    }
    "#)?;

    let result = evaluate_expression("Patient.name.given", fhir_resource)?;
    println!("Given names: {:?}", result);
    
    Ok(())
}
```

### Working with Different Data Types

```rust
use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::model::FhirPathValue;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let patient = json!({
        "resourceType": "Patient",
        "id": "example",
        "name": [{
            "family": "Smith",
            "given": ["John"]
        }],
        "gender": "male",
        "active": true
    });

    // String values
    let family_name = evaluate_expression("name.family", patient.clone())?;
    println!("Family name: {:?}", family_name);

    // Boolean values
    let is_active = evaluate_expression("active", patient.clone())?;
    println!("Is active: {:?}", is_active);

    // Array values
    let given_names = evaluate_expression("name.given", patient.clone())?;
    println!("Given names: {:?}", given_names);

    Ok(())
}
```

## Advanced Usage

### Error Handling

```rust
use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::error::FhirPathError;
use serde_json::Value;

fn safe_evaluate(expression: &str, resource: Value) -> Result<String, String> {
    match evaluate_expression(expression, resource) {
        Ok(result) => Ok(format!("{:?}", result)),
        Err(FhirPathError::SyntaxError { message, position }) => {
            Err(format!("Syntax error at position {}: {}", position, message))
        }
        Err(FhirPathError::EvaluationError { message }) => {
            Err(format!("Evaluation error: {}", message))
        }
        Err(e) => Err(format!("Unknown error: {:?}", e))
    }
}
```

### Working with Complex Resources

```rust
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = json!({
        "resourceType": "Bundle",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "name": [{"given": ["Alice"]}]
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "valueQuantity": {
                        "value": 120,
                        "unit": "mmHg"
                    }
                }
            }
        ]
    });

    // Extract all resource types
    let resource_types = evaluate_expression(
        "entry.resource.resourceType", 
        bundle.clone()
    )?;
    println!("Resource types: {:?}", resource_types);

    // Extract patient names
    let patient_names = evaluate_expression(
        "entry.resource.where(resourceType = 'Patient').name.given",
        bundle.clone()
    )?;
    println!("Patient names: {:?}", patient_names);

    // Extract observation values
    let obs_values = evaluate_expression(
        "entry.resource.where(resourceType = 'Observation').valueQuantity.value",
        bundle
    )?;
    println!("Observation values: {:?}", obs_values);

    Ok(())
}
```

### Expression Validation

```rust
use fhirpath_core::parser::parse_expression;

fn validate_expression(expression: &str) -> bool {
    match parse_expression(expression) {
        Ok(_) => {
            println!("Expression '{}' is valid", expression);
            true
        }
        Err(e) => {
            println!("Expression '{}' is invalid: {:?}", expression, e);
            false
        }
    }
}

fn main() {
    validate_expression("Patient.name.given");     // Valid
    validate_expression("Patient..invalid");       // Invalid
    validate_expression("name.where(use = 'official')"); // Valid
}
```

## Integration Patterns

### Building a FHIRPath Service

```rust
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::Value;
use std::collections::HashMap;

pub struct FhirPathService {
    resources: HashMap<String, Value>,
}

impl FhirPathService {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn add_resource(&mut self, id: String, resource: Value) {
        self.resources.insert(id, resource);
    }

    pub fn evaluate(&self, id: &str, expression: &str) -> Result<String, String> {
        let resource = self.resources.get(id)
            .ok_or_else(|| format!("Resource '{}' not found", id))?;

        match evaluate_expression(expression, resource.clone()) {
            Ok(result) => Ok(format!("{:?}", result)),
            Err(e) => Err(format!("Evaluation failed: {:?}", e))
        }
    }
}

// Usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut service = FhirPathService::new();
    
    let patient = serde_json::from_str(r#"
    {
        "resourceType": "Patient",
        "name": [{"given": ["John"]}]
    }
    "#)?;
    
    service.add_resource("patient1".to_string(), patient);
    
    let result = service.evaluate("patient1", "name.given")?;
    println!("Result: {}", result);
    
    Ok(())
}
```

### Async Integration

```rust
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::Value;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resource: Value = serde_json::from_str(r#"
    {
        "resourceType": "Patient",
        "name": [{"given": ["Alice"]}]
    }
    "#)?;

    // FHIRPath evaluation is CPU-bound, so use spawn_blocking
    let result = tokio::task::spawn_blocking(move || {
        evaluate_expression("name.given", resource)
    }).await??;

    println!("Async result: {:?}", result);
    Ok(())
}
```

## Performance Considerations

### Reusing Parsed Expressions

For better performance when evaluating the same expression multiple times:

```rust
use fhirpath_core::parser::parse_expression;
use fhirpath_core::evaluator::evaluate_parsed;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse expression once
    let parsed_expr = parse_expression("name.given")?;
    
    let resources = vec![
        serde_json::from_str(r#"{"resourceType": "Patient", "name": [{"given": ["John"]}]}"#)?,
        serde_json::from_str(r#"{"resourceType": "Patient", "name": [{"given": ["Jane"]}]}"#)?,
    ];

    // Evaluate against multiple resources
    for resource in resources {
        let result = evaluate_parsed(&parsed_expr, resource)?;
        println!("Result: {:?}", result);
    }

    Ok(())
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_patient_name_extraction() {
        let patient = json!({
            "resourceType": "Patient",
            "name": [{"given": ["John"], "family": "Doe"}]
        });

        let given_result = evaluate_expression("name.given", patient.clone()).unwrap();
        let family_result = evaluate_expression("name.family", patient).unwrap();

        // Add your assertions here
        assert!(!given_result.is_empty());
        assert!(!family_result.is_empty());
    }
}
```

## Next Steps

- Explore more [usage examples](/examples/usage-examples/)
- Learn about [performance optimization](/development/performance/)
- Check out [Node.js integration](/usage/nodejs/)
- Read the [API reference](/reference/api/)
