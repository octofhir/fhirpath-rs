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

//! FHIRPath implementation in Rust
//!
//! A complete implementation of FHIRPath expression language for FHIR resources.

pub mod ast;
pub mod compiler;
pub mod diagnostics;
pub mod evaluator;
pub mod model;
pub mod parser;
pub mod pipeline;
pub mod registry;

// Re-export main types
pub use evaluator::{EvaluationContext, FhirPathEngine};
pub use model::{FhirPathValue, SmartCollection, SmartCollectionBuilder};
pub use parser::{ParseError, parse_expression as parse};
pub use registry::FunctionRegistry;

// Re-export ModelProvider from fhir-model-rs
pub use octofhir_fhir_model as fhir_model;
pub use octofhir_fhir_model::provider::ModelProvider;

// Re-export from fhirpath-core
pub mod engine;
pub mod error;
pub mod types;
pub mod value_ext;

pub use engine::*;
pub use error::*;
pub use types::*;
