# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a FHIRPath implementation in Rust organized as a **workspace with 9 specialized crates**:

### Workspace Structure
- **octofhir-fhirpath**: Main library crate that re-exports and integrates all components. Contains development-only binaries for benchmarking and testing (excluded from publishing)
- **fhirpath-core**: Core types, errors, and evaluation results  
- **fhirpath-ast**: Abstract Syntax Tree definitions, operators, and visitor patterns
- **fhirpath-diagnostics**: Error reporting, diagnostic formatting, and LSP integration
- **fhirpath-parser**: Tokenizer and parser using nom library (version 8)
- **fhirpath-model**: Value types, ModelProvider trait, FHIR data model, and resource handling
- **fhirpath-evaluator**: Expression evaluation engine with context management and optimizations
- **fhirpath-registry**: Function and operator registry with built-in implementations
- **fhirpath-analyzer**: Code analysis and validation tools for FHIRPath expressions

### Development Binaries (Not Published)
The main crate includes development-only binaries accessible via the `dev-tools` feature:
- **fhirpath-bench**: Performance benchmarking and profiling tool
- **test-coverage**: Test coverage analysis and report generation
- **test-runner**: Individual test file runner for debugging

### Key Architecture Components

- **Three-stage pipeline**: Tokenizer → Parser → Evaluator with arena-based memory management  
- **ModelProvider Architecture**: Async trait for FHIR type resolution and validation (required since v0.3.0)
- **Registry system**: Modular function and operator registration with caching and fast-path optimizations
- **Performance optimization**: Specialized evaluators, memory pools, and streaming evaluation
- **Reference Resolution**: Enhanced Bundle support with `resolve()` function for cross-resource references
- **Extension framework**: Support for custom functions and CDA/FHIR-specific extensions
- **Memory efficiency**: Arc-based root resource sharing to reduce memory usage during evaluation
- **FHIRPath Compliance**: All evaluation results are collections per specification
- **Zero warnings**: Clean codebase with all compiler warnings resolved

### Data Flow Architecture
```
Input JSON → ModelProvider → Parser (AST) → Evaluator (Context) → FhirPathValue
                ↓              ↓                ↓
           Type Validation  Error Recovery  Registry Lookup
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

# Fix all formatting and clippy issues automatically
just fix
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

# Clean build artifacts
just clean

# Security audit
just audit

# Install development tools
just install-tools

# Watch for changes and run tests
just watch

# Watch for changes (check only)
just watch-check

# Expand macros for debugging
just expand [ITEM]

# Install profiling tools
just install-profiling-tools
```

### Test-Specific Commands
```bash
# Run specific test case
just test-case test-case-name

# Run failed expression tests
just test-failed

# Test coverage with tarpaulin (HTML report)
just coverage

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

### Profiling Commands
```bash
# Profile specific expression
just profile "Patient.name.given"

# Profile with Patient data
just profile-patient "Patient.name.where(use = 'official').family"

# Profile with Bundle data
just profile-bundle "Bundle.entry.resource.count()"

# Run example profiling sessions
just profile-examples
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
cargo test --package fhirpath-parser

# Run a single test with full output
cargo test test_name -- --nocapture

# Test coverage with tarpaulin (HTML report)
just coverage
```

## Guidelines

Apply the following guidelines when developing fhirpath-rs:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Coding Guidelines](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Style Guide](https://rust-lang.github.io/rust-style-guide/)

## IMPORTANT: Mandatory Implementation Patterns

**⚠️ CRITICAL ARCHITECTURAL REQUIREMENTS - MUST BE FOLLOWED ⚠️**

### JSON Processing with sonic_rs
- **ALWAYS use sonic_rs::Value** instead of serde_json::Value for all JSON processing
- **NEVER mix serde_json and sonic_rs types** - use consistent sonic_rs throughout
- Convert from serde_json to sonic_rs using: `sonic_rs::from_str(&serde_json::to_string(&value).unwrap()).unwrap()`
- For new code, use `sonic_rs::json!()` macro instead of `serde_json::json!()`

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
- **NEVER use serde_json types in new code** - use sonic_rs consistently

### Implementation Standards
- Always check FHIRPath specification for function and operator behavior
- Extend existing registry systems rather than creating new ones
- Maintain registry-based architecture for extensibility
- Follow existing patterns in unified implementations
- **CRITICAL**: Always use `ensure_collection_result()` to wrap evaluation results in collections per FHIRPath specification
- **CRITICAL**: Use `Arc<FhirPathValue>` for root resource sharing to optimize memory usage
- **CRITICAL**: Implement proper equivalence logic for unordered collection comparisons

## Specifications and Dependencies

- FHIRPath specification reference in [HL7](https://build.fhir.org/ig/HL7/FHIRPath/)
- Official test cases in `specs/fhirpath/tests/` 
- FHIRSchema spec: https://fhir-schema.github.io/fhir-schema/intro.html
- Uses nom library version 8 for parsing
- For UCUM units: use https://github.com/octofhir/ucum-rs or local path `./…/ucum-rs`

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

ALL tests files must be under the tests directory for the current crate

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

### Recent Architectural Improvements

The following critical improvements have been implemented to enhance performance and FHIRPath specification compliance:

#### Memory Optimization with Arc
- **Problem**: Context root resource was being cloned for every evaluation, causing memory inefficiency
- **Solution**: Implemented Arc (Atomic Reference Counting) for root resource sharing
- **Impact**: Significantly reduced memory usage during evaluation by sharing references instead of cloning
- **Implementation**: Changed `EvaluationContext.root` from `FhirPathValue` to `Arc<FhirPathValue>`

#### FHIRPath Specification Compliance 
- **Problem**: Some evaluation results were not wrapped in collections, violating FHIRPath spec requirement
- **Solution**: Enhanced `ensure_collection_result()` to wrap ALL non-collection values in collections
- **Impact**: Full compliance with FHIRPath specification that mandates all results be collections
- **Critical Fix**: Empty values and scalar results are now properly wrapped in collections

#### Collection Equivalence Operator
- **Problem**: Collection equivalence (~) operator failed on unordered comparisons like `(1 | 2 | 3) ~ (3 | 2 | 1)`
- **Solution**: Implemented proper unordered collection comparison logic
- **Impact**: Fixed 50+ test failures and achieved proper equivalence semantics

#### Decimal Serialization
- **Problem**: Decimal values were serialized as strings instead of numbers in JSON output  
- **Solution**: Modified `FhirPathValue::From` implementation to output numeric values
- **Impact**: Tests expecting `[1.5875]` now correctly get numbers instead of `"1.5875"`

These improvements significantly enhanced test pass rates, with 37 test suites achieving 100% pass rates and overall specification compliance improving substantially.

## Library Usage

The main library crate provides a clean API using **sonic_rs::Value** for high performance JSON handling:

### Basic Usage with MockModelProvider (for testing/simple use cases)

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue, MockModelProvider};
use sonic_rs::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // MockModelProvider for testing and simple use cases
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

### Production Usage with FhirSchemaModelProvider (with multi-tier caching)

For production use and full FHIR compliance with high-performance multi-tier caching:

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use octofhir_fhirpath_model::FhirSchemaModelProvider;
use sonic_rs::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // FhirSchemaModelProvider with multi-tier caching (default)
    // Provides full FHIR type information and intelligent caching
    let model_provider = FhirSchemaModelProvider::new().await?;
    let mut engine = FhirPathEngine::with_model_provider(Box::new(model_provider));
    
    let patient = json!({"resourceType": "Patient", "active": true});
    
    // Type operations work efficiently with hot/warm/cold cache hierarchy
    let type_result = engine.evaluate("Patient.active.type()", patient.clone()).await?;
    println!("Type: {:?}", type_result); // Should show System.Boolean
    
    // Type checking operations are cached automatically
    let is_result = engine.evaluate("Patient.active is Boolean", patient.clone()).await?;
    println!("Is Boolean: {:?}", is_result); // Should show true
    
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

From `octofhir-fhirpath-model`:
- `FhirSchemaModelProvider`: High-performance ModelProvider with multi-tier caching (default)
- `CacheManager`: Multi-tier cache system with hot/warm/cold storage
- `PrecomputedTypeRegistry`: High-performance type registry for fast type operations
- `ModelProvider`: Async trait for FHIR type introspection

## Performance Characteristics

This implementation is optimized for high-performance with:
- **Tokenizer**: 10M+ operations/second
- **Parser**: 1M+ operations/second  
- **Evaluator**: Arena-based memory management with specialized evaluation paths and Arc-optimized resource sharing
- **Multi-tier Caching**: Hot cache (lock-free, <100ns), Warm cache (<1μs), Cold storage (<10μs)
- **Intelligent Caching**: Access pattern tracking with predictive cache warming
- **Memory Efficiency**: Reduced memory usage through Arc-based root resource sharing and smart cache tiers
- **Benchmarks**: Comprehensive suite testing all components and cache performance
- **Test Coverage**: Significantly improved specification compliance (37 test suites at 100% pass rate)
- **Code Quality**: Zero compiler warnings with clean, maintainable codebase

## Architecture Decision Records (ADRs)

Major architectural decisions are documented in `docs/adr/`:

- **ADR-003**: FHIRPath Type Reflection System - Type reflection and enhanced type() function implementation with FHIRSchema integration

## Future Development

Planned major features documented in ADRs and tasks:
- **Type Reflection System**: Enhanced type() function with FHIRSchema integration (ADR-003)
- **CDA Support**: Clinical Document Architecture extensions and functions
- **Advanced Type Operations**: Enhanced type coercion and validation
- Cross-platform distribution and Docker support

## Implementation Notes

- Always check FHIRPath specification for functions and operator behavior
- Use the unified registry system for all function and operator implementations
- Maintain async-first architecture throughout the codebase
- Follow the three-stage pipeline: Tokenizer → Parser → Evaluator
- Prioritize performance with arena-based memory management and caching

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.

      
      IMPORTANT: this context may or may not be relevant to your tasks. You should not respond to this context unless it is highly relevant to your task.