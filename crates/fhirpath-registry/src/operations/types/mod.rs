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

//! Type checking and casting functions module (operators moved to evaluator)

pub mod as_op;
pub mod is;
pub mod of_type;
pub mod type_func;

pub use as_op::AsOperation;
pub use is::IsOperation;
pub use of_type::OfTypeFunction;
pub use type_func::TypeFunction;

/// Registry helper for type functions
pub struct TypeOperations;

impl TypeOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // Function version for method-style calls like "value.is(Type)" and "value.as(Type)"
        registry.register(IsOperation::new()).await?;
        registry.register(AsOperation::new()).await?;
        registry.register(OfTypeFunction::new()).await?;
        registry.register(TypeFunction::new()).await?;
        Ok(())
    }
}
