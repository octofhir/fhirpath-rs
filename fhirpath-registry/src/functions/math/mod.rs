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
    registry.register(AbsFunction);
    registry.register(AvgFunction);
    registry.register(CeilingFunction);
    registry.register(ExpFunction);
    registry.register(FloorFunction);
    registry.register(LnFunction);
    registry.register(LogFunction);
    registry.register(MaxFunction);
    registry.register(MinFunction);
    registry.register(PowerFunction);
    registry.register(PrecisionFunction);
    registry.register(RoundFunction);
    registry.register(SqrtFunction);
    registry.register(SumFunction);
    registry.register(TruncateFunction);
}