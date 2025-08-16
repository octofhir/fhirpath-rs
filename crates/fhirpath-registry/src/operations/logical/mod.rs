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

//! Logical operators module

pub mod and;
pub mod implies;
pub mod not;
pub mod or;
pub mod xor;

pub use and::AndOperation;
pub use implies::ImpliesOperation;
pub use not::NotOperation;
pub use or::OrOperation;
pub use xor::XorOperation;

/// Registry helper for logical operations
pub struct LogicalOperations;

impl LogicalOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(AndOperation::new()).await?;
        registry.register(OrOperation::new()).await?;
        registry.register(NotOperation::new()).await?;
        registry.register(XorOperation::new()).await?;
        registry.register(ImpliesOperation::new()).await?;
        Ok(())
    }
}
