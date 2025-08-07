//! Filtering and selection functions for FHIRPath expressions

mod of_type;
mod select;
mod skip;
mod take;
mod r#where;

pub use of_type::OfTypeFunction;
pub use select::SelectFunction;
pub use skip::SkipFunction;
pub use take::TakeFunction;
pub use r#where::WhereFunction;

use crate::registry::function::FunctionRegistry;

/// Register all filtering functions
pub fn register_filtering_functions(registry: &mut FunctionRegistry) {
    registry.register_async(OfTypeFunction);
    registry.register(SelectFunction);
    registry.register_async(SkipFunction);
    registry.register_async(TakeFunction);
    registry.register(WhereFunction);
}
