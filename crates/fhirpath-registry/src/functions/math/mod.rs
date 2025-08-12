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

//! Mathematical functions for FHIRPath expressions

mod abs;
mod avg;
mod ceiling;
mod exp;
mod floor;
mod ln;
mod log;
mod max;
mod min;
mod power;
mod precision;
mod round;
mod sqrt;
mod sum;
mod truncate;

pub use abs::AbsFunction;
pub use avg::AvgFunction;
pub use ceiling::CeilingFunction;
pub use exp::ExpFunction;
pub use floor::FloorFunction;
pub use ln::LnFunction;
pub use log::LogFunction;
pub use max::MaxFunction;
pub use min::MinFunction;
pub use power::PowerFunction;
pub use precision::PrecisionFunction;
pub use round::RoundFunction;
pub use sqrt::SqrtFunction;
pub use sum::SumFunction;
pub use truncate::TruncateFunction;

use crate::function::FunctionRegistry;

/// Register all math functions
pub fn register_math_functions(registry: &mut FunctionRegistry) {
    registry.register_async(AbsFunction);
    registry.register_async(AvgFunction);
    registry.register_async(CeilingFunction);
    registry.register_async(ExpFunction);
    registry.register_async(FloorFunction);
    registry.register_async(LnFunction);
    registry.register_async(LogFunction);
    registry.register_async(MaxFunction);
    registry.register_async(MinFunction);
    registry.register_async(PowerFunction);
    registry.register_async(PrecisionFunction);
    registry.register_async(RoundFunction);
    registry.register_async(SqrtFunction);
    registry.register_async(SumFunction);
    registry.register_async(TruncateFunction);
}
