# fhirpath-model

Data model and value types for FHIRPath expressions.

## Overview

This crate provides the core data types and model provider infrastructure for FHIRPath evaluation:

- **FhirPathValue**: The fundamental value type that represents all possible FHIRPath values
- **FhirResource**: Wrapper for FHIR resources with property access methods
- **Quantity**: UCUM-aware quantity type with unit conversion support
- **ModelProvider**: Trait for providing FHIR type information
- **FhirSchema**: Support for loading and using FHIR Schema definitions

## Features

- `async-schema`: Enable async FHIR Schema loading from URLs
- `diagnostics`: Enable integration with fhirpath-diagnostics

## Usage

### Basic Value Types

```rust
use fhirpath_model::{FhirPathValue, Quantity};
use rust_decimal::Decimal;

// Create different value types
let bool_val = FhirPathValue::Boolean(true);
let int_val = FhirPathValue::Integer(42);
let str_val = FhirPathValue::String("hello".to_string());
let qty_val = FhirPathValue::quantity(
    Decimal::from(5),
    Some("mg".to_string())
);

// Collections
let collection = FhirPathValue::collection(vec![
    FhirPathValue::Integer(1),
    FhirPathValue::Integer(2),
    FhirPathValue::Integer(3),
]);

// Convert from JSON
let json = serde_json::json!({
    "resourceType": "Patient",
    "id": "123"
});
let resource = FhirPathValue::from(json);
```

### Working with Quantities

```rust
use fhirpath_model::Quantity;
use rust_decimal::Decimal;

let q1 = Quantity::new(Decimal::from(5), Some("mg".to_string()));
let q2 = Quantity::new(Decimal::from(3), Some("mg".to_string()));

// Arithmetic with compatible units
let sum = q1.add(&q2)?;
assert_eq!(sum.value, Decimal::from(8));

// Multiplication by scalar
let doubled = q1.multiply_scalar(Decimal::from(2));
assert_eq!(doubled.value, Decimal::from(10));
```

### Model Provider

```rust
use fhirpath_model::{ModelProvider, FhirVersion};

// Use the empty provider for basic functionality
let provider = fhirpath_model::provider::EmptyModelProvider;

// With async-schema feature:
#[cfg(feature = "async-schema")]
{
    use fhirpath_model::FhirSchemaProvider;
    
    let provider = FhirSchemaProvider::from_url(
        "https://fhir-schema.github.io/fhir-schema/r5/fhir.schema.json"
    ).await?;
    
    // Query type information
    if let Some(type_info) = provider.get_type_info("Patient") {
        println!("Patient is a: {:?}", type_info);
    }
    
    // Get property types
    if let Some(prop_type) = provider.get_property_type("Patient", "active") {
        println!("Patient.active type: {:?}", prop_type);
    }
}
```

## Type System

The crate provides a comprehensive type system that mirrors FHIRPath's type model:

- **Primitive Types**: Boolean, Integer, Decimal, String, Date, DateTime, Time
- **Complex Types**: Quantity, Resource
- **Collections**: All values can be collections
- **Type Modifiers**: Optional, Union types

## FHIR Schema Integration

When the `async-schema` feature is enabled, the crate can load and use FHIR Schema definitions for accurate type information across different FHIR versions (R4, R4B, R5).

## License

This project is licensed under the Apache-2.0 license.