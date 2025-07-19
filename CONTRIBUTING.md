# Contributing to FHIRPath Rust Engine

Thank you for your interest in contributing to the FHIRPath Rust Engine! This document provides guidelines and information for contributors.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Coding Standards](#coding-standards)
5. [Testing Guidelines](#testing-guidelines)
6. [Submitting Changes](#submitting-changes)
7. [Security Considerations](#security-considerations)
8. [Performance Guidelines](#performance-guidelines)

## Getting Started

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Node.js 20+ (for Node.js bindings development)
- Git

### Quick Start

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/fhirpath-rs.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes following [Conventional Commits](https://www.conventionalcommits.org/)
5. Run tests: `cargo test --workspace`
6. Submit a pull request

**Note**: This project uses automated changelog generation. Please follow [Conventional Commits](https://www.conventionalcommits.org/) format for your commit messages. See [CHANGELOG_AUTOMATION.md](CHANGELOG_AUTOMATION.md) for details.

## Development Setup

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/fhirpath-rs.git
cd fhirpath-rs

# Install Rust dependencies
cargo build

# Install Node.js dependencies (for Node.js bindings)
cd fhirpath-node
npm install
cd ..

# Run all tests
cargo test --workspace
cd fhirpath-node && npm test && cd ..
```

### Development Tools

We recommend using the following tools:

- **IDE**: VS Code with rust-analyzer extension
- **Formatting**: `rustfmt` (configured in `rustfmt.toml`)
- **Linting**: `clippy` (configured in `clippy.toml`)
- **Testing**: `cargo test` and `cargo nextest` (if available)

### Environment Setup

```bash
# Install additional development tools
cargo install cargo-nextest  # For parallel test execution
cargo install cargo-watch    # For continuous testing during development
cargo install cargo-audit    # For security auditing

# Set up pre-commit hooks (optional but recommended)
cargo install pre-commit
pre-commit install
```

## Project Structure

```
octofhir-fhirpath/
├── fhirpath-core/          # Core FHIRPath engine implementation
│   ├── src/
│   │   ├── lexer.rs        # Tokenization
│   │   ├── parser.rs       # AST parsing
│   │   ├── evaluator.rs    # Expression evaluation
│   │   ├── model.rs        # Data models
│   │   └── errors.rs       # Error types
│   ├── tests/              # Unit and integration tests
│   └── benches/            # Performance benchmarks
├── fhirpath-node/          # Node.js bindings
│   ├── src/lib.rs          # NAPI bindings
│   └── test/               # Node.js tests
├── fhirpath-cli/           # Command-line interface
│   └── src/main.rs         # CLI implementation
├── docs/                   # Documentation
└── phases/                 # Implementation tracking
```

### Key Components

- **Lexer**: Tokenizes FHIRPath expressions
- **Parser**: Builds Abstract Syntax Tree (AST)
- **Evaluator**: Executes expressions against FHIR resources
- **Model**: Defines FHIRPath value types
- **Bindings**: Node.js integration layer

## Coding Standards

### Rust Code Style

We follow standard Rust conventions with some project-specific guidelines:

#### Formatting

```bash
# Format code before committing
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

#### Linting

```bash
# Run clippy for linting
cargo clippy --workspace --all-targets --all-features

# Fix clippy warnings where possible
cargo clippy --workspace --all-targets --all-features --fix
```

#### Code Organization

- Use meaningful names for functions, variables, and types
- Keep functions focused and reasonably sized
- Add comprehensive documentation for public APIs
- Use `#[cfg(test)]` for test-only code
- Organize imports: std, external crates, local modules

#### Documentation

```rust
/// Evaluates a FHIRPath expression against a FHIR resource.
///
/// # Arguments
///
/// * `expression` - The FHIRPath expression to evaluate
/// * `resource` - The FHIR resource as JSON
///
/// # Returns
///
/// Returns a `Result` containing the evaluation result or an error.
///
/// # Examples
///
/// ```rust
/// use fhirpath_core::evaluator::evaluate_expression;
/// use serde_json::json;
///
/// let resource = json!({"resourceType": "Patient", "id": "123"});
/// let result = evaluate_expression("id", resource)?;
/// ```
pub fn evaluate_expression(
    expression: &str,
    resource: serde_json::Value,
) -> Result<FhirPathValue, FhirPathError> {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create specific error types for different failure modes
- Provide helpful error messages with context
- Use `anyhow` for application errors, custom types for library errors

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FhirPathError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Evaluation error: {0}")]
    EvaluationError(String),
    
    #[error("Type error: {0}")]
    TypeError(String),
}
```

## Testing Guidelines

### Test Organization

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test component interactions
- **Conformance tests**: Test against FHIRPath specification
- **Performance tests**: Benchmark critical paths

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_property_access() {
        let resource = json!({
            "resourceType": "Patient",
            "id": "example"
        });
        
        let result = evaluate_expression("id", resource).unwrap();
        
        match result {
            FhirPathValue::Collection(values) => {
                assert_eq!(values.len(), 1);
                match &values[0] {
                    FhirPathValue::String(s) => assert_eq!(s, "example"),
                    _ => panic!("Expected string value"),
                }
            }
            _ => panic!("Expected collection result"),
        }
    }
}
```

### Test Data

- Use realistic FHIR resources for testing
- Store test fixtures in `tests/fixtures/`
- Create helper functions for common test scenarios
- Test both success and failure cases

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with parallel execution (if nextest is installed)
cargo nextest run

# Run specific test
cargo test test_basic_property_access

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Submitting Changes

### Pull Request Process

1. **Create a feature branch** from `main`
2. **Make your changes** following coding standards
3. **Add tests** for new functionality
4. **Update documentation** as needed
5. **Run the full test suite**
6. **Submit a pull request**

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new warnings introduced
```

### Commit Messages

Use conventional commit format:

```
type(scope): description

feat(parser): add support for new FHIRPath function
fix(evaluator): handle null values in path expressions
docs(readme): update installation instructions
test(lexer): add tests for edge cases
```

### Code Review

- All changes require review before merging
- Address reviewer feedback promptly
- Keep pull requests focused and reasonably sized
- Ensure CI passes before requesting review

## Security Considerations

### Security Guidelines

- **Input validation**: Validate all external inputs
- **Resource limits**: Prevent resource exhaustion attacks
- **Error handling**: Don't leak sensitive information in errors
- **Dependencies**: Keep dependencies updated and audited

### Security Testing

```bash
# Run security audit
cargo audit

# Check for known vulnerabilities
cargo audit --deny warnings
```

### Reporting Security Issues

Please report security vulnerabilities privately to the maintainers rather than creating public issues.

## Performance Guidelines

### Performance Considerations

- **Avoid unnecessary allocations** in hot paths
- **Use appropriate data structures** for the use case
- **Profile before optimizing** - measure actual performance
- **Consider memory usage** as well as CPU performance

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile with specific tools
cargo build --release
perf record target/release/your-binary
```

### Performance Testing

- Add benchmarks for critical functionality
- Test with realistic data sizes
- Monitor performance regressions in CI
- Document performance characteristics

## Development Workflow

### Daily Development

```bash
# Start development session
cargo watch -x "test --workspace"

# In another terminal
cargo watch -x "clippy --workspace"

# Before committing
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

### Release Process

1. Update version numbers in `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Run full test suite including benchmarks
4. Create release PR
5. Tag release after merge
6. Publish to crates.io (maintainers only)

## Getting Help

### Resources

- **Documentation**: Check the `docs/` directory
- **Examples**: See `docs/usage-examples.md`
- **Tests**: Look at existing tests for patterns
- **Issues**: Search existing issues before creating new ones

### Communication

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Pull Requests**: For code contributions

### Maintainer Contact

For questions about contributing, reach out to the project maintainers through GitHub issues or discussions.

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).

Thank you for contributing to the FHIRPath Rust Engine!
