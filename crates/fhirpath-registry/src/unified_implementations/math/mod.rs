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

//! Unified math function implementations for FHIRPath
//!
//! This module contains all mathematical functions implemented using the unified
//! function trait with rich metadata and optimized execution paths.

pub mod abs;
pub mod ceiling;
pub mod exp;
pub mod floor;
pub mod ln;
pub mod log;
pub mod round;
pub mod sqrt;
pub mod truncate;
pub mod precision;
pub mod power;

// Re-export all function implementations
pub use abs::*;
pub use ceiling::*;
pub use exp::*;
pub use floor::*;
pub use ln::*;
pub use log::*;
pub use round::*;
pub use sqrt::*;
pub use truncate::*;
pub use precision::*;
pub use power::*;