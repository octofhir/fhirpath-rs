//! Type checking functions
//!
//! Functions for runtime type checking and type operations.

pub mod is_function;
pub mod type_function;

pub use is_function::IsFunctionEvaluator;
pub use type_function::TypeFunctionEvaluator;