# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **fhirpath-rs** (octofhir-fhirpath), a high-performance FHIRPath implementation in Rust for healthcare data processing. It provides a complete implementation of the FHIRPath expression language for FHIR resources with **90.9% specification compliance** (1003/1104 tests passing).

## Common Commands

All development tasks use the `justfile` system. Essential commands:

### Build and Test
- `just build` - Build entire workspace
- `just test` - Run all tests
- `just test-coverage` - Generate test coverage report (may timeout on first run)
- `just test-coverage-mock` - Fast test coverage using MockModelProvider
- `just test-official` - Run official FHIRPath specification tests

### Code Quality  
- `just qa` - Complete quality assurance (format + lint + test)
- `just fix` - Auto-fix formatting and clippy issues
- `just clippy` - Run linting
- `just fmt` - Format code
- `just check` - Quick compilation check

### Performance
- `just bench` - Run unified benchmark suite
- `just bench-full` - Complete benchmark suite with report generation
- `just profile "expression"` - Profile specific FHIRPath expressions

### CLI Development
- `just cli-evaluate "expression"` - Test CLI evaluation (reads from stdin)
- `just cli-evaluate "expression" file.json` - Evaluate against specific file
- `just cli-parse "expression"` - Parse expression to AST
- `just cli-validate "expression"` - Validate syntax only
- `just cli-analyze "expression"` - Analyze expression with optimization suggestions

### Interactive REPL
- `just repl` - Start interactive FHIRPath REPL for rapid prototyping and debugging
- `just repl file.json` - Start REPL with initial resource loaded from file
- `just repl --fhir-version r5` - Start REPL with specific FHIR version (r4, r4b, r5)

#### Enhanced Output Formats
The CLI supports multiple output formats for better integration and user experience:

- `just cli-pretty "expression" [file.json]` - Colorized, emoji-rich output with execution metrics
- `just cli-json "expression" [file.json]` - Structured JSON output for machine parsing
- `just cli-table "expression" [file.json]` - Formatted table output for complex results

You can also use the main CLI directly with `--output-format`:
- `--output-format raw` - Default plain text output
- `--output-format pretty` - Colorized output with emojis and performance info  
- `--output-format json` - Structured JSON with metadata
- `--output-format table` - Formatted table for result collections

Global flags:
- `--no-color` - Disable colored output (also via `FHIRPATH_NO_COLOR` env var)
- `--quiet` - Suppress informational messages
- `--verbose` - Enable additional details

### Documentation
- `just doc` - Generate API documentation
- `just docs` - Generate all documentation including benchmarks

## Architecture

### Modular Workspace Structure
The project uses 11 specialized crates for flexibility and maintainability:

```
crates/
├── octofhir-fhirpath/     # Main library (core functionality, published)
├── fhirpath-core/         # Core types, errors, evaluation results (published)
├── fhirpath-ast/          # Abstract syntax tree definitions (published)
├── fhirpath-parser/       # Tokenizer and parser (nom-based) (published)
├── fhirpath-evaluator/    # Expression evaluation engine (published)
├── fhirpath-model/        # Value types and FHIR data model (published)
├── fhirpath-registry/     # Function and operator registry (published)
├── fhirpath-analyzer/     # Static analysis and validation (published)
├── fhirpath-diagnostics/  # Error handling and reporting (published)
├── fhirpath-cli/          # Command-line interface (NOT published)
└── fhirpath-dev-tools/    # Development tools (NOT published)
```

### Key Design Principles
1. **Performance**: Zero-copy parsing, arena allocation, efficient data structures
2. **Safety**: 100% memory-safe Rust, no unsafe blocks
3. **Async-First**: ModelProvider architecture supports async operations without over-engineering
4. **Thread-Safe**: Full Send + Sync support for FhirPathEngine
5. **Modular**: Clean separation via workspace crates
6. **Simplicity**: Clean, maintainable code without unnecessary overhead or complexity
7. **Separation of Concerns**: Core library, CLI, and dev tools are cleanly separated

### Crate Architecture Details

#### Published Crates (for library users)
- **octofhir-fhirpath**: Main library with minimal dependencies, includes MockModelProvider
- **fhirpath-core**: Core types and errors
- **fhirpath-ast**: AST definitions  
- **fhirpath-parser**: Parsing functionality
- **fhirpath-evaluator**: Expression evaluation
- **fhirpath-model**: Value types and ModelProvider trait
- **fhirpath-registry**: Function registry
- **fhirpath-analyzer**: Static analysis
- **fhirpath-diagnostics**: Error handling

#### Private Crates (NOT published)
- **fhirpath-cli**: Complete CLI application with FhirSchemaModelProvider, REPL, server
- **fhirpath-dev-tools**: Test runners, benchmarking, coverage analysis

### JSON Processing
**Important**: This codebase uses `serde_json::Value` for all JSON processing. Maintain consistency by always using `serde_json` throughout the codebase. Do not introduce other JSON libraries unless there is a compelling performance or compatibility reason.

## Interactive REPL

The FHIRPath REPL provides an interactive environment for rapid prototyping and debugging of FHIRPath expressions. It includes the following features:

### REPL Commands
- `<expression>` - Evaluate any FHIRPath expression
- `:load <file>` - Load FHIR resource from file
- `:set <name> <value>` - Set variable value 
- `:unset <name>` - Remove variable
- `:vars` - List all variables and context
- `:resource` - Show current resource information
- `:type <expression>` - Show type information (planned)
- `:explain <expression>` - Show evaluation steps (planned)
- `:help [function]` - Show help for commands or functions
- `:history` - Show command history
- `:quit` - Exit REPL

### Features
- **Interactive line editing** with history and arrow key navigation
- **Auto-completion** for function names and properties
- **Colored output** for better readability
- **Variable management** for complex expression building
- **Resource loading** from JSON files
- **Command history** with persistent storage
- **Help system** with function documentation
- **Error handling** with clear, actionable messages

### Usage Examples
```bash
# Start REPL
just repl

# Start REPL with initial resource
just repl examples/patient.json

# Start REPL with specific FHIR version
just repl --fhir-version r5

# Example REPL session
fhirpath> :load examples/patient.json
Loaded Patient resource (id: example-1)

fhirpath> Patient.name.given.first()
"John"

fhirpath> :set myVar "test"
Variable 'myVar' set

fhirpath> :vars
%context = Patient resource
myVar = "test"

fhirpath> :help first
first() - Returns the first item in a collection
Usage: collection.first()
Returns: single item or empty if collection is empty

fhirpath> :quit
```

## Development Patterns

### Testing Strategy
- **Unit Tests**: Each crate has comprehensive unit tests
- **Integration Tests**: Cross-crate functionality testing
- **Specification Compliance**: 1104 official FHIRPath tests (90.9% pass rate)
- **Performance Tests**: Automated benchmarking and regression detection
- Always run `just test-coverage` to update compliance report

### Code Quality Standards
- **Zero Warnings**: All clippy warnings must be resolved
- **Documentation**: All public APIs must have doc comments
- **Formatting**: Uses rustfmt with 100-character line limit
- **Performance**: Maintain 100K+ ops/sec for parser, 1K+ ops/sec for evaluator

### Error Handling
- Uses comprehensive diagnostic system with source location tracking
- All errors include helpful context and suggestions
- Parser has error recovery capabilities

### FHIRPath Function Implementation
When implementing new FHIRPath functions:
1. Add function to appropriate category in `fhirpath-registry/src/operations/`
2. Register in the registry with proper signature
3. Add comprehensive tests including edge cases
4. Update test coverage by running official test suites
5. Consider performance implications and add benchmarks if needed

### Performance Considerations  
- Use `SmallVec` for small collections to avoid heap allocation
- Prefer arena allocation in evaluation contexts
- Profile with `just bench` before and after changes
- Memory usage is critical for large Bundle resources

### CLI Development
The main CLI binary is now in `crates/fhirpath-cli/src/main.rs`. All CLI functionality has been separated from the main library and uses FhirSchemaModelProvider for full FHIR schema support. The CLI provides consistent output formatting and includes REPL, server, and various output modes.

## Testing and Validation

### Running Tests
- `just test` - All tests
- `just test-official` - Official FHIRPath specification tests  
- `cargo test specific_test_name -- --nocapture` - Individual test with output
- `cargo test --package crate-name` - Tests for specific crate

### Test Coverage
Current status: **90.9%** (1003/1104 tests passing)
- Run `just test-coverage` to update TEST_COVERAGE.md
- Focus on improving coverage in areas marked 🟠 or 🔴 in test report
- All new functionality must include tests

### Performance Benchmarks
- `just bench` provides comprehensive performance metrics
- Parser target: 100K+ operations/second  
- Evaluator target: 1K+ operations/second with Bundle resolution
- Memory efficiency is crucial for healthcare applications
- **Important**: This codebase uses **Divan** for all benchmarking. Always use Divan instead of Criterion when creating new benchmarks.

## Environment Variables

### Development
- `RUST_LOG=debug` - Enable debug logging
- `RUST_BACKTRACE=1` - Enable backtraces
- `FHIRPATH_USE_MOCK_PROVIDER=1` - Use MockModelProvider for faster testing

### CLI Usage
- `FHIRPATH_MODEL` - Default model provider (mock, r4, r5)
- `FHIRPATH_TIMEOUT` - Default timeout in seconds
- `FHIRPATH_OUTPUT_FORMAT` - Default output format (raw, pretty, json, table)
- `FHIRPATH_NO_COLOR` - Disable colored output (same as NO_COLOR)

## Key Files and Directories

- `justfile` - All development commands
- `Cargo.toml` - Workspace configuration with serde_json, tokio, and healthcare-specific dependencies
- `specs/fhirpath/tests/` - Official FHIRPath test suite (1104 tests)
- `TEST_COVERAGE.md` - Auto-generated compliance report
- `benchmark.md` - Performance benchmark results
- `docs/ARCHITECTURE.md` - Detailed technical architecture
- `docs/DEVELOPMENT.md` - Comprehensive development guide
- `CLI.md` - Complete CLI reference

## Special Considerations

### FHIRPath Compliance
- Follow FHIRPath specification exactly (http://hl7.org/fhirpath/)
- Any deviations must be documented with rationale
- Test against official test suites regularly
- Current focus: improving from 90.9% to 95%+ compliance

### Healthcare Data Processing
- Large Bundle resources are common (hundreds of entries)
- Reference resolution across Bundle entries is critical
- Type safety and validation are essential for medical data
- Performance matters for high-throughput healthcare systems

### ModelProvider Architecture
Starting from v0.3.0, ModelProvider is mandatory:
- `MockModelProvider` - Fast, simple provider for development/testing (main library)
- `FhirSchemaModelProvider` - Full FHIR R4/R5 schema integration (CLI and dev tools only)
- Async operations for external data fetching
- Caching is essential for performance

**Important**: The main library (`octofhir-fhirpath`) now only includes MockModelProvider to keep dependencies minimal. For full FHIR schema support with FhirSchemaModelProvider, use the CLI crate (`fhirpath-cli`) or dev tools (`fhirpath-dev-tools`).

**Critical**: Do NOT hardcode FHIR properties, choice types, or resource types in ModelProvider implementations. All FHIR schema information (properties, types, choices, resource definitions) MUST be dynamically retrieved from FHIRSchema. This ensures:
- Accurate compliance with official FHIR specifications
- Support for all FHIR versions and profiles
- Automatic updates when FHIR schemas change
- Consistency with the broader FHIR ecosystem

### Release Process
- Run `just release-prep` for complete quality assurance
- Update CHANGELOG.md for significant changes
- Use semantic versioning (currently v0.4.x)
- All releases require 90%+ test compliance

## Current Priorities

1. **Improve FHIRPath compliance** from 90.9% to 95%+
2. **Optimize performance** for complex expressions on large Bundles  
3. **Enhance error messages** with better diagnostics
4. **Complete missing functions** (see TEST_COVERAGE.md for specifics)
5. **Add analyzer integration** for static analysis capabilities

## Development Philosophy

**Develop as a Professional Rust Developer and FHIR Expert:**
- Write idiomatic, clean Rust code that follows best practices
- Apply deep FHIR domain knowledge to design decisions
- Prioritize async-first architecture without over-engineering or unnecessary overhead
- Keep the codebase simple, readable, and maintainable
- Focus on performance and correctness without sacrificing code clarity
- Use established patterns and avoid premature optimization
- Ensure all code is production-ready and enterprise-grade

## Integration Notes

This codebase integrates with broader healthcare ecosystem:
- Uses octofhir-* family of healthcare crates
- Compatible with FHIR R4 and R5 specifications
- Designed for integration with EHR systems and healthcare APIs
- Thread-safe for server applications