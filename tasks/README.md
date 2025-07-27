# FHIRPath-RS Development Tasks

This directory contains the comprehensive development plan for bringing fhirpath-rs to full FHIRPath specification compliance and LSP-ready performance.

## Quick Start

1. **Review the roadmap**: Start with `00-development-roadmap.md`
2. **Check task status**: Use `00-task-index.md` for current progress
3. **Begin development**: Start with `phase1-01-parser-error-analysis.md`

## File Organization

### Planning Documents
- `00-development-roadmap.md` - High-level development strategy
- `00-task-index.md` - Master task tracking system
- `README.md` - This file

### Phase 1: Critical Parser Fixes (CRITICAL)
- `phase1-01-parser-error-analysis.md` - Diagnose parser foundation issues
- `phase1-02-function-call-parser-fix.md` - Fix function call parsing

### Phase 2: Core Missing Functions (HIGH)
- `phase2-01-collection-operations.md` - first(), last(), tail(), indexer
- `phase2-02-aggregate-functions.md` - sum(), avg(), min(), max()
- `phase2-03-set-operations.md` - combine(), exclude(), join()
- `phase2-04-fhir-type-system.md` - Quantity, precision, inheritance

### Future Phases
Additional task files will be created as development progresses through:
- Phase 3: Partial Implementation Completion
- Phase 4: Edge Case Resolution  
- Phase 5: Performance Optimization
- Phase 6: Advanced Features

## Current Status

- **Test Coverage**: 32.2% (324/1005 tests passing)
- **Total Tasks**: 29 planned across 6 phases
- **Immediate Priority**: Complete partially implemented core functionality
- **Timeline**: 15-20 weeks to completion

## Key Metrics

| Phase | Tasks | Timeline | Impact |
|-------|-------|----------|---------|
| Phase 1 | 6 | 4-5 weeks | Increase coverage to 55-60% |
| Phase 2 | 7 | 5-6 weeks | Increase coverage to 75-80% |
| Phase 3 | 6 | 3-4 weeks | Increase coverage to 85-90% |
| Phase 4 | 3 | 1-2 weeks | Increase coverage to 92-95% |
| Phase 5 | 4 | 2-3 weeks | Sub-ms parsing performance |
| Phase 6 | 3 | 2-3 weeks | 98%+ coverage + advanced features |

## Development Workflow

1. **Task Selection**: Choose next task from critical path
2. **Branch Creation**: `feature/[task-id]-[short-name]`
3. **Implementation**: Follow task acceptance criteria
4. **Testing**: Ensure test coverage improvements
5. **Review**: Code review and documentation update
6. **Integration**: Merge and update task status

## Progress Tracking

- Update `00-task-index.md` weekly
- Run `./scripts/update-test-coverage.sh` after major completions
- Track phase progress percentages
- Identify and resolve blockers promptly

## Critical Path

**Must complete first** (blocking other work):
1. phase1-01 - Parser Error Analysis
2. phase1-02 - Function Call Parser Fix  
3. phase2-01 - Collection Operations

**High impact** (major coverage improvements):
- phase2-02 - Aggregate Functions
- phase2-03 - Set Operations
- phase2-04 - FHIR Type System

## Success Criteria

- **Functional**: 95%+ test coverage across all FHIRPath features
- **Performance**: Sub-millisecond parsing for typical expressions
- **Quality**: Clean, maintainable, well-documented code
- **Compliance**: Full FHIRPath specification adherence

## Getting Help

- Review task dependencies before starting
- Check existing test coverage for context
- Refer to FHIRPath specification in `specs/` directory
- Follow Rust performance guidelines in `CLAUDE.md`

---

*Generated on: 2025-07-27*
*Last Updated: 2025-07-27*
