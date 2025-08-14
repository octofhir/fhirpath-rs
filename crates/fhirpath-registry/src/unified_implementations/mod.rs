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

//! Unified function implementations for the new registry

pub mod aggregates;
pub mod boolean;
pub mod cda;
pub mod collection;
pub mod datetime;
pub mod fhir;
pub mod filtering;
pub mod math;
pub mod string;
pub mod string_extended;
pub mod tree_navigation;
pub mod type_checking;
pub mod type_conversion;
pub mod utility;

// Re-export unified implementations
pub use aggregates::*;
pub use boolean::*;
pub use cda::*;
pub use collection::*;
pub use datetime::*;
pub use fhir::*;
pub use filtering::*;
pub use math::*;
pub use string::*;
pub use string_extended::*;
pub use tree_navigation::*;
pub use type_checking::*;
pub use type_conversion::*;
pub use utility::*;