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

//! Unified type conversion function implementations for FHIRPath
//!
//! This module contains all type conversion functions implemented using the unified
//! function trait with rich metadata and optimized execution paths.

pub mod to_string;
pub mod to_integer;
pub mod to_decimal;
pub mod to_boolean;
pub mod to_quantity;
pub mod converts_to_string;
pub mod converts_to_integer;
pub mod converts_to_decimal;
pub mod converts_to_boolean;
pub mod converts_to_quantity;

// Re-export all type conversion functions
pub use to_string::UnifiedToStringFunction;
pub use to_integer::UnifiedToIntegerFunction;
pub use to_decimal::UnifiedToDecimalFunction;
pub use to_boolean::UnifiedToBooleanFunction;
pub use to_quantity::UnifiedToQuantityFunction;
pub use converts_to_string::UnifiedConvertsToStringFunction;
pub use converts_to_integer::UnifiedConvertsToIntegerFunction;
pub use converts_to_decimal::UnifiedConvertsToDecimalFunction;
pub use converts_to_boolean::UnifiedConvertsToBooleanFunction;
pub use converts_to_quantity::UnifiedConvertsToQuantityFunction;
