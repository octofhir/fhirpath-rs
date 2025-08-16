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

//! Utility functions module

pub mod comparable;
pub mod decode;
pub mod define_variable;
pub mod encode;
pub mod escape;
pub mod has_value;
pub mod iif;
pub mod trace;
pub mod unescape;

pub use comparable::ComparableFunction;
pub use decode::DecodeFunction;
pub use define_variable::DefineVariableFunction;
pub use encode::EncodeFunction;
pub use escape::EscapeFunction;
pub use has_value::HasValueFunction;
pub use iif::IifFunction;
pub use trace::TraceFunction;
pub use unescape::UnescapeFunction;

/// Registry helper for utility operations
pub struct UtilityOperations;

impl UtilityOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(IifFunction::new()).await?;
        registry.register(TraceFunction::new()).await?;
        registry.register(HasValueFunction::new()).await?;
        registry.register(EncodeFunction::new()).await?;
        registry.register(DecodeFunction::new()).await?;
        registry.register(EscapeFunction::new()).await?;
        registry.register(UnescapeFunction::new()).await?;
        registry.register(DefineVariableFunction::new()).await?;
        registry.register(ComparableFunction::new()).await?;
        Ok(())
    }
}
