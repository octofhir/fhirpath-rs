# ADR-001: FHIRPath Core Architecture Overview

## Status
Proposed

## Context
We need a high-performance FHIRPath engine that provides:
- Extensible operator and function registry
- Multi-version FHIR support via FHIR Schema
- Excellent diagnostics for both humans and machines
- Best-in-class performance
- Comprehensive test compliance

## Decision

### 1. Function Registry Architecture

```rust
// Trait-based function registry for extensibility
pub trait FhirPathFunction: Send + Sync {
    fn name(&self) -> &str;
    fn min_arity(&self) -> usize;
    fn max_arity(&self) -> Option<usize>;
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo;
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue>;
}

pub struct FunctionRegistry {
    functions: HashMap<String, Arc<dyn FhirPathFunction>>,
    // Function overloads by signature
    overloads: HashMap<String, Vec<FunctionSignature>>,
}

// Example built-in function
pub struct WhereFunction;
impl FhirPathFunction for WhereFunction {
    fn name(&self) -> &str { "where" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Implementation
    }
}
```

### 2. Operator Registry Architecture

```rust
pub trait FhirPathOperator: Send + Sync {
    fn symbol(&self) -> &str;
    fn precedence(&self) -> u8;
    fn associativity(&self) -> Associativity;
    fn evaluate(&self, left: &FhirPathValue, right: &FhirPathValue) -> Result<FhirPathValue>;
}

pub struct OperatorRegistry {
    binary_ops: HashMap<String, Arc<dyn FhirPathOperator>>,
    unary_ops: HashMap<String, Arc<dyn FhirPathOperator>>,
}
```

### 3. Model Provider with FHIR Schema

```rust
pub trait ModelProvider: Send + Sync {
    fn get_type_info(&self, type_name: &str) -> Option<TypeInfo>;
    fn get_property_type(&self, parent_type: &str, property: &str) -> Option<TypeInfo>;
    fn is_polymorphic(&self, property: &str) -> bool;
    fn get_search_params(&self, resource_type: &str) -> Vec<SearchParameter>;
}

pub struct FhirSchemaModelProvider {
    schema: FhirSchema,
    version: FhirVersion,
    // Cache for performance
    type_cache: Arc<RwLock<HashMap<String, TypeInfo>>>,
}

impl FhirSchemaModelProvider {
    pub fn from_schema_url(url: &str) -> Result<Self> {
        // Load FHIR Schema from URL
    }

    pub fn from_schema_file(path: &Path) -> Result<Self> {
        // Load FHIR Schema from file
    }
}
```

### 4. Diagnostic System

```rust
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub location: SourceLocation,
    pub suggestions: Vec<Suggestion>,
    pub related: Vec<RelatedInformation>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
    pub source_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub location: SourceLocation,
}

// LSP-compatible diagnostic producer
pub trait DiagnosticProducer {
    fn to_lsp_diagnostic(&self) -> lsp_types::Diagnostic;
    fn to_human_readable(&self) -> String;
}
```

### 5. Performance Optimizations

```rust
pub struct OptimizedEngine {
    // Expression cache with LRU eviction
    expression_cache: LruCache<String, Arc<ExpressionNode>>,

    // JIT-like optimization for hot paths
    hot_expressions: HashMap<String, OptimizedExpression>,

    // Thread-local evaluation contexts
    context_pool: ThreadLocal<RefCell<EvaluationContext>>,
}

// Zero-copy string handling
pub struct InternedString(Arc<str>);

// Optimized collection operations
pub struct LazyCollection<T> {
    source: CollectionSource<T>,
    operations: Vec<Operation>,
}
```

## Consequences

### Positive
- Highly extensible via trait-based design
- Type-safe with compile-time guarantees
- Excellent performance through caching and optimization
- Rich diagnostics for tooling integration
- Clean separation of concerns

### Negative
- More complex initial implementation
- Requires careful memory management
- Need to maintain compatibility across FHIR versions

### Risks
- FHIR Schema format changes
- Performance regression with complex expressions
- Memory usage with large datasets

## Implementation Guide

### Phase 1: Core Registry System

1. **Function Registry Implementation**
```rust
// fhirpath-core/src/registry/function.rs
impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
            signatures: HashMap::new(),
        };

        // Register all built-in functions
        registry.register_builtin_functions();
        registry
    }

    fn register_builtin_functions(&mut self) {
        // Collection functions
        self.register(WhereFunction::new());
        self.register(SelectFunction::new());
        self.register(FirstFunction::new());

        // String functions
        self.register(StartsWithFunction::new());
        self.register(ContainsFunction::new());

        // Math functions
        self.register(AbsFunction::new());
        self.register(RoundFunction::new());

        // Type functions
        self.register(OfTypeFunction::new());
        self.register(IsFunction::new());
    }
}

// Example function implementation with error handling
struct WhereFunction;
impl FhirPathFunction for WhereFunction {
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        let collection = context.input.to_collection();
        let mut result = Vec::new();

        for item in collection {
            let item_context = context.with_input(item.clone());
            if let FhirPathValue::Boolean(true) = evaluate_ast(&args[0], &item_context)? {
                result.push(item);
            }
        }

        Ok(FhirPathValue::Collection(result))
    }
}
```

2. **Operator Registry with Precedence**
```rust
// fhirpath-core/src/registry/operator.rs
impl OperatorRegistry {
    pub fn resolve_precedence(&self, ops: &[String]) -> Vec<(String, u8)> {
        ops.iter()
            .filter_map(|op| {
                self.binary_ops.get(op)
                    .map(|handler| (op.clone(), handler.precedence()))
            })
            .collect()
    }

    pub fn evaluate_binary(
        &self,
        op: &str,
        left: &FhirPathValue,
        right: &FhirPathValue
    ) -> Result<FhirPathValue> {
        self.binary_ops
            .get(op)
            .ok_or_else(|| FhirPathError::unknown_operator(op))?
            .evaluate(left, right)
    }
}
```

### Phase 2: Model Provider Implementation

1. **FHIR Schema LoadYou ing**
```rust
// fhirpath-core/src/model/schema.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FhirSchema {
    pub url: String,
    pub version: String,
    pub date: String,
    pub definitions: HashMap<String, TypeDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefinition {
    pub url: String,
    pub base: Option<String>,
    pub kind: TypeKind,
    pub derivation: Option<String>,
    pub elements: HashMap<String, ElementDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDefinition {
    pub type: Vec<TypeReference>,
    pub min: u32,
    pub max: String, // "1", "*", etc.
    pub fixed: Option<Value>,
    pub pattern: Option<Value>,
    pub binding: Option<Binding>,
}

impl FhirSchemaModelProvider {
    pub async fn from_url(url: &str) -> Result<Self> {
        let schema_data = fetch_schema(url).await?;
        let schema: FhirSchema = serde_json::from_str(&schema_data)?;

        Ok(Self {
            schema,
            version: detect_fhir_version(&schema),
            type_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn resolve_type(&self, type_name: &str) -> Option<&TypeDefinition> {
        self.schema.definitions.get(type_name)
    }

    pub fn get_element_definition(
        &self,
        type_name: &str,
        path: &str
    ) -> Option<&ElementDefinition> {
        self.resolve_type(type_name)?
            .elements
            .get(path)
    }
}
```

2. **Type System Integration**
```rust
// fhirpath-core/src/types/system.rs
pub struct TypeSystem {
    model_provider: Arc<dyn ModelProvider>,
    type_cache: Arc<RwLock<HashMap<String, TypeInfo>>>,
}

impl TypeSystem {
    pub fn check_compatibility(
        &self,
        value: &FhirPathValue,
        expected_type: &TypeInfo
    ) -> Result<bool> {
        match (value, expected_type) {
            (FhirPathValue::String(_), TypeInfo::String) => Ok(true),
            (FhirPathValue::Integer(_), TypeInfo::Integer) => Ok(true),
            (FhirPathValue::Resource(res), TypeInfo::Resource(type_name)) => {
                Ok(self.is_type_compatible(res.resource_type(), type_name))
            }
            _ => Ok(false),
        }
    }

    fn is_type_compatible(&self, actual: Option<&str>, expected: &str) -> bool {
        if let Some(actual_type) = actual {
            if actual_type == expected {
                return true;
            }

            // Check inheritance chain
            self.model_provider
                .get_type_info(actual_type)
                .map(|info| info.is_subtype_of(expected))
                .unwrap_or(false)
        } else {
            false
        }
    }
}
```

### Phase 3: Diagnostic System

1. **Diagnostic Builder Pattern**
```rust
// fhirpath-core/src/diagnostics/builder.rs
pub struct DiagnosticBuilder {
    severity: Severity,
    code: DiagnosticCode,
    message: String,
    location: Option<SourceLocation>,
    suggestions: Vec<Suggestion>,
    related: Vec<RelatedInformation>,
}

impl DiagnosticBuilder {
    pub fn error(code: DiagnosticCode) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: String::new(),
            location: None,
            suggestions: Vec::new(),
            related: Vec::new(),
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_location(mut self, start: usize, end: usize) -> Self {
        self.location = Some(SourceLocation {
            start: Position { line: 0, column: start },
            end: Position { line: 0, column: end },
            source_text: None,
        });
        self
    }

    pub fn suggest(mut self, message: impl Into<String>, replacement: Option<String>) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            replacement,
            location: self.location.clone().unwrap_or_default(),
        });
        self
    }

    pub fn build(self) -> Diagnostic {
        Diagnostic {
            severity: self.severity,
            code: self.code,
            message: self.message,
            location: self.location.unwrap_or_default(),
            suggestions: self.suggestions,
            related: self.related,
        }
    }
}

// Usage example
let diagnostic = DiagnosticBuilder::error(DiagnosticCode::UnknownFunction)
    .with_message("Unknown function 'wher'")
    .with_location(10, 14)
    .suggest("Did you mean 'where'?", Some("where".to_string()))
    .build();
```

2. **Parser Integration with Diagnostics**
```rust
// fhirpath-core/src/parser/diagnostic.rs
pub struct DiagnosticParser<'a> {
    input: &'a str,
    position: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> DiagnosticParser<'a> {
    pub fn parse_with_diagnostics(input: &str) -> (Option<ExpressionNode>, Vec<Diagnostic>) {
        let mut parser = Self {
            input,
            position: 0,
            diagnostics: Vec::new(),
        };

        let result = parser.parse_expression();
        (result, parser.diagnostics)
    }

    fn report_error(&mut self, code: DiagnosticCode, message: &str) {
        let diagnostic = DiagnosticBuilder::error(code)
            .with_message(message)
            .with_location(self.position, self.position + 1)
            .build();

        self.diagnostics.push(diagnostic);
    }
}
```

### Phase 4: Performance Optimizations

1. **Expression Caching with Statistics**
```rust
// fhirpath-core/src/engine/cache.rs
pub struct ExpressionCache {
    cache: LruCache<String, Arc<CompiledExpression>>,
    stats: CacheStatistics,
}

#[derive(Debug, Default)]
pub struct CacheStatistics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl ExpressionCache {
    pub fn get_or_compile(&mut self, expr: &str) -> Result<Arc<CompiledExpression>> {
        if let Some(compiled) = self.cache.get(expr) {
            self.stats.hits.fetch_add(1, Ordering::Relaxed);
            Ok(compiled.clone())
        } else {
            self.stats.misses.fetch_add(1, Ordering::Relaxed);
            let compiled = Arc::new(compile_expression(expr)?);

            if self.cache.len() >= self.cache.cap() {
                self.stats.evictions.fetch_add(1, Ordering::Relaxed);
            }

            self.cache.put(expr.to_string(), compiled.clone());
            Ok(compiled)
        }
    }
}
```

2. **Lazy Evaluation for Collections**
```rust
// fhirpath-core/src/model/lazy.rs
pub enum LazyCollection {
    Materialized(Vec<FhirPathValue>),
    Lazy(Box<dyn Iterator<Item = FhirPathValue> + Send>),
}

impl LazyCollection {
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&FhirPathValue) -> bool + Send + 'static
    {
        match self {
            Self::Materialized(vec) => {
                Self::Lazy(Box::new(vec.into_iter().filter(predicate)))
            }
            Self::Lazy(iter) => {
                Self::Lazy(Box::new(iter.filter(predicate)))
            }
        }
    }

    pub fn take(self, n: usize) -> Self {
        match self {
            Self::Materialized(vec) => {
                Self::Materialized(vec.into_iter().take(n).collect())
            }
            Self::Lazy(iter) => {
                Self::Lazy(Box::new(iter.take(n)))
            }
        }
    }

    pub fn materialize(self) -> Vec<FhirPathValue> {
        match self {
            Self::Materialized(vec) => vec,
            Self::Lazy(iter) => iter.collect(),
        }
    }
}
```

### Phase 5: Testing Infrastructure

1. **Test Runner for atomic-ehr Tests**
```rust
// fhirpath-core/tests/runner.rs
pub struct TestRunner {
    engine: FhirPathEngine,
    results: TestResults,
}

#[derive(Debug, Default)]
pub struct TestResults {
    passed: usize,
    failed: usize,
    skipped: usize,
    errors: Vec<TestError>,
}

impl TestRunner {
    pub fn run_test_file(&mut self, path: &Path) -> Result<()> {
        let test_data = fs::read_to_string(path)?;
        let test_suite: TestSuite = serde_json::from_str(&test_data)?;

        for test in test_suite.tests {
            match self.run_single_test(&test) {
                TestResult::Pass => self.results.passed += 1,
                TestResult::Fail(err) => {
                    self.results.failed += 1;
                    self.results.errors.push(err);
                }
                TestResult::Skip => self.results.skipped += 1,
            }
        }

        Ok(())
    }

    fn run_single_test(&mut self, test: &Test) -> TestResult {
        if test.tags.contains(&"skip".to_string()) {
            return TestResult::Skip;
        }

        match self.engine.evaluate(&test.expression, test.input.clone()) {
            Ok(result) => {
                if result == test.expected {
                    TestResult::Pass
                } else {
                    TestResult::Fail(TestError {
                        test_name: test.name.clone(),
                        expected: test.expected.clone(),
                        actual: result,
                        message: "Result mismatch".to_string(),
                    })
                }
            }
            Err(e) => TestResult::Fail(TestError {
                test_name: test.name.clone(),
                expected: test.expected.clone(),
                actual: FhirPathValue::Empty,
                message: format!("Evaluation error: {}", e),
            }),
        }
    }
}
```

### Integration Example

```rust
// Example of using the complete system
use fhirpath_core::{FhirPathEngine, FhirSchemaModelProvider};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize model provider with R5 schema
    let model_provider = FhirSchemaModelProvider::from_url(
        "https://fhir-schema.github.io/fhir-schema/r5/fhir.schema.json"
    ).await?;

    // Create engine with model provider
    let mut engine = FhirPathEngine::with_model_provider(
        Arc::new(model_provider)
    );

    // Load patient data
    let patient_data = serde_json::from_str(r#"{
        "resourceType": "Patient",
        "name": [{
            "given": ["John"],
            "family": "Doe"
        }]
    }"#)?;

    // Evaluate expression with diagnostics
    let (result, diagnostics) = engine.evaluate_with_diagnostics(
        "name.given.first() + ' ' + name.family",
        patient_data
    )?;

    // Handle any warnings
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic.to_human_readable());
    }

    println!("Result: {}", result);
    Ok(())
}
```
