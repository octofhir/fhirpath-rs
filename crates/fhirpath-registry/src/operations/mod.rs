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

//! Core FHIRPath operations for the unified registry
//!
//! This module contains high-performance implementations of all standard FHIRPath
//! operations organized by category. Each operation supports both sync and async
//! evaluation paths for optimal performance.

use crate::FhirPathRegistry;
use octofhir_fhirpath_model::{FhirPathValue, provider::ModelProvider};
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// Evaluation context for operations that includes variables and model provider
#[derive(Clone)]
pub struct EvaluationContext {
    /// Current input value being evaluated
    pub input: FhirPathValue,

    /// Root input value (for %context and $resource variables)
    pub root: FhirPathValue,

    /// Environment variables for the evaluation
    pub variables: FxHashMap<String, FhirPathValue>,

    /// Registry for functions and operators
    pub registry: Arc<FhirPathRegistry>,

    /// Model provider for type checking and validation (required)
    pub model_provider: Arc<dyn ModelProvider>,
}

impl EvaluationContext {
    /// Create a new evaluation context (ModelProvider and registry required)
    pub fn new(
        input: FhirPathValue,
        registry: Arc<FhirPathRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            root: input.clone(),
            input,
            variables: FxHashMap::default(),
            registry,
            model_provider,
        }
    }

    /// Create a new evaluation context with initial variables
    pub fn with_variables(
        input: FhirPathValue,
        registry: Arc<FhirPathRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        variables: FxHashMap<String, FhirPathValue>,
    ) -> Self {
        Self {
            root: input.clone(),
            input,
            variables,
            registry,
            model_provider,
        }
    }

    /// Create a child context with new input value
    pub fn with_input(&self, input: FhirPathValue) -> Self {
        Self {
            root: self.root.clone(),
            input,
            variables: self.variables.clone(),
            registry: self.registry.clone(),
            model_provider: self.model_provider.clone(),
        }
    }

    /// Create a new evaluation context preserving root from another context
    pub fn with_preserved_root(
        input: FhirPathValue,
        original_root: FhirPathValue,
        registry: Arc<FhirPathRegistry>,
        model_provider: Arc<dyn ModelProvider>,
    ) -> Self {
        Self {
            root: original_root,
            input,
            variables: FxHashMap::default(),
            registry,
            model_provider,
        }
    }

    /// Create a new context with different focus/input value (alias for with_input)
    pub fn with_focus(&self, input: FhirPathValue) -> Self {
        self.with_input(input)
    }

    /// Get a variable value by name
    pub fn get_variable(&self, name: &str) -> Option<&FhirPathValue> {
        self.variables.get(name)
    }

    /// Set a variable value
    pub fn set_variable(&mut self, name: String, value: FhirPathValue) {
        self.variables.insert(name, value);
    }
}

pub mod cda;
pub mod collection;
pub mod comparison;
pub mod conversion;
pub mod datetime;
pub mod fhir;
pub mod logical;
pub mod math;
pub mod string;
pub mod types;
pub mod utility;

// Re-export for convenience
pub use cda::{CdaOperations, HasTemplateIdOfFunction};
pub use collection::CollectionOperations;

pub use conversion::ConversionOperations;
pub use string::StringOperations;

pub use comparison::ComparisonOperations;
pub use datetime::DateTimeOperations;
pub use fhir::FhirOperations;
pub use logical::LogicalOperations;
pub use math::MathOperations;
pub use types::TypeOperations;
pub use utility::UtilityOperations;
