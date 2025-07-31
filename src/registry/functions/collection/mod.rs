//! Collection manipulation functions for FHIRPath expressions

mod aggregate;
mod children;
mod combine;
mod count;
mod descendants;
mod distinct;
mod empty;
mod exclude;
mod exists;
mod first;
mod intersect;
mod last;
mod length;
mod single;
mod sort;
mod subset_of;
mod superset_of;
mod tail;

pub use aggregate::AggregateFunction;
pub use children::ChildrenFunction;
pub use combine::CombineFunction;
pub use count::CountFunction;
pub use descendants::DescendantsFunction;
pub use distinct::DistinctFunction;
pub use empty::EmptyFunction;
pub use exclude::ExcludeFunction;
pub use exists::ExistsFunction;
pub use first::FirstFunction;
pub use intersect::IntersectFunction;
pub use last::LastFunction;
pub use length::LengthFunction;
pub use single::SingleFunction;
pub use sort::SortFunction;
pub use subset_of::SubsetOfFunction;
pub use superset_of::SupersetOfFunction;
pub use tail::TailFunction;

use crate::registry::function::FunctionRegistry;

/// Register all collection functions
pub fn register_collection_functions(registry: &mut FunctionRegistry) {
    registry.register(AggregateFunction);
    registry.register(ChildrenFunction);
    registry.register(CombineFunction);
    registry.register(CountFunction);
    registry.register(DescendantsFunction);
    registry.register(DistinctFunction);
    registry.register(EmptyFunction);
    registry.register(ExcludeFunction);
    registry.register(ExistsFunction);
    registry.register(FirstFunction);
    registry.register(IntersectFunction);
    registry.register(LastFunction);
    registry.register(LengthFunction);
    registry.register(SingleFunction);
    registry.register(SortFunction);
    registry.register(SubsetOfFunction);
    registry.register(SupersetOfFunction);
    registry.register(TailFunction);
}
