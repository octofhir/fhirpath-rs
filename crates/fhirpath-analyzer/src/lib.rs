//! # FHIRPath Static Analysis Engine
//!
//! This crate provides comprehensive static analysis capabilities for FHIRPath expressions,
//! including type checking, semantic validation, performance optimization analysis,
//! and advanced diagnostic reporting.
//!
//! ## Key Features
//!
//! - **Multi-phase analysis**: Lexical, semantic, property resolution, function validation
//! - **Rich diagnostics**: Error recovery, suggestions, multiple output formats
//! - **Type system**: Full FHIR type awareness and validation
//! - **Performance optimization**: Analysis of expression efficiency
//! - **Integration ready**: Designed for IDE, CLI, and server integration
//!
//! ## Architecture
//!
//! The analyzer follows a modular, multi-phase approach:
//!
//! 1. **Lexical Analysis**: Token-level validation and basic syntax checking
//! 2. **Semantic Analysis**: Context-aware validation and type inference
//! 3. **Property Resolution**: FHIR property and path validation
//! 4. **Function Validation**: Function signature and usage validation
//! 5. **Optimization Analysis**: Performance and efficiency recommendations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use octofhir_fhirpath_analyzer::core::analyzer_engine::FhirPathAnalyzer;
//! use octofhir_fhirpath_analyzer::providers::MockFhirProvider;
//! use std::sync::Arc;
//!
//! async fn analyze_expression() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = Arc::new(MockFhirProvider::new());
//!     let mut analyzer = FhirPathAnalyzer::new(provider);
//!     
//!     let result = analyzer.analyze("Patient.name.family").await?;
//!     
//!     if result.has_errors() {
//!         for diagnostic in result.diagnostics() {
//!             println!("Error: {}", diagnostic);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod phases;
pub mod diagnostics;
pub mod validators;
pub mod providers;

// Re-export commonly used types for convenience
pub use core::{
    analyzer_engine::{
        FhirPathAnalyzer, AnalysisResult, AnalyzerConfig, NodeId,
        SymbolResolution, PropertyResolution, FunctionResolution, VariableResolution,
        OptimizationHint, OptimizationHintType, CompletionItem, CompletionKind,
        HoverInfo, AnalysisMetadata, MemoryStats
    },
    context::{
        AnalysisContext, ScopeInfo, LambdaContext, PathSegment, AnalysisPhase, IterationType
    },
    symbol_table::{
        SymbolTable, SymbolTableError, Scope, ScopeType, VariableBinding, FunctionBinding
    },
    type_system::{
        FhirType, PrimitiveType, ResourceType, BackboneElementType, 
        TypeInformation, TypeSystem, Cardinality, TypeConstraint, ConstraintType, ConstraintSeverity
    },
    error::AnalysisError,
};

pub use providers::{
    fhir_provider::{FhirProvider, PropertyInfo, FhirProviderError},
    function_provider::{FunctionProvider, FunctionSignature, FunctionProviderError},
    cache_provider::CacheProvider,
};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");