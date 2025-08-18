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
//! A complete implementation of the FHIRPath expression language for FHIR resources.

// Import workspace crates
pub use octofhir_fhirpath_ast as ast;
pub use octofhir_fhirpath_core as core;
pub use octofhir_fhirpath_diagnostics as diagnostics;
pub use octofhir_fhirpath_evaluator as evaluator;
pub use octofhir_fhirpath_model as model;
pub use octofhir_fhirpath_parser as parser;
pub use octofhir_fhirpath_registry as registry;

// Main implementation modules
pub mod pipeline;
pub mod utils;

// Primary engine - use this for all new code
pub use octofhir_fhirpath_evaluator::{EvaluationConfig, EvaluationContext, FhirPathEngine};
pub use octofhir_fhirpath_model::{
    FhirPathValue, JsonValue, SmartCollection, SmartCollectionBuilder,
};
pub use octofhir_fhirpath_parser::{ParseError, parse_expression as parse};
pub use octofhir_fhirpath_registry::{FhirPathRegistry, create_standard_registry};

// Re-export from workspace crates
pub use octofhir_fhirpath_ast::{
    BinaryOpData, BinaryOperator, ConditionalData, ExpressionNode, FunctionCallData, LambdaData,
    LiteralValue, MethodCallData, UnaryOperator,
};
pub use octofhir_fhirpath_core::{FhirPathError, Result};
pub use octofhir_fhirpath_diagnostics::{
    Diagnostic, DiagnosticBuilder, DiagnosticCode, DiagnosticReporter, DiagnosticSeverity,
};

// Re-export ModelProvider from fhir-model-rs
pub use octofhir_fhirpath_model::ModelProvider;
pub use octofhir_fhirpath_model::fhir_model;

// Re-export from local modules (minimal local integration code)
pub mod value_ext;

// Re-export conversion utilities for easier access
pub use utils::{
    JsonResult, fhir_value_to_serde, from_sonic, parse_as_fhir_value, parse_json, parse_with_serde,
    reformat_json, serde_to_fhir_value, serde_to_sonic, sonic_to_serde, to_sonic,
};
