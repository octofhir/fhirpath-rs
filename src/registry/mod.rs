//! Function and operator registry for FHIRPath
//!
//! This crate provides trait-based registries for FHIRPath functions and operators,
//! allowing for extensible and type-safe evaluation.

#![warn(missing_docs)]

pub mod cache;
pub mod compiled_signatures;
pub mod extension;
pub mod fast_path;
pub mod function;
pub mod functions;
pub mod operator;
pub mod operators;
pub mod signature;

pub use compiled_signatures::{
    CompilationStats, CompiledSignature, CompiledSignatureRegistry, SpecializedSignature,
};
pub use fast_path::{FastPathFunction, FastPathRegistry};
pub use function::{FhirPathFunction, FunctionRegistry};
pub use operator::{Associativity, FhirPathOperator, OperatorRegistry};
pub use signature::{FunctionSignature, OperatorSignature};

/// Create a standard registry with all built-in functions and operators
pub fn create_standard_registries() -> (FunctionRegistry, OperatorRegistry) {
    let mut functions = FunctionRegistry::new();
    let mut operators = OperatorRegistry::new();

    // Register built-in functions
    function::register_builtin_functions(&mut functions);

    // Register built-in operators
    operator::register_builtin_operators(&mut operators);

    (functions, operators)
}
