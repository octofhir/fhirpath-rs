# Development Guide

This document provides comprehensive guidance for developers who want to contribute to or work with octofhir-fhirpath.

## Table of Contents

- [Quick Setup](#quick-setup)
- [Development Commands](#development-commands)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Performance and Benchmarking](#performance-and-benchmarking)
- [Code Quality](#code-quality)
- [Contributing](#contributing)
- [Tools and Utilities](#tools-and-utilities)

## Quick Setup

### Prerequisites

- **Rust 1.87+**: Install from [rustup.rs](https://rustup.rs/)
- **Just**: Command runner - `cargo install just`
- **Git**: Version control

### Clone and Setup

```bash
# Clone the repository
git clone https://github.com/octofhir/fhirpath-rs.git
cd fhirpath-rs

# Build the project
just build

# Run tests to verify setup
just test

# Check code quality
just qa
```

## Development Commands

All common development tasks are managed through the `justfile`. Here are the essential commands:

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
```

### Code Quality

```bash
# Fix all formatting and clippy issues automatically
just fix

# Run clippy linting
just clippy

# Format code
just fmt

# Check code without building
just check

# Quality assurance (format + lint + test)
just qa
```

### Performance and Benchmarking

```bash
# Run simplified unified benchmark suite
just bench

# Run full benchmark suite (same as bench - simplified)
just bench-full

# Update benchmark documentation
just bench-update-docs
```

### Test-Specific Commands

```bash
# Run specific test case from official FHIRPath test suite
just test-case test-case-name

# Example: Run literals tests
just test-case literals

# Run failed expression tests
just test-failed
```

### Documentation

```bash
# Generate API documentation
just doc

# Generate complete documentation (including dependencies)
just doc-all

# Generate all documentation (API + benchmarks)
just docs
```

### CLI Development

```bash
# Evaluate FHIRPath expression (read FHIR resource from stdin)
just cli-evaluate "Patient.name.given"

# Evaluate FHIRPath expression with specific file
just cli-evaluate "Patient.name.given" path/to/resource.json

# Parse FHIRPath expression to AST
just cli-parse "Patient.name.given"

# Validate FHIRPath expression syntax
just cli-validate "Patient.name.given"

# Show CLI help
just cli-help
```

### Utility Commands

```bash
# Clean build artifacts
just clean

# Security audit
just audit

# Install development tools
just install-tools

# Watch for changes and run tests
just watch

# Expand macros for debugging
just expand [ITEM]

# Release preparation (full QA + docs)
just release-prep
```

## Project Structure

### Workspace Architecture

The project uses a modular workspace structure with specialized crates:

```
fhirpath-rs/
├── crates/
│   ├── octofhir-fhirpath/     # Main library (re-exports)
│   ├── fhirpath-core/         # Core types and errors
│   ├── fhirpath-parser/       # Tokenizer and parser
│   ├── fhirpath-evaluator/    # Expression evaluation
│   ├── fhirpath-model/        # Value types and FHIR model
│   ├── fhirpath-registry/     # Function and operator registry
│   ├── fhirpath-tools/        # CLI tools and utilities
│   └── fhirpath-bench/        # Benchmarking and profiling
├── docs/                      # Documentation
├── specs/                     # FHIRPath specification and tests
├── scripts/                   # Build and release scripts  
└── tasks/                     # Development task tracking
```

### Key Files

- **`justfile`**: Development command definitions
- **`CLAUDE.md`**: Project-specific AI assistant instructions
- **`CONTRIBUTING.md`**: Contribution guidelines
- **`TEST_COVERAGE.md`**: Automatically generated test coverage report
- **`benchmark.md`**: Performance benchmark results

## Development Workflow

### 1. Creating a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

### 2. Development Cycle

```bash
# Make your changes...

# Check formatting and linting
just fix

# Run tests
just test

# Run quality assurance
just qa
```

### 3. Testing Changes

```bash
# Run specific tests
cargo test test_name -- --nocapture

# Run tests for specific crate
cargo test --package fhirpath-parser

# Test against official FHIRPath specification
just test-official

# Update test coverage report
just test-coverage
```

### 4. Performance Testing

```bash
# Run benchmarks to check for regressions
just bench

# Profile specific expressions (if needed)
just bench-update-docs
```

### 5. Final Checks

```bash
# Complete quality assurance
just qa

# Ensure documentation builds
just doc

# Final test run
just test
```

## Testing

### Test Categories

1. **Unit Tests**: Individual crate functionality
2. **Integration Tests**: Cross-crate functionality  
3. **Specification Tests**: Official FHIRPath test suites (1017 tests)
4. **Performance Tests**: Benchmarking and regression testing
5. **Property Tests**: Fuzzing and edge case validation

### Running Tests

```bash
# All tests
just test

# Specific test case
cargo test test_function_name -- --nocapture

# Tests with output
cargo test -- --nocapture

# Specific crate tests
cargo test --package fhirpath-evaluator

# Official FHIRPath compliance tests
just test-official

# Coverage report generation
just test-coverage
```

### Test Coverage Tracking

The project maintains automated test coverage tracking:

```bash
# Generate coverage report
just test-coverage

# View results
cat TEST_COVERAGE.md
```

**Current Coverage**: 87.0% (885/1017 official tests passing)

### Adding New Tests

#### Unit Tests
Add tests in the appropriate crate's `src/` directory:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_functionality() {
        // Test implementation
    }

    #[tokio::test]  // For async tests
    async fn test_async_functionality() {
        // Async test implementation
    }
}
```

#### Integration Tests
Add integration tests in `crates/{crate}/tests/`:

```rust
use octofhir_fhirpath::*;

#[tokio::test]
async fn test_end_to_end_workflow() {
    // Integration test implementation
}
```

## Performance and Benchmarking

### Running Benchmarks

```bash
# Run unified benchmark suite
just bench

# Update benchmark documentation
just bench-update-docs
```

### Benchmark Categories

1. **Parser Performance**: Expression parsing speed
2. **Evaluator Performance**: Expression evaluation speed
3. **Function Performance**: Individual function benchmarks
4. **Memory Usage**: Memory consumption analysis

### Adding Benchmarks

Add benchmarks to `crates/fhirpath-bench/src/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_your_feature(c: &mut Criterion) {
    c.bench_function("your_feature", |b| {
        b.iter(|| {
            // Benchmark code
            black_box(your_function())
        })
    });
}

criterion_group!(benches, benchmark_your_feature);
criterion_main!(benches);
```

### Performance Guidelines

- **Parser Target**: 100K+ operations/second
- **Evaluator Target**: 1K+ operations/second (with Bundle resolution)
- **Memory Usage**: Minimize allocations in hot paths
- **Profiling**: Use `cargo flamegraph` for detailed profiling

## Code Quality

### Automated Quality Checks

```bash
# Fix all issues automatically
just fix

# Manual quality assurance
just qa
```

The project maintains **zero warnings** through:

- **Clippy**: Comprehensive linting
- **rustfmt**: Consistent code formatting
- **CI/CD**: Automated quality enforcement

### Code Style Guidelines

1. **Follow Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
2. **Use Clippy**: All clippy warnings must be resolved
3. **Documentation**: All public APIs must be documented
4. **Error Handling**: Use proper error types and handling
5. **Testing**: All new functionality must have tests

### Documentation Standards

```rust
/// Brief description of the function.
/// 
/// # Arguments
/// 
/// * `param` - Description of parameter
/// 
/// # Returns
/// 
/// Description of return value
/// 
/// # Errors
/// 
/// Description of possible errors
/// 
/// # Examples
/// 
/// ```rust
/// // Example usage
/// ```
pub fn your_function(param: Type) -> Result<ReturnType, Error> {
    // Implementation
}
```

## Contributing

### Before Starting

1. **Check Issues**: Look for existing issues or create one
2. **Discuss Changes**: For large changes, discuss in issues first
3. **Review Guidelines**: Read `CONTRIBUTING.md` thoroughly

### Development Process

1. **Fork and Clone**: Fork the repository and clone your fork
2. **Create Branch**: Create feature branch from `main`
3. **Make Changes**: Follow development workflow
4. **Test Thoroughly**: Ensure all tests pass
5. **Submit PR**: Create pull request with clear description

### Pull Request Requirements

- [ ] All tests pass (`just test`)
- [ ] Code quality checks pass (`just qa`)
- [ ] Documentation is updated if needed
- [ ] Performance regressions are addressed
- [ ] CHANGELOG.md is updated (if applicable)

### Code Review Process

1. **Automated Checks**: CI/CD runs automated checks
2. **Maintainer Review**: Core maintainers review code
3. **Performance Review**: Performance impact is evaluated
4. **Documentation Review**: Documentation completeness is checked

## Tools and Utilities

### Required Tools

Install development tools:

```bash
just install-tools
```

This installs:
- `cargo-watch` - File watching for development
- `cargo-audit` - Security auditing
- `cargo-tarpaulin` - Code coverage (Linux)
- Additional development utilities

### Debugging Tools

#### Parser Debugging
For debugging parser issues, create simple tests:

```rust
#[test]
fn debug_parser_issue() {
    let expr = "problematic expression";
    match parse(expr) {
        Ok(ast) => println!("AST: {:#?}", ast),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

#### Evaluator Debugging
Use the trace function for runtime debugging:

```rust
// In FHIRPath expression
"Patient.name.trace('debug_name')"
```

#### Performance Debugging
Use cargo profiling tools:

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench your_benchmark
```

### IDE Integration

#### VS Code
Recommended extensions:
- `rust-analyzer` - Rust language server
- `CodeLLDB` - Debugging support
- `crates` - Crate management

#### IntelliJ IDEA/CLion  
- Install Rust plugin
- Configure with Cargo project

### Environment Variables

Set up development environment:

```bash
# Enable debug logging
export RUST_LOG=debug

# Enable backtraces
export RUST_BACKTRACE=1

# Optimize compilation for development
export CARGO_PROFILE_DEV_CODEGEN_UNITS=256
```

### Troubleshooting

#### Common Issues

**Build Failures:**
```bash
# Clean and rebuild
just clean
just build
```

**Test Failures:**
```bash
# Run specific failing test with output
cargo test failing_test_name -- --nocapture
```

**Performance Regressions:**
```bash
# Run benchmarks to identify issues
just bench
```

#### Getting Help

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/octofhir/fhirpath-rs/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/octofhir/fhirpath-rs/discussions)  
- 📧 **Email**: funyloony@gmail.com
- 📚 **Documentation**: Check docs/ directory

## Architecture Decision Records (ADRs)

Major architectural decisions are documented in `docs/adr/`. Before making significant changes:

1. Review existing ADRs
2. Create new ADR if needed
3. Follow ADR template format
4. Update task files in `tasks/` directory

## Release Process

### Preparing for Release

```bash
# Complete quality assurance
just release-prep

# This runs:
# - Full test suite
# - Documentation generation
# - Benchmark updates
# - Coverage report updates
```

### Version Management

The project uses semantic versioning:
- **Major**: Breaking changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes

### Migration Support

For users upgrading between versions:
- Check `MIGRATION_GUIDE.md` 
- Use migration scripts in `scripts/`
- Test with verification scripts

This comprehensive development guide should help you get started contributing to octofhir-fhirpath. For additional questions, please use the support channels listed above.