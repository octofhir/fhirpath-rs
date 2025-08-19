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

//! String functions for FHIRPath

use crate::fhirpath_registry::FhirPathRegistry;
use octofhir_fhirpath_core::Result;

pub mod contains;
pub mod ends_with;
pub mod index_of;
pub mod join;
pub mod last_index_of; // NEW
pub mod length;
pub mod lower;
pub mod matches;
pub mod matches_full;
pub mod replace;
pub mod replace_matches;
pub mod split;
pub mod starts_with;
pub mod substring;
pub mod to_chars;
pub mod trim;
pub mod upper;

pub use contains::ContainsFunction;
pub use ends_with::EndsWithFunction;
pub use index_of::IndexOfFunction;
pub use join::JoinFunction;
pub use last_index_of::LastIndexOfFunction; // NEW
pub use length::LengthFunction;
pub use lower::LowerFunction;
pub use matches::MatchesFunction;
pub use matches_full::MatchesFullFunction;
pub use replace::ReplaceFunction;
pub use replace_matches::ReplaceMatchesFunction;
pub use split::SplitFunction;
pub use starts_with::StartsWithFunction;
pub use substring::SubstringFunction;
pub use to_chars::ToCharsFunction;
pub use trim::TrimFunction;
pub use upper::UpperFunction;

/// Utility struct for registering all string operations
pub struct StringOperations;

impl StringOperations {
    /// Register all string operations in the registry
    pub async fn register_all(registry: &FhirPathRegistry) -> Result<()> {
        // Existing
        registry.register(LengthFunction::new()).await?;

        // Basic string search functions
        registry.register(IndexOfFunction::new()).await?;
        registry.register(LastIndexOfFunction::new()).await?; // NEW
        registry.register(SubstringFunction::new()).await?;
        registry.register(StartsWithFunction::new()).await?;
        registry.register(EndsWithFunction::new()).await?;
        registry.register(ContainsFunction::new()).await?;

        // String transformation functions
        registry.register(UpperFunction::new()).await?;
        registry.register(LowerFunction::new()).await?;
        registry.register(TrimFunction::new()).await?;
        registry.register(ToCharsFunction::new()).await?;

        // String manipulation functions
        registry.register(ReplaceFunction::new()).await?;
        registry.register(SplitFunction::new()).await?;
        registry.register(JoinFunction::new()).await?;

        // Regular expression functions
        registry.register(MatchesFunction::new()).await?;
        registry.register(MatchesFullFunction::new()).await?;
        registry.register(ReplaceMatchesFunction::new()).await?;

        Ok(())
    }
}
