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

pub mod aggregate;
pub mod children;
pub mod descendants;
pub mod of_type;
pub mod repeat;
pub mod select;
pub mod single;
pub mod sort_lambda;
pub mod where_fn;

pub use aggregate::AggregateFunction;
pub use children::ChildrenFunction;
pub use descendants::DescendantsFunction;
pub use of_type::OfTypeFunction;
pub use repeat::RepeatFunction;
pub use select::SelectFunction;
pub use single::SingleFunction;
pub use sort_lambda::SortLambdaFunction;
pub use where_fn::WhereFunction;

/// Registry helper for lambda operations
pub struct LambdaOperations;

impl LambdaOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // Register the hybrid functions that support both regular and lambda modes
        registry.register(WhereFunction::new()).await?;
        registry.register(SelectFunction::new()).await?;
        registry.register(AggregateFunction::new()).await?;
        registry.register(RepeatFunction::new()).await?;
        registry.register(OfTypeFunction::new()).await?;
        registry.register(SingleFunction::new()).await?;
        registry.register(ChildrenFunction::new()).await?;
        registry.register(DescendantsFunction::new()).await?;
        registry.register(SortLambdaFunction::new()).await?;
        Ok(())
    }
}
