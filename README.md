# FHIRPath for Rust 🚀

[![Crates.io](https://img.shields.io/crates/v/octofhir-fhirpath.svg)](https://crates.io/crates/octofhir-fhirpath)
[![Documentation](https://docs.rs/octofhir-fhirpath/badge.svg)](https://docs.rs/octofhir-fhirpath)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/octofhir/fhirpath-rs/blob/main/LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.87+-blue.svg)](https://www.rust-lang.org)

**Fast, safe, and production-ready FHIRPath implementation in Rust.**

FHIRPath is the standard query language for navigating FHIR healthcare data. This library provides high-performance evaluation with **90.9% specification compliance**.

## ✨ Quick Example

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider().await?;
    
    let patient = json!({
        "resourceType": "Patient",
        "name": [
            {"given": ["Alice"], "family": "Smith"},
            {"given": ["Dr. Alice"], "family": "Smith", "use": "professional"}
        ],
        "active": true
    });
    
    // Get all given names
    let names = engine.evaluate("Patient.name.given", patient.clone()).await?;
    println!("Names: {}", names); // ["Alice", "Dr. Alice"]
    
    // Filter by use and get family name
    let professional = engine.evaluate(
        "Patient.name.where(use = 'professional').family.first()", 
        patient.clone()
    ).await?;
    println!("Professional name: {}", professional); // "Smith"
    
    Ok(())
}
```

## 🚀 Installation

### As a Library

```toml
[dependencies]
octofhir-fhirpath = "0.4"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### As a CLI Tool

```bash
cargo install octofhir-fhirpath
```

## 📖 Getting Started

### Basic Library Usage

```rust
use octofhir_fhirpath::FhirPathEngine;
use serde_json::json;

// Create engine (use MockModelProvider for simple cases)
let engine = FhirPathEngine::with_mock_provider().await?;

// Your FHIR data
let patient = json!({"resourceType": "Patient", "active": true});

// Evaluate expressions
let result = engine.evaluate("Patient.active", patient).await?;
println!("Active: {}", result); // [true]
```

### Production Usage with Full FHIR Support

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use octofhir_fhirpath_model::FhirSchemaModelProvider;

// Create engine with full FHIR R5 schema support
let model_provider = FhirSchemaModelProvider::r5().await?;
let engine = FhirPathEngine::with_model_provider(Box::new(model_provider)).await?;

// Now supports advanced features like type checking, resolve(), etc.
let result = engine.evaluate("Patient.active is Boolean", patient).await?;
```

### Command Line Usage

```bash
# Evaluate expression against JSON file
octofhir-fhirpath evaluate "Patient.name.given" --input patient.json

# Interactive REPL for rapid prototyping
octofhir-fhirpath repl --input patient.json

# Enhanced output formats
octofhir-fhirpath evaluate "Patient.name" --output-format pretty --input patient.json
octofhir-fhirpath evaluate "Patient.name.given" --output-format table --input patient.json

# Pipe JSON data
echo '{"resourceType":"Patient","active":true}' | \
  octofhir-fhirpath evaluate "Patient.active"

# Use environment variables
octofhir-fhirpath evaluate "age > %minAge" \
  --input patient.json \
  --variable "minAge=18"

# Web interface and HTTP server
octofhir-fhirpath server --port 8080
```

See **[CLI.md](CLI.md)** for complete command-line reference.

## 🎯 Key Features

- **Fast**: High-performance tokenizer, parser, and evaluator
- **Safe**: 100% memory-safe Rust with zero unsafe blocks
- **Compliant**: 90.9% FHIRPath specification compliance (1003/1104 tests)
- **Production Ready**: Thread-safe, async-first, comprehensive error handling
- **FHIR Support**: Full R4/R5 support with Bundle resolution and type checking
- **Interactive**: Rich REPL with auto-completion, history, and help system
- **Developer Friendly**: Multiple output formats, web interface, extensive documentation

## 🔧 Common Use Cases

### Healthcare Data Query
```rust
// Find all active patients over 18
engine.evaluate("Bundle.entry.resource.where(resourceType='Patient' and active=true and birthDate < today()-18 'years')", bundle).await?

// Get medication names from prescriptions
engine.evaluate("Bundle.entry.resource.where(resourceType='MedicationRequest').medicationReference.resolve().code.coding.display", bundle).await?
```

### Data Validation
```rust
// Check if patient has required fields
engine.evaluate("Patient.name.exists() and Patient.birthDate.exists()", patient).await?

// Validate phone number format
engine.evaluate("Patient.telecom.where(system='phone').value.matches('[0-9-()]+')", patient).await?
```

### Clinical Decision Support
```rust
// Find high-risk patients
engine.evaluate("Patient.extension.where(url='http://example.org/risk-score').valueInteger > 8", patient).await?

// Calculate medication dosage
engine.evaluate("MedicationRequest.dosageInstruction.doseAndRate.doseQuantity.value * 2", medication).await?
```

## 📚 Documentation

| Guide | Description |
|-------|-------------|
| **[CLI.md](CLI.md)** | Complete command-line tool reference with REPL and server docs |
| **[API Documentation](https://docs.rs/octofhir-fhirpath)** | Full Rust API documentation |
| **[Examples](examples/)** | Code examples and patterns |
| **[Specification Compliance](TEST_COVERAGE.md)** | Detailed compliance report |
| **[Architecture Guide](docs/ARCHITECTURE.md)** | Technical architecture and design patterns |
| **[Development Guide](docs/DEVELOPMENT.md)** | Contributing and development setup |

## 🧪 Test Runners (Dev Tools)

- Convert official R5 XML suite to grouped JSON:
  - `cargo run --bin convert-r5-xml-to-json -- specs/fhirpath/tests/tests-fhir-r5.xml`
- Run existing JSON test suites:
  - `cargo run --bin test-runner -- specs/fhirpath/tests/<suite>.json`

## ⚡ Performance

Built for high-throughput healthcare applications with optimized parsing and evaluation engine designed for production workloads.

## 🤝 Contributing

We welcome contributions! 

```bash
# Get started
git clone https://github.com/octofhir/fhirpath-rs.git
cd fhirpath-rs
just test

# See development guide
just --list
```

## 📋 Specification Compliance: 90.9%

✅ **Fully Supported (100%)**
- Path navigation and filtering
- Collection operations (`where`, `select`, `first`, `last`, `count`, etc.)
- Mathematical operations and arithmetic
- String manipulation functions
- Boolean logic and comparisons
- Date/time operations
- Type operations (`is`, `as`, `ofType`)

🟡 **Well Supported (85%+)**
- FHIR-specific functions (`resolve`, `extension`, `children`)
- Advanced filtering with environment variables
- Lambda expressions and complex iterations
- Aggregate functions and advanced collections

See **[TEST_COVERAGE.md](TEST_COVERAGE.md)** for detailed compliance status.

## 🛠️ Architecture

Modern, modular Rust architecture:

- **9 specialized crates** for flexibility and maintainability
- **Async-first design** for scalable healthcare applications  
- **Zero-copy parsing** with arena allocation for performance
- **Comprehensive error handling** with helpful diagnostic messages
- **Thread-safe by design** with full `Send + Sync` support

## 🔗 Resources

- **[FHIRPath Specification](http://hl7.org/fhirpath/)** - Official specification
- **[FHIR R5](https://hl7.org/fhir/)** - FHIR standard documentation  
- **[HL7 FHIR](https://www.hl7.org/fhir/)** - Healthcare interoperability standards

## 📞 Support & Community

- 🐛 **[Issues](https://github.com/octofhir/fhirpath-rs/issues)** - Bug reports and feature requests
- 💬 **[Discussions](https://github.com/octofhir/fhirpath-rs/discussions)** - Questions and community
- 📧 **Email**: funyloony@gmail.com
- 💝 **Sponsor**: [Boosty](https://boosty.to/octoshikari) - Support development

## 📄 License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.

---

**Built with ❤️ for healthcare interoperability**
