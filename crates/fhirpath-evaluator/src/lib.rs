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
//! This module provides clean, focused evaluation functionality for FHIRPath expressions.
//! It implements both traditional AST interpretation and high-performance bytecode VM execution
//! with automatic hybrid strategy selection for optimal performance.

/// Memory-efficient FHIR Bundle processing with Arc-based sharing
pub mod bundle_arc;
pub mod collections;
mod context;
mod engine;
// mod error; // Using fhirpath-core error types instead
pub mod function_optimizer;
pub mod navigation;
mod shared_context;
pub mod type_checker;
pub mod validator;

// Essential evaluation functionality - clean and focused
pub use context::{EvaluationContext, VariableScope};
pub use engine::FhirPathEngine;
pub use function_optimizer::{
    CacheStats, CollectionOpType, DispatchInfo, FunctionOptimizer, OptimizedSignature,
};
pub use navigation::TypeAwareNavigator;
pub use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
pub use shared_context::{
    ContextInheritance, FunctionClosureOptimizer, SharedContextBuilder, SharedEvaluationContext,
};
pub use type_checker::TypeChecker;
pub use validator::{RuntimeValidator, ValidationMode, ValidationResult};

// Collection optimization utilities
pub use collections::{
    BundleEntryIterator, CollectionUtils, FilterOps, OptimizedCollectionBuilder, SizeHint,
};

// Tests for evaluator functionality
#[cfg(test)]
mod tests {
    mod environment_variables;
}
