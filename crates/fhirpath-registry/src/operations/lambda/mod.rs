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

pub mod where_fn;
pub mod where_lambda;
pub mod select;
pub mod select_lambda;
pub mod aggregate;
pub mod repeat;
pub mod of_type;
pub mod single;
pub mod children;
pub mod sort_lambda;

pub use where_fn::WhereFunction;
pub use where_lambda::WhereLambdaFunction;
pub use select::SelectFunction;
pub use select_lambda::SelectLambdaFunction;
pub use aggregate::AggregateFunction;
pub use repeat::RepeatFunction;
pub use of_type::OfTypeFunction;
pub use single::SingleFunction;
pub use children::ChildrenFunction;
pub use sort_lambda::SortLambdaFunction;

/// Registry helper for lambda operations
pub struct LambdaOperations;

impl LambdaOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // Register the lambda version of where and select instead of the old ones
        registry.register(WhereLambdaFunction::new()).await?;
        registry.register(SelectLambdaFunction::new()).await?;
        registry.register(AggregateFunction::new()).await?;
        registry.register(RepeatFunction::new()).await?;
        registry.register(OfTypeFunction::new()).await?;
        registry.register(SingleFunction::new()).await?;
        registry.register(ChildrenFunction::new()).await?;
        registry.register(SortLambdaFunction::new()).await?;
        Ok(())
    }
}
