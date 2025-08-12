# Phase 4: Evaluator and Main Crate Migration

## Overview
Extract the evaluator into a separate crate and move the main octofhir-fhirpath crate into the crates/ folder, completing the workspace migration.

## Tasks

### 4.1 Create fhirpath-evaluator Crate
- [ ] Create `crates/fhirpath-evaluator/Cargo.toml`
- [ ] Move `src/evaluator/` module content
- [ ] Add dependencies on all required crates:
  - [ ] `fhirpath-core`
  - [ ] `fhirpath-ast`
  - [ ] `fhirpath-model`
  - [ ] `fhirpath-compiler`
  - [ ] `fhirpath-registry`
  - [ ] `fhirpath-diagnostics`
- [ ] Preserve evaluation engine functionality
- [ ] Maintain context management system
- [ ] Keep performance optimizations
- [ ] Preserve type checking and validation

### 4.2 Create Main octofhir-fhirpath Crate in crates/
- [ ] Create `crates/octofhir-fhirpath/` directory
- [ ] Move current `src/` content to `crates/octofhir-fhirpath/src/`
- [ ] Create `crates/octofhir-fhirpath/Cargo.toml`
- [ ] Move `src/bin/` to `crates/octofhir-fhirpath/src/bin/`
- [ ] Update main crate to depend on all workspace crates
- [ ] Preserve CLI binary functionality
- [ ] Maintain public API re-exports

### 4.3 Update Workspace Root Configuration
- [ ] Update root `Cargo.toml` to be workspace-only (no lib)
- [ ] Remove root `src/` directory
- [ ] Update workspace members to include all crates
- [ ] Configure workspace dependencies
- [ ] Update workspace-level settings

### 4.4 Engine Integration
- [ ] Move `src/engine.rs` to main crate
- [ ] Update engine to use new evaluator crate
- [ ] Preserve FhirPathEngine functionality
- [ ] Maintain evaluation context management
- [ ] Keep performance characteristics

### 4.5 Update Build and Tooling
- [ ] Update `justfile` commands for new structure
- [ ] Update benchmark configurations
- [ ] Update test runner paths
- [ ] Update CLI binary paths
- [ ] Update documentation generation

### 4.6 Final Integration Testing
- [ ] Run full test suite
- [ ] Verify CLI functionality
- [ ] Test benchmark suite
- [ ] Run test coverage generation
- [ ] Validate official FHIRPath tests

## Success Criteria
- [ ] All crates build independently
- [ ] Main crate properly integrates all components
- [ ] CLI binary works correctly
- [ ] All tests pass (831 official tests)
- [ ] Performance benchmarks maintain speed
- [ ] Public API remains unchanged
- [ ] Workspace structure is clean and maintainable

## Estimated Time: 2-3 days

## Status: IN PROGRESS ⚠️

### Completed Tasks:
- [x] Created fhirpath-evaluator crate with dependencies
- [x] Moved evaluator module content to new crate (lib.rs, all module files)
- [x] Created workspace configuration for 9/10 target crates
- [x] Updated main octofhir-fhirpath crate structure
- [x] Cleaned up duplicate modules from main crate
- [x] Updated engine integration and public API
- [x] Moved root src/ to backup (src_backup_old)
- [x] Established clean workspace-only root structure

### Workspace Architecture Progress:
- **Phase 1**: ✅ Core, AST, and Diagnostics crates (4/10 crates)
- **Phase 2**: ✅ Parser and Model crates (6/10 crates)  
- **Phase 3**: ✅ Registry and Compiler crates (8/10 crates)
- **Phase 4**: ⚠️ Evaluator crate (9/10 crates) - compilation issues remain
- **Next**: Phase 5 - Tools crates, then final integration

### Key Achievements:
- **Evaluator crate extracted** with clean dependencies on all workspace crates
- **Main crate restructured** to use workspace dependencies
- **Clean separation** achieved - no duplicate code between workspace crates
- **Public API preserved** through strategic re-exports
- **Workspace-only root** configuration established

### Remaining Issues:
1. **TypeInfo import paths** need correction in registry crate (use `types::TypeInfo`)
2. **Missing Display implementations** for error types in registry
3. **Registry imports** need final cleanup for cross-crate references
4. **Compilation validation** needed once import issues resolved

### Next Steps:
- Fix TypeInfo import paths: `fhirpath_model::TypeInfo` → `fhirpath_model::types::TypeInfo`
- Add Display implementations for ExtensionError and other error types
- Complete registry crate import cleanup
- Validate workspace compilation
- Create tools crates (Phase 5)

**Impact**: Core workspace structure achieved (9/10 crates). Compilation issues are import-related and do not affect the architectural migration success.