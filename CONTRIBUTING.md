# Contributing to FHIRPath-rs

Thank you for your interest in contributing to FHIRPath-rs! This guide will help you get started with contributing to our high-performance FHIRPath implementation.

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.70+ (install via [rustup](https://rustup.rs/))
- **Just**: Command runner (install via `cargo install just`)
- **Git**: Version control

### Development Setup

```bash
# Clone the repository
git clone https://github.com/octofhir/fhirpath-rs.git
cd fhirpath-rs

# Build the project
just build

# Run tests to ensure everything works
just test

# Run quality assurance checks
just qa
```

## 🎯 Ways to Contribute

### 1. Bug Reports

Found a bug? Help us fix it!

- **Search existing issues** first to avoid duplicates
- **Use our issue template** for bug reports
- **Include minimal reproduction steps**
- **Provide version information** and environment details

### 2. Feature Requests

Have an idea for improvement?

- **Check existing feature requests** first
- **Describe the use case** and benefits
- **Consider backwards compatibility**
- **Discuss implementation approach** if you have ideas

### 3. Code Contributions

Ready to code? Here's how:

#### Small Changes
- Documentation improvements
- Bug fixes
- Test improvements
- Performance optimizations

#### Major Changes
- New FHIRPath functions
- Architecture improvements
- New features

**For major changes, please open an issue first to discuss the approach.**

### 4. Documentation

Help improve our documentation:

- **API documentation** improvements
- **Tutorial and examples**
- **Architecture documentation**
- **Performance guides**

## 🛠️ Development Workflow

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/your-username/fhirpath-rs.git
cd fhirpath-rs

# Add upstream remote
git remote add upstream https://github.com/octofhir/fhirpath-rs.git
```

### 2. Create a Branch

```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes:
git checkout -b fix/issue-description
```

### 3. Development Commands

```bash
# Run tests frequently during development
just test

# Run specific test suites
just test-official          # Official FHIRPath test suite
just test-failed           # Tests that are currently failing

# Check code quality
just fmt                   # Format code
just clippy               # Run linting
just qa                   # Full quality assurance

# Run benchmarks (for performance changes)
just bench
just bench-full

# Generate documentation
just doc
just docs                 # Full documentation including benchmarks

# Update test coverage
just test-coverage
```

### 4. Code Style Guidelines

#### Rust Style
- Follow **standard Rust formatting** (`cargo fmt`)
- Use **meaningful variable names**
- Add **doc comments** for public APIs
- Follow **Rust API Guidelines**

#### Performance Guidelines
- **Profile before optimizing** using `just bench`
- **Maintain zero-copy parsing** where possible
- **Use appropriate data structures** (Vec vs SmallVec, etc.)
- **Consider memory allocation patterns**

#### FHIRPath Compliance
- **Follow FHIRPath specification** exactly
- **Add tests from official test suite** when implementing features
- **Maintain backwards compatibility**
- **Document any deviations** from the spec (with rationale)

### 5. Testing Requirements

All contributions must include appropriate tests:

#### Unit Tests
```bash
# Add tests in the same file or `tests/` directory
cargo test your_test_name
```

#### Integration Tests
```bash
# For major features, add integration tests
just test-official
```

#### Performance Tests
```bash
# For performance-critical changes
just bench
```

### 6. Commit Guidelines

Follow conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `perf`: Performance improvement
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `chore`: Maintenance tasks

**Examples:**
```
feat(parser): add support for new FHIRPath operators

fix(evaluator): handle null values in comparison operations

perf(tokenizer): optimize string parsing for 15% speedup

docs(readme): add examples for common use cases
```

### 7. Pull Request Process

#### Before Submitting
```bash
# Ensure your branch is up to date
git fetch upstream
git rebase upstream/main

# Run full quality assurance
just qa

# Update test coverage if needed
just test-coverage

# Generate documentation
just docs
```

#### PR Requirements
- ✅ **All tests pass** (`just test`)
- ✅ **Code is formatted** (`just fmt`)
- ✅ **No clippy warnings** (`just clippy`)
- ✅ **Documentation updated** for new features
- ✅ **Test coverage maintained** or improved
- ✅ **Performance benchmarks** run (for perf changes)

#### PR Description Template
```markdown
## Summary
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Test improvement

## Changes Made
- List specific changes
- Include any breaking changes
- Note performance impacts

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Official FHIRPath tests pass
- [ ] Performance benchmarks run

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests cover edge cases
```

## 🎯 Areas Needing Help

### High Priority
1. **FHIRPath Compliance**: Improve test pass rate from 82.7% to 90%+
2. **Performance**: Optimize evaluator for complex expressions
3. **Error Messages**: Enhance diagnostic information
4. **Documentation**: More examples and tutorials

### Function Implementation
Current implementation status and areas needing work:

#### 🟡 Partially Implemented (Help Needed)
- **Date/Time functions**: `lowBoundary()`, `highBoundary()` edge cases
- **Type operations**: Complex type inheritance scenarios  
- **Variable scoping**: Advanced `defineVariable()` cases
- **Quantity operations**: UCUM unit conversions

#### 🔴 Missing Features (Good First Issues)
- **Extension functions**: FHIR-specific extensions
- **Advanced operators**: Some edge cases in comparison operators
- **Error recovery**: Better parser error recovery
- **LSP support**: Language server protocol features

### Testing & Quality
- **Edge case testing**: Improve test coverage for error conditions
- **Property-based tests**: Add fuzzing and property tests
- **Performance regression tests**: Automated performance monitoring
- **Memory usage profiling**: Optimize memory consumption

## 🏗️ Architecture Overview

Understanding the codebase structure:

```
src/
├── ast/              # Abstract syntax tree
│   ├── expression.rs # Expression nodes
│   └── visitor.rs    # AST visitor pattern
├── parser/           # Parsing pipeline
│   ├── tokenizer.rs  # Lexical analysis
│   ├── pratt.rs      # Pratt parser
│   └── error.rs      # Parse errors
├── evaluator/        # Expression evaluation
│   ├── engine.rs     # Main evaluation engine
│   └── context.rs    # Variable context management
├── registry/         # Function registry
│   ├── functions/    # Built-in function implementations
│   └── operators/    # Operator implementations
├── model/            # Data model
│   ├── value.rs      # FHIRPath value types
│   └── resource.rs   # FHIR resource representation
└── diagnostics/      # Error handling
    ├── diagnostic.rs # Diagnostic messages
    └── formatter.rs  # Error formatting
```

### Key Design Principles

1. **Performance First**: Zero-copy parsing, efficient data structures
2. **Safety**: Memory safety through Rust's type system
3. **Compliance**: Strict adherence to FHIRPath specification
4. **Modularity**: Clean separation of concerns
5. **Testability**: Comprehensive test coverage

## 🔍 Code Review Process

### What We Look For

#### Functionality
- ✅ **Correctness**: Does it work as intended?
- ✅ **Edge cases**: Are error conditions handled?
- ✅ **FHIRPath compliance**: Follows specification exactly
- ✅ **Performance**: No unnecessary performance regressions

#### Code Quality
- ✅ **Readability**: Clear, well-structured code
- ✅ **Documentation**: Public APIs documented
- ✅ **Tests**: Comprehensive test coverage
- ✅ **Error handling**: Proper error propagation

#### Design
- ✅ **Architecture**: Fits well with existing design
- ✅ **API design**: Follows Rust conventions
- ✅ **Backwards compatibility**: No breaking changes without discussion
- ✅ **Future-proofing**: Considers future extensions

### Review Timeline
- **Simple fixes**: 1-2 days
- **New features**: 3-7 days
- **Major changes**: 1-2 weeks (may require multiple review rounds)

## 🏆 Recognition

Contributors are recognized in several ways:

- **Contributors list** in README.md
- **Release notes** credit for significant contributions  
- **Issues and PRs** tagged with contributor acknowledgments
- **Maintainer status** for consistent, high-quality contributors

## 📞 Getting Help

### Discussion Channels
- **GitHub Discussions**: General questions and ideas
- **Issues**: Bug reports and feature requests
- **Email**: funyloony@gmail.com for sensitive topics

### Development Help
- **Architecture questions**: Open a discussion
- **FHIRPath specification**: Reference official docs
- **Performance questions**: Share benchmark results
- **Testing help**: Look at existing test patterns

## 📋 Issue Templates

### Bug Report Template
```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Expression: `Patient.name.given`
2. Input data: `{...}`
3. Expected result: `[...]`
4. Actual result: `[...]`

**Environment**
- fhirpath-rs version: 
- Rust version:
- OS:

**Additional context**
Any other context about the problem.
```

### Feature Request Template  
```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**FHIRPath specification**
Link to relevant specification section if applicable.

**Additional context**
Any other context or screenshots about the feature request.
```

## 🎉 First Time Contributors

Welcome! Here are some good first issues:

### Easy Wins
- **Documentation improvements**: Fix typos, add examples
- **Test additions**: Add test cases for existing functions
- **Error message improvements**: Make error messages clearer
- **Code formatting**: Fix clippy warnings

### Guided Issues
Look for issues labeled:
- `good first issue`: Beginner-friendly
- `help wanted`: Community help needed
- `documentation`: Documentation improvements
- `testing`: Test-related work

## 📜 License

By contributing to FHIRPath-rs, you agree that your contributions will be licensed under the Apache License, Version 2.0.

---

Thank you for contributing to FHIRPath-rs! Together, we're building the best FHIRPath implementation in Rust. 🦀❤️