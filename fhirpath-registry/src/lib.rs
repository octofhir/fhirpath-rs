//! Function and operator registry for FHIRPath
//!
//! This crate provides trait-based registries for FHIRPath functions and operators,
//! allowing for extensible and type-safe evaluation.

#![warn(missing_docs)]

pub mod function;
pub mod functions;
pub mod operator;
pub mod signature;

pub use function::{FhirPathFunction, FunctionRegistry};
pub use operator::{FhirPathOperator, OperatorRegistry, Associativity};
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