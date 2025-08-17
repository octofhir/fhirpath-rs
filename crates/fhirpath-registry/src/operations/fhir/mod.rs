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

//! FHIR-specific functions module

pub mod children;
pub mod conforms_to;
pub mod descendants;
pub mod extension;
pub mod resolve;

pub use children::ChildrenFunction;
pub use conforms_to::ConformsToFunction;
pub use descendants::DescendantsFunction;
pub use extension::ExtensionFunction;
pub use resolve::ResolveFunction;

/// Registry helper for FHIR operations
pub struct FhirOperations;

impl FhirOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(ResolveFunction::new()).await?;
        registry.register(ExtensionFunction::new()).await?;
        registry.register(ConformsToFunction::new()).await?;
        registry.register(ChildrenFunction::new()).await?;
        registry.register(DescendantsFunction::new()).await?;
        Ok(())
    }
}
