//! Utility functions for FHIRPath expressions

mod conforms_to;
mod define_variable;
mod has_value;
mod iif;
mod repeat;
mod trace;

pub use conforms_to::ConformsToFunction;
pub use define_variable::DefineVariableFunction;
pub use has_value::HasValueFunction;
pub use iif::IifFunction;
pub use repeat::RepeatFunction;
pub use trace::TraceFunction;

use crate::registry::function::FunctionRegistry;

/// Register all utility functions
pub fn register_utility_functions(registry: &mut FunctionRegistry) {
    registry.register_async(ConformsToFunction::new());
    registry.register_async(DefineVariableFunction);
    registry.register_async(HasValueFunction);
    registry.register_async(IifFunction);
    registry.register_async(RepeatFunction);
    registry.register_async(TraceFunction);
}
