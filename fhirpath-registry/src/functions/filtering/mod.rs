//! Filtering and selection functions for FHIRPath expressions

mod of_type;
mod select;
mod skip;
mod take;
mod r#where;

pub use of_type::OfTypeFunction;
pub use r#where::WhereFunction;
pub use select::SelectFunction;
pub use skip::SkipFunction;
pub use take::TakeFunction;

use crate::function::FunctionRegistry;

/// Register all filtering functions
pub fn register_filtering_functions(registry: &mut FunctionRegistry) {
    registry.register(OfTypeFunction);
    registry.register(SelectFunction);
    registry.register(SkipFunction);
    registry.register(TakeFunction);
    registry.register(WhereFunction);
}