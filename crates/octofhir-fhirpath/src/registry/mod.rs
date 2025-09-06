//! FHIRPath function and operator registry
//!
//! This module provides a comprehensive registry system for FHIRPath functions,
//! supporting both synchronous and asynchronous functions with metadata,
//! validation, and dispatch capabilities.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use crate::core::{
    FhirPathError, FhirPathValue, ModelProvider, Result,
    error_code::FP0054,
};
use crate::evaluator::TerminologyService;

pub use terminology_provider::{
    ConceptDetails, DefaultTerminologyProvider, MockTerminologyProvider, TerminologyProvider,
};
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
pub mod fhir_utils;
pub mod logic;
pub mod math;
pub mod numeric;
pub mod string;
pub mod terminology;
pub mod terminology_provider;
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

/// Function execution context
pub struct FunctionContext<'a> {
    pub input: &'a [FhirPathValue],
    pub arguments: &'a [FhirPathValue],
    pub model_provider: &'a dyn ModelProvider,
    pub variables: &'a HashMap<String, FhirPathValue>,
    pub resource_context: Option<&'a FhirPathValue>,
    /// Optional terminology service available to functions needing terminology operations
    pub terminology: Option<&'a dyn TerminologyService>,
}

/// Sync function signature
pub type SyncFunction = Arc<dyn Fn(&FunctionContext) -> Result<Vec<FhirPathValue>> + Send + Sync>;

/// Async function signature
pub type AsyncFunction = Arc<
    dyn for<'a> Fn(
            &'a FunctionContext<'a>,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<FhirPathValue>>> + Send + 'a>>
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

    pub fn list_functions_by_category(&self, category: FunctionCategory) -> Vec<FunctionMetadata> {
        self.list_functions()
            .into_iter()
            .filter(|metadata| metadata.category == category)
            .collect()
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
