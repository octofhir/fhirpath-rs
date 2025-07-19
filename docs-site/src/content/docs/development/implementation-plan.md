---
title: Implementation Plan
description: Comprehensive plan for implementing the FHIRPath engine in Rust
---

# FHIRPath Rust Implementation Plan

> **Implementation Status**: For the current implementation status of this plan, please see the implementation status overview and individual phase status files.

## Additional Deliverables to Consider

- **CLI tool** (`fhirpath-cli`) – Useful for manual testing, demos, and CI validation
- **WASM build** – For browsers and Cloudflare Workers using `napi-experimental` that targets Wasm
- **Security audit checklist** – To ensure protection against untrusted inputs (unlimited recursion, resource exhaustion)
- **Benchmark corpus** – Compare to HAPI Java, HL7 JavaScript reference, and Google's fhir-py with public dashboard
- **Docker integration** – Provide a containerized version for easy deployment
- **Python bindings** – Consider PyO3 integration for Python ecosystem support
- **VS Code extension** – Simple syntax highlighting and validation for FHIRPath expressions


## Overview
This document outlines the plan to implement a FHIRPath engine in Rust, with Node.js bindings to make it usable from JavaScript/TypeScript applications. The implementation will follow the [FHIRPath specification](https://build.fhir.org/ig/HL7/FHIRPath/). The implementation will target FHIR R4 as the source-of-truth specification and be released under the Apache-2.0 license to ensure compatibility with the broader FHIR ecosystem.

## Phase 1: Project Setup

1. **Create project structure**
   - Initialize a Cargo workspace with multiple crates (`fhirpath-core`, `fhirpath-cli`, `fhirpath-node`)
   - Set up directory structure for lexer, parser, evaluator, and Node.js bindings
   - Configure Cargo.toml with pinned dependencies
   - Add a `devcontainer.json` or `Dockerfile` for consistent development environment
   - Add repository hygiene tools (Clippy, rustfmt) to CI checks

2. **Set up testing framework**
   - Configure test suite using Rust's built-in testing framework and `cargo nextest` for parallel testing
   - Download and commit the official FHIRPath test cases from the specification
   - Create a build script that can update test fixtures when needed
   - Set up CI/CD pipeline (GitHub Actions) with matrix testing

3. **Set up Node.js binding infrastructure**
   - Add napi-rs for Node.js bindings with support for both CommonJS and ES Modules
   - Configure build process for multiple platforms using `cargo zigbuild` or `cross + zig`
   - Set up TypeScript type definitions generation

## Phase 2: Core FHIRPath Engine Implementation

1. **Lexer implementation**
   - Implement tokenization of FHIRPath expressions
   - Handle identifiers, literals, operators, and functions
   - Add comprehensive tests for lexer functionality
   - Create diagnostic error types with source span information for better error messages

2. **Parser implementation**
   - Create abstract syntax tree (AST) representation
   - Decide between `nom` (streaming) or `pest` (PEG) parser approach and document the choice
   - Implement recursive descent parser for FHIRPath grammar
   - Handle operator precedence and associativity
   - Add comprehensive tests for parser functionality
   - Place the official FHIRPath grammar file (`.g4`/`.ebnf`) in `docs/spec/` for reference

3. **Data model implementation**
   - Evaluate existing FHIR Rust libraries (`fhirbolt`, `fhir-rs`) before creating custom structures
   - Define Rust structures for FHIRPath types (including collections)
   - Consider a "thin wrapper" view over `serde_json::Value` to avoid O(n) cloning
   - Implement FHIR resource representation compatible with FHIRPath
   - Create serialization/deserialization for JSON FHIR resources

4. **Expression evaluator implementation**
   - Build the evaluator to operate on an iterator stack (lazy evaluation)
   - Implement AST visitor/evaluator with hooks for debug tracing (`#[cfg(feature = "trace")]`)
   - Implement all FHIRPath functions and operators
   - Add type checking and type conversion
   - Handle collections and collection operators
   - Add benchmarks with `criterion = { version = "0.5", features = ["html_reports"] }`

## Phase 3: Advanced Features

1. **Implement navigation features**
   - Path navigation and traversal
   - Context management for evaluation
   - Handling of complex paths and recursion
   - Add "streaming mode" for very large resources using `serde_json::Deserializer::from_reader`

2. **Implement filtering and projection**
   - Where, select, and other collection operations
   - Implement collection manipulation functions
   - Optimize for memory efficiency on large collections

3. **Implement polymorphic behavior**
   - Type-based function dispatch
   - Handling of null/empty values
   - Implement FHIRPath equality semantics
   - Handle time-zone and precision rules for date/time types (`@2012-04` vs `@2012-04-01T00:00:00Z`)

4. **Optimization**
   - Implement expression optimization
   - Add caching strategies
   - Benchmark and profile for performance improvements
   - Compare performance with existing FHIRPath implementations (HAPI Java, HL7 JavaScript, Google's fhir-py)

## Phase 4: Node.js Integration

1. **Create Node.js binding layer**
   - Expose core FHIRPath functionality to JavaScript
   - Handle data conversion between Rust and JavaScript
   - Implement error handling and propagation
   - Use `napi-rs`'s `#[napi(task)]` attribute for CPU-bound operations to offload to a thread pool

2. **Create TypeScript type definitions**
   - Generate TypeScript interfaces for the library
   - Add JSDoc comments for better developer experience
   - Support TypeScript's strict mode

3. **Create JavaScript-friendly API**
   - Design an idiomatic JavaScript API
   - Implement Promise-based async interfaces where appropriate
   - Add JavaScript-specific convenience functions
   - Produce both CommonJS and ESModule entry points for maximum compatibility

## Phase 5: Testing and Documentation

1. **Comprehensive testing**
   - Unit tests for all components with `cargo nextest` for parallel execution
   - Integration tests with real FHIR resources
   - Conformance tests against FHIRPath specification examples
   - Performance benchmarks with Criterion and published dashboards
   - Run the same test matrix in Node.js to validate bindings

2. **Documentation**
   - Generate a `docs/` microsite (Docusaurus/VitePress) from one source of markdown
   - Rust API documentation with rustdoc (`cargo doc --document-private-items`)
   - JavaScript API documentation (TypeScript declarations via `api-extractor`)
   - Usage examples and tutorials
   - Contributing guidelines
   - Security considerations and audit checklist

3. **Create example applications**
   - CLI tool (`fhirpath-cli`) for manual testing and demos
   - Integration example with a Node.js application
   - Web/browser demo using the WASM build

## Phase 6: Packaging and Distribution

1. **Package for distribution**
   - Set up npm packaging
   - Automate cross-platform builds with `cargo zigbuild` or `cross + zig`
   - Provide pre-compiled binaries for all major platforms:
     - arm64-linux, x64-linux (both musl and gnu)
     - darwin-arm64, darwin-x64
     - win32-x64
   - Set up crates.io publishing
   - Create WASM build for browsers/Cloudflare Workers

2. **Release strategy**
   - Define versioning strategy
   - Set up release automation
   - Create change log management
   - Publish benchmark dashboard to GitHub Pages

## Implementation Details

### Key Dependencies

- **Rust Core**
  - `nom = "7.1.3"` or `pest = "2.7"` for parser combinators
  - `serde = "1.0.219"` for JSON serialization/deserialization
  - `serde_json = "1.0.140"` for JSON handling
  - `thiserror = "1.0.69"` for error handling
  - `criterion = { version = "0.5.1", features = ["html_reports"] }` for benchmarking

- **Node.js Bindings**
  - `napi-rs = "2.2.2"` for Node.js Native API bindings
  - `napi-build = "2.2.2"` for build scripts

### Performance Considerations

- Lazy evaluation where appropriate
- Minimizing memory allocations
- Effective caching strategies for parsed expressions
- Benchmarking against existing JavaScript implementations

### Testing Strategy

- Use the official FHIRPath test suite
- Create custom test cases for edge cases
- Fuzz testing for parser robustness
- Performance regression testing

## Timeline Estimates

- **Phase 1:** 1-2 weeks
- **Phase 2:** 4-6 weeks
- **Phase 3:** 3-4 weeks
- **Phase 4:** 2-3 weeks
- **Phase 5:** 2-3 weeks
- **Phase 6:** 1 week

**Total estimated time:** 13-19 weeks for an experienced 2-3 person team with full-time focus

Notes:
- Add 25-30% contingency for part-time contributors or those less experienced with Rust parsers or napi-rs
- Early focus on parser error handling and memory model will prevent costly refactors later

## Milestones

1. **M1:** Basic lexer and parser implementation
2. **M2:** Core evaluation engine with basic operations
3. **M3:** Complete FHIRPath function implementation
4. **M4:** Working Node.js integration
5. **M5:** Passing all specification tests
6. **M6:** Performance optimization complete
7. **M7:** First stable release

## Risks and Mitigations

- **Complex grammar implementation**
  - Start with a subset of features and expand
  - Use established parser combinator libraries
  - Place the official grammar file in the repo for easy reference
  - Invest early in good error reporting with source spans

- **Node.js binding complexity**
  - Leverage napi-rs which simplifies much of the work
  - Start with simple bindings and expand functionality
  - Test both CommonJS and ESModule loading patterns early

- **Performance challenges**
  - Profile early and often
  - Implement incremental optimizations
  - Use Criterion benchmarks to catch performance regressions
  - Optimize for both small and large FHIR resources
  - Consider streaming evaluation for large bundles

- **FHIR model complexity**
  - Focus initially on core functionality needed for FHIRPath
  - Consider using existing FHIR Rust libraries if available
  - Implement time/date handling with precision and timezone rules early
  - Handle edge cases like empty/null values consistently

## Next Steps

1. Set up the initial project structure with Cargo workspace
2. Download and commit the official FHIRPath test cases
3. Create CI configuration with Clippy and rustfmt checks
4. Set up Criterion benchmarks with a simple baseline
5. Implement a basic lexer for FHIRPath expressions with good error reporting
6. Create the initial test framework using the official test cases
7. Begin work on the parser implementation
8. Create a simple CLI tool for manual testing
