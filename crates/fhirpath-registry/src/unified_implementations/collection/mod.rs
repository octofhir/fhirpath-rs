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

//! Collection function implementations for the unified registry

// Individual function modules
pub mod all;
pub mod any;
pub mod combine;
pub mod count;
pub mod distinct;
pub mod empty;
pub mod exclude;
pub mod is_distinct;
pub mod exists;
pub mod first;
pub mod flatten;
pub mod index_of;
pub mod intersect;
pub mod last;
pub mod single;
pub mod skip;
pub mod sort_enhanced;
pub mod subset_of;
pub mod superset_of;
pub mod tail;
pub mod take;
pub mod union;

// Re-export unified implementations
pub use all::UnifiedAllFunction;
pub use any::UnifiedAnyFunction;
pub use combine::UnifiedCombineFunction;
pub use count::UnifiedCountFunction;
pub use distinct::UnifiedDistinctFunction;
pub use empty::UnifiedEmptyFunction;
pub use exclude::UnifiedExcludeFunction;
pub use is_distinct::UnifiedIsDistinctFunction;
pub use exists::UnifiedExistsFunction;
pub use first::UnifiedFirstFunction;
pub use flatten::UnifiedFlattenFunction;
pub use index_of::UnifiedIndexOfFunction;
pub use intersect::UnifiedIntersectFunction;
pub use last::UnifiedLastFunction;
pub use single::UnifiedSingleFunction;
pub use skip::UnifiedSkipFunction;
pub use sort_enhanced::EnhancedSortFunction;
pub use subset_of::UnifiedSubsetOfFunction;
pub use superset_of::UnifiedSupersetOfFunction;
pub use tail::UnifiedTailFunction;
pub use take::UnifiedTakeFunction;
pub use union::UnifiedUnionFunction;
