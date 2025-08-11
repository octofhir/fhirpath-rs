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

//! Utility functions for FHIRPath expressions

mod conforms_to;
mod define_variable;
mod has_value;
mod iif;
mod repeat;
mod trace;

pub use conforms_to::ConformsToFunction;
pub use define_variable::DefineVariableFunction;
pub use has_value::HasValueFunction;
pub use iif::IifFunction;
pub use repeat::RepeatFunction;
pub use trace::TraceFunction;

use crate::registry::function::FunctionRegistry;

/// Register all utility functions
pub fn register_utility_functions(registry: &mut FunctionRegistry) {
    registry.register_async(ConformsToFunction::new());
    registry.register_async(DefineVariableFunction);
    registry.register_async(HasValueFunction);
    registry.register_async(IifFunction);
    registry.register_async(RepeatFunction);
    registry.register_async(TraceFunction);
}
