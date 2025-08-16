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

//! Comparison operators module

pub mod equals;
pub mod equivalent;
pub mod greater_than;
pub mod greater_than_or_equal;
pub mod less_than;
pub mod less_than_or_equal;
pub mod not_equals;
pub mod not_equivalent;

pub use equals::EqualsOperation;
pub use equivalent::EquivalentOperation;
pub use greater_than::GreaterThanOperation;
pub use greater_than_or_equal::GreaterThanOrEqualOperation;
pub use less_than::LessThanOperation;
pub use less_than_or_equal::LessThanOrEqualOperation;
pub use not_equals::NotEqualsOperation;
pub use not_equivalent::NotEquivalentOperation;

// Collection membership operators
pub use crate::operations::collection::contains_op::ContainsOperation;
// pub use crate::operations::collection::in_op::InOperation; // TODO: Still needs implementation

// Type checking operators
pub use crate::operations::types::is_operator::IsBinaryOperator;

/// Registry helper for comparison operations
pub struct ComparisonOperations;

impl ComparisonOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(EqualsOperation::new()).await?;
        registry.register(NotEqualsOperation::new()).await?;
        registry.register(LessThanOperation::new()).await?;
        registry.register(GreaterThanOperation::new()).await?;
        registry
            .register(GreaterThanOrEqualOperation::new())
            .await?;
        registry.register(LessThanOrEqualOperation::new()).await?;
        registry.register(EquivalentOperation::new()).await?;
        registry.register(NotEquivalentOperation::new()).await?;

        // Type checking operators
        registry.register(IsBinaryOperator::new()).await?;

        // Collection membership operations - note: contains is now handled by StringOperations
        // registry.register(InOperation::new()).await?; // TODO: Still needs implementation
        Ok(())
    }
}
