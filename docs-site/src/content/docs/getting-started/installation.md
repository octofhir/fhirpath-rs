---
title: Installation
description: How to install OctoFHIR FHIRPath components
---

OctoFHIR FHIRPath provides multiple installation options depending on your use case.

## CLI Tool

The command-line interface is the easiest way to get started with FHIRPath evaluation.

### Install from Source

```bash
# Clone the repository
git clone https://github.com/octofhir/fhirpath-rs
cd fhirpath-rs

# Install the CLI tool
cargo install --path fhirpath-cli
```

### Verify Installation

```bash
# Check if the CLI tool is installed correctly
octofhir-fhirpath --help
```

## Rust Library

To use FHIRPath in your Rust project, add it as a dependency.

### Add to Cargo.toml

```toml
[dependencies]
fhirpath-core = "0.1.0"
```

### Basic Usage

```rust
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fhir_resource: Value = serde_json::from_str(r#"
    {
        "resourceType": "Patient",
        "name": [{"given": ["John"]}]
    }
    "#)?;
    
    let result = evaluate_expression("name.given", fhir_resource)?;
    println!("Result: {:?}", result);
    Ok(())
}
```

## Node.js Package

For JavaScript and TypeScript projects, install the Node.js bindings.

### Install via npm

```bash
npm install fhirpath-node
```

### Install via yarn

```bash
yarn add fhirpath-node
```

### Basic Usage

```javascript
const { evaluateExpression } = require('fhirpath-node');

const patient = {
    resourceType: "Patient",
    name: [{ given: ["John"] }]
};

const result = evaluateExpression("name.given", patient);
console.log("Result:", result);
```

## Requirements

- **Rust**: Version 1.70 or higher for building from source
- **Node.js**: Version 16 or higher for Node.js bindings
- **Operating System**: Linux, macOS, or Windows

## Next Steps

Once you have installed OctoFHIR FHIRPath, check out the [Quick Start guide](/getting-started/quick-start/) to learn how to use it effectively.
