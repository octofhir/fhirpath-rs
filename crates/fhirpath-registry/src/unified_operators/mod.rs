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

//! Unified operator implementations with enhanced metadata

pub mod arithmetic;
pub mod comparison;
pub mod logical;
pub mod type_checking;
pub mod collection;
pub mod string;

pub use arithmetic::*;
pub use comparison::*;
pub use logical::*;
pub use type_checking::*;
pub use collection::*;
pub use string::*;

#[cfg(test)]
mod tests;