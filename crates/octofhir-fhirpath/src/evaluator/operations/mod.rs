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

// Re-export operator evaluators explicitly
pub use add_operator::AddOperatorEvaluator;
pub use and_operator::AndOperatorEvaluator;
pub use concatenate_operator::ConcatenateOperatorEvaluator;
pub use contains_operator::ContainsOperatorEvaluator;
pub use divide_operator::DivideOperatorEvaluator;
pub use equals_operator::EqualsOperatorEvaluator;
pub use equivalent_operator::EquivalentOperatorEvaluator;
pub use greater_equal_operator::GreaterEqualOperatorEvaluator;
pub use greater_than_operator::GreaterThanOperatorEvaluator;
pub use implies_operator::ImpliesOperatorEvaluator;
pub use in_operator::InOperatorEvaluator;
pub use integer_divide_operator::IntegerDivideOperatorEvaluator;
pub use less_equal_operator::LessEqualOperatorEvaluator;
pub use less_than_operator::LessThanOperatorEvaluator;
pub use modulo_operator::ModuloOperatorEvaluator;
pub use multiply_operator::MultiplyOperatorEvaluator;
pub use negate_operator::NegateOperatorEvaluator;
pub use not_equals_operator::NotEqualsOperatorEvaluator;
pub use not_equivalent_operator::NotEquivalentOperatorEvaluator;
pub use or_operator::OrOperatorEvaluator;
pub use subtract_operator::SubtractOperatorEvaluator;
pub use type_operators::{AsOperatorEvaluator, IsOperatorEvaluator};
pub use union_operator::UnionOperatorEvaluator;
pub use xor_operator::XorOperatorEvaluator;
