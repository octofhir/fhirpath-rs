//! Function dispatch system for executing registered functions

use super::{FunctionRegistry, FunctionContext, FunctionMetadata};
use crate::core::{FhirPathValue, FhirPathError, Result};
use crate::core::error_code::{FP0053, FP0054};

pub struct FunctionDispatcher {
    registry: FunctionRegistry,
}

impl FunctionDispatcher {
    pub fn new(registry: FunctionRegistry) -> Self {
        Self { registry }
    }

    pub fn dispatch_sync(
        &self,
        function_name: &str,
        context: &FunctionContext,
    ) -> Result<Vec<FhirPathValue>> {
        match self.registry.get_sync_function(function_name) {
            Some((function, metadata)) => {
                // Validate parameters
                if let Err(err) = self.validate_parameters(context, &metadata) {
                    return Err(err);
                }

                // Execute function
                (function)(context)
            }
            None => {
                if self.registry.get_async_function(function_name).is_some() {
                    Err(FhirPathError::evaluation_error(
                        FP0054,
                        format!(
                            "Function '{}' is async and cannot be called in sync context",
                            function_name
                        ),
                    ))
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0054,
                        format!("Unknown function: '{}'", function_name),
                    ))
                }
            }
        }
    }

    pub async fn dispatch_async(
        &self,
        function_name: &str,
        context: &FunctionContext<'_>,
    ) -> Result<Vec<FhirPathValue>> {
        match self.registry.get_async_function(function_name) {
            Some((function, metadata)) => {
                // Validate parameters
                if let Err(err) = self.validate_parameters(context, &metadata) {
                    return Err(err);
                }

                // Execute async function
                (function)(context).await
            }
            None => {
                if let Some((sync_function, metadata)) = self.registry.get_sync_function(function_name) {
                    // Validate parameters
                    if let Err(err) = self.validate_parameters(context, &metadata) {
                        return Err(err);
                    }

                    // Execute sync function in async context
                    (sync_function)(context)
                } else {
                    Err(FhirPathError::evaluation_error(
                        FP0054,
                        format!("Unknown function: '{}'", function_name),
                    ))
                }
            }
        }
    }

    fn validate_parameters(
        &self,
        context: &FunctionContext,
        metadata: &FunctionMetadata,
    ) -> Result<()> {
        let required_params = metadata.parameters.iter().filter(|p| !p.is_optional).count();
        let provided_params = context.arguments.len();

        if provided_params < required_params {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                format!(
                    "Function '{}' requires {} parameters but {} were provided",
                    metadata.name, required_params, provided_params
                ),
            ));
        }

        if provided_params > metadata.parameters.len() {
            return Err(FhirPathError::evaluation_error(
                FP0053,
                format!(
                    "Function '{}' accepts at most {} parameters but {} were provided",
                    metadata.name, metadata.parameters.len(), provided_params
                ),
            ));
        }

        // Type validation can be added here
        Ok(())
    }

    pub fn get_registry(&self) -> &FunctionRegistry {
        &self.registry
    }

    pub fn get_function_help(&self, function_name: &str) -> Option<String> {
        self.registry.get_function_metadata(function_name).map(|metadata| {
            let mut help = format!("{}(", metadata.name);
            
            for (i, param) in metadata.parameters.iter().enumerate() {
                if i > 0 {
                    help.push_str(", ");
                }
                help.push_str(&param.name);
                if let Some(ref type_constraint) = param.type_constraint {
                    help.push_str(": ");
                    help.push_str(type_constraint);
                }
                if param.is_optional {
                    help.push('?');
                }
            }
            
            help.push(')');
            
            if let Some(ref return_type) = metadata.return_type {
                help.push_str(" -> ");
                help.push_str(return_type);
            }
            
            help.push_str("\n\n");
            help.push_str(&metadata.description);
            
            if !metadata.examples.is_empty() {
                help.push_str("\n\nExamples:\n");
                for example in &metadata.examples {
                    help.push_str("  ");
                    help.push_str(example);
                    help.push('\n');
                }
            }
            
            help
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FhirPathValue, ModelProvider};
    use crate::mock_provider::MockModelProvider;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_sync_function_dispatch() {
        let registry = FunctionRegistry::default();
        let dispatcher = FunctionDispatcher::new(registry);
        
        let model_provider = MockModelProvider::default();
        let variables = HashMap::new();
        let input = vec![];
        let arguments = vec![];
        
        let context = FunctionContext {
            input: &input,
            arguments: &arguments,
            model_provider: &model_provider,
            variables: &variables,
            resource_context: None,
            terminology: None,
        };
        
        // Test empty function
        let result = dispatcher.dispatch_sync("empty", &context);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], FhirPathValue::boolean(true)); // empty collection should return true
    }

    #[test]
    fn test_function_help() {
        let registry = FunctionRegistry::default();
        let dispatcher = FunctionDispatcher::new(registry);
        
        let help = dispatcher.get_function_help("empty");
        assert!(help.is_some());
        
        let help_text = help.unwrap();
        assert!(help_text.contains("empty()"));
        assert!(help_text.contains("Returns true if the input collection is empty"));
    }
}
