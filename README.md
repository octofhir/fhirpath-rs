# octofhir-fhirpath

[![Crates.io](https://img.shields.io/crates/v/octofhir-fhirpath.svg)](https://crates.io/crates/octofhir-fhirpath)
[![Documentation](https://docs.rs/octofhir-fhirpath/badge.svg)](https://docs.rs/octofhir-fhirpath)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.87+-blue.svg)](https://www.rust-lang.org)

A high-performance, memory-safe FHIRPath implementation in Rust with **82.7% compliance** with the official FHIRPath specification.

## 🎯 Overview

FHIRPath is a path-based navigation and extraction language for FHIR (Fast Healthcare Interoperability Resources) data. This Rust implementation provides a complete FHIRPath engine optimized for performance, safety, and standards compliance.

### Key Features

- ✅ **High Specification Compliance**: 82.7% pass rate on official FHIRPath test suites (831/1005 tests)
- 🚀 **High Performance**: Optimized tokenizer (10M+ ops/sec), parser (1M+ ops/sec), and evaluator
- ⚡ **Bytecode Compiler**: Advanced compilation to bytecode with VM execution for maximum performance
- 🔒 **Memory Safe**: Zero-copy parsing with safe Rust memory management and arena allocation
- 🛠️ **Complete Toolchain**: Parser, evaluator, compiler, CLI tools, and comprehensive diagnostics
- 📊 **Production Ready**: Extensive test coverage, simplified benchmarking, and zero warnings
- 🔧 **Developer Friendly**: Rich error messages, IDE integration support, and comprehensive documentation

## 🚀 Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
octofhir-fhirpath = "0.2.0"
```

### Basic Usage

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine
    let engine = FhirPathEngine::new();
    
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
    let result = engine.evaluate("Patient.name.given", &patient)?;
    println!("Given names: {:?}", result);
    
    let phone = engine.evaluate("Patient.telecom.where(system='phone').value", &patient)?;
    println!("Phone: {:?}", phone);
    
    Ok(())
}
```

## 📚 Core Concepts

### FHIRPath Engine

The `FhirPathEngine` is the main entry point for evaluating FHIRPath expressions:

```rust
use octofhir_fhirpath::FhirPathEngine;

let engine = FhirPathEngine::new();
let result = engine.evaluate("Patient.name.family", &fhir_resource)?;
```

### Value System

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

### Expression Parsing

Parse and analyze FHIRPath expressions:

```rust
use octofhir_fhirpath::parser::parse;

let expression = parse("Patient.name.where(use = 'official').given")?;
println!("Parsed AST: {:#?}", expression);
```

## 🎯 Supported Features

### Core Language Features

- **Path Navigation**: `Patient.name.given`, `Bundle.entry.resource`
- **Filtering**: `Patient.name.where(use = 'official')`
- **Indexing**: `Patient.name[0]`, `Patient.telecom.first()`
- **Boolean Logic**: `and`, `or`, `xor`, `implies`, `not()`
- **Arithmetic**: `+`, `-`, `*`, `/`, `div`, `mod`
- **Comparison**: `=`, `!=`, `<`, `<=`, `>`, `>=`, `~`
- **Collections**: `union`, `intersect`, `exclude`, `distinct`
- **Type Operations**: `is`, `as`, `ofType()`

### Function Library

#### Collection Functions (100% Complete)
- `count()`, `empty()`, `exists()`, `all()`, `first()`, `last()`, `tail()`, `skip()`, `take()`
- `where()`, `select()`, `distinct()`, `intersect()`, `exclude()`, `combine()`

#### String Functions (100% Complete)  
- `contains()`, `startsWith()`, `endsWith()`, `matches()`, `length()`
- `substring()`, `indexOf()`, `split()`, `join()`, `replace()`, `trim()`
- `upper()`, `lower()`, `toChars()`, `encode()`, `decode()`

#### Math Functions (100% Complete)
- `abs()`, `ceiling()`, `floor()`, `round()`, `truncate()`, `sqrt()`, `exp()`, `ln()`, `log()`
- `power()`, `sum()`, `avg()`, `min()`, `max()`, `precision()`

#### DateTime Functions (100% Complete)
- `now()`, `today()`, `lowBoundary()`, `highBoundary()`

#### Type Conversion Functions (100% Complete)
- `toString()`, `toInteger()`, `toDecimal()`, `toBoolean()`, `toQuantity()`
- `convertsToString()`, `convertsToInteger()`, etc.

#### Utility Functions (90%+ Complete)
- `iif()`, `trace()`, `defineVariable()`, `hasValue()`, `conformsTo()`

## 📊 Standards Compliance

Current compliance with official FHIRPath specification test suites:

| Test Category | Pass Rate | Status |
|---------------|-----------|---------|
| **Overall Compliance** | **82.7%** (831/1005) | 🟢 **Production Ready** |
| Core Language | 95%+ | ✅ Excellent |
| String Functions | 100% | ✅ Complete |
| Math Functions | 100% | ✅ Complete |  
| Collection Functions | 100% | ✅ Complete |
| Boolean Logic | 100% | ✅ Complete |
| Type System | 90%+ | ✅ Very Good |
| Date/Time | 90%+ | ✅ Very Good |
| Advanced Features | 70%+ | 🟡 Good |

### Fully Compliant Areas (100%)
- String manipulation and pattern matching
- Mathematical operations and functions
- Collection operations and filtering
- Boolean logic and comparisons
- Core path navigation
- Type checking and conversion

### Well-Implemented Areas (70-99%)
- Complex type operations
- Advanced filtering with variables
- Quantity and unit handling
- Error handling and edge cases

## 🛠️ Development Tools

### Command Line Interface

```bash
# Install CLI tools
cargo install octofhir-fhirpath

# Evaluate expressions
echo '{"resourceType": "Patient", "name": [{"given": ["John"]}]}' | \
  octofhir-fhirpath evaluate "Patient.name.given"

# Parse expressions to AST
octofhir-fhirpath parse "Patient.name.where(use = 'official')"

# Validate syntax
octofhir-fhirpath validate "Patient.name.given.first()" 
```

### Development Commands

```bash
# Run tests
just test

# Run official FHIRPath test suite
just test-official

# Generate test coverage report  
just test-coverage

# Run performance benchmarks
just bench

# Quality assurance
just qa
```

## 🚀 Performance

octofhir-fhirpath is optimized for high-performance use cases:

- **Tokenizer**: 10M+ operations/second
- **Parser**: 1M+ operations/second  
- **Evaluator**: Efficient context management and caching
- **Bytecode VM**: High-performance virtual machine execution
- **Memory**: Zero-copy parsing with minimal allocations
- **Optimization**: Constant folding, strength reduction, and dead code elimination

### Benchmark Results

```bash
just bench  # Run simplified, comprehensive performance tests
```

Benchmarks are simplified into a single unified suite testing all components:  
- Tokenizer performance across complexity levels
- Parser performance with various expressions
- Evaluator performance with context management
- Throughput testing for high-volume operations

## 🏗️ Architecture

octofhir-fhirpath uses a modular architecture:

```
src/
├── ast/           # Abstract syntax tree definitions
├── parser/        # Tokenizer and parser (nom-based)
├── evaluator/     # Expression evaluation engine  
├── compiler/      # Bytecode compiler and virtual machine
├── registry/      # Function registry and built-ins
├── model/         # Value types and FHIR data model
├── diagnostics/   # Error handling and reporting
└── bin/           # CLI tools and utilities
```

### Performance Architecture

- **Three-stage pipeline**: Tokenizer → Parser → Evaluator with arena-based memory management
- **Bytecode compilation**: AST compilation to optimized bytecode with VM execution
- **Registry system**: Modular function and operator registration with caching
- **Memory optimization**: Specialized evaluators, memory pools, and streaming evaluation

## 🔍 Error Handling

Rich diagnostic information with source location tracking:

```rust
match engine.evaluate("Patient.name.invalidFunction()", &resource) {
    Ok(result) => println!("Result: {:?}", result),
    Err(e) => {
        println!("Error: {}", e);
        // Includes line/column information and suggestions
    }
}
```

## 🧪 Testing

Comprehensive test coverage including:

- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end workflow testing  
- **Official Test Suite**: 1005 tests from FHIRPath specification
- **Performance Tests**: Benchmarking and regression testing
- **Property Tests**: Fuzzing and edge case validation

```bash
# Run all tests
cargo test

# Run with coverage
just test-coverage
```

## 🤝 Contributing

We welcome contributions! Please see our [contribution guidelines](CONTRIBUTING.md).

### Development Setup

```bash
# Clone repository
git clone https://github.com/octofhir/fhirpath-rs.git
cd fhirpath-rs

# Install dependencies
cargo build

# Run tests
just test

# Check code quality
just qa
```

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🔗 Links

- **Crates.io**: https://crates.io/crates/octofhir-fhirpath
- **Documentation**: https://docs.rs/octofhir-fhirpath
- **Repository**: https://github.com/octofhir/fhirpath-rs
- **FHIRPath Specification**: http://hl7.org/fhirpath/
- **FHIR Specification**: https://hl7.org/fhir/

## 📞 Support

- 🐛 [Issue Tracker](https://github.com/octofhir/fhirpath-rs/issues)
- 💬 [Discussions](https://github.com/octofhir/fhirpath-rs/discussions)
- 📧 **Email**: funyloony@gmail.com

---

Built with ❤️ by the [OctoFHIR Team](https://github.com/octofhir)
