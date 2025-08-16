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

//! FHIRPath Expression Evaluator
//!
//! This module provides the unified FHIRPath evaluation engine.

mod context;
pub mod engine;

// Primary engine
pub use engine::{EvaluationConfig, FhirPathEngine};

// Essential evaluation functionality
pub use context::{EvaluationContext, LambdaContextBuilder, LambdaMetadata, VariableScope};
pub use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};

// Comprehensive test suite for evaluator functionality
#[cfg(test)]
mod tests;
