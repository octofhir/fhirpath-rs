# FHIRPath CLI Examples

This directory contains example code demonstrating various ways to use the FHIRPath CLI library programmatically.

## Running Examples

Run any example using cargo:

```bash
# From the workspace root
cargo run --package fhirpath-cli --example basic_evaluation

# From the CLI crate directory
cd crates/fhirpath-cli
cargo run --example basic_evaluation
```

## Available Examples

### 1. Basic Evaluation (`basic_evaluation.rs`)

Demonstrates fundamental FHIRPath expression evaluation:
- Creating a CLI context
- Loading FHIR resources
- Evaluating simple expressions
- Using FHIRPath functions (where, count, first, etc.)
- Working with variables

**Run:**
```bash
cargo run --example basic_evaluation
```

**Key Concepts:**
- Context creation and configuration
- Model provider initialization
- Expression evaluation workflow
- Variable substitution

### 2. Batch Processing (`batch_processing.rs`)

Shows how to process multiple FHIR resources:
- Creating multiple test files
- Processing files in batch
- Collecting and aggregating results
- Error handling strategies

**Run:**
```bash
cargo run --example batch_processing
```

**Key Concepts:**
- File iteration patterns
- Result collection
- Batch validation
- Error recovery

### 3. Custom Formatter (`custom_formatter.rs`)

Demonstrates different output formatting options:
- Pretty format (Ariadne-style diagnostics)
- JSON format (machine-readable)
- Raw format (plain text)
- Formatting errors and empty results
- Handling collections

**Run:**
```bash
cargo run --example custom_formatter
```

**Key Concepts:**
- Output format selection
- Custom formatting logic
- Error output handling
- Collection formatting

## Common Patterns

### Creating a CLI Context

```rust
use fhirpath_cli::cli::context::CliContext;
use fhirpath_cli::cli::output::OutputFormat;

let context = CliContext::new(
    OutputFormat::Pretty,  // Output format
    false,                 // no_color
    false,                 // quiet
    false,                 // verbose
    "r4".to_string(),      // FHIR version
    vec![],                // packages
    false,                 // profile
);
```

### Initializing Model Provider

```rust
use fhirpath_cli::EmbeddedModelProvider;
use octofhir_fhir_model::provider::FhirVersion;
use std::sync::Arc;

let model_provider = Arc::new(
    EmbeddedModelProvider::new(FhirVersion::R4)
);
```

### Evaluating an Expression

```rust
use fhirpath_cli::cli::handlers;

handlers::handle_evaluate(
    "Patient.name.family",      // Expression
    Some("patient.json"),       // Input file
    &[],                        // Variables
    false,                      // pretty (for JSON)
    false,                      // analyze
    &context,                   // CLI context
    &model_provider,            // Model provider
).await;
```

### Using Variables

```rust
handlers::handle_evaluate(
    "Patient.name.where(use = %nameUse).family",
    Some("patient.json"),
    &["nameUse=official".to_string()],  // Variable definition
    false,
    false,
    &context,
    &model_provider,
).await;
```

### Creating Test Resources

```rust
use tempfile::NamedTempFile;
use std::io::Write;

let patient_json = r#"
{
    "resourceType": "Patient",
    "id": "example",
    "active": true
}
"#;

let temp_file = NamedTempFile::new()?;
std::fs::write(temp_file.path(), patient_json)?;
```

## Integration with Your Application

### As a Library

You can use the CLI handlers in your own Rust applications:

```rust
use fhirpath_cli::{
    EmbeddedModelProvider,
    cli::context::CliContext,
    cli::handlers,
    cli::output::OutputFormat,
};
use octofhir_fhir_model::provider::FhirVersion;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup
    let context = CliContext::new(
        OutputFormat::Json,
        false,
        true,  // quiet mode
        false,
        "r4".to_string(),
        vec![],
        false,
    );
    
    let model_provider = Arc::new(
        EmbeddedModelProvider::new(FhirVersion::R4)
    );
    
    // Evaluate
    handlers::handle_evaluate(
        "Patient.active",
        Some("patient.json"),
        &[],
        false,
        false,
        &context,
        &model_provider,
    ).await;
    
    Ok(())
}
```

### As a CLI Tool

The examples can also guide CLI usage:

```bash
# Equivalent CLI command for the above code
octofhir-fhirpath evaluate "Patient.active" \
    --input patient.json \
    --output-format json \
    --quiet
```

## Testing Your Code

Use these examples as templates for testing:

```rust
#[tokio::test]
async fn test_patient_name_extraction() {
    let context = CliContext::new(
        OutputFormat::Json,
        false,
        true,
        false,
        "r4".to_string(),
        vec![],
        false,
    );
    
    let model_provider = Arc::new(
        EmbeddedModelProvider::new(FhirVersion::R4)
    );
    
    // Test evaluation
    handlers::handle_evaluate(
        "Patient.name.family",
        Some("tests/fixtures/patient.json"),
        &[],
        false,
        false,
        &context,
        &model_provider,
    ).await;
    
    // Assert results...
}
```

## Error Handling

All examples demonstrate proper error handling:

```rust
// Using anyhow for error propagation
async fn main() -> anyhow::Result<()> {
    // Operations that may fail
    let file = std::fs::read_to_string("patient.json")?;
    
    // Async operations
    handlers::handle_evaluate(...).await;
    
    Ok(())
}
```

## Performance Considerations

- Model provider initialization is expensive - reuse when possible
- Use quiet mode for batch processing to reduce I/O overhead
- Consider JSON output format for machine parsing
- Use Arc for sharing model providers across evaluations

## Further Reading

- [CLI README](../README.md) - Full CLI documentation
- [Main README](../../../README.md) - Project overview
- [API Docs](https://docs.rs/octofhir-fhirpath) - FHIRPath library documentation
- [FHIRPath Spec](http://hl7.org/fhirpath/) - FHIRPath specification

## Contributing

Feel free to contribute more examples! Please ensure they:
- Compile without warnings
- Include comprehensive comments
- Demonstrate a specific use case
- Follow the existing code style
- Include error handling

## Questions?

- Open an issue on GitHub
- Check the main documentation
- Review existing examples for patterns
