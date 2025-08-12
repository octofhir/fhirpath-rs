# ADR-002: FHIRPath Analyzer Crate Implementation

## Status

Proposed

## Context

The implementation of the MCP server's `explain_expression` tool requires sophisticated static analysis capabilities for FHIRPath expressions. Currently, our codebase has basic type checking in `fhirpath-evaluator/src/type_checker.rs`, but lacks comprehensive static analysis features needed for:

1. **Expression Explanation**: Detailed step-by-step breakdown of FHIRPath expressions
2. **Type Inference**: Static type analysis without execution
3. **Complexity Analysis**: Performance characteristics and optimization suggestions
4. **Diagnostic Generation**: Syntax errors, type mismatches, and warnings
5. **Symbol Resolution**: Identifier, function, and path resolution for IDE support

### Research Findings

**From atomic-ehr/fhirpath Repository:**
- Comprehensive async analyzer with static type analysis
- Expression parsing and validation with error recovery
- Debugging and inspection with execution traces
- Registry API for function/operator introspection
- LSP-compatible diagnostic and symbol information

**Current Codebase Analysis:**
- Basic `TypeChecker` in `fhirpath-evaluator` for runtime type operations (`is`, `as`, `ofType`)
- Disabled analyzer tests in `tests/analyzer_tests.rs.disabled` showing comprehensive test coverage
- Existing `fhirpath-ast/src/visitor.rs` pattern for AST traversal
- ModelProvider integration for async type information

### Problem Statement

For the MCP server's `explain_expression` tool to provide educational value, we need:
- Static analysis of expressions without evaluation
- Type inference and validation
- Complexity metrics and performance insights
- Detailed explanations of expression semantics
- Integration with existing AST and ModelProvider infrastructure

## Decision

We will implement a new `fhirpath-analyzer` crate that provides comprehensive static analysis capabilities for FHIRPath expressions, designed specifically to support the MCP server's explanation and validation tools.

### Architecture Design

#### 1. Crate Structure
```
crates/fhirpath-analyzer/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Main analyzer API
│   ├── analyzer.rs               # Core analyzer implementation
│   ├── type_inference/           # Static type analysis
│   │   ├── mod.rs
│   │   ├── inference_engine.rs   # Type inference engine
│   │   ├── type_context.rs       # Type resolution context
│   │   └── type_rules.rs         # FHIRPath type rules
│   ├── expression_analysis/      # Expression-level analysis
│   │   ├── mod.rs
│   │   ├── complexity.rs         # Complexity metrics
│   │   ├── semantics.rs          # Semantic analysis
│   │   └── optimization.rs       # Optimization suggestions
│   ├── diagnostics/              # Error and warning system
│   │   ├── mod.rs
│   │   ├── diagnostic_engine.rs  # Diagnostic generation
│   │   ├── error_codes.rs        # Standard error codes
│   │   └── validator.rs          # Expression validation
│   ├── explanation/              # Expression explanation
│   │   ├── mod.rs
│   │   ├── explainer.rs          # Main explanation engine
│   │   ├── step_builder.rs       # Step-by-step breakdown
│   │   └── documentation.rs      # Function/operator docs
│   ├── symbol_resolution/        # Symbol and reference resolution
│   │   ├── mod.rs
│   │   ├── symbol_resolver.rs    # Symbol resolution engine
│   │   ├── scope_manager.rs      # Variable scope tracking
│   │   └── reference_finder.rs   # Reference finding
│   └── visitor/                  # AST traversal patterns
│       ├── mod.rs
│       ├── analysis_visitor.rs   # Base analysis visitor
│       └── typed_visitor.rs      # Type-aware visitor
```

#### 2. Core Components

**Main Analyzer Interface**
```rust
pub struct FhirPathAnalyzer {
    provider: Arc<dyn ModelProvider>,
    type_inference: TypeInferenceEngine,
    expression_analyzer: ExpressionAnalyzer,
    diagnostic_engine: DiagnosticEngine,
    explainer: ExpressionExplainer,
    symbol_resolver: SymbolResolver,
}

pub struct AnalysisResult {
    pub type_info: TypeAnalysisResult,
    pub complexity: ComplexityMetrics,
    pub diagnostics: Vec<Diagnostic>,
    pub explanation: ExpressionExplanation,
    pub symbols: Vec<SymbolInfo>,
    pub suggestions: Vec<OptimizationSuggestion>,
}
```

**Type Inference Engine**
```rust
pub struct TypeInferenceEngine {
    provider: Arc<dyn ModelProvider>,
    context: TypeInferenceContext,
}

pub struct TypeAnalysisResult {
    pub inferred_type: Option<String>,
    pub is_collection: bool,
    pub confidence: f32,
    pub referenced_types: Vec<String>,
    pub type_constraints: Vec<TypeConstraint>,
}
```

**Expression Analysis**
```rust
pub struct ExpressionAnalyzer {
    complexity_analyzer: ComplexityAnalyzer,
    semantic_analyzer: SemanticAnalyzer,
    optimization_analyzer: OptimizationAnalyzer,
}

pub struct ComplexityMetrics {
    pub node_count: usize,
    pub depth: usize,
    pub navigation_count: usize,
    pub function_call_count: usize,
    pub estimated_cost: u32,
    pub performance_class: PerformanceClass,
}
```

**Expression Explanation**
```rust
pub struct ExpressionExplainer {
    step_builder: StepBuilder,
    documentation_provider: DocumentationProvider,
}

pub struct ExpressionExplanation {
    pub overall_description: String,
    pub steps: Vec<ExplanationStep>,
    pub context_type: Option<String>,
    pub return_type: Option<String>,
    pub examples: Vec<String>,
}

pub struct ExplanationStep {
    pub operation: String,
    pub description: String,
    pub input_type: Option<String>,
    pub output_type: Option<String>,
    pub notes: Vec<String>,
}
```

**Diagnostic System**
```rust
pub struct DiagnosticEngine {
    validators: Vec<Box<dyn ExpressionValidator>>,
    error_codes: ErrorCodeRegistry,
}

pub struct Diagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub location: Option<SourceLocation>,
    pub suggestions: Vec<String>,
}
```

#### 3. Integration Points

**With Existing Crates:**
- **fhirpath-ast**: Use existing AST types and visitor patterns
- **fhirpath-model**: Leverage ModelProvider for type information
- **fhirpath-registry**: Access function/operator metadata
- **fhirpath-parser**: Integrate with parsing errors and source locations
- **fhirpath-evaluator**: Reference existing TypeChecker for consistency

**For MCP Server:**
- Provides `explain_expression` tool implementation
- Supports `parse_fhirpath` tool with detailed validation
- Enhances `evaluate_fhirpath` with pre-analysis optimization

#### 4. Key Features

**Static Type Inference**
- Infer types without evaluation using ModelProvider
- Handle collections, polymorphic types, and union types
- Confidence scoring for type inferences
- Type constraint validation

**Complexity Analysis**
- Node counting and depth analysis
- Performance classification (Simple, Moderate, Complex, Expensive)
- Memory usage estimation
- Optimization opportunities identification

**Comprehensive Diagnostics**
- Syntax error detection with recovery suggestions
- Type mismatch identification
- Unknown function/property warnings
- Performance warnings for expensive operations

**Educational Explanations**
- Step-by-step expression breakdown
- Function documentation with examples
- Context-aware explanations based on input type
- Common pattern recognition and suggestions

**Symbol Resolution**
- Identifier resolution to FHIR types/properties
- Function and operator documentation
- Variable scope tracking ($this, $index, etc.)
- Reference finding for IDE navigation

#### 5. Implementation Strategy

**Phase 1: Core Infrastructure (Week 1)**
- Set up crate structure and dependencies
- Implement basic analyzer interface
- Create AST visitor patterns for analysis
- Basic type inference for literals and simple expressions

**Phase 2: Type System (Week 2)**
- Complete type inference engine
- ModelProvider integration for FHIR types
- Type constraint validation
- Collection and polymorphic type handling

**Phase 3: Analysis Features (Week 3)**
- Complexity analysis implementation
- Diagnostic system with error codes
- Basic expression explanation
- Function/operator documentation integration

**Phase 4: Advanced Features (Week 4)**
- Symbol resolution and scope management
- Optimization suggestion engine
- Performance analysis and warnings
- Integration testing with MCP server

**Phase 5: Documentation and Testing (Week 5)**
- Comprehensive test suite
- Performance benchmarks
- API documentation
- Integration examples

### API Design

**Primary Interface**
```rust
impl FhirPathAnalyzer {
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self;
    
    pub async fn analyze(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
    ) -> Result<AnalysisResult, AnalysisError>;
    
    pub async fn explain(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
    ) -> Result<ExpressionExplanation, AnalysisError>;
    
    pub async fn validate(
        &self,
        expression: &ExpressionNode,
        context_type: Option<&str>,
    ) -> Result<Vec<Diagnostic>, AnalysisError>;
}
```

**Builder Pattern for Configuration**
```rust
let analyzer = AnalyzerBuilder::new(provider)
    .with_detailed_type_inference(true)
    .with_complexity_analysis(true)
    .with_optimization_suggestions(true)
    .with_max_depth(50)
    .build();
```

## Consequences

### Positive
- **Educational Value**: Enables comprehensive expression explanation for MCP server
- **Code Quality**: Improves overall codebase with static analysis capabilities
- **Developer Experience**: Supports IDE integration and debugging tools
- **Performance**: Identifies optimization opportunities before evaluation
- **Modularity**: Clean separation of analysis from evaluation logic
- **Extensibility**: Foundation for future analysis features

### Negative
- **Complexity**: Adds another significant crate to maintain
- **Dependencies**: Requires coordination with multiple existing crates
- **Performance Overhead**: Static analysis adds computational cost
- **API Surface**: Increases public API that needs to be maintained

### Neutral
- **Code Size**: Substantial addition but follows existing patterns
- **Testing Requirements**: Needs comprehensive test coverage
- **Documentation Burden**: Requires extensive documentation for educational use

## Implementation Checklist

### Phase 1: Foundation
- [ ] Create `fhirpath-analyzer` crate with proper workspace integration
- [ ] Implement basic analyzer interface and builder pattern
- [ ] Create AST visitor patterns for analysis traversal
- [ ] Set up integration with existing AST types
- [ ] Basic type inference for literal expressions

### Phase 2: Type System
- [ ] Complete type inference engine with ModelProvider integration
- [ ] Implement type constraint validation
- [ ] Handle collection types and polymorphic expressions
- [ ] Add confidence scoring for type inferences
- [ ] Integration testing with real FHIR schemas

### Phase 3: Analysis Features
- [ ] Implement complexity analysis with performance classification
- [ ] Build diagnostic system with standard error codes
- [ ] Create expression explanation engine
- [ ] Integrate function/operator documentation
- [ ] Add optimization suggestion system

### Phase 4: Advanced Features
- [ ] Implement symbol resolution and scope management
- [ ] Add reference finding capabilities
- [ ] Performance analysis and warning system
- [ ] Complete MCP server integration
- [ ] End-to-end testing with explain_expression tool

### Phase 5: Quality and Documentation
- [ ] Comprehensive test suite with real FHIRPath expressions
- [ ] Performance benchmarks and optimization
- [ ] Complete API documentation with examples
- [ ] Integration guides for MCP server usage
- [ ] Educational examples and tutorials

## References

- [atomic-ehr/fhirpath Analyzer Implementation](https://github.com/atomic-ehr/fhirpath)
- [FHIRPath Specification - Type System](http://hl7.org/fhirpath/#type-system)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [Existing TypeChecker Implementation](../../crates/fhirpath-evaluator/src/type_checker.rs)
- [Disabled Analyzer Tests](../../tests/analyzer_tests.rs.disabled)