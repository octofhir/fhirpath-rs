# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a FHIRPath implementation in Rust organized as a **workspace with 11 specialized crates**:

### Workspace Structure
- **octofhir-fhirpath**: Main library crate that re-exports and integrates all components
- **octofhir-fhirpath-core**: Core types, errors, and evaluation results
- **octofhir-fhirpath-ast**: Abstract syntax tree definitions and visitor patterns
- **octofhir-fhirpath-parser**: Tokenizer and parser using nom library (version 8)
- **octofhir-fhirpath-model**: Value types, ModelProvider trait, FHIR data model, and resource handling
- **octofhir-fhirpath-evaluator**: Expression evaluation engine with context management and optimizations
- **octofhir-fhirpath-compiler**: Bytecode compilation and VM execution with optimizer
- **octofhir-fhirpath-registry**: Function and operator registry with built-in implementations
- **octofhir-fhirpath-diagnostics**: Error handling, diagnostic reporting, and LSP support
- **octofhir-fhirpath-tools**: CLI tools, test runners, and coverage analysis
- **octofhir-fhirpath-benchmarks**: Performance testing and profiling utilities

### Migration Status
The codebase has been migrated from a monolithic structure to this modular workspace. Legacy code exists in `src_backup_old/` for reference but the active implementation is in the `crates/` workspace structure.

### Key Architecture Components

- **Three-stage pipeline**: Tokenizer → Parser → Evaluator with arena-based memory management
- **Bytecode compilation**: AST compilation to bytecode with VM execution and optimization passes
- **ModelProvider Architecture**: Async trait for FHIR type resolution and validation (required since v0.3.0)
- **Registry system**: Modular function and operator registration with caching and fast-path optimizations
- **Performance optimization**: Specialized evaluators, memory pools, and streaming evaluation
- **Reference Resolution**: Enhanced Bundle support with `resolve()` function for cross-resource references
- **Extension framework**: Support for custom functions and CDA/FHIR-specific extensions
- **Zero warnings**: Clean codebase with all compiler warnings resolved

### Data Flow Architecture
```
Input JSON → ModelProvider → Parser (AST) → Compiler (Bytecode) → Evaluator (Context) → FhirPathValue
                ↓              ↓                ↓                     ↓
           Type Validation  Error Recovery  Optimization         Registry Lookup
```

## Development Commands

### Building and Testing
```bash
# Build entire workspace
just build

# Build with release optimization  
just build-release

# Run all tests
just test

# Run integration tests (official FHIRPath test suites)
just test-official

# Update test coverage report
just test-coverage

# Run simplified unified benchmark suite
just bench

# Run full benchmark suite (same as bench - simplified)
just bench-full

# Update benchmark documentation
just bench-update-docs
```

### Performance and Quality
```bash
# Run clippy linting
just clippy

# Format code
just fmt

# Check code without building
just check

# Quality assurance (format + lint + test)
just qa

# All benchmarks now unified in single suite
# (legacy commands removed - use 'just bench')

# Clean build artifacts
just clean
just clean-bench
```

### Test-Specific Commands
```bash
# Run specific test case
just test-case test-case-name

# Run failed expression tests
just test-failed

# Release preparation (full QA + docs)
just release-prep
```

### Documentation Commands
```bash
# Generate API documentation
just doc

# Generate complete documentation (including dependencies)
just doc-all

# Generate all documentation (API + benchmarks)
just docs
```

### CLI Commands
```bash
# Evaluate FHIRPath expression (read FHIR resource from stdin)
just cli-evaluate "Patient.name.given"

# Evaluate FHIRPath expression with specific file
just cli-evaluate "Patient.name.given" path/to/resource.json

# Evaluate with environment variables
just cli-evaluate "%customVar" --variable "customVar=hello world"

# Multiple variables
just cli-evaluate "age > %minAge" --variable "minAge=18" --variable "maxAge=65"

# Parse FHIRPath expression to AST
just cli-parse "Patient.name.given"

# Validate FHIRPath expression syntax
just cli-validate "Patient.name.given"

# Show CLI help
just cli-help

# Run CLI with custom arguments
just cli [arguments...]
```

### Environment Variables Support

The implementation supports FHIRPath environment variables per the specification:

#### Standard Environment Variables
- `%context` - The original node in the input context
- `%resource` - The resource containing the original node
- `%rootResource` - The container resource (for contained resources)
- `%sct` - SNOMED CT URL (`http://snomed.info/sct`)
- `%loinc` - LOINC URL (`http://loinc.org`)
- `%"vs-[name]"` - HL7 value set URLs

#### Custom Variables
```bash
# Set custom variables via CLI
echo '{"age": 25}' | octofhir-fhirpath evaluate 'age > %threshold' --variable 'threshold=18'

# JSON values
echo '{}' | octofhir-fhirpath evaluate '%config.enabled' --variable 'config={"enabled": true}'
```

See [docs/ENVIRONMENT_VARIABLES.md](docs/ENVIRONMENT_VARIABLES.md) for complete documentation.

### Debug Utilities
For debugging parser errors, create simple tests instead of dedicated binaries.
The project includes utilities for troubleshooting:
```bash
# Run specific test case from official FHIRPath test suite
just test-case test-case-name

# Example: just test-case literals
# This runs specs/fhirpath/tests/literals.json

# Alternative test coverage with MockModelProvider (faster, no network)
just test-coverage-mock

# Run single test by name
cargo test test_name -- --nocapture

# Run tests for specific crate
cargo test --package octofhir-fhirpath-parser
```

## Guidelines

Apply the following guidelines when developing fhirpath-core:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Coding Guidelines](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Style Guide](https://rust-lang.github.io/rust-style-guide/)

## IMPORTANT: Mandatory Implementation Patterns

**⚠️ CRITICAL ARCHITECTURAL REQUIREMENTS - MUST BE FOLLOWED ⚠️**

### Async-First Architecture
- **ALL features MUST be async-first** with non-blocking behavior
- No synchronous blocking operations in public APIs
- Use `async/await` pattern throughout the codebase
- Ensure all I/O operations are non-blocking

### Unified Function Registry Usage
- **ALWAYS use UnifiedFunctionRegistry** for function implementations
- **NEVER create hardcoded function implementations**
- If existing functions need improvement, enhance them to support both regular and lambda evaluation
- Functions must be registered through the registry system, not implemented as static methods

### Operator Registry Requirements
- **ALWAYS use operator registry** for all operator operations
- **NEVER implement static or hardcoded operator solutions**
- All operators (arithmetic, logical, comparison, etc.) must go through the registry
- No direct operator implementation in evaluator code

### Prohibited Patterns
- **NEVER create fallback or simplified solutions** - implement full functionality
- **NEVER bypass registry systems** with direct implementations
- **NEVER create hardcoded function mappings** or static dispatch
- **NEVER implement operators outside the registry system**
- **NEVER create synchronous wrappers** around async functionality

### Implementation Standards
- Always check FHIRPath specification for function and operator behavior
- Extend existing registry systems rather than creating new ones
- Maintain registry-based architecture for extensibility
- Follow existing patterns in unified implementations

## Specifications and Dependencies

- FHIRPath specification reference in `specs/` folder
- Official test cases in `specs/fhirpath/tests/`
- FHIRSchema spec: https://fhir-schema.github.io/fhir-schema/intro.html
- Uses nom library version 8 for parsing
- For UCUM units: use https://github.com/octofhir/ucum-rs or local path `./…/ucum-rs`
- Divan for all benchmarking (no Criterion)


## Development Process

### Architecture Decision Records (ADRs)
Before implementing major features:
1. Create ADR following: https://github.com/joelparkerhenderson/architecture-decision-record
2. Split implementation into phases/tasks stored in `tasks/` directory
3. Update task files with implementation status

### Task Management
For every ADR implementation split record into phases/tasks and store in `tasks/` directory. Maintain a specific task file when working on it. Before starting on the first task, create all tasks for future use. After implementing features from a task file update its status.

### Debug Workflow
For debugging cases create a simple test inside the test directory and delete it after resolving the issue.


## Test Coverage

To track progress and maintain visibility into implementation completeness:

### Updating Test Coverage Report
Run the automated test coverage generator:
```bash
just test-coverage
```

This command:
- Builds the test infrastructure
- Runs all official FHIRPath test suites
- Generates a comprehensive report in `TEST_COVERAGE.md`
- Provides statistics on pass rates and identifies missing functionality

The coverage report should be updated after completing any major functionality to track progress.

## Library Usage

The main library crate provides a clean API:

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue, MockModelProvider};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ModelProvider is required since v0.3.0
    let model_provider = MockModelProvider::new();
    let mut engine = FhirPathEngine::with_model_provider(Box::new(model_provider));
    
    let patient = json!({"resourceType": "Patient", "name": [{"given": ["John"]}]});
    
    // Basic evaluation
    let result = engine.evaluate("Patient.name.given", patient.clone()).await?;
    println!("Result: {:?}", result);
    
    // Evaluation with environment variables
    let mut variables = HashMap::new();
    variables.insert("myVar".to_string(), FhirPathValue::String("test".into()));
    let result = engine.evaluate_with_variables("%myVar", patient, variables).await?;
    println!("Variable result: {:?}", result);
    
    Ok(())
}
```

Main exports from `octofhir-fhirpath`:
- `FhirPathEngine`: Main evaluation engine (async, requires ModelProvider)
- `FhirPathValue`: Value types and smart collections
- `parse()`: Parse FHIRPath expressions to AST
- `FunctionRegistry`: Function registry for extensions
- `EvaluationContext`: Context for expression evaluation
- `MockModelProvider`: Basic ModelProvider for testing/simple use cases

## Performance Characteristics

This implementation is optimized for high-performance with:
- **Tokenizer**: 10M+ operations/second
- **Parser**: 1M+ operations/second
- **Evaluator**: Arena-based memory management with specialized evaluation paths
- **Bytecode VM**: High-performance virtual machine with optimization passes
- **Benchmarks**: Simplified unified suite testing all components efficiently
- **Test Coverage**: 88.1% specification compliance with official FHIRPath test suites
- **Code Quality**: Zero compiler warnings with clean, maintainable codebase

## Architecture Decision Records (ADRs)

Major architectural decisions are documented in `docs/adr/`:

- **ADR-001**: Model Context Protocol (MCP) Server Implementation - Plan for exposing FHIRPath functionality through MCP for AI assistants
- **ADR-002**: FHIRPath Analyzer Crate - Static analysis and expression explanation capabilities

## Future Development

Planned major features documented in ADRs:
- **fhirpath-mcp-server**: MCP server crate for AI assistant integration
- **fhirpath-analyzer**: Static analysis and expression explanation
- Cross-platform distribution and Docker support

- always check fhirpath specifcaiton for gunctions and opertaor behaviour
