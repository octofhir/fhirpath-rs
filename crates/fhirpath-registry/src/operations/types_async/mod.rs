//! Type operations - async implementations
//!
//! These are type operations that work with the FunctionRegistry system.

pub mod as_op;
pub mod is;
pub mod of_type;
pub mod type_func;

pub use as_op::AsOperation;
pub use is::IsOperation;
pub use of_type::OfTypeFunction;
pub use type_func::TypeFunction;
