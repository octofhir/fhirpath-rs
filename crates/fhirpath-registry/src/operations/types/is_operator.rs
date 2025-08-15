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

//! Is binary operator implementation - type checking operator

use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType, PerformanceComplexity, Associativity}};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;

/// Is binary operator - checks if value is of specified type (x is Type syntax)
#[derive(Debug, Clone)]
pub struct IsBinaryOperator;

impl IsBinaryOperator {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("is", OperationType::BinaryOperator {
            precedence: 8,
            associativity: Associativity::Left,
        })
            .description("Type checking binary operator - returns true if the input is of the specified type")
            .example("Patient.active is Boolean")
            .example("Patient.name is Collection")  
            .example("Patient is Patient")
            .parameter("value", TypeConstraint::Any, false)
            .parameter("type", TypeConstraint::Specific(FhirPathType::String), false)
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    async fn check_type_with_provider(
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        // Reuse the same type checking logic from the function version
        crate::operations::types::is::IsOperation::check_type_with_provider(value, type_name, context).await
    }

    /// Handle both direct strings and single-element collections containing strings or identifiers
    fn extract_type_name(type_arg: &FhirPathValue) -> Result<String> {
        match type_arg {
            FhirPathValue::String(s) => Ok(s.as_ref().to_string()),
            FhirPathValue::Collection(items) => {
                match items.len() {
                    0 => Err(FhirPathError::TypeError {
                        message: "is operator type argument cannot be empty".to_string()
                    }),
                    1 => {
                        Self::extract_type_name(items.first().unwrap())
                    },
                    _ => Err(FhirPathError::TypeError {
                        message: "is operator type argument must be a single value".to_string()
                    }),
                }
            },
            // Handle TypeInfoObject (identifiers like Integer, String, etc.)
            FhirPathValue::TypeInfoObject { namespace, name } => {
                // For type identifiers, use just the name (e.g., "Integer" from "System.Integer")
                Ok(name.as_ref().to_string())
            },
            // Handle other value types that might represent identifiers
            value => {
                // Try to convert any value to its string representation
                // This handles cases where identifiers are parsed as other types
                match value.to_string_value() {
                    Some(s) => Ok(s),
                    None => Err(FhirPathError::TypeError {
                        message: format!("is operator type argument must be convertible to string, got {}", value.type_name())
                    }),
                }
            },
        }
    }
}

#[async_trait]
impl FhirPathOperation for IsBinaryOperator {
    fn identifier(&self) -> &str {
        "is"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::BinaryOperator {
            precedence: 8,
            associativity: Associativity::Left,
        }
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            IsBinaryOperator::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount { 
                function_name: self.identifier().to_string(), 
                expected: 2, 
                actual: args.len() 
            });
        }

        // First argument is the value, second is the type
        let value = &args[0];
        let type_name = Self::extract_type_name(&args[1])?;

        let result = Self::check_type_with_provider(value, &type_name, context).await?;

        Ok(FhirPathValue::Collection(Collection::from(vec![
            FhirPathValue::Boolean(result)
        ])))
    }

    fn try_evaluate_sync(&self, _args: &[FhirPathValue], _context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Type checking requires async ModelProvider calls, so cannot be done synchronously
        None
    }

    fn supports_sync(&self) -> bool {
        false  // Type checking requires async ModelProvider calls
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if args.len() != 2 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 2,
                actual: args.len(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}