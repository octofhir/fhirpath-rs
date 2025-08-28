# FHIRPath Analyzer Integration Guide

This guide shows how to integrate the FHIRPath analyzer into different contexts and use cases.

## Table of Contents

1. [Core Integration](#core-integration)
2. [Engine Integration](#engine-integration)
3. [IDE/LSP Integration](#idelsip-integration)
4. [Testing Integration](#testing-integration)
5. [Performance Considerations](#performance-considerations)
6. [Migration Guide](#migration-guide)

## Core Integration

### Basic Analyzer Setup

```rust
use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, AnalyzerConfig};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

// Create analyzer with default configuration
let provider = Arc::new(MockModelProvider::new());
let analyzer = FhirPathAnalyzer::new(provider);

// Or with custom configuration
let config = AnalyzerConfig {
    settings: AnalysisSettings {
        enable_type_inference: true,
        enable_function_validation: true,
        enable_union_analysis: true,
        max_analysis_depth: 50,
    },
    cache_size: 5000,
    enable_profiling: false,
};

let analyzer = FhirPathAnalyzer::with_config(provider, config);
```

### Function Registry Integration

```rust
use octofhir_fhirpath_registry::create_standard_registry;

// Create analyzer with function registry for full validation
let provider = Arc::new(MockModelProvider::new());
let registry = Arc::new(create_standard_registry().await?);
let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);

// Analyze expressions with function validation
let result = analyzer.analyze("count()").await?;
for error in result.validation_errors {
    println!("Validation error: {}", error.message);
    if !error.suggestions.is_empty() {
        println!("Suggestions: {}", error.suggestions.join(", "));
    }
}
```

## Engine Integration

### FHIRPath Engine with Analysis

The analyzer integrates seamlessly with the existing FHIRPath engine:

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use octofhir_fhirpath_analyzer::FhirPathAnalyzer;
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Shared model provider
    let provider = Arc::new(MockModelProvider::new());
    
    // Create both engine and analyzer
    let mut engine = FhirPathEngine::with_model_provider(
        Box::new(MockModelProvider::new())
    );
    let analyzer = FhirPathAnalyzer::new(provider);
    
    let expression = "Patient.name.given";
    let patient = json!({
        "resourceType": "Patient", 
        "name": [{"given": ["John", "Doe"]}]
    });
    
    // 1. Analyze expression first
    let analysis = analyzer.analyze(expression).await?;
    
    // Check for validation errors before evaluation
    if !analysis.validation_errors.is_empty() {
        println!("Expression has validation errors:");
        for error in analysis.validation_errors {
            println!("  - {}", error.message);
        }
        return Ok(());
    }
    
    // 2. Evaluate if analysis passes
    let result = engine.evaluate(expression, patient).await?;
    println!("Result: {:?}", result);
    
    // 3. Use type information from analysis
    for (node_id, semantic_info) in analysis.type_annotations {
        if let Some(fhir_type) = semantic_info.fhir_path_type {
            println!("Node {} has FHIRPath type: {}", node_id, fhir_type);
        }
    }
    
    Ok(())
}
```

### Pre-validation Wrapper

Create a wrapper that validates before evaluation:

```rust
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};
use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationError};
use serde_json::Value;
use std::sync::Arc;

pub struct ValidatingFhirPathEngine {
    engine: FhirPathEngine,
    analyzer: FhirPathAnalyzer,
}

impl ValidatingFhirPathEngine {
    pub async fn new(
        provider: Arc<dyn octofhir_fhirpath_model::ModelProvider>
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let engine_provider = provider.clone();
        let analyzer_provider = provider;
        
        let engine = FhirPathEngine::with_model_provider(
            // Clone provider for engine (requires Box)
            Box::new(octofhir_fhirpath_model::mock_provider::MockModelProvider::new())
        );
        let analyzer = FhirPathAnalyzer::new(analyzer_provider);
        
        Ok(Self { engine, analyzer })
    }
    
    pub async fn validate_and_evaluate(
        &mut self,
        expression: &str,
        data: Value,
    ) -> Result<Vec<FhirPathValue>, ValidatingEngineError> {
        // 1. Validate expression first
        let analysis = self.analyzer.analyze(expression).await
            .map_err(ValidatingEngineError::AnalysisError)?;
        
        if !analysis.validation_errors.is_empty() {
            return Err(ValidatingEngineError::ValidationErrors(analysis.validation_errors));
        }
        
        // 2. Evaluate if validation passes
        let result = self.engine.evaluate(expression, data).await
            .map_err(ValidatingEngineError::EvaluationError)?;
        
        Ok(result)
    }
}

#[derive(Debug)]
pub enum ValidatingEngineError {
    AnalysisError(octofhir_fhirpath_analyzer::AnalysisError),
    ValidationErrors(Vec<ValidationError>),
    EvaluationError(Box<dyn std::error::Error>),
}
```

## IDE/LSP Integration

### Language Server Protocol Support

```rust
use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use tower_lsp::{jsonrpc::Result, lsp_types::*, Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct FhirPathLanguageServer {
    client: Client,
    analyzer: FhirPathAnalyzer,
}

impl FhirPathLanguageServer {
    async fn validate_document(&self, uri: &Url, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        // Analyze the FHIRPath expression
        match self.analyzer.analyze(text).await {
            Ok(result) => {
                // Convert validation errors to LSP diagnostics
                for error in result.validation_errors {
                    let severity = match error.error_type {
                        ValidationErrorType::InvalidFunction => DiagnosticSeverity::ERROR,
                        ValidationErrorType::TypeMismatch => DiagnosticSeverity::WARNING,
                        ValidationErrorType::InvalidExpression => DiagnosticSeverity::ERROR,
                        ValidationErrorType::InvalidTypeOperation => DiagnosticSeverity::WARNING,
                        ValidationErrorType::FunctionSignature => DiagnosticSeverity::ERROR,
                    };
                    
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: 0, character: error.position as u32 },
                            end: Position { line: 0, character: (error.position + error.length) as u32 },
                        },
                        severity: Some(severity),
                        source: Some("fhirpath-analyzer".to_string()),
                        message: error.message,
                        code: Some(NumberOrString::String(format!("{:?}", error.error_type))),
                        ..Default::default()
                    });
                }
            }
            Err(e) => {
                // Parse error
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: text.len() as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("fhirpath-analyzer".to_string()),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                });
            }
        }
        
        diagnostics
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for FhirPathLanguageServer {
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = &params.content_changes[0].text;
        
        let diagnostics = self.validate_document(&uri, text).await;
        
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        
        // Use analyzer to provide context-aware completions
        // Implementation would analyze partial expressions and suggest functions/properties
        
        Ok(None) // Simplified for example
    }
}
```

### Hover Information

```rust
impl LanguageServer for FhirPathLanguageServer {
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let text = get_document_text(uri).await?; // Implementation specific
        
        match self.analyzer.analyze(&text).await {
            Ok(result) => {
                // Find hover information based on cursor position
                for (node_id, semantic_info) in result.type_annotations {
                    // Check if cursor position intersects with this node
                    // Implementation would map positions to AST nodes
                    
                    let mut contents = Vec::new();
                    
                    if let Some(fhir_type) = semantic_info.fhir_path_type {
                        contents.push(MarkedString::String(format!("**Type**: {}", fhir_type)));
                    }
                    
                    if let Some(model_type) = semantic_info.model_type {
                        contents.push(MarkedString::String(format!("**Model Type**: {}", model_type)));
                    }
                    
                    contents.push(MarkedString::String(format!("**Cardinality**: {:?}", semantic_info.cardinality)));
                    contents.push(MarkedString::String(format!("**Confidence**: {:?}", semantic_info.confidence)));
                    
                    return Ok(Some(Hover {
                        contents: HoverContents::Array(contents),
                        range: None,
                    }));
                }
                
                Ok(None)
            }
            Err(_) => Ok(None),
        }
    }
}
```

## Testing Integration

### Unit Test Integration

```rust
use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
use octofhir_fhirpath_model::mock_provider::MockModelProvider;
use std::sync::Arc;

#[tokio::test]
async fn test_expression_validation() {
    let provider = Arc::new(MockModelProvider::new());
    let analyzer = FhirPathAnalyzer::new(provider);
    
    // Test valid expression
    let result = analyzer.analyze("Patient.name.given").await.unwrap();
    assert!(result.validation_errors.is_empty(), "Valid expression should not have errors");
    assert!(!result.type_annotations.is_empty(), "Should have type annotations");
    
    // Test invalid expression
    let result = analyzer.analyze("Patient.nonexistentField").await.unwrap();
    // Assertions based on expected validation behavior
}

#[tokio::test]
async fn test_function_validation() {
    let provider = Arc::new(MockModelProvider::new());
    let registry = Arc::new(octofhir_fhirpath_registry::create_standard_registry().await.unwrap());
    let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);
    
    // Test valid function
    let result = analyzer.analyze("count()").await.unwrap();
    assert!(result.validation_errors.is_empty());
    assert!(!result.function_calls.is_empty());
    
    // Test invalid function
    let result = analyzer.analyze("invalidFunction()").await.unwrap();
    assert!(!result.validation_errors.is_empty());
    assert!(result.validation_errors.iter().any(|e| 
        matches!(e.error_type, ValidationErrorType::InvalidFunction)
    ));
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    async fn test_analyzer_never_panics(expression in "\\PC{0,100}") {
        let provider = Arc::new(MockModelProvider::new());
        let analyzer = FhirPathAnalyzer::new(provider);
        
        // Analyzer should never panic, even on invalid input
        let result = analyzer.analyze(&expression).await;
        
        // Either succeeds with analysis or fails gracefully
        match result {
            Ok(_) => {}, // Success is fine
            Err(_) => {}, // Graceful error is fine
        }
    }
}
```

## Performance Considerations

### Caching Strategy

```rust
use octofhir_fhirpath_analyzer::{AnalyzerConfig, AnalysisSettings};

// Configure for high-throughput scenarios
let config = AnalyzerConfig {
    settings: AnalysisSettings {
        enable_type_inference: true,
        enable_function_validation: true,
        enable_union_analysis: false, // Disable expensive features if not needed
        max_analysis_depth: 30, // Lower depth for performance
    },
    cache_size: 10000, // Large cache for repeated expressions
    enable_profiling: false, // Disable in production
};

let analyzer = FhirPathAnalyzer::with_config(provider, config);
```

### Concurrent Usage

```rust
use std::sync::Arc;
use tokio::task::JoinSet;

async fn analyze_expressions_concurrently(
    analyzer: Arc<FhirPathAnalyzer>,
    expressions: Vec<String>
) -> Vec<Result<octofhir_fhirpath_analyzer::AnalysisResult, octofhir_fhirpath_analyzer::AnalysisError>> {
    let mut set = JoinSet::new();
    
    for expression in expressions {
        let analyzer = analyzer.clone();
        set.spawn(async move {
            analyzer.analyze(&expression).await
        });
    }
    
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }
    
    results
}
```

### Memory Management

```rust
// Periodically clear caches in long-running applications
use std::time::{Duration, Instant};

struct ManagedAnalyzer {
    analyzer: FhirPathAnalyzer,
    last_cache_clear: Instant,
}

impl ManagedAnalyzer {
    pub async fn analyze(&mut self, expression: &str) -> Result<AnalysisResult, AnalysisError> {
        // Clear cache every hour to prevent memory bloat
        if self.last_cache_clear.elapsed() > Duration::from_secs(3600) {
            self.analyzer.clear_cache().await;
            self.last_cache_clear = Instant::now();
        }
        
        self.analyzer.analyze(expression).await
    }
}
```

## Migration Guide

### From Direct Parser Usage

**Before:**
```rust
use octofhir_fhirpath_parser::parse;

let ast = parse("Patient.name")?;
// Manual AST walking and analysis
```

**After:**
```rust
use octofhir_fhirpath_analyzer::FhirPathAnalyzer;

let analyzer = FhirPathAnalyzer::new(provider);
let result = analyzer.analyze("Patient.name").await?;
// Rich semantic information available
```

### Adding Analysis to Existing Engine

**Before:**
```rust
let mut engine = FhirPathEngine::new();
let result = engine.evaluate(expression, data).await?;
```

**After:**
```rust
let provider = Arc::new(MockModelProvider::new());
let analyzer = FhirPathAnalyzer::new(provider.clone());

// Optional pre-validation
let analysis = analyzer.analyze(expression).await?;
if !analysis.validation_errors.is_empty() {
    // Handle validation errors
    return Err("Expression has validation errors".into());
}

let mut engine = FhirPathEngine::with_model_provider(
    Box::new(MockModelProvider::new())
);
let result = engine.evaluate(expression, data).await?;
```

## Best Practices

### Error Handling

```rust
match analyzer.analyze(expression).await {
    Ok(result) => {
        // Always check validation errors
        for error in result.validation_errors {
            match error.error_type {
                ValidationErrorType::InvalidFunction => {
                    // Handle function errors with suggestions
                    eprintln!("Function error: {}", error.message);
                    if !error.suggestions.is_empty() {
                        eprintln!("Try: {}", error.suggestions.join(", "));
                    }
                }
                ValidationErrorType::TypeMismatch => {
                    // Handle type errors
                    eprintln!("Type error: {}", error.message);
                }
                _ => eprintln!("Other error: {}", error.message),
            }
        }
    }
    Err(e) => {
        // Handle parse/analysis errors
        eprintln!("Analysis failed: {}", e);
    }
}
```

### Configuration Tuning

```rust
// Development configuration - comprehensive analysis
let dev_config = AnalyzerConfig {
    settings: AnalysisSettings {
        enable_type_inference: true,
        enable_function_validation: true,
        enable_union_analysis: true,
        max_analysis_depth: 100,
    },
    cache_size: 1000,
    enable_profiling: true,
};

// Production configuration - performance optimized
let prod_config = AnalyzerConfig {
    settings: AnalysisSettings {
        enable_type_inference: true,
        enable_function_validation: true,
        enable_union_analysis: false, // Disable expensive features
        max_analysis_depth: 50,
    },
    cache_size: 10000,
    enable_profiling: false,
};
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**: Reduce cache size or clear caches periodically
2. **Slow Analysis**: Disable union analysis or reduce max analysis depth
3. **Missing Function Validation**: Ensure function registry is provided
4. **Incomplete Type Information**: Check ModelProvider implementation

### Debug Configuration

```rust
let debug_config = AnalyzerConfig {
    settings: AnalysisSettings::default(),
    cache_size: 100, // Small cache for debugging
    enable_profiling: true, // Enable timing information
};
```

For more examples and detailed usage, see the `examples/` directory in the analyzer crate.