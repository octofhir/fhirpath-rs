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

//! Type checking and casting operations module

pub mod is;
pub mod is_operator;
pub mod as_op;
pub mod type_func;

pub use is::IsOperation;
pub use is_operator::IsBinaryOperator;
pub use as_op::AsOperation;
pub use type_func::TypeFunction;

/// Registry helper for type operations
pub struct TypeOperations;

impl TypeOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(IsOperation::new()).await?;
        registry.register(IsBinaryOperator::new()).await?;
        registry.register(AsOperation::new()).await?;
        registry.register(TypeFunction::new()).await?;
        Ok(())
    }
}
