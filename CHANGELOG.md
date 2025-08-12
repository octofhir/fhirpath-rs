# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Architecture Decision Records (ADRs) for major architectural decisions
  - ADR-001: Model Context Protocol (MCP) Server Implementation
  - ADR-002: FHIRPath Analyzer Crate for static analysis and expression explanation
- Enhanced CLAUDE.md with improved workspace architecture documentation
- Simple and complete usage examples in README.md
- CLI quick start example with practical usage
- Comprehensive contributor guide updates
- Future development roadmap with MCP server and analyzer crate plans

### Changed
- Updated README.md with modular workspace architecture description
- Improved Quick Start section with both simple and complete examples
- Enhanced CLI documentation with practical examples
- Updated CONTRIBUTING.md to reflect workspace structure
- Updated specification compliance rate to 88.1%
- Enhanced development workflow documentation
- Updated architecture overview to show 11 specialized crates

### Planned
- Model Context Protocol (MCP) server implementation for AI assistant integration
- FHIRPath analyzer crate for static analysis and expression explanation
- Cross-platform distribution with GitHub releases and Docker images

## [0.4.0] - 2024-XX-XX

### Added
- Comprehensive CI/CD pipeline with GitHub Actions
- Canary releases on every main branch push
- Automated release workflow with manual trigger
- Automatic crates.io publishing on tag push
- GitHub issue templates for bug reports and feature requests
- Pull request template with comprehensive checklist
- Dependabot configuration for automated dependency updates
- Code coverage reporting with codecov
- Security audit workflow
- MSRV (Minimum Supported Rust Version) testing

### Changed
- Package name from `fhirpath-rs` to `octofhir-fhirpath` for crates.io compatibility
- Library name from `fhirpath_rs` to `octofhir_fhirpath` for consistency
- Updated package metadata with better description and categorization
- Added MSRV specification (Rust 1.87.0)

### Fixed
- Resolved deprecation warnings in benchmark files
- Fixed clippy warning about infinite loop in parser
- Fixed compilation errors in diagnostic reporter
- Fixed hanging cache tests with proper eviction logic
- Fixed union type compatibility and signature matching
- All tests now pass (122 passed, 0 failed)

## [0.2.0] - 2024-07-31

### Added
- Comprehensive FHIRPath implementation with 82.7% official spec compliance
- Modular architecture with separate components for parsing, evaluation, and registry
- High-performance tokenizer and parser using nom library
- Function registry with built-in FHIRPath functions
- Type system with comprehensive type checking
- Diagnostic system with detailed error reporting
- Caching infrastructure for improved performance
- Benchmark suite for performance monitoring
- CLI tool for FHIRPath evaluation
- Integration with official FHIRPath test suites

### Changed
- Consolidated from multi-crate workspace to single crate with modular structure
- Improved error handling and diagnostic reporting
- Enhanced type system with better compatibility checking

### Fixed
- Numerous parsing edge cases
- Type coercion and compatibility issues
- Performance bottlenecks in evaluation engine

## [0.1.0] - Initial Release

### Added
- Basic FHIRPath parser and evaluator
- Core functionality for FHIRPath expressions
- Initial test suite
- Basic documentation