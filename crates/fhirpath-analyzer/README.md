# FHIRPath Analyzer

A high-performance, specification-compliant static analysis engine for FHIRPath expressions.

## Features

- 🔍 **Type Inference** - Automatic type detection for literals, identifiers, and complex expressions
- ✅ **Function Validation** - Comprehensive signature validation against the function registry  
- 🔗 **Union Type Analysis** - Advanced support for children() function and choice types
- ⚡ **High Performance** - <100μs analysis time with aggressive caching
- 🔧 **Zero AST Changes** - External mapping preserves existing functionality
- 📊 **Rich Diagnostics** - Detailed error messages with helpful suggestions

## Quick Start

### Basic Usage

```rust
use octofhir_fhirpath_analyzer::{FhirPathAnalyzer};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create analyzer with ModelProvider
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);
    
    // Analyze FHIRPath expression
    let result = analyzer.analyze("Patient.name.given").await?;
    
    // Inspect analysis results
    println!("Type annotations: {}", result.type_annotations.len());
    println!("Validation errors: {}", result.validation_errors.len());
    println!("Function calls: {}", result.function_calls.len());
    
    Ok(())
}
```

### Engine Integration

```rust
use octofhir_fhirpath::FhirPathEngine;
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use sonic_rs::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with model provider
    let provider = Box::new(MockModelProvider::new());
    let mut engine = FhirPathEngine::with_model_provider(provider);
    
    let patient = json!({"resourceType": "Patient", "name": [{"given": ["John"]}]});
    
    // Regular evaluation
    let result = engine.evaluate("Patient.name.given", patient).await?;
    
    println!("Result: {:?}", result);
    
    Ok(())
}
```

## Architecture

The analyzer uses an external mapping approach to provide rich analysis without modifying the existing AST:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Expression    │───▶│  Parser (AST)    │───▶│   Analyzer      │
│   "Patient.name"│    │  ExpressionNode  │    │  + Semantic     │
└─────────────────┘    └──────────────────┘    │    Mapping      │
                                               └─────────────────┘
                                                        │
                                               ┌─────────────────┐
                                               │ Analysis Result │
                                               │ • Type Info     │
                                               │ • Validation    │
                                               │ • Suggestions   │
                                               └─────────────────┘
```

## Advanced Features

### Function Registry Integration

```rust
use octofhir_fhirpath_analyzer::FhirPathAnalyzer;
use octofhir_fhirpath_registry::create_standard_registry;
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(create_standard_registry().await?);
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

    // Validates function signatures
    let result = analyzer.analyze("substring('hello', 1, 3)").await?;
    
    // Check for validation errors
    for error in result.validation_errors {
        println!("Error: {}", error.message);
        println!("Suggestions: {:?}", error.suggestions);
    }
    
    Ok(())
}
```

### Children() Function Analysis

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);
    
    // Analyzes union types from children() function
    let result = analyzer.analyze("Patient.children().ofType(HumanName)").await?;

    // Check for union type analysis
    if !result.union_types.is_empty() {
        println!("Found union types from children() analysis");
        for (node_id, union_info) in result.union_types {
            println!("Node {}: {} types, collection: {}", 
                node_id, 
                union_info.constituent_types.len(),
                union_info.is_collection
            );
        }
    }

    // Provides suggestions for invalid type operations
    for error in result.validation_errors {
        println!("Error: {}", error.message);
        if !error.suggestions.is_empty() {
            println!("Suggestions: {}", error.suggestions.join(", "));
        }
    }
    
    Ok(())
}
```

### Custom Configuration

```rust
use octofhir_fhirpath_analyzer::{AnalyzerConfig, AnalysisSettings};

let config = AnalyzerConfig {
    settings: AnalysisSettings {
        enable_type_inference: true,
        enable_function_validation: true,
        enable_union_analysis: false, // Disable for performance
        max_analysis_depth: 50,
    },
    cache_size: 5000,
    enable_profiling: true,
};

let analyzer = FhirPathAnalyzer::with_config(provider, config);
```

## CLI Usage

The analyzer integrates with the FHIRPath CLI tools:

```bash
# Analyze expression
just cli-evaluate "Patient.name.given"

# Parse and validate expression
cargo run --bin octofhir-fhirpath -- parse "Patient.children().ofType(HumanName)"

# Get help
cargo run --bin octofhir-fhirpath -- --help
```

## Performance

- **Analysis Speed**: <100μs for typical expressions with caching
- **Memory Overhead**: <10% increase when enabled
- **Cache Hit Rate**: >90% for repeated expressions  
- **Concurrent Support**: 1000+ concurrent operations

## Error Messages

The analyzer provides detailed error messages with suggestions:

```
❌ Validation Error: Function 'children' expects 0 parameters, got 1
   Suggestions: children() takes no arguments

❌ Validation Error: Type 'InvalidType' is not a valid child type for ofType operation on children()
   Suggestions: HumanName, Identifier, ContactPoint, Address, CodeableConcept
```

## Integration with Existing Code

The analyzer is designed for zero-impact integration:

- ✅ No AST modifications required
- ✅ Existing tests continue to pass
- ✅ Optional analysis features
- ✅ Backward compatible API

## Specification Compliance

Supports FHIRPath specification features:

- ✅ All literal types (String, Integer, Decimal, Boolean, Date, DateTime, Time, Quantity)
- ✅ Function signature validation  
- ✅ Union type analysis for children() and choice types
- ✅ Type filtering operations (ofType, is, as)
- ✅ Complex expression analysis
- ✅ Boolean logic operators (and, or, xor, implies, not)

## Examples

See the `examples/` directory for comprehensive usage examples:

- `basic_analysis.rs` - Type inference and basic validation
- `function_validation.rs` - Function signature validation  
- `children_analysis.rs` - Children function and union type analysis

Run examples with:
```bash
cargo run --example basic_analysis
cargo run --example function_validation  
cargo run --example children_analysis
```

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development setup and guidelines.

## License

Licensed under MIT OR Apache-2.0.