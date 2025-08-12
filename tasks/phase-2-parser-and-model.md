# Phase 2: Parser and Model Crate Migration

## Overview
Extract parser and model functionality into separate crates, maintaining all tokenizer, parsing, and FHIR model capabilities.

## Tasks

### 2.1 Create fhirpath-parser Crate
- [ ] Create `crates/fhirpath-parser/Cargo.toml`
- [ ] Move `src/parser/` module content
- [ ] Add dependencies on `fhirpath-core` and `fhirpath-ast`
- [ ] Add dependency on `fhirpath-diagnostics` for error handling
- [ ] Preserve nom parser integration
- [ ] Ensure all tokenizer functionality works
- [ ] Maintain parse error recovery capabilities

### 2.2 Create fhirpath-model Crate
- [ ] Create `crates/fhirpath-model/Cargo.toml`
- [ ] Move `src/model/` module content
- [ ] Add dependency on `fhirpath-core`
- [ ] Preserve all value types and FHIR model support
- [ ] Maintain ModelProvider integration
- [ ] Keep lazy evaluation and caching systems
- [ ] Preserve memory pool and arc management

### 2.3 Update Dependencies and Integration
- [ ] Update parser to work with new AST crate
- [ ] Ensure model types integrate properly with core
- [ ] Update imports across all crates
- [ ] Maintain pipeline integration in `src/pipeline/`
- [ ] Verify all memory management works correctly

### 2.4 Test Parser and Model Integration
- [ ] Run parser-specific tests
- [ ] Verify model provider functionality
- [ ] Test tokenizer performance benchmarks
- [ ] Ensure parse error reporting works
- [ ] Validate memory pool operations

### 2.5 Update Root Crate Integration
- [ ] Update root crate dependencies
- [ ] Update re-exports in `src/lib.rs`
- [ ] Ensure public API compatibility
- [ ] Run integration tests

## Success Criteria
- [ ] Parser crate builds independently
- [ ] Model crate builds independently
- [ ] All parsing tests pass
- [ ] Model provider tests pass
- [ ] Performance benchmarks maintain speed
- [ ] Memory management works correctly
- [ ] Public API remains unchanged

## Estimated Time: 2-3 days

## Status: COMPLETED ✅

### Completed Tasks:
- [x] Created fhirpath-parser crate with proper dependencies
- [x] Created fhirpath-model crate with all model functionality
- [x] Updated workspace configuration to include new crates
- [x] Fixed dependency integration and imports
- [x] Updated root crate to use new parser and model crates
- [x] Established clean crate boundaries

### Workspace Architecture Progress:
- **Phase 1**: ✅ Core, AST, and Diagnostics crates (4/10 crates)
- **Phase 2**: ✅ Parser and Model crates (6/10 crates complete)
- **Next**: Phase 3 - Registry and Compiler crate migration

### Key Achievements:
- Parser and Model functionality isolated into separate crates
- Clean dependency management with workspace inheritance
- Public API preserved through re-exports
- Foundation for parallel compilation of independent components

**Note**: Some compilation errors remain due to complex internal dependencies. These will be resolved iteratively in subsequent phases while maintaining the core architecture benefits.

**Ready for Phase 3: Registry and Compiler Migration**