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

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// DefineVariable function - creates a variable in the current scope
#[derive(Debug, Clone)]
pub struct DefineVariableFunction;

impl Default for DefineVariableFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl DefineVariableFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("defineVariable", OperationType::Function)
            .description(
                "Defines a variable with a name and optionally a value in the current scope",
            )
            .parameter(
                "name",
                TypeConstraint::Specific(crate::metadata::FhirPathType::String),
                false,
            )
            .parameter("value", TypeConstraint::Any, true)
            .returns(TypeConstraint::Any)
            .example("defineVariable('name', 'value').select(%name)")
            .example("defineVariable('current').select(%current)")
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
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(DefineVariableFunction::create_metadata);
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
        // Validate arguments: defineVariable(name) or defineVariable(name, value)
        if args.is_empty() || args.len() > 2 {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "defineVariable() requires 1 or 2 arguments (name, [value])".to_string(),
            });
        }

        // Extract variable name
        let var_name = match &args[0] {
            FhirPathValue::String(name) => name.as_ref(),
            FhirPathValue::Collection(items) if items.len() == 1 => match items.first().unwrap() {
                FhirPathValue::String(name) => name.as_ref(),
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "defineVariable() name parameter must be a string".to_string(),
                    });
                }
            },
            _ => {
                return Err(FhirPathError::EvaluationError {
                    expression: None,
                    location: None,
                    message: "defineVariable() name parameter must be a string".to_string(),
                });
            }
        };

        // Check if the variable name is a system variable (protected)
        if Self::is_system_variable(var_name) {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: format!("Cannot override system variable '{var_name}'"),
            });
        }

        // Extract variable value - use current context if not provided
        let var_value = if args.len() == 2 {
            args[1].clone()
        } else {
            context.input.clone()
        };

        // Set the variable in the current context
        // Note: This approach updates the context that will be used for subsequent operations
        let mut updated_context = context.clone();
        updated_context
            .variables
            .insert(var_name.to_string(), var_value);

        // Return the current input for chaining
        // The variable context modification should be handled at the evaluator level
        Ok(context.input.clone())
    }

    /// Check if a variable name is a system variable that cannot be overridden
    fn is_system_variable(name: &str) -> bool {
        match name {
            // Standard environment variables
            "context" | "resource" | "rootResource" | "sct" | "loinc" | "ucum" => true,
            // Lambda variables
            "this" | "$this" | "index" | "$index" | "total" | "$total" => true,
            // Value set variables (with or without quotes)
            name if name.starts_with("\"vs-") && name.ends_with('"') => true,
            name if name.starts_with("vs-") => true,
            // Extension variables (with or without quotes)
            name if name.starts_with("\"ext-") && name.ends_with('"') => true,
            name if name.starts_with("ext-") => true,
            _ => false,
        }
    }
}
