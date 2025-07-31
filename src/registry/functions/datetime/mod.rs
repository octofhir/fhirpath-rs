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
    registry.register(NowFunction);
    registry.register(TodayFunction);
    registry.register(LowBoundaryFunction);
    registry.register(HighBoundaryFunction);
}
