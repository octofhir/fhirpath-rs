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

//! Boolean logic functions for FHIRPath expressions

mod all;
mod all_true;
mod any;
mod is_distinct;
mod not;

pub use all::AllFunction;
pub use all_true::AllTrueFunction;
pub use any::AnyFunction;
pub use is_distinct::IsDistinctFunction;
pub use not::NotFunction;

use crate::registry::function::FunctionRegistry;

/// Register all boolean functions
pub fn register_boolean_functions(registry: &mut FunctionRegistry) {
    registry.register(AllFunction);
    registry.register_async(AllTrueFunction);
    registry.register_async(AnyFunction);
    registry.register_async(IsDistinctFunction);
    registry.register_async(NotFunction);
}
