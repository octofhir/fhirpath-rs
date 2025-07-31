//! Date and time functions for FHIRPath expressions

mod now;
mod today;
mod boundary;

pub use now::NowFunction;
pub use today::TodayFunction;
pub use boundary::{LowBoundaryFunction, HighBoundaryFunction};

use crate::function::FunctionRegistry;

/// Register all datetime functions
pub fn register_datetime_functions(registry: &mut FunctionRegistry) {
    registry.register(NowFunction);
    registry.register(TodayFunction);
    registry.register(LowBoundaryFunction);
    registry.register(HighBoundaryFunction);
}