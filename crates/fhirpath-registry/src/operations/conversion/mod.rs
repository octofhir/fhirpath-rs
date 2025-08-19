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

//! Conversion functions module

pub mod converts_to_boolean;
pub mod converts_to_date;
pub mod converts_to_datetime;
pub mod converts_to_decimal;
pub mod converts_to_integer;
pub mod converts_to_long; // NEW
pub mod converts_to_quantity;
pub mod converts_to_string;
pub mod converts_to_time;
pub mod to_boolean;
pub mod to_date;
pub mod to_datetime;
pub mod to_decimal;
pub mod to_integer;
pub mod to_long; // NEW
pub mod to_quantity;
pub mod to_string;
pub mod to_time;
// TODO: Add other conversion modules as they are implemented

pub use converts_to_boolean::ConvertsToBooleanFunction;
pub use converts_to_date::ConvertsToDateFunction;
pub use converts_to_datetime::ConvertsToDateTimeFunction;
pub use converts_to_decimal::ConvertsToDecimalFunction;
pub use converts_to_integer::ConvertsToIntegerFunction;
pub use converts_to_long::ConvertsToLongFunction; // NEW
pub use converts_to_quantity::ConvertsToQuantityFunction;
pub use converts_to_string::ConvertsToStringFunction;
pub use converts_to_time::ConvertsToTimeFunction;
pub use to_boolean::ToBooleanFunction;
pub use to_date::ToDateFunction;
pub use to_datetime::ToDateTimeFunction;
pub use to_decimal::ToDecimalFunction;
pub use to_integer::ToIntegerFunction;
pub use to_long::ToLongFunction; // NEW
pub use to_quantity::ToQuantityFunction;
pub use to_string::ToStringFunction;
pub use to_time::ToTimeFunction;

/// Registry helper for conversion operations
pub struct ConversionOperations;

impl ConversionOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(ConvertsToBooleanFunction::new()).await?;
        registry.register(ToBooleanFunction::new()).await?;
        registry.register(ConvertsToQuantityFunction::new()).await?;
        registry.register(ToQuantityFunction::new()).await?;
        registry.register(ConvertsToStringFunction::new()).await?;
        registry.register(ToStringFunction::new()).await?;
        registry.register(ConvertsToIntegerFunction::new()).await?;
        registry.register(ToIntegerFunction::new()).await?;
        registry.register(ConvertsToDecimalFunction::new()).await?;
        registry.register(ToDecimalFunction::new()).await?;
        registry.register(ConvertsToDateFunction::new()).await?;
        registry.register(ToDateFunction::new()).await?;
        registry.register(ConvertsToDateTimeFunction::new()).await?;
        registry.register(ToDateTimeFunction::new()).await?;
        registry.register(ConvertsToTimeFunction::new()).await?;
        registry.register(ToTimeFunction::new()).await?;

        // NEW: Long integer functions
        registry.register(ToLongFunction::new()).await?;
        registry.register(ConvertsToLongFunction::new()).await?;

        // TODO: Register other conversion functions as they are implemented
        Ok(())
    }
}
