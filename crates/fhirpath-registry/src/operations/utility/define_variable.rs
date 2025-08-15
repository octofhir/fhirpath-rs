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

//! DefineVariable function implementation - creates scoped variables

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// DefineVariable function - creates a variable in the current scope
#[derive(Debug, Clone)]
pub struct DefineVariableFunction;

impl DefineVariableFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("defineVariable", OperationType::Function)
            .description("Defines a variable with a name and value in the current scope")
            .returns(TypeConstraint::Any)
            .example("defineVariable('name', 'value').select(%name)")
            .example("defineVariable('patient', Patient).name.defineVariable('firstName', %patient.name.given.first())")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for DefineVariableFunction {
    fn identifier(&self) -> &str {
        "defineVariable"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            DefineVariableFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(args, context) {
            return result;
        }

        self.evaluate_define_variable(args, context)
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(self.evaluate_define_variable(args, context))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DefineVariableFunction {
    fn evaluate_define_variable(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments: defineVariable(name, value)
        if args.len() != 2 {
            return Err(FhirPathError::EvaluationError {
                message: "defineVariable() requires exactly 2 arguments (name, value)".to_string(),
            });
        }

        // Extract variable name
        let var_name = match &args[0] {
            FhirPathValue::String(name) => name.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.first().unwrap() {
                    FhirPathValue::String(name) => name.as_ref(),
                    _ => return Err(FhirPathError::EvaluationError {
                        message: "defineVariable() name parameter must be a string".to_string(),
                    }),
                }
            },
            _ => return Err(FhirPathError::EvaluationError {
                message: "defineVariable() name parameter must be a string".to_string(),
            }),
        };

        // Extract variable value
        let var_value = args[1].clone();

        // Create a new context with the variable defined
        let mut new_context = context.clone();
        new_context.set_variable(var_name.to_string(), var_value);

        // Return the current input (for chaining) but with new variable context
        // Note: In a full implementation, this would need to return a special context
        // that carries the variable scope forward for subsequent operations
        Ok(context.input.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::provider::MockModelProvider;
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use crate::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_define_variable_basic() {
        let define_var_fn = DefineVariableFunction::new();
        
        let input = FhirPathValue::String("test_input".into());
        let context = create_test_context(input.clone());
        
        let args = vec![
            FhirPathValue::String("myVar".into()),
            FhirPathValue::String("myValue".into())
        ];
        
        let result = define_var_fn.evaluate(&args, &context).await.unwrap();
        
        // Should return the input for chaining
        assert_eq!(result, input);
    }

    #[tokio::test]
    async fn test_define_variable_error_conditions() {
        let define_var_fn = DefineVariableFunction::new();
        let context = create_test_context(FhirPathValue::Empty);
        
        // Test wrong number of arguments
        let args = vec![FhirPathValue::String("var".into())];
        let result = define_var_fn.evaluate(&args, &context).await;
        assert!(result.is_err());
        
        // Test non-string variable name
        let args = vec![
            FhirPathValue::Integer(42),
            FhirPathValue::String("value".into())
        ];
        let result = define_var_fn.evaluate(&args, &context).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_evaluation() {
        let define_var_fn = DefineVariableFunction::new();
        let input = FhirPathValue::String("test".into());
        let context = create_test_context(input.clone());
        
        let args = vec![
            FhirPathValue::String("var".into()),
            FhirPathValue::String("value".into())
        ];
        
        let sync_result = define_var_fn.try_evaluate_sync(&args, &context).unwrap().unwrap();
        assert_eq!(sync_result, input);
        assert!(define_var_fn.supports_sync());
    }
}