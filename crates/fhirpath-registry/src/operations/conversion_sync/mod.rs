//! Synchronous conversion operations module
//!
//! This module provides sync implementations of all FHIRPath conversion operations,
//! replacing the complex async system with simple sync implementations for pure
//! data transformation operations.

pub mod converts_to_boolean;
pub mod converts_to_date;
pub mod converts_to_datetime;
pub mod converts_to_decimal;
pub mod converts_to_integer;
pub mod converts_to_long;
pub mod converts_to_quantity;
pub mod converts_to_string;
pub mod converts_to_time;
pub mod to_boolean;
pub mod to_date;
pub mod to_datetime;
pub mod to_decimal;
pub mod to_integer;
pub mod to_long;
pub mod to_quantity;
pub mod to_string;
pub mod to_time;

// Re-export available sync conversion operations
pub use converts_to_boolean::ConvertsToBooleanFunction;
pub use converts_to_date::ConvertsToDateFunction;
pub use converts_to_datetime::ConvertsToDateTimeFunction;
pub use converts_to_decimal::ConvertsToDecimalFunction;
pub use converts_to_integer::ConvertsToIntegerFunction;
pub use converts_to_long::ConvertsToLongFunction;
pub use converts_to_quantity::ConvertsToQuantityFunction;
pub use converts_to_string::ConvertsToStringFunction;
pub use converts_to_time::ConvertsToTimeFunction;
pub use to_boolean::ToBooleanFunction;
pub use to_date::ToDateFunction;
pub use to_datetime::ToDateTimeFunction;
pub use to_decimal::ToDecimalFunction;
pub use to_integer::ToIntegerFunction;
pub use to_long::ToLongFunction;
pub use to_quantity::ToQuantityFunction;
pub use to_string::ToStringFunction;
pub use to_time::ToTimeFunction;

use crate::registry::FunctionRegistry;

/// Registry helper for sync conversion operations
pub struct SyncConversionOperations;

impl SyncConversionOperations {
    /// Register all sync conversion operations with the registry
    pub async fn register_all(registry: &FunctionRegistry) {
        // Type checking operations (converts_to_*)
        registry
            .register_sync(Box::new(ConvertsToBooleanFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToDateFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToDateTimeFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToDecimalFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToIntegerFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToLongFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToStringFunction))
            .await;
        registry
            .register_sync(Box::new(ConvertsToTimeFunction))
            .await;

        // Type conversion operations (to_*)
        registry.register_sync(Box::new(ToBooleanFunction)).await;
        registry.register_sync(Box::new(ToDateFunction)).await;
        registry.register_sync(Box::new(ToDateTimeFunction)).await;
        registry.register_sync(Box::new(ToDecimalFunction)).await;
        registry.register_sync(Box::new(ToIntegerFunction)).await;
        registry.register_sync(Box::new(ToLongFunction)).await;
        registry.register_sync(Box::new(ToStringFunction)).await;
        registry.register_sync(Box::new(ToTimeFunction)).await;
    }
}
