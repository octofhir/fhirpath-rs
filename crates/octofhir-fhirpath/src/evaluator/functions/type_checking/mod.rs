//! Type checking functions
//!
//! Functions for runtime type checking and type operations.

pub mod as_function;
pub mod conforms_to_function;
pub mod is_function;
pub mod type_function;

pub use as_function::AsFunctionEvaluator;
pub use conforms_to_function::ConformsToFunctionEvaluator;
pub use is_function::IsFunctionEvaluator;
pub use type_function::TypeFunctionEvaluator;
