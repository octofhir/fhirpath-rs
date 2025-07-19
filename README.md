# OctoFHIR FHIRPath

[![CI](https://github.com/octofhir/fhirpath-rs/workflows/CI/badge.svg)](https://github.com/octofhir/fhirpath-rs/actions/workflows/ci.yml)
[![Release](https://github.com/octofhir/fhirpath-rs/workflows/Release/badge.svg)](https://github.com/octofhir/fhirpath-rs/actions/workflows/release.yml)
[![Crates.io](https://img.shields.io/crates/v/fhirpath-core.svg)](https://crates.io/crates/fhirpath-core)
[![npm](https://img.shields.io/npm/v/@octofhir/fhirpath-node.svg)](https://www.npmjs.com/package/@octofhir/fhirpath-node)

A high-performance FHIRPath implementation in Rust with multiple language bindings.

## Overview

OctoFHIR FHIRPath is a comprehensive implementation of the [FHIRPath specification](http://hl7.org/fhirpath/) written in Rust. FHIRPath is a path-based navigation and extraction language for FHIR (Fast Healthcare Interoperability Resources) data.

This project provides:
- A fast, memory-safe core engine written in Rust
- A command-line interface for FHIRPath evaluation
- Node.js bindings for JavaScript/TypeScript integration

## Features

- ✅ **High Performance**: Written in Rust for maximum speed and memory safety
- ✅ **Multiple Interfaces**: CLI tool, Rust library, and Node.js bindings
- ✅ **FHIRPath Compliance**: Implements the official FHIRPath specification
- ✅ **Expression Validation**: Syntax validation for FHIRPath expressions
- ✅ **Multiple Output Formats**: JSON and pretty-printed output
- ✅ **Streaming Support**: Efficient processing of large FHIR resources
- ✅ **Test Compliance**: Comprehensive testing against official FHIRPath test suite

## Documentation

This project includes comprehensive documentation available at the documentation site:

- **Live Site**: [https://octofhir.github.io/fhirpath-rs](https://octofhir.github.io/fhirpath-rs)
- **Test Compliance**: View current status of official FHIRPath test suite compliance
- **API Reference**: Detailed API documentation for all components
- **Usage Examples**: Practical examples and tutorials
- **Implementation Status**: Track development progress and supported features
- **Interactive Playground**: Test FHIRPath expressions in your browser

### Local Development

To build and view the documentation locally:

```bash
# Build WASM package first
cd fhirpath-wasm
npm run build

# Install dependencies and run dev server
cd ../docs-site
npm install
npm run prebuild  # Copies WASM and comparison data
npm run dev
```

### Deployment

The documentation site is automatically deployed to GitHub Pages when changes are pushed to the main branch. See [docs-site/DEPLOYMENT.md](docs-site/DEPLOYMENT.md) for detailed information about the deployment process.

### Test Compliance Status

The implementation is continuously tested against the official FHIRPath test suite. Current status:

- **179 tests passing** (25.2% compliance)
- **501 tests failing** (features not yet implemented)
- **31 tests skipped** (missing test data or dependencies)
- **711 total tests** in the official suite

View detailed test compliance information in the documentation site under Reference → Test Compliance.

## Components

### fhirpath-core
The core FHIRPath engine that provides:
- Lexical analysis and tokenization
- Expression parsing using nom parser combinators
- Expression evaluation against FHIR resources
- Comprehensive error handling

### fhirpath-cli
A command-line interface that allows you to:
- Evaluate FHIRPath expressions against FHIR resources
- Validate FHIRPath expression syntax
- Output results in multiple formats

### fhirpath-node
Node.js bindings that enable:
- JavaScript/TypeScript integration
- Native performance in Node.js applications
- Seamless JSON handling

## Installation

### CLI Tool

```bash
# Install from source
git clone https://github.com/octofhir/fhirpath-rs
cd fhirpath-rs
cargo install --path fhirpath-cli
```

### Rust Library

Add to your `Cargo.toml`:

```toml
[dependencies]
fhirpath-core = "0.1.0"
```

### Node.js Package

```bash
npm install fhirpath-node
```

## Usage

### Command Line Interface

The CLI provides several commands:

#### Evaluate FHIRPath expressions

```bash
# Evaluate an expression against a FHIR resource
octofhir-fhirpath eval "Patient.name.given" patient.json

# Specify output format
octofhir-fhirpath eval "Patient.name.given" patient.json --format json
octofhir-fhirpath eval "Patient.name.given" patient.json --format pretty
```

#### Validate FHIRPath expressions

```bash
# Check if an expression is syntactically valid
octofhir-fhirpath validate "Patient.name.given"
octofhir-fhirpath validate "Patient.invalid..syntax"
```

#### Show parsed AST

```bash
# Display the Abstract Syntax Tree of an expression
octofhir-fhirpath ast "Patient.name.given"
octofhir-fhirpath ast "Patient.name.given" --format debug
```

#### Generate shell completions

```bash
# Generate completion scripts for your shell
octofhir-fhirpath completion bash > ~/.bash_completion.d/octofhir-fhirpath
octofhir-fhirpath completion zsh > ~/.zsh/completions/_octofhir-fhirpath
octofhir-fhirpath completion fish > ~/.config/fish/completions/octofhir-fhirpath.fish
```

### Rust Library

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
    
    let expression = "Patient.name.given";
    let result = evaluate_expression(expression, &fhir_resource)?;
    
    println!("Result: {:?}", result);
    Ok(())
}
```

### Node.js

```javascript
const { evaluateExpression } = require('fhirpath-node');

const fhirResource = {
    resourceType: "Patient",
    name: [
        {
            given: ["John", "Doe"]
        }
    ]
};

const expression = "Patient.name.given";
const result = evaluateExpression(expression, fhirResource);
console.log('Result:', result);
```

## Development

### Prerequisites

- Rust 1.70+ with 2024 edition support
- Node.js 20+ (for Node.js bindings)
- Cargo

### Building from Source

```bash
# Clone the repository
git clone https://github.com/octofhir/fhirpath-rs
cd fhirpath-rs

# Build all components
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Building Node.js Bindings

```bash
cd fhirpath-node
npm install
npm run build
npm test
```

### Project Structure

```
octofhir-fhirpath/
├── fhirpath-core/          # Core FHIRPath engine
│   ├── src/
│   │   ├── lexer.rs        # Tokenization
│   │   ├── parser.rs       # Expression parsing
│   │   ├── evaluator.rs    # Expression evaluation
│   │   └── model.rs        # Data models
│   └── tests/              # Core tests
├── fhirpath-cli/           # Command-line interface
│   └── src/main.rs         # CLI implementation
├── fhirpath-node/          # Node.js bindings
│   └── src/lib.rs          # NAPI bindings
└── docs/                   # Documentation
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific component
cargo test -p fhirpath-core
cargo test -p fhirpath-cli

# Run with debug output
cargo test -- --nocapture
```

## Releases

This project uses an automated release process that creates tags and publishes packages automatically.

### Automatic Tag Creation

When you update the version in the root `Cargo.toml` file and push to the `main` branch, a GitHub Actions workflow will automatically:

1. **Detect the version change** from the `Cargo.toml` file
2. **Validate the change** by ensuring the version has actually changed
3. **Check for unreleased changes** in `CHANGELOG.md` under the `[Unreleased]` section
4. **Create and push a git tag** in the format `v{VERSION}` (e.g., `v0.2.0`)
5. **Trigger the release workflow** which builds and publishes all packages

### Release Process for Maintainers

To create a new release:

1. **Update the changelog**: Add your changes to the `[Unreleased]` section in `CHANGELOG.md`
2. **Bump the version**: Update the version in the root `Cargo.toml` file
3. **Update package versions**: Ensure versions are synchronized in:
   - `fhirpath-node/package.json`
   - `fhirpath-wasm/package.json`
   - Individual `Cargo.toml` files (if they override workspace version)
4. **Commit and push**: Push your changes to the `main` branch
5. **Automatic processing**: The auto-tag workflow will handle the rest

### Manual Tag Creation

If you need to manually create a tag or force tag creation:

```bash
# Navigate to Actions tab in GitHub
# Run "Auto Tag Release" workflow manually
# Check "Force create tag even if it already exists" if needed
```

### What Gets Published

Each release automatically publishes:
- **Rust crates** to [crates.io](https://crates.io):
  - `fhirpath-core`
  - `octofhir-fhirpath` (CLI)
- **Node.js packages** to [npm](https://npmjs.com):
  - `@octofhir/fhirpath-node`
  - `@octofhir/fhirpath-wasm`
- **GitHub release** with:
  - Cross-platform binaries (Linux, macOS, Windows)
  - Automatically generated changelog
  - Release artifacts

### Versioning

This project follows [Semantic Versioning (SemVer)](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions  
- **PATCH**: Backwards-compatible bug fixes

See [VERSIONING.md](VERSIONING.md) for detailed versioning strategy.

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for your changes
5. Ensure all tests pass (`cargo test`)
6. Run clippy for linting (`cargo clippy`)
7. Format code (`cargo fmt`)
8. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/) (`git commit -m 'feat: add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

**Note**: This project uses automated changelog generation based on conventional commits. See [CHANGELOG_AUTOMATION.md](CHANGELOG_AUTOMATION.md) for details on commit message format and changelog automation.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [HL7 FHIR](https://www.hl7.org/fhir/) for the FHIRPath specification
- The Rust community for excellent tooling and libraries
- Contributors and maintainers of this project

## Support

- 📖 [Documentation](https://github.com/octofhir/fhirpath-rs/docs)
- 🐛 [Issue Tracker](https://github.com/octofhir/fhirpath-rs/issues)
- 💬 [Discussions](https://github.com/octofhir/fhirpath-rs/discussions)
