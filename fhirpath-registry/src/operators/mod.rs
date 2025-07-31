//! Operator implementations for FHIRPath expressions

mod arithmetic;
mod comparison;
mod logical;
mod string;
mod collection;

// Re-export all operators
pub use arithmetic::*;
pub use comparison::*;
pub use logical::*;
pub use string::*;
pub use collection::*;

use crate::operator::OperatorRegistry;

/// Register all built-in operators
pub fn register_builtin_operators(registry: &mut OperatorRegistry) {
    // Arithmetic operators
    arithmetic::register_arithmetic_operators(registry);
    
    // Comparison operators
    comparison::register_comparison_operators(registry);
    
    // Logical operators
    logical::register_logical_operators(registry);
    
    // String operators
    string::register_string_operators(registry);
    
    // Collection operators
    collection::register_collection_operators(registry);
}