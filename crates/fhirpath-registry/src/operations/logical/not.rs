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

//! Logical NOT operator implementation

use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{
        FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
        TypeConstraint,
    },
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Logical NOT operator
#[derive(Debug, Clone)]
pub struct NotOperation;

impl Default for NotOperation {
    fn default() -> Self {
        Self::new()
    }
}

impl NotOperation {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("not", OperationType::Function)
            .description("Logical NOT function with three-valued logic")
            .example("true.not()")
            .example("false.not()")
            .example("empty().not()")
            .returns(TypeConstraint::Specific(FhirPathType::Boolean))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn to_boolean(value: &FhirPathValue) -> Result<Option<bool>> {
        match value {
            FhirPathValue::Empty => Ok(None),
            FhirPathValue::Boolean(b) => Ok(Some(*b)),
            FhirPathValue::Integer(i) => Ok(Some(*i != 0)), // 0 = false, non-zero = true
            FhirPathValue::Decimal(d) => Ok(Some(!d.is_zero())), // 0.0 = false, non-zero = true
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(None)
                } else if c.len() == 1 {
                    Self::to_boolean(c.first().unwrap())
                } else {
                    Err(FhirPathError::TypeError {
                        message: "Cannot convert collection with multiple items to boolean"
                            .to_string(),
                    })
                }
            }
            _ => Err(FhirPathError::TypeError {
                message: format!("Cannot convert {} to boolean", value.type_name()),
            }),
        }
    }
}

#[async_trait]
impl FhirPathOperation for NotOperation {
    fn identifier(&self) -> &str {
        "not"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(NotOperation::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        let value = Self::to_boolean(&context.input)?;

        // Three-valued logic for NOT
        let result = match value {
            Some(true) => Some(false),
            Some(false) => Some(true),
            None => None, // NOT of empty is empty
        };

        match result {
            Some(b) => Ok(FhirPathValue::Boolean(b)),
            None => Ok(FhirPathValue::Empty),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            }));
        }

        match Self::to_boolean(&context.input) {
            Ok(value) => {
                let result = match value {
                    Some(true) => Some(false),
                    Some(false) => Some(true),
                    None => None,
                };

                match result {
                    Some(b) => Some(Ok(FhirPathValue::Boolean(b))),
                    None => Some(Ok(FhirPathValue::Empty)),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len(),
            });
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
