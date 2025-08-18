# octofhir-fhirpath

[![Crates.io](https://img.shields.io/crates/v/octofhir-fhirpath.svg)](https://crates.io/crates/octofhir-fhirpath)
[![Documentation](https://docs.rs/octofhir-fhirpath/badge.svg)](https://docs.rs/octofhir-fhirpath)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/octofhir/fhirpath-rs/blob/main/LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.87+-blue.svg)](https://www.rust-lang.org)
[![Support on Boosty](https://img.shields.io/badge/support-Boosty-orange)](https://boosty.to/octoshikari)

A production-ready FHIRPath implementation in Rust with **87.0% specification compliance**. Fast, safe, and built for healthcare applications that need reliable FHIR data navigation.

## Why octofhir-fhirpath?

**FHIRPath is essential for modern healthcare interoperability** - it's the standard query language for FHIR resources used across electronic health records, clinical decision support, and healthcare data exchange. This Rust implementation brings memory safety, high performance, and reliability to FHIRPath evaluation.

### Key Numbers

- **87.0% FHIRPath specification compliance** (885/1017 official tests passing)
- **473K parser operations/sec** - Fast expression parsing
- **4K+ evaluator operations/sec** - Efficient expression evaluation 
- **100% safe Rust** - Memory safe with zero unsafe blocks
- **11 modular crates** - Flexible architecture for any use case
- **Zero warnings** - Clean, maintainable codebase

### Why Developers Choose This Library

- **Production Ready**: Battle-tested with comprehensive test coverage and real-world usage
- **Thread Safe**: Full `Send + Sync` support for concurrent applications  
- **Developer Friendly**: Rich error messages, excellent documentation, and helpful CLI tools
- **Standards Compliant**: Follows official FHIRPath specification with ongoing compliance improvements
- **Performant**: Optimized for high-throughput healthcare applications

## Quick Start

### Installation

```toml
[dependencies]
octofhir-fhirpath = "0.4.0"
```

### Basic Usage

```rust
use octofhir_fhirpath::FhirPathEngine;
use sonic_rs::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = FhirPathEngine::with_mock_provider();
    
    let patient = json!({
        "resourceType": "Patient",
        "name": [{"given": ["Alice"], "family": "Smith"}]
    });
    
    let result = engine.evaluate("Patient.name.given", patient).await?;
    println!("Names: {:?}", result); // ["Alice"]
    
    Ok(())
}
```

### CLI Usage

```bash
cargo install octofhir-fhirpath

octofhir-fhirpath evaluate "Patient.name.given" \
  --input '{"resourceType":"Patient","name":[{"given":["Alice"]}]}'
```

## Documentation

- 📖 **[Examples and Usage](docs/EXAMPLES.md)** - Comprehensive examples and patterns
- 🏗️ **[Architecture Guide](docs/ARCHITECTURE.md)** - Technical architecture and design  
- 🎯 **[Feature List](docs/FEATURES.md)** - Complete feature documentation with compliance status
- 🖥️ **[CLI Reference](docs/CLI.md)** - Command-line tool documentation
- 🔧 **[Development Guide](docs/DEVELOPMENT.md)** - Contributing and development setup

## Specification Compliance

**87.0% compliance** with official FHIRPath specification test suites (885/1017 tests passing).

### ✅ Fully Supported (100% pass rate)
- Core path navigation and filtering
- Collection operations (`where`, `select`, `first`, `count`, etc.)
- Mathematical functions and arithmetic
- DateTime operations and boundaries
- Boolean logic and comparisons

### 🟢 Well Supported (85-99% pass rate)  
- String manipulation and pattern matching
- Type operations and conversions
- Advanced filtering with variables
- Lambda functions and iterations

### 🟡 Partially Supported (70-84% pass rate)
- Complex FHIR-specific functions
- Advanced type system features
- Some edge cases in date/time handling

See **[docs/FEATURES.md](docs/FEATURES.md)** for detailed feature documentation and compliance status.

## Performance

High-performance implementation optimized for production use:

- **473K+ parser operations/sec** - Fast expression parsing
- **4K+ evaluator operations/sec** - Efficient evaluation with Bundle resolution
- **Thread-safe design** - Full `Send + Sync` support for concurrent applications  
- **Memory efficient** - Zero-copy parsing with arena allocation
- **Optimized for healthcare** - Built for high-throughput FHIR processing

## Contributing

We welcome contributions! See **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** for setup instructions.

```bash
git clone https://github.com/octofhir/fhirpath-rs.git
cd fhirpath-rs
just test
```

## License

Licensed under either of Apache-2.0 or MIT at your option.

## Links

- **Crates.io**: https://crates.io/crates/octofhir-fhirpath
- **Documentation**: https://docs.rs/octofhir-fhirpath  
- **Repository**: https://github.com/octofhir/fhirpath-rs
- **FHIRPath Specification**: http://hl7.org/fhirpath/
- **FHIR R5 Specification**: https://hl7.org/fhir/

## Support

- 🐛 [Report Issues](https://github.com/octofhir/fhirpath-rs/issues)
- 💬 [Discussions](https://github.com/octofhir/fhirpath-rs/discussions)  
- 📧 **Email**: funyloony@gmail.com
- 💝 **Support Development**: [Boosty](https://boosty.to/octoshikari) - Help us build better FHIR tools

---

Built with ❤️ for healthcare interoperability
