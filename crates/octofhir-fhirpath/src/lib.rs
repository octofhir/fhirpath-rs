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

// Import workspace crates
pub use fhirpath_ast as ast;
pub use fhirpath_compiler as compiler;
pub use fhirpath_core as core;
pub use fhirpath_diagnostics as diagnostics;
pub use fhirpath_evaluator as evaluator;
pub use fhirpath_model as model;
pub use fhirpath_parser as parser;
pub use fhirpath_registry as registry;

// Main implementation modules
pub mod pipeline;

// Re-export main types
pub use engine::{FhirPathEngineWithCache, IntegratedFhirPathEngine};
pub use fhirpath_evaluator::{EvaluationContext, FhirPathEngine};
pub use fhirpath_model::{FhirPathValue, SmartCollection, SmartCollectionBuilder};
pub use fhirpath_parser::{ParseError, parse_expression as parse};
pub use fhirpath_registry::FunctionRegistry;

// Re-export from workspace crates
pub use fhirpath_ast::{
    BinaryOpData, BinaryOperator, ConditionalData, ExpressionNode, FunctionCallData, LambdaData,
    LiteralValue, MethodCallData, UnaryOperator,
};
pub use fhirpath_core::{FhirPathError, FhirTypeRegistry, Result};
pub use fhirpath_diagnostics::{
    Diagnostic, DiagnosticBuilder, DiagnosticCode, DiagnosticReporter, DiagnosticSeverity,
};

// Re-export ModelProvider from fhir-model-rs
pub use fhirpath_model::ModelProvider;
pub use fhirpath_model::fhir_model;

// Re-export from local modules (minimal local integration code)
pub mod engine;
pub mod value_ext;

pub use engine::*;
