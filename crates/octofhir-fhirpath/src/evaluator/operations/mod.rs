//! FHIRPath operation implementations
//!
//! This module contains individual operator implementations that are registered
//! with the OperatorRegistry for evaluation.

// Logical operators
pub mod and_operator;
pub mod implies_operator;
pub mod or_operator;
pub mod xor_operator;

// Comparison operators
pub mod equals_operator;
pub mod equivalent_operator;
pub mod greater_equal_operator;
pub mod greater_than_operator;
pub mod less_equal_operator;
pub mod less_than_operator;
pub mod not_equals_operator;
pub mod not_equivalent_operator;

// Arithmetic operators
pub mod add_operator;
pub mod divide_operator;
pub mod integer_divide_operator;
pub mod modulo_operator;
pub mod multiply_operator;
pub mod subtract_operator;

// Unary operators
pub mod negate_operator;

// Collection operators
pub mod contains_operator;
pub mod in_operator;
pub mod union_operator;

// String operators
pub mod concatenate_operator;

// Type operators
pub mod type_operators;

// Re-export all operators for convenience
pub use add_operator::*;
pub use and_operator::*;
pub use concatenate_operator::*;
pub use contains_operator::*;
pub use divide_operator::*;
pub use equals_operator::*;
pub use equivalent_operator::*;
pub use greater_equal_operator::*;
pub use greater_than_operator::*;
pub use implies_operator::*;
pub use in_operator::*;
pub use integer_divide_operator::*;
pub use less_equal_operator::*;
pub use less_than_operator::*;
pub use modulo_operator::*;
pub use multiply_operator::*;
pub use negate_operator::*;
pub use not_equals_operator::*;
pub use not_equivalent_operator::*;
pub use or_operator::*;
pub use subtract_operator::*;
pub use type_operators::*;
pub use union_operator::*;
pub use xor_operator::*;
