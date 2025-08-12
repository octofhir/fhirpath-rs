# Migration to Multi-Crate Architecture

## Overview
Migrate the current single-crate FHIRPath implementation to a multi-crate workspace architecture to improve compilation times, maintainability, and developer experience while preserving all existing functionality.

## Architecture Goals
- **Faster compilation**: Parallel compilation of independent crates
- **Better maintainability**: Clear module boundaries and dependencies
- **Easier development**: Multiple developers can work on different components
- **Preserved functionality**: Zero breaking changes to public API
- **Single published crate**: Only `octofhir-fhirpath` is published

## Target Crate Structure

### Published Crate
- **octofhir-fhirpath** (root crate) - Main library and CLI binary

### Internal Crates (not published)
1. **fhirpath-core** - Core types, traits, and abstractions
2. **fhirpath-ast** - AST definitions and visitor pattern
3. **fhirpath-parser** - Tokenizer and parser using nom
4. **fhirpath-evaluator** - Expression evaluation engine
5. **fhirpath-compiler** - Bytecode compilation and VM
6. **fhirpath-registry** - Function and operator registry
7. **fhirpath-model** - Value types and FHIR model support
8. **fhirpath-diagnostics** - Error handling and diagnostic reporting
9. **fhirpath-tools** - Test coverage and utility tools
10. **fhirpath-benchmarks** - Benchmarking infrastructure

## Dependency Graph
```
octofhir-fhirpath
├── fhirpath-evaluator
│   ├── fhirpath-compiler
│   │   ├── fhirpath-ast
│   │   │   └── fhirpath-core
│   │   └── fhirpath-registry
│   │       └── fhirpath-core
│   ├── fhirpath-model
│   │   └── fhirpath-core
│   └── fhirpath-diagnostics
│       └── fhirpath-core
├── fhirpath-parser
│   ├── fhirpath-ast
│   └── fhirpath-diagnostics
└── fhirpath-tools (dev-dependency)
    └── fhirpath-benchmarks (dev-dependency)
```

## Migration Benefits
1. **Compilation Speed**: Parallel compilation of independent crates
2. **Development Workflow**: Multiple developers can work simultaneously on different components
3. **Clean Dependencies**: Clear separation of concerns and dependency management
4. **Testing Isolation**: Unit tests can run faster on individual crates
5. **Feature Development**: New features can be developed in isolation
6. **Code Organization**: Better code organization with clear module boundaries

## Status: PLANNED
Next steps: Execute migration phases as defined in individual task files.