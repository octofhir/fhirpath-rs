//! Boolean logic functions for FHIRPath expressions

mod all;
mod all_true;
mod any;
mod is_distinct;
mod not;

pub use all::AllFunction;
pub use all_true::AllTrueFunction;
pub use any::AnyFunction;
pub use is_distinct::IsDistinctFunction;
pub use not::NotFunction;

use crate::function::FunctionRegistry;

/// Register all boolean functions
pub fn register_boolean_functions(registry: &mut FunctionRegistry) {
    registry.register(AllFunction);
    registry.register(AllTrueFunction);
    registry.register(AnyFunction);
    registry.register(IsDistinctFunction);
    registry.register(NotFunction);
}