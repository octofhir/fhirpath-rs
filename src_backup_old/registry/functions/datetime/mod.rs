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

//! Date and time functions for FHIRPath expressions

mod boundary;
mod now;
mod today;

pub use boundary::{HighBoundaryFunction, LowBoundaryFunction};
pub use now::NowFunction;
pub use today::TodayFunction;

use crate::registry::function::FunctionRegistry;

/// Register all datetime functions
pub fn register_datetime_functions(registry: &mut FunctionRegistry) {
    registry.register_async(NowFunction);
    registry.register_async(TodayFunction);
    registry.register_async(LowBoundaryFunction);
    registry.register_async(HighBoundaryFunction);
}
