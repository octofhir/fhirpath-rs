//! Operator implementations for FHIRPath expressions

pub mod arithmetic;
mod collection;
mod comparison;
mod logical;
mod string;

// Re-export all operators
pub use arithmetic::*;
pub use collection::*;
pub use comparison::*;
pub use logical::*;
pub use string::*;

use crate::registry::operator::OperatorRegistry;

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
