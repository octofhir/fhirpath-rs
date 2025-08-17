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

//! Lambda-supporting functions module

pub mod children;
pub mod descendants;
pub mod of_type;
pub mod single;

pub use children::ChildrenFunction;
pub use descendants::DescendantsFunction;
pub use of_type::OfTypeFunction;
pub use single::SingleFunction;

/// Registry helper for lambda operations
pub struct LambdaOperations;

impl LambdaOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // Register the remaining lambda functions that are not handled in engine
        // Main lambda functions (where, select, aggregate, sort, repeat) are now handled directly in the engine
        registry.register(OfTypeFunction::new()).await?;
        registry.register(SingleFunction::new()).await?;
        registry.register(ChildrenFunction::new()).await?;
        registry.register(DescendantsFunction::new()).await?;
        Ok(())
    }
}
