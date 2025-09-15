//! FHIRPath function and operator registry
//!
//! This module provides a comprehensive registry system for FHIRPath functions,
//! supporting both synchronous and asynchronous functions with metadata,
//! validation, and dispatch capabilities.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use crate::core::{FhirPathError, FhirPathValue, Result, error_code::FP0054};

pub use terminology_provider::{
    ConceptDetails, DefaultTerminologyProvider, MockTerminologyProvider, TerminologyProvider,
};
// ConcreteTerminologyService removed - using terminology provider from fhir-model instead
pub use terminology_utils::{
    Coding, ConceptDesignation, ConceptProperty, ConceptTranslation, PropertyValue,
    TerminologyUtils,
};

pub mod builder;
pub mod collection;
pub mod conversion;
pub mod conversion_utils;
pub mod datetime;
pub mod datetime_utils;
pub mod defaults;
pub mod dispatcher;
pub mod fhir;
pub mod lambda_functions;
pub mod fhir_utils;
pub mod logic;
pub mod math;
pub mod numeric;
pub mod string;
pub mod terminology;
pub mod terminology_provider;
// terminology_service removed - using terminology provider from fhir-model instead
pub mod terminology_utils;
pub mod type_utils;
pub mod types;

pub use collection::CollectionUtils;
pub use conversion_utils::ConversionUtils;
pub use datetime_utils::{DateTimeDuration, DateTimeUtils};
pub use fhir_utils::FhirUtils;
pub use math::ArithmeticOperations;
pub use string::{RegexCache, StringUtils};
pub use type_utils::TypeUtils;
pub use types::{FhirPathType, TypeChecker};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod type_tests;

#[cfg(test)]
mod datetime_tests;

#[cfg(test)]
mod terminology_tests;

#[cfg(test)]
mod terminology_provider_tests;

#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    pub name: String,
    pub category: FunctionCategory,
    pub description: String,
    pub parameters: Vec<ParameterMetadata>,
    pub return_type: Option<String>,
    pub is_async: bool,
    pub examples: Vec<String>,
    /// Function requires ModelProvider access
    pub requires_model_provider: bool,
    /// Function requires TerminologyProvider access
    pub requires_terminology_provider: bool,
    /// Function does not propagate empty (returns non-empty even with empty input)
    pub does_not_propagate_empty: bool,
}

#[derive(Debug, Clone)]
pub struct ParameterMetadata {
    pub name: String,
    pub type_constraint: Option<String>,
    pub is_optional: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionCategory {
    Collection,
    Math,
    String,
    Type,
    Conversion,
    DateTime,
    Fhir,
    Terminology,
    Logic,
    Utility,
}

/// Function execution context with provider access through EvaluationContext
pub struct FunctionContext<'a> {
    pub input: FhirPathValue,
    pub arguments: FhirPathValue,
    pub context: &'a crate::evaluator::EvaluationContext,
    /// Track whether current evaluation is within FHIR navigation context
    pub is_fhir_navigation: bool,
}

/// Sync function signature
pub type SyncFunction = Arc<dyn Fn(&FunctionContext) -> Result<FhirPathValue> + Send + Sync>;

/// Async function signature
pub type AsyncFunction = Arc<
    dyn for<'a> Fn(
            &'a FunctionContext<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<FhirPathValue>> + Send + 'a>>
        + Send
        + Sync,
>;

/// Thread-safe function registry with metadata support using RwLock for concurrent access
pub struct FunctionRegistry {
    sync_functions: RwLock<HashMap<String, (SyncFunction, FunctionMetadata)>>,
    async_functions: RwLock<HashMap<String, (AsyncFunction, FunctionMetadata)>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        Self {
            sync_functions: RwLock::new(HashMap::new()),
            async_functions: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_sync_function(
        &self,
        name: impl Into<String>,
        function: SyncFunction,
        metadata: FunctionMetadata,
    ) -> Result<()> {
        let name = name.into();

        // Check if function already exists
        {
            let sync_functions = self.sync_functions.read().unwrap();
            let async_functions = self.async_functions.read().unwrap();
            if sync_functions.contains_key(&name) || async_functions.contains_key(&name) {
                return Err(FhirPathError::evaluation_error(
                    FP0054,
                    format!("Function '{}' is already registered", name),
                ));
            }
        }

        // Insert the function
        let mut sync_functions = self.sync_functions.write().unwrap();
        sync_functions.insert(name, (function, metadata));
        Ok(())
    }

    pub fn register_async_function(
        &self,
        name: impl Into<String>,
        function: AsyncFunction,
        metadata: FunctionMetadata,
    ) -> Result<()> {
        let name = name.into();

        // Check if function already exists
        {
            let sync_functions = self.sync_functions.read().unwrap();
            let async_functions = self.async_functions.read().unwrap();
            if sync_functions.contains_key(&name) || async_functions.contains_key(&name) {
                return Err(FhirPathError::evaluation_error(
                    FP0054,
                    format!("Function '{}' is already registered", name),
                ));
            }
        }

        // Insert the function
        let mut async_functions = self.async_functions.write().unwrap();
        async_functions.insert(name, (function, metadata));
        Ok(())
    }

    pub fn get_sync_function(&self, name: &str) -> Option<(SyncFunction, FunctionMetadata)> {
        self.sync_functions.read().unwrap().get(name).cloned()
    }

    pub fn get_async_function(&self, name: &str) -> Option<(AsyncFunction, FunctionMetadata)> {
        self.async_functions.read().unwrap().get(name).cloned()
    }

    pub fn is_function_async(&self, name: &str) -> Option<bool> {
        let sync_functions = self.sync_functions.read().unwrap();
        let async_functions = self.async_functions.read().unwrap();

        if sync_functions.contains_key(name) {
            Some(false)
        } else if async_functions.contains_key(name) {
            Some(true)
        } else {
            None
        }
    }

    pub fn get_function_metadata(&self, name: &str) -> Option<FunctionMetadata> {
        let sync_functions = self.sync_functions.read().unwrap();
        if let Some((_, metadata)) = sync_functions.get(name) {
            return Some(metadata.clone());
        }

        let async_functions = self.async_functions.read().unwrap();
        async_functions
            .get(name)
            .map(|(_, metadata)| metadata.clone())
    }

    pub fn list_functions(&self) -> Vec<FunctionMetadata> {
        let mut functions = Vec::new();

        // Collect sync functions
        let sync_functions = self.sync_functions.read().unwrap();
        for (_, metadata) in sync_functions.values() {
            functions.push(metadata.clone());
        }

        // Collect async functions
        let async_functions = self.async_functions.read().unwrap();
        for (_, metadata) in async_functions.values() {
            functions.push(metadata.clone());
        }

        functions
    }

    /// Evaluate a function with providers (legacy method - use evaluate_function_with_args)
    pub async fn evaluate_function(
        &self,
        name: &str,
        input: &crate::core::Collection,
        _args: &[crate::ast::ExpressionNode],
        context: &crate::evaluator::EvaluationContext,
    ) -> Result<crate::core::Collection> {
        // Legacy method - delegate to new method with empty args for compatibility
        let empty_args = Vec::new();
        self.evaluate_function_with_args(name, input, &empty_args, context).await
    }

    /// Evaluate a function with pre-evaluated arguments
    pub async fn evaluate_function_with_args(
        &self,
        name: &str,
        input: &crate::core::Collection,
        args: &[crate::core::Collection],
        context: &crate::evaluator::EvaluationContext,
    ) -> Result<crate::core::Collection> {
        // Convert input collection to FhirPathValue
        let input_value = if input.is_empty() {
            FhirPathValue::Empty
        } else if input.len() == 1 {
            input.first().unwrap().clone()
        } else {
            FhirPathValue::Collection(input.clone())
        };

        // Convert evaluated arguments to FhirPathValue
        let args_value = if args.is_empty() {
            FhirPathValue::Collection(crate::core::Collection::empty())
        } else if args.len() == 1 {
            if args[0].is_empty() {
                FhirPathValue::Empty
            } else if args[0].len() == 1 {
                args[0].first().unwrap().clone()
            } else {
                FhirPathValue::Collection(args[0].clone())
            }
        } else {
            // Multiple arguments - convert to collection of collections
            let arg_values: Vec<FhirPathValue> = args.iter()
                .map(|arg_collection| {
                    if arg_collection.is_empty() {
                        FhirPathValue::Empty
                    } else if arg_collection.len() == 1 {
                        arg_collection.first().unwrap().clone()
                    } else {
                        FhirPathValue::Collection(arg_collection.clone())
                    }
                })
                .collect();
            FhirPathValue::Collection(crate::core::Collection::from_values(arg_values))
        };

        // Create function context
        let function_context = FunctionContext {
            input: input_value,
            arguments: args_value,
            context,
            is_fhir_navigation: false,
        };

        // Try sync function first
        if let Some((function, _metadata)) = self.get_sync_function(name) {
            let result = function(&function_context)?;
            return Ok(self.fhirpath_value_to_collection(result));
        }

        // Try async function
        if let Some((function, _metadata)) = self.get_async_function(name) {
            let result = function(&function_context).await?;
            return Ok(self.fhirpath_value_to_collection(result));
        }

        // Function not found
        Err(FhirPathError::evaluation_error(
            FP0054,
            format!("Unknown function: {}", name),
        ))
    }

    /// Convert FhirPathValue to Collection
    fn fhirpath_value_to_collection(&self, value: FhirPathValue) -> crate::core::Collection {
        match value {
            FhirPathValue::Empty => crate::core::Collection::empty(),
            FhirPathValue::Collection(collection) => collection,
            single_value => crate::core::Collection::single(single_value),
        }
    }

    pub fn list_functions_by_category(&self, category: FunctionCategory) -> Vec<FunctionMetadata> {
        self.list_functions()
            .into_iter()
            .filter(|metadata| metadata.category == category)
            .collect()
    }

    /// Validate that context provides required providers for a function
    pub fn validate_function_providers(&self, function_name: &str, context: &crate::evaluator::EvaluationContext) -> Result<()> {
        let metadata = self.get_function_metadata(function_name)
            .ok_or_else(|| FhirPathError::evaluation_error(
                FP0054,
                format!("Unknown function: '{}'", function_name)
            ))?;

        // Validate ModelProvider requirement (always available)
        if metadata.requires_model_provider {
            // ModelProvider is always available through context, so no check needed
        }

        // Validate TerminologyProvider requirement
        if metadata.requires_terminology_provider && !context.has_terminology_provider() {
            return Err(FhirPathError::evaluation_error(
                FP0054,
                format!("Function '{}' requires terminology provider but none configured", function_name)
            ));
        }

        Ok(())
    }

    /// Check if function requires providers
    pub fn requires_providers(&self, function_name: &str) -> Option<(bool, bool)> {
        self.get_function_metadata(function_name)
            .map(|m| (m.requires_model_provider, m.requires_terminology_provider))
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        let registry = Self::new();
        registry
            .register_default_functions()
            .expect("Failed to register default functions");
        registry
    }
}

/// Create a standard FHIRPath function registry with all built-in functions
pub async fn create_standard_registry() -> FunctionRegistry {
    FunctionRegistry::default()
}
