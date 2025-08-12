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

//! Extension system for FHIRPath functions and variables

pub mod builtin;
pub mod error;
pub mod manager;
pub mod metadata;
pub mod registry;

pub use error::{ExtensionError, ExtensionResult};
pub use manager::ExtensionManager;
pub use metadata::ExtensionMetadata;
pub use registry::ExtensionRegistry;

use crate::function::{EvaluationContext, FunctionImpl};
use octofhir_fhirpath_model::FhirPathValue;
use std::sync::Arc;

/// Type alias for variable resolvers
pub type VariableResolver =
    Arc<dyn Fn(&str, &EvaluationContext) -> Option<FhirPathValue> + Send + Sync>;

/// Trait for implementing FHIRPath extensions
pub trait FhirPathExtension: Send + Sync {
    /// Get extension metadata
    fn metadata(&self) -> &ExtensionMetadata;

    /// Register extension functions
    fn register_functions(&self, registry: &mut ExtensionRegistry) -> ExtensionResult<()>;

    /// Register extension variables (optional)
    fn register_variables(&self, _registry: &mut ExtensionRegistry) -> ExtensionResult<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Initialize extension (called after registration)
    fn initialize(&self) -> ExtensionResult<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Cleanup extension resources (called on unload)
    fn cleanup(&self) -> ExtensionResult<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Resolution result for namespace-qualified function lookups
#[derive(Clone)]
pub enum FunctionResolution {
    /// Core function (no namespace)
    Core(Arc<FunctionImpl>),

    /// Extension function with namespace
    Extension {
        /// The namespace containing the function
        namespace: String,
        /// The function implementation
        function: Arc<FunctionImpl>,
    },

    /// Function exists in multiple namespaces (ambiguous)
    Ambiguous(Vec<String>),

    /// Function not found
    NotFound,
}

impl FunctionResolution {
    /// Get the function implementation if resolved
    pub fn function(&self) -> Option<&Arc<FunctionImpl>> {
        match self {
            FunctionResolution::Core(func) => Some(func),
            FunctionResolution::Extension { function, .. } => Some(function),
            _ => None,
        }
    }

    /// Check if resolution is ambiguous
    pub fn is_ambiguous(&self) -> bool {
        matches!(self, FunctionResolution::Ambiguous(_))
    }

    /// Check if function was found
    pub fn is_found(&self) -> bool {
        matches!(
            self,
            FunctionResolution::Core(_) | FunctionResolution::Extension { .. }
        )
    }
}
