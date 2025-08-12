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

//! Function and operator registry for FHIRPath implementation
//!
//! This crate provides the comprehensive function registry with built-in functions,
//! operators, and extension system for FHIRPath expressions.

pub mod cache;
pub mod compiled_signatures;
pub mod extension;
pub mod fast_path;
pub mod function;
pub mod functions;
pub mod operator;
pub mod operators;
pub mod signature;
// Re-export main types
pub use extension::ExtensionRegistry;
pub use function::FunctionRegistry;
pub use operator::OperatorRegistry;
pub use signature::{FunctionSignature, ParameterInfo};

/// Create a standard registry with all built-in functions and operators
pub fn create_standard_registries() -> (FunctionRegistry, OperatorRegistry) {
    let mut functions = FunctionRegistry::new();
    let mut operators = OperatorRegistry::new();

    // Register built-in functions and operators
    function::register_builtin_functions(&mut functions);
    operator::register_builtin_operators(&mut operators);

    (functions, operators)
}

// Re-export from workspace crates for convenience
pub use fhirpath_ast::{BinaryOperator, ExpressionNode, UnaryOperator};
pub use fhirpath_core::{FhirPathError, Result};
pub use fhirpath_model::{FhirPathValue, ModelProvider};
