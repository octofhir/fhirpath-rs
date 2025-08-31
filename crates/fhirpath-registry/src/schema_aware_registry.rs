//! Schema-Aware Function Registry
//!
//! This module provides an enhanced function registry with schema awareness
//! for improved type checking and FHIR compliance.

use crate::registry::{AsyncRegistry, SyncRegistry};
use crate::type_registry::{FhirPathTypeRegistry, RegistryError};
use octofhir_fhirschema::package::FhirSchemaPackageManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Enhanced function registry with schema awareness
pub struct SchemaAwareFunctionRegistry {
    #[allow(dead_code)]
    sync_registry: Arc<SyncRegistry>,
    #[allow(dead_code)]
    async_registry: Arc<AsyncRegistry>,
    type_registry: Arc<FhirPathTypeRegistry>,
    schema_manager: Arc<FhirSchemaPackageManager>,
    #[allow(dead_code)]
    function_cache: Arc<RwLock<HashMap<String, FunctionType>>>,
}

/// Function type for caching dispatch decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum FunctionType {
    Sync,
    Async,
}

impl SchemaAwareFunctionRegistry {
    /// Create a new schema-aware function registry
    pub async fn new(schema_manager: Arc<FhirSchemaPackageManager>) -> Result<Self, RegistryError> {
        let type_registry = Arc::new(FhirPathTypeRegistry::new(schema_manager.clone()).await?);

        let registry = Self {
            sync_registry: Arc::new(SyncRegistry::new()),
            async_registry: Arc::new(AsyncRegistry::new()),
            type_registry,
            schema_manager,
            function_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        Ok(registry)
    }

    /// Evaluate a function with smart sync/async dispatch (placeholder)
    pub async fn evaluate_function(
        &self,
        function_name: &str,
        args: &[octofhir_fhirpath_model::FhirPathValue],
        context: &crate::traits::EvaluationContext,
    ) -> Result<octofhir_fhirpath_model::FhirPathValue, octofhir_fhirpath_core::FhirPathError> {
        use octofhir_fhirpath_model::FhirPathValue;

        // Simple implementation for core type functions
        match function_name {
            "ofType" => {
                if args.len() != 1 {
                    return Err(
                        octofhir_fhirpath_core::FhirPathError::InvalidArgumentCount {
                            function_name: "ofType".to_string(),
                            expected: 1,
                            actual: args.len(),
                        },
                    );
                }

                let type_name = args[0].as_string().ok_or_else(|| {
                    octofhir_fhirpath_core::FhirPathError::TypeMismatch {
                        expected: "string".to_string(),
                        actual: "unknown".to_string(),
                        context: Some("ofType requires a string argument".to_string()),
                    }
                })?;

                // Use O(1) type registry for type checking
                if self.type_registry.is_resource_type(&type_name)
                    || self.type_registry.is_data_type(&type_name)
                {
                    // Filter by type - simplified implementation
                    if context.input.type_name() == type_name {
                        Ok(FhirPathValue::Collection(
                            vec![context.input.clone()].into(),
                        ))
                    } else {
                        Ok(FhirPathValue::Collection(vec![].into()))
                    }
                } else {
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }

            "is" => {
                if args.len() != 1 {
                    return Err(
                        octofhir_fhirpath_core::FhirPathError::InvalidArgumentCount {
                            function_name: "is".to_string(),
                            expected: 1,
                            actual: args.len(),
                        },
                    );
                }

                let type_name = args[0].as_string().ok_or_else(|| {
                    octofhir_fhirpath_core::FhirPathError::TypeMismatch {
                        expected: "string".to_string(),
                        actual: "unknown".to_string(),
                        context: Some("is requires a string argument".to_string()),
                    }
                })?;

                let is_type = context.input.type_name() == type_name;
                Ok(FhirPathValue::Boolean(is_type))
            }

            "as" => {
                if args.len() != 1 {
                    return Err(
                        octofhir_fhirpath_core::FhirPathError::InvalidArgumentCount {
                            function_name: "as".to_string(),
                            expected: 1,
                            actual: args.len(),
                        },
                    );
                }

                let type_name = args[0].as_string().ok_or_else(|| {
                    octofhir_fhirpath_core::FhirPathError::TypeMismatch {
                        expected: "string".to_string(),
                        actual: "unknown".to_string(),
                        context: Some("as requires a string argument".to_string()),
                    }
                })?;

                if context.input.type_name() == type_name {
                    Ok(FhirPathValue::Collection(
                        vec![context.input.clone()].into(),
                    ))
                } else {
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }

            "conformsTo" => {
                // Placeholder - always returns false for now
                Ok(FhirPathValue::Boolean(false))
            }

            _ => Err(octofhir_fhirpath_core::FhirPathError::EvaluationError {
                message: format!("Function '{}' not found", function_name),
                expression: Some(function_name.to_string()),
                location: None,
                error_type: Some("function_not_found".to_string()),
            }),
        }
    }

    /// Get the type registry for external use
    pub fn type_registry(&self) -> &Arc<FhirPathTypeRegistry> {
        &self.type_registry
    }

    /// Get the schema manager for external use
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}
