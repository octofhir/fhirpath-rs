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

//! Collection functions for FHIRPath

use crate::fhirpath_registry::FhirPathRegistry;
use octofhir_fhirpath_core::Result;

pub mod all;
pub mod all_false;
pub mod all_true;
pub mod any_false;
pub mod any_true;
pub mod combine;
pub mod contains_op;
pub mod count;
pub mod distinct;
pub mod empty;
pub mod exclude;
pub mod exists;
pub mod first;
pub mod intersect;
pub mod is_distinct;
pub mod last;
pub mod skip;
// Sort is handled by lambda functions
pub mod subset_of;
pub mod superset_of;
pub mod tail;
pub mod take;
pub mod union;
pub mod union_operator;
// pub mod in_op;

pub use all::AllFunction;
pub use all_false::AllFalseFunction;
pub use all_true::AllTrueFunction;
pub use any_false::AnyFalseFunction;
pub use any_true::AnyTrueFunction;
pub use combine::CombineFunction;
pub use contains_op::ContainsOperation;
pub use count::CountFunction;
pub use distinct::DistinctFunction;
pub use empty::EmptyFunction;
pub use exclude::ExcludeFunction;
pub use exists::ExistsFunction;
pub use first::FirstFunction;
pub use intersect::IntersectFunction;
pub use is_distinct::IsDistinctFunction;
pub use last::LastFunction;
pub use skip::SkipFunction;
// Sort is handled by lambda functions
pub use subset_of::SubsetOfFunction;
pub use superset_of::SupersetOfFunction;
pub use tail::TailFunction;
pub use take::TakeFunction;
pub use union::UnionFunction;
pub use union_operator::UnionOperator;
// use crate::operations::collection::in_op::InOperation;

/// Utility struct for registering all collection operations
pub struct CollectionOperations;

impl CollectionOperations {
    /// Register all collection operations in the registry
    pub async fn register_all(registry: &FhirPathRegistry) -> Result<()> {
        // Existing collection functions
        registry.register(CountFunction::new()).await?;
        registry.register(DistinctFunction::new()).await?;
        registry.register(EmptyFunction::new()).await?;
        registry.register(ExistsFunction::new()).await?;
        // registry.register(InOperation::new()).await?;

        // Navigation functions
        registry.register(FirstFunction::new()).await?;
        registry.register(LastFunction::new()).await?;
        registry.register(TailFunction::new()).await?;
        registry.register(SkipFunction::new()).await?;
        registry.register(TakeFunction::new()).await?;
        // Sort is handled by lambda functions

        // Boolean collection functions
        registry.register(AllFunction::new()).await?;
        registry.register(AllTrueFunction::new()).await?;
        registry.register(AnyTrueFunction::new()).await?;
        registry.register(AllFalseFunction::new()).await?;
        registry.register(AnyFalseFunction::new()).await?;

        // Set operations
        registry.register(SubsetOfFunction::new()).await?;
        registry.register(SupersetOfFunction::new()).await?;
        registry.register(IsDistinctFunction::new()).await?;
        registry.register(IntersectFunction::new()).await?;
        registry.register(ExcludeFunction::new()).await?;
        registry.register(UnionFunction::new()).await?;
        registry.register(CombineFunction::new()).await?;

        // Union operator (|)
        registry.register(UnionOperator::new()).await?;

        Ok(())
    }
}
