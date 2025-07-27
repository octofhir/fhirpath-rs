//! # FHIRPath Core Library
//!
//! A high-performance FHIRPath engine implementation in Rust.
//!
//! This library provides a complete implementation of the FHIRPath v2.0.0 specification
//! with a focus on performance, safety, and specification compliance.

// Re-export from component crates
pub use fhirpath_ast as ast;
pub use fhirpath_evaluator as evaluator;
pub use fhirpath_registry as registry;

pub mod engine;
pub mod error;
pub mod value_ext;
pub mod types;

