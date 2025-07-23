# ADR-005: FHIRPath Rust Library Development Plan

## Status
PROPOSED

## Context
The fhirpath-rs project aims to create a high-performance, production-ready FHIRPath evaluation engine in Rust. Based on comprehensive analysis of the FHIRPath 2.0.0 specification, reference implementations in Java (HAPI FHIR) and JavaScript (HL7 fhirpath.js), and the current implementation state, this ADR provides a structured development plan.

### Current State Analysis
- **Test Compliance**: 179/711 tests passing (25.2%)
- **Architecture**: Solid foundation with parser, AST, evaluator, and type system
- **Major Gaps**: Function library, advanced evaluation features, compilation issues
- **Strengths**: Good Rust practices, comprehensive error handling, performance-conscious design

## Decision
We will implement FHIRPath functionality in phases, prioritizing core compliance and stability before advanced features and optimizations.

## Development Plan

### Phase 1: Foundation Stabilization (Weeks 1-2)
**Goal: Achieve compilation and basic functionality**

#### 1.1 Fix Critical Issues
- Resolve compilation errors in `parser.rs`
- Fix incomplete datetime parsing logic
- Ensure all modules compile without warnings
- Validate test suite integration

#### 1.2 Core Infrastructure
- Stabilize AST representation for all expression types
- Ensure robust error propagation
- Validate parser coverage for FHIRPath grammar
- Test basic expression evaluation

**Success Criteria**: 
- Clean compilation with no errors/warnings
- Basic literal and path expressions work
- Test framework runs without crashes

### Phase 2: Essential Functions Implementation (Weeks 3-6)
**Goal: Implement core FHIRPath functions to reach 50% test compliance**

#### 2.1 Collection Functions (Priority 1)
```rust
// Core collection operations
exists()        // Check if collection is non-empty
empty()         // Check if collection is empty
count()         // Get collection size
first()         // Get first element
last()          // Get last element
tail()          // Get all but first
take(n)         // Get first n elements
skip(n)         // Skip first n elements
where(expr)     // Filter collection
select(expr)    // Transform collection
all(expr)       // Universal quantifier
any(expr)       // Existential quantifier
```

#### 2.2 String Functions (Priority 1)
```rust
// String manipulation
length()        // String length
substring(start, length) // Extract substring
contains(str)   // Check if contains substring
startsWith(str) // Check prefix
endsWith(str)   // Check suffix
matches(regex)  // Regular expression matching
replace(regex, replacement) // String replacement
split(separator) // Split string
lower()         // Convert to lowercase
upper()         // Convert to uppercase
```

#### 2.3 Math Functions (Priority 2)
```rust
// Mathematical operations
abs()           // Absolute value
ceiling()       // Round up
floor()         // Round down
round()         // Round to nearest
truncate()      // Truncate decimal
sqrt()          // Square root
power(exp)      // Power function
exp()           // Natural exponential
ln()            // Natural logarithm
log(base)       // Logarithm with base
```

#### 2.4 Type Conversion Functions (Priority 1)
```rust
// Type conversions
toString()      // Convert to string
toInteger()     // Convert to integer
toDecimal()     // Convert to decimal
toBoolean()     // Convert to boolean
toDate()        // Convert to date
toDateTime()    // Convert to datetime
toTime()        // Convert to time
toQuantity()    // Convert to quantity
convertsToString() // Check if convertible
convertsToInteger() // Check if convertible
// ... other conversion checks
```

**Success Criteria**: 
- 350+ tests passing (50% compliance)
- All basic collection operations work
- String and math functions functional
- Type conversions handle edge cases correctly

### Phase 3: Advanced Language Features (Weeks 7-10)
**Goal: Implement advanced FHIRPath features to reach 70% test compliance**

#### 3.1 Advanced Path Navigation
- Complex property access with filtering
- Method chaining with proper context
- Variable binding and scoping (`$this`, `$index`, `$total`)
- Polymorphic type handling (`ofType()`)

#### 3.2 Date/Time Operations
```rust
// Temporal functions
now()           // Current datetime
today()         // Current date
timeOfDay()     // Current time
+ operator      // Date arithmetic
- operator      // Date arithmetic
convertsToDate()
convertsToDateTime()
convertsToTime()
```

#### 3.3 Advanced Collection Operations
```rust
// Complex collection functions
repeat(expr)    // Recursive evaluation
aggregate(expr) // Collection aggregation
union()         // Set union
intersect()     // Set intersection
exclude()       // Set difference
distinct()      // Remove duplicates
flatten()       // Flatten nested collections
```

#### 3.4 Utility Functions
```rust
// Debugging and utility
trace(name)     // Debug output
iif(condition, true_result, false_result) // Conditional
```

**Success Criteria**: 
- 500+ tests passing (70% compliance)
- Complex path expressions work correctly
- Date/time arithmetic implemented
- Variable scoping functional

### Phase 4: FHIR-Specific Extensions (Weeks 11-12)
**Goal: Implement FHIR-specific functions and reach 85% test compliance**

#### 4.1 FHIR-Specific Functions
```rust
// FHIR extensions
extension(url)  // Extract extension by URL
resolve()       // Resolve references
memberOf(vs)    // Check value set membership
subsumes(code)  // Code subsumption
subsumedBy(code) // Reverse subsumption
htmlChecks()    // FHIR narrative validation
```

#### 4.2 FHIR Type Integration
- Proper handling of FHIR choice types (e.g., `value[x]`)
- Reference resolution mechanisms
- Extension handling patterns
- Coding and CodeableConcept equivalence

#### 4.3 Environment Variables
- Implement `%resource`, `%context`, `%ucum`
- Terminology service integration placeholders
- External function injection mechanism

**Success Criteria**: 
- 600+ tests passing (85% compliance)
- FHIR-specific functions work with real FHIR data
- Extension and reference handling functional

### Phase 5: Performance and Production Readiness (Weeks 13-16)
**Goal: Optimize performance and achieve production readiness**

#### 5.1 Performance Optimizations
- AST optimization and compilation
- Advanced expression caching strategies
- Memory usage optimization
- Benchmark against reference implementations

#### 5.2 Advanced Error Handling
- Detailed error messages with position information
- Recovery strategies for partial evaluation
- Debugging and tracing capabilities

#### 5.3 Production Features
- Thread safety validation
- Memory safety verification
- Security audit for expression evaluation
- Performance profiling and optimization

#### 5.4 API Stabilization
- Finalize public API design
- Comprehensive documentation
- Usage examples and guides
- Version compatibility strategy

**Success Criteria**: 
- 650+ tests passing (90%+ compliance)
- Performance competitive with reference implementations
- Production-ready stability and security
- Complete documentation and examples

## Implementation Guidelines

### Architecture Principles
1. **Modular Design**: Maintain clear separation between parser, evaluator, and type system
2. **Performance Focus**: Optimize for speed while maintaining memory safety
3. **Error Handling**: Provide detailed, actionable error messages
4. **Type Safety**: Leverage Rust's type system for compile-time correctness
5. **Extensibility**: Design for future FHIR version compatibility

### Development Standards
1. **Test-Driven Development**: Use failing official tests to guide implementation
2. **Documentation**: Document all public APIs with examples
3. **Security**: Never allow arbitrary code execution through FHIRPath
4. **Compatibility**: Maintain compatibility with FHIR R4/R5 specifications
5. **Performance**: Profile and benchmark all major features

### Quality Gates
- **All phases**: Zero compilation warnings
- **Phase 1**: Basic functionality tests pass
- **Phase 2**: 50% official test compliance
- **Phase 3**: 70% official test compliance  
- **Phase 4**: 85% official test compliance
- **Phase 5**: 90%+ compliance with production performance

## Integration with Existing Architecture

### Leverage Current Strengths
- **Parser**: Extend existing nom-based parser rather than rewrite
- **Type System**: Build on existing `FhirPathValue` and type registry
- **Error Handling**: Extend current error types for new functionality
- **Testing**: Use existing test infrastructure and comparison framework

### Follow Established Patterns
- **ADR Process**: Create ADRs for major architectural decisions
- **UCUM Integration**: Continue using external UCUM library with defensive programming
- **Performance Patterns**: Follow existing caching and optimization strategies

## Success Metrics

### Functional Metrics
- **Test Compliance**: Target 90%+ of official FHIRPath tests passing
- **Feature Coverage**: All core FHIRPath functions implemented
- **FHIR Integration**: Full support for FHIR-specific extensions

### Performance Metrics
- **Evaluation Speed**: Competitive with Java reference implementation
- **Memory Usage**: Minimal allocation during expression evaluation  
- **Compilation Time**: Sub-second for typical FHIRPath expressions

### Quality Metrics
- **Security**: No arbitrary code execution vulnerabilities
- **Stability**: Zero crashes on valid input, graceful errors on invalid input
- **Documentation**: 100% of public API documented with examples

## Risks and Mitigation

### Technical Risks
- **Complexity**: FHIRPath specification has many edge cases
  - *Mitigation*: Incremental implementation guided by test failures
- **Performance**: Rust's safety might impact evaluation speed
  - *Mitigation*: Profile early and optimize hot paths
- **FHIR Integration**: Complex FHIR data model interactions  
  - *Mitigation*: Study reference implementations, create focused ADRs

### Project Risks
- **Scope Creep**: Feature requests beyond core FHIRPath
  - *Mitigation*: Strict adherence to official specification
- **Resource Allocation**: Implementation complexity may exceed estimates
  - *Mitigation*: Prioritize core functionality, defer advanced features

## Decision Rationale

This phased approach balances several competing priorities:

1. **Stability First**: Fixing compilation issues and basic functionality ensures a solid foundation
2. **Test-Driven Priority**: Using official test compliance as success criteria ensures specification adherence
3. **Incremental Value**: Each phase delivers usable functionality
4. **Performance Consideration**: Defers optimization until core functionality is stable
5. **FHIR Integration**: Separates FHIRPath core from FHIR-specific features for clarity

The plan leverages the existing strong architectural foundation while addressing the major gaps identified in the current implementation. By following this structured approach, the project can achieve production-ready FHIRPath evaluation capability that meets or exceeds the performance and functionality of reference implementations.