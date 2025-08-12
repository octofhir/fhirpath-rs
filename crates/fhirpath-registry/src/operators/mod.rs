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

//! Operator implementations for FHIRPath expressions

pub mod arithmetic;
mod collection;
mod comparison;
mod logical;
mod string;

// Re-export all operators
pub use arithmetic::*;
pub use collection::*;
pub use comparison::*;
pub use logical::*;
pub use string::*;

use crate::operator::OperatorRegistry;

/// Register all built-in operators
pub fn register_builtin_operators(registry: &mut OperatorRegistry) {
    // Arithmetic operators
    arithmetic::register_arithmetic_operators(registry);

    // Comparison operators
    comparison::register_comparison_operators(registry);

    // Logical operators
    logical::register_logical_operators(registry);

    // String operators
    string::register_string_operators(registry);

    // Collection operators
    collection::register_collection_operators(registry);
}
