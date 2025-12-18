#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_char_add_str)]
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

//! FHIRPath Development Tools
//!
//! This crate provides development and testing utilities for the FHIRPath implementation,
//! including test runners, coverage analysis, and benchmarking tools.

pub mod common;
pub mod metadata;
pub mod test_support;

// Re-export common functionality
pub use common::*;
pub use test_support::*;
// Re-export model providers from fhirschema crate
pub use octofhir_fhirschema::EmbeddedSchemaProvider;
