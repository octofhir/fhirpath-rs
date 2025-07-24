//! Registry system for FHIRPath functions and operators
//!
//! This module provides a trait-based extensible system for registering
//! and managing FHIRPath functions and operators.

pub mod function;
pub mod operator;

pub use function::{FhirPathFunction, FunctionRegistry, FunctionSignature};
pub use operator::{FhirPathOperator, OperatorRegistry, Associativity};

// Re-export TypeInfo from fhirpath-model crate
pub use fhirpath_model::TypeInfo;

/// Create a standard registry with all built-in functions and operators
pub fn create_standard_registry() -> (FunctionRegistry, OperatorRegistry) {
    let mut functions = FunctionRegistry::new();
    let mut operators = OperatorRegistry::new();
    
    // Register built-in functions
    function::register_builtin_functions(&mut functions);
    
    // Register built-in operators
    operator::register_builtin_operators(&mut operators);
    
    (functions, operators)
}