//! Expression context for FHIRPath analysis
//!
//! This module provides context tracking during expression analysis,
//! including input types, variable scoping, and model provider access.

use std::collections::HashMap;
use std::sync::Arc;

use octofhir_fhir_model::{ModelProvider, TypeInfo};

/// Context for expression analysis that tracks types and variables
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    /// The input type for the current expression context
    pub input_type: TypeInfo,
    /// System variables ($this, $index, etc.)
    pub system_variables: HashMap<String, TypeInfo>,
    /// User-defined variables (%var)
    pub user_variables: HashMap<String, TypeInfo>,
    /// Model provider for type resolution
    pub model_provider: Option<Arc<dyn ModelProvider + Send + Sync>>,
    /// Whether this is the head of a navigation chain
    pub is_chain_head: bool,
}

impl ExpressionContext {
    /// Create a new expression context with the given input type
    pub fn new(input_type: TypeInfo) -> Self {
        let mut system_variables = HashMap::new();

        // Add standard system variables
        system_variables.insert("$this".to_string(), input_type.clone());
        system_variables.insert(
            "$index".to_string(),
            TypeInfo {
                type_name: "Integer".to_string(),
                singleton: Some(true),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Integer".to_string()),
            },
        );
        system_variables.insert(
            "$total".to_string(),
            TypeInfo {
                type_name: "Any".to_string(),
                singleton: Some(false),
                is_empty: Some(false),
                namespace: Some("System".to_string()),
                name: Some("Any".to_string()),
            },
        );

        Self {
            input_type,
            system_variables,
            user_variables: HashMap::new(),
            model_provider: None,
            is_chain_head: true,
        }
    }

    /// Create a new context with a model provider
    pub fn with_model_provider(mut self, model_provider: Arc<dyn ModelProvider + Send + Sync>) -> Self {
        self.model_provider = Some(model_provider);
        self
    }

    /// Create a new context with updated input type
    pub fn with_input_type(&self, input_type: TypeInfo) -> Self {
        let mut new_context = self.clone();
        new_context.input_type = input_type.clone();
        new_context
            .system_variables
            .insert("$this".to_string(), input_type);
        new_context.is_chain_head = false;
        new_context
    }

    /// Fork the context for parallel analysis (like union operations)
    pub fn fork(&self) -> Self {
        self.clone()
    }

    /// Add a user variable to the context
    pub fn with_user_variable(&mut self, name: String, type_info: TypeInfo) {
        self.user_variables.insert(name, type_info);
    }

    /// Add a system variable to the context
    pub fn with_system_variable(&mut self, name: String, type_info: TypeInfo) {
        self.system_variables.insert(name, type_info);
    }

    /// Get a system variable type
    pub fn get_system_variable(&self, name: &str) -> Option<&TypeInfo> {
        self.system_variables.get(name)
    }

    /// Get a user variable type
    pub fn get_user_variable(&self, name: &str) -> Option<&TypeInfo> {
        self.user_variables.get(name)
    }

    /// Check if we have a model provider
    pub fn has_model_provider(&self) -> bool {
        self.model_provider.is_some()
    }

    /// Get the model provider if available
    pub fn get_model_provider(&self) -> Option<&Arc<dyn ModelProvider + Send + Sync>> {
        self.model_provider.as_ref()
    }
}

impl Default for ExpressionContext {
    fn default() -> Self {
        Self::new(TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(false),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        })
    }
}
