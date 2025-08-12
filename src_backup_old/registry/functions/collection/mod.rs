// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    // Lambda functions (still using old trait)
    registry.register(AggregateFunction);
    registry.register(ExistsFunction);
    registry.register(SortFunction);

    // Async collection functions
    registry.register_async(ChildrenFunction);
    registry.register_async(CombineFunction);
    registry.register_async(CountFunction);
    registry.register_async(DescendantsFunction);
    registry.register_async(DistinctFunction);
    registry.register_async(EmptyFunction);
    registry.register_async(ExcludeFunction);
    registry.register_async(FirstFunction);
    registry.register_async(IntersectFunction);
    registry.register_async(LastFunction);
    registry.register_async(LengthFunction);
    registry.register_async(SingleFunction);
    registry.register_async(SubsetOfFunction);
    registry.register_async(SupersetOfFunction);
    registry.register_async(TailFunction);
}
