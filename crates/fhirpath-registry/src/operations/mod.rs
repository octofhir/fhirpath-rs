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

//! Core FHIRPath operations for the unified registry
//!
//! This module contains high-performance implementations of all standard FHIRPath
//! operations organized by category. Each operation supports both sync and async
//! evaluation paths for optimal performance.

pub mod arithmetic;
pub mod collection;
pub mod string;

// Re-export for convenience
pub use arithmetic::{
    ArithmeticOperations,
    AdditionOperation,
    SubtractionOperation,
    MultiplicationOperation,
    DivisionOperation,
    ModuloOperation,
    IntegerDivisionOperation,
    UnaryMinusOperation,
    UnaryPlusOperation,
};

pub use collection::{
    CountFunction,
    EmptyFunction,
    ExistsFunction,
    FirstFunction,
    LastFunction,
    SingleFunction,
};

pub use string::{
    LengthFunction,
    ContainsFunction,
    StartsWithFunction,
    EndsWithFunction,
    SubstringFunction,
};