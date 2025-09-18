//! Conversion functions for FHIRPath
//!
//! This module contains conversion functions that convert values between different types.

// Conversion functions
pub mod to_boolean_function;
pub mod to_date_function;
pub mod to_datetime_function;
pub mod to_decimal_function;
pub mod to_integer_function;
pub mod to_quantity_function;
pub mod to_string_function;
pub mod to_time_function;

// Conversion test functions
pub mod converts_to_boolean_function;
pub mod converts_to_date_function;
pub mod converts_to_datetime_function;
pub mod converts_to_decimal_function;
pub mod converts_to_integer_function;
pub mod converts_to_quantity_function;
pub mod converts_to_string_function;
pub mod converts_to_time_function;

// Re-export all conversion functions
pub use to_boolean_function::*;
pub use to_date_function::*;
pub use to_datetime_function::*;
pub use to_decimal_function::*;
pub use to_integer_function::*;
pub use to_quantity_function::*;
pub use to_string_function::*;
pub use to_time_function::*;

pub use converts_to_boolean_function::*;
pub use converts_to_date_function::*;
pub use converts_to_datetime_function::*;
pub use converts_to_decimal_function::*;
pub use converts_to_integer_function::*;
pub use converts_to_quantity_function::*;
pub use converts_to_string_function::*;
pub use converts_to_time_function::*;
