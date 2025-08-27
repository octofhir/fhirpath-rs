//! # FHIRPath Static Analysis and Type-Enriched AST Engine
//!
//! This crate provides comprehensive static analysis capabilities for FHIRPath expressions
//! with a focus on specification compliance and rich functionality.
//!
//! ## Core Components
//!
//! - [`FhirPathAnalyzer`] - Main analysis engine
//! - [`AnalysisResult`] - Rich analysis information  
//! - [`SemanticInfo`] - Type and semantic metadata
//! - [`ValidationError`] - Detailed error information
//!
//! ## Quick Start
//!
//! ```rust
//! use octofhir_fhirpath_analyzer::{FhirPathAnalyzer};
//! use octofhir_fhirpath_model::mock_provider::MockModelProvider;
//! use std::sync::Arc;
//!
//! # tokio_test::block_on(async {
//! let provider = Arc::new(MockModelProvider::new());
//! let analyzer = FhirPathAnalyzer::new(provider);
//!
//! let result = analyzer.analyze("Patient.name.given").await?;
//! println!("Found {} type annotations", result.type_annotations.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```
//!
//! ## Advanced Usage
//!
//! ### Function Registry Integration
//!
//! ```rust
//! use octofhir_fhirpath_analyzer::FhirPathAnalyzer;
//! use octofhir_fhirpath_registry::create_standard_registry;
//! use octofhir_fhirpath_model::mock_provider::MockModelProvider;
//! use std::sync::Arc;
//!
//! # tokio_test::block_on(async {
//! let provider = Arc::new(MockModelProvider::new());
//! let registry = Arc::new(create_standard_registry().await);
//! let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);
//!
//! // Function signature validation
//! let result = analyzer.analyze("count()").await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```
//!
//! ### Children() Function Analysis
//!
//! ```rust
//! # use octofhir_fhirpath_analyzer::FhirPathAnalyzer;
//! # use octofhir_fhirpath_model::mock_provider::MockModelProvider;
//! # use std::sync::Arc;
//! # tokio_test::block_on(async {
//! # let provider = Arc::new(MockModelProvider::new());
//! # let analyzer = FhirPathAnalyzer::new(provider);
//! // Union type analysis for children() function
//! let result = analyzer.analyze("Patient.children().ofType(HumanName)").await?;
//!
//! // Check for union type information
//! if !result.union_types.is_empty() {
//!     println!("Found union types from children() analysis");
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```
//!
//! ## Error Handling
//!
//! The analyzer provides detailed validation errors with suggestions:
//!
//! ```rust
//! # use octofhir_fhirpath_analyzer::{FhirPathAnalyzer, ValidationErrorType};
//! # use octofhir_fhirpath_model::mock_provider::MockModelProvider;
//! # use std::sync::Arc;
//! # tokio_test::block_on(async {
//! # let provider = Arc::new(MockModelProvider::new());
//! # let registry = Arc::new(octofhir_fhirpath_registry::create_standard_registry().await);
//! # let analyzer = FhirPathAnalyzer::with_function_registry(provider, registry);
//! let result = analyzer.analyze("unknownFunction()").await?;
//!
//! for error in result.validation_errors {
//!     match error.error_type {
//!         ValidationErrorType::InvalidFunction => {
//!             println!("Function error: {}", error.message);
//!             println!("Suggestions: {:?}", error.suggestions);
//!         }
//!         _ => println!("Other error: {}", error.message),
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```
//!
//! ## Performance Considerations
//!
//! The analyzer is designed for high performance:
//!
//! - **Caching**: Aggressive caching of analysis results
//! - **External Mapping**: No AST modifications for zero overhead when disabled
//! - **Concurrent**: Thread-safe operations with DashMap
//!
//! Typical performance targets:
//! - Analysis: <100μs for basic expressions
//! - Memory: <10% overhead when enabled
//! - Cache hit rate: >90% for repeated expressions

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::result_large_err)]

pub mod analyzer;
pub mod cache;
pub mod children_analyzer;
pub mod config;
pub mod error;
pub mod field_validator;
pub mod function_analyzer;
pub mod model_provider_ext;
pub mod types;

// Re-export main types
pub use analyzer::FhirPathAnalyzer;
pub use cache::{AnalysisCache, ExpressionAnalysisMap};
pub use children_analyzer::ChildrenFunctionAnalyzer;
pub use config::AnalyzerConfig;
pub use error::{AnalysisError, ValidationError, ValidationErrorType};
pub use field_validator::FieldValidator;
pub use function_analyzer::FunctionAnalyzer;
pub use model_provider_ext::ModelProviderChildrenExt;
pub use types::{AnalysisContext, AnalysisResult, AnalysisSettings, SemanticInfo};
