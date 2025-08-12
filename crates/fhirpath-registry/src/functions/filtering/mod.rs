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

//! Filtering and selection functions for FHIRPath expressions

mod of_type;
mod select;
mod skip;
mod take;
mod r#where;

pub use of_type::OfTypeFunction;
pub use select::SelectFunction;
pub use skip::SkipFunction;
pub use take::TakeFunction;
pub use r#where::WhereFunction;

use crate::function::FunctionRegistry;

/// Register all filtering functions
pub fn register_filtering_functions(registry: &mut FunctionRegistry) {
    registry.register_async(OfTypeFunction);
    registry.register(SelectFunction);
    registry.register_async(SkipFunction);
    registry.register_async(TakeFunction);
    registry.register(WhereFunction);
}
