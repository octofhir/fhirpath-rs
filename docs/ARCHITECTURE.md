# Architecture Guide

This document describes the technical architecture and design decisions behind octofhir-fhirpath.

## Overview

octofhir-fhirpath uses a **modular workspace architecture** with 11 specialized crates designed for flexibility, performance, and maintainability. The core philosophy is to provide a unified, thread-safe engine while allowing fine-grained control over individual components.

## Workspace Structure

```
crates/
├── octofhir-fhirpath/    # Main library (re-exports all components)
├── fhirpath-core/        # Core types, errors, and evaluation results
├── fhirpath-ast/         # Abstract syntax tree definitions
├── fhirpath-parser/      # Tokenizer and parser (nom-based)
├── fhirpath-evaluator/   # Expression evaluation engine  
├── fhirpath-compiler/    # Bytecode compiler and virtual machine
├── fhirpath-registry/    # Function registry and built-ins
├── fhirpath-model/       # Value types and FHIR data model
├── fhirpath-diagnostics/ # Error handling and reporting
├── fhirpath-tools/       # CLI tools and test utilities
└── fhirpath-benchmarks/  # Performance testing and profiling
```

### Crate Responsibilities

#### octofhir-fhirpath (Main Library)
- **Purpose**: Primary entry point and unified API
- **Exports**: `FhirPathEngine`, `FhirPathValue`, parser functions, and all public APIs
- **Dependencies**: Re-exports components from all other crates
- **Key Components**: 
  - Unified `FhirPathEngine` with thread-safety guarantees
  - High-level convenience APIs
  - Integration testing

#### fhirpath-core  
- **Purpose**: Foundational types and error handling
- **Key Components**:
  - Core error types and result handling
  - Evaluation context definitions
  - Shared constants and utilities
- **Design**: Minimal dependencies, stable API surface

#### fhirpath-parser
- **Purpose**: Tokenization and parsing of FHIRPath expressions
- **Technology**: Built on nom parser combinator library (version 8)
- **Key Components**:
  - Tokenizer with support for all FHIRPath tokens
  - Recursive descent parser with error recovery
  - AST construction with source location tracking
- **Performance**: 117K-473K operations/second depending on expression complexity

#### fhirpath-evaluator
- **Purpose**: Expression evaluation engine with context management
- **Key Components**:
  - Context management and variable scoping
  - Specialized evaluators for different operation types
  - Lambda function support with optimization
  - Memory management with arena allocation
- **Performance**: 4K+ operations/second with Bundle resolution

#### fhirpath-model
- **Purpose**: Value types, FHIR data model, and ModelProvider integration
- **Key Components**:
  - `FhirPathValue` enum with smart collections
  - ModelProvider trait for async FHIR type resolution
  - Mock provider for testing and simple use cases
  - Type coercion and validation logic
- **Architecture**: Async-first design with caching capabilities

#### fhirpath-registry
- **Purpose**: Function and operator registry with built-in implementations
- **Key Components**:
  - Unified function registry system
  - 100+ built-in FHIRPath functions
  - Operator registry with precedence handling
  - Signature-based dispatch and caching
- **Extensibility**: Plugin architecture for custom functions

## Unified Engine Architecture

The core design principle is the **unified `FhirPathEngine`** that consolidates all evaluation capabilities into a single, thread-safe interface.

### Thread Safety Design

```rust
pub struct FhirPathEngine {
    // Thread-safe internal state
    // Immutable configuration
    // Cached registry lookups
}

unsafe impl Send for FhirPathEngine {}
unsafe impl Sync for FhirPathEngine {}
```

**Key Guarantees:**
- **Send**: Engine can be transferred between threads
- **Sync**: Engine can be shared between threads safely
- **No Locking**: Lock-free design for maximum performance
- **Immutable State**: Configuration is set at creation time

### Three-Stage Processing Pipeline

```
Input Expression → [Tokenizer] → Tokens → [Parser] → AST → [Evaluator] → Result
                       ↓             ↓            ↓
                   Error Recovery  AST Caching  Context Management
```

#### Stage 1: Tokenization
- **Input**: Raw FHIRPath expression string
- **Output**: Stream of typed tokens
- **Features**: 
  - Unicode support
  - Error recovery for invalid characters
  - Position tracking for diagnostics
- **Performance**: 10M+ operations/second (estimated)

#### Stage 2: Parsing  
- **Input**: Token stream
- **Output**: Abstract Syntax Tree (AST)
- **Features**:
  - Operator precedence handling
  - Error recovery and suggestions
  - Source location preservation
- **Performance**: 117K-473K operations/second

#### Stage 3: Evaluation
- **Input**: AST + FHIR resource + context
- **Output**: `FhirPathValue` result
- **Features**:
  - Context management and variable scoping
  - Reference resolution with Bundle support
  - Lambda optimization with early exit
- **Performance**: 4K+ operations/second

## JSON Processing with sonic_rs

### Why sonic_rs Over serde_json

octofhir-fhirpath uses **sonic_rs** instead of the more common serde_json for all JSON processing. This architectural decision brings significant performance and feature benefits:

#### Performance Advantages
- **Faster Parsing**: sonic_rs is built on SIMD-optimized JSON parsing, providing significantly faster JSON parsing than serde_json
- **Lower Memory Usage**: More efficient memory allocation patterns during JSON parsing and manipulation
- **Zero-Copy Operations**: Better support for zero-copy JSON operations where possible
- **Optimized for Large Documents**: Particularly beneficial for large FHIR Bundle documents common in healthcare

#### Healthcare-Specific Benefits
- **Large Bundle Handling**: Healthcare applications often process large Bundle resources with hundreds of entries. sonic_rs excels at this scale
- **High Throughput**: Production healthcare systems require processing thousands of FHIR resources per second. sonic_rs's performance characteristics align with these requirements
- **Memory Efficiency**: Healthcare applications often run in memory-constrained environments. sonic_rs's efficient memory usage is crucial

#### Technical Implementation
```rust
use sonic_rs::{json, Value as SonicValue};

// All JSON processing uses sonic_rs
let fhir_resource = json!({
    "resourceType": "Patient", 
    "name": [{"given": ["Alice"]}]
});

// Consistent type usage throughout the codebase
fn process_fhir_data(value: SonicValue) -> Result<FhirPathValue> {
    // Processing logic using sonic_rs types
}
```

#### Migration Strategy
The codebase has been systematically migrated from serde_json to sonic_rs:

1. **Core Types**: All internal JSON handling uses `sonic_rs::Value`
2. **API Consistency**: Public APIs accept sonic_rs types for optimal performance  
3. **Backward Compatibility**: Conversion utilities available for mixed environments
4. **Performance Testing**: Benchmarks demonstrate measurable improvements in JSON-heavy operations

#### Conversion Between Libraries
When integration with existing serde_json code is required:

```rust
// Convert from serde_json to sonic_rs
let serde_value: serde_json::Value = serde_json::json!({"key": "value"});
let sonic_value: sonic_rs::Value = sonic_rs::from_str(&serde_json::to_string(&serde_value)?)?;

// Convert from sonic_rs to serde_json (if needed)  
let sonic_value = sonic_rs::json!({"key": "value"});
let serde_value: serde_json::Value = serde_json::from_str(&sonic_rs::to_string(&sonic_value)?)?;
```

This strategic choice of sonic_rs reflects the library's commitment to high-performance healthcare data processing while maintaining developer-friendly APIs.

## Memory Management

### Arena-Based Allocation

The evaluator uses arena-based memory management to minimize allocation overhead:

```rust
struct EvaluationContext {
    arena: Arena<FhirPathValue>,
    variable_stack: Vec<VariableScope>,
    // ... other context data
}
```

**Benefits:**
- **Reduced GC Pressure**: Bulk deallocation at context end
- **Cache Locality**: Related data allocated together
- **Performance**: Fewer malloc/free calls

### String Interning

Commonly used strings are interned to reduce memory usage:

```rust
static INTERNER: OnceCell<StringInterner> = OnceCell::new();

fn intern_string(s: &str) -> InternedString {
    INTERNER.get_or_init(|| StringInterner::new()).intern(s)
}
```

### Smart Collections

The `FhirPathValue::Collection` type uses smart allocation strategies:

```rust
pub enum SmartCollection {
    Empty,
    Single(Box<FhirPathValue>),
    Small(SmallVec<[FhirPathValue; 4]>),
    Large(Vec<FhirPathValue>),
}
```

## ModelProvider Architecture

Starting from v0.3.0, the ModelProvider is mandatory for type resolution and validation:

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn get_type_info(&self, resource_type: &str) -> Result<TypeInfo>;
    async fn validate_path(&self, base_type: &str, path: &str) -> Result<PathInfo>;
    // ... other methods
}
```

### Implementation Types

1. **MockModelProvider**: Simple implementation for testing
2. **FhirSchemaProvider**: Full FHIR schema integration  
3. **CachedProvider**: Wrapper adding caching capabilities
4. **Custom Providers**: User-defined implementations

## Registry System

### Function Registry

Functions are registered with signature-based dispatch:

```rust
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: TypeInfo,
    pub is_lambda: bool,
}

impl FunctionRegistry {
    pub fn register<F>(&mut self, signature: FunctionSignature, implementation: F)
    where F: Fn(&[FhirPathValue], &EvaluationContext) -> Result<FhirPathValue> + Send + Sync;
}
```

### Operator Registry

Operators are handled through precedence-based parsing and registry lookup:

```rust
pub struct OperatorInfo {
    pub symbol: String,
    pub precedence: u8,
    pub associativity: Associativity,
    pub implementation: OperatorImpl,
}
```

### Caching Strategy

The registry employs multiple caching layers:

1. **Signature Cache**: Pre-compiled function signatures
2. **Dispatch Cache**: Fast lookup for function implementations  
3. **Type Cache**: Cached type resolution results
4. **AST Cache**: Parsed expression caching (optional)

## Reference Resolution

Enhanced Bundle support with sophisticated reference resolution:

### Resolution Algorithm

1. **Parse Reference**: Extract type and ID from reference string
2. **Check Context**: Determine if we're in a Bundle context  
3. **Resolve Strategy**:
   - Contained resources: `#id` within same resource
   - Bundle entries: Cross-reference via `fullUrl`
   - Relative references: `ResourceType/id` patterns
   - Absolute URLs: Full URL resolution

### Bundle Context Management

```rust
struct BundleContext {
    entries: HashMap<String, ResourceEntry>,
    fullurl_map: HashMap<String, usize>,
    id_map: HashMap<String, usize>,
}
```

## Performance Optimizations

### Lambda Optimization

Lambda functions like `where()`, `select()`, and `all()` include early exit patterns:

```rust
fn evaluate_where_lambda(collection: &[FhirPathValue], predicate: &Lambda) -> Result<Vec<FhirPathValue>> {
    let mut results = Vec::new();
    for item in collection {
        if evaluate_lambda(predicate, item)?.is_truthy() {
            results.push(item.clone());
        }
        // Early exit for `any()` when first truthy found
        // Continue for `all()` until first falsy found
    }
    Ok(results)
}
```

### Registry Fast-Path

Common functions and operators have optimized dispatch paths:

```rust
match function_name {
    "count" => fast_path_count(args),
    "first" => fast_path_first(args), 
    "where" => fast_path_where(args, context),
    _ => registry_dispatch(function_name, args, context),
}
```

### Memory Pool Reuse

Evaluation contexts are pooled and reused to minimize allocation:

```rust
static CONTEXT_POOL: OnceCell<Pool<EvaluationContext>> = OnceCell::new();
```

## Error Handling and Diagnostics

### Rich Error Information

Errors include detailed diagnostic information:

```rust
pub struct FhirPathError {
    pub kind: ErrorKind,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestions: Vec<String>,
    pub context: Vec<String>,
}
```

### Error Recovery

The parser includes error recovery to provide helpful suggestions:

1. **Token Recovery**: Skip invalid tokens and continue parsing
2. **Suggestion Generation**: Suggest corrections for typos
3. **Context Preservation**: Maintain parsing state for better errors

## Extension Architecture

The system supports extensions through the registry:

```rust
// Custom function registration
engine.registry_mut().register_function(
    "myCustomFunction",
    |args, context| {
        // Custom implementation
        Ok(FhirPathValue::String("result".into()))
    }
);

// Custom operator registration  
engine.registry_mut().register_operator(
    "~=",
    Precedence::Comparison,
    |left, right, context| {
        // Custom operator implementation
        fuzzy_match(left, right)
    }
);
```

## Testing Architecture

### Test Categories

1. **Unit Tests**: Individual crate testing
2. **Integration Tests**: Cross-crate functionality
3. **Specification Tests**: Official FHIRPath test suite (1017 tests)
4. **Performance Tests**: Benchmarking and regression testing
5. **Property Tests**: Fuzzing and edge case validation

### Mock Infrastructure

Comprehensive mocking for testing:

```rust
pub struct MockModelProvider {
    type_info: HashMap<String, TypeInfo>,
    validation_rules: Vec<ValidationRule>,
}
```

## Code Quality

### Zero Warnings Policy

The codebase maintains zero compiler warnings through:

- Comprehensive linting with clippy
- Automated formatting with rustfmt
- CI/CD enforcement of quality standards
- Regular dependency updates

### Documentation Standards

- **API Documentation**: All public APIs documented
- **Architecture Decision Records**: Major decisions documented in `docs/adr/`
- **Examples**: Comprehensive examples in documentation
- **Migration Guides**: Version migration assistance

This architecture provides a solid foundation for high-performance, maintainable FHIRPath evaluation while remaining flexible enough for diverse use cases in healthcare applications.