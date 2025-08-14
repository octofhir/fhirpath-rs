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

//! Unified string function implementations for FHIRPath
//!
//! This module contains all string manipulation functions implemented using the unified
//! function trait with rich metadata and optimized execution paths.

pub mod length;
pub mod substring;
pub mod contains;
pub mod starts_with;
pub mod ends_with;
pub mod upper;
pub mod lower;
pub mod trim;
pub mod to_chars;
pub mod escape;
pub mod unescape;

pub use length::UnifiedLengthFunction;
pub use substring::UnifiedSubstringFunction;
pub use contains::UnifiedContainsFunction;
pub use starts_with::UnifiedStartsWithFunction;
pub use ends_with::UnifiedEndsWithFunction;
pub use upper::UnifiedUpperFunction;
pub use lower::UnifiedLowerFunction;
pub use trim::UnifiedTrimFunction;
pub use to_chars::UnifiedToCharsFunction;
pub use escape::UnifiedEscapeFunction;
pub use unescape::UnifiedUnescapeFunction;
