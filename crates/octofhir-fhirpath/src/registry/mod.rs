//! FHIRPath function and operator registry
//!
//! This module provides the registry system for FHIRPath functions and operators,
//! enabling extensible function libraries and standard operation support.

use std::collections::HashMap;
use crate::core::{FhirPathError, Result, Collection};

/// Function signature for FHIRPath functions
pub type FunctionImpl = fn(&[Collection]) -> Result<Collection>;

/// Registry for FHIRPath functions and operators
#[derive(Debug)]
pub struct FunctionRegistry {
    /// Registered functions by name
    functions: HashMap<String, FunctionImpl>,
}

impl FunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register_function(&mut self, name: impl Into<String>, func: FunctionImpl) {
        self.functions.insert(name.into(), func);
    }

    /// Get a function by name
    pub fn get_function(&self, name: &str) -> Option<&FunctionImpl> {
        self.functions.get(name)
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names
    pub fn function_names(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a standard FHIRPath function registry with all built-in functions
pub async fn create_standard_registry() -> FunctionRegistry {
    let mut registry = FunctionRegistry::new();
    
    // TODO: Register all standard FHIRPath functions
    // For now, just create an empty registry
    
    registry
}
