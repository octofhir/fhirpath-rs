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

//! SecondOf function implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::Timelike;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::any::Any;

/// SecondOf function - extracts second component from DateTime or Time (0-59)
#[derive(Debug, Clone)]
pub struct SecondOfFunction;

impl Default for SecondOfFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SecondOfFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("secondOf", OperationType::Function)
            .description("Extract second component (0-59) from DateTime or Time value")
            .example("@2023-05-15T14:30:45.secondOf() = 45")
            .example("DiagnosticReport.issued.secondOf()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SecondOfFunction {
    fn identifier(&self) -> &str {
        "secondOf"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(SecondOfFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "secondOf() takes no arguments".to_string(),
            });
        }

        let mut results = Vec::new();

        for value in context.input.clone().to_collection().iter() {
            match value {
                FhirPathValue::DateTime(datetime) => {
                    results.push(FhirPathValue::Integer(datetime.datetime.second() as i64));
                }
                FhirPathValue::Time(time) => {
                    results.push(FhirPathValue::Integer(time.time.second() as i64));
                }
                _ => {
                    return Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message: "secondOf() can only be called on DateTime or Time values"
                            .to_string(),
                    });
                }
            }
        }

        Ok(FhirPathValue::Collection(results.into()))
    }

    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        if !args.is_empty() {
            return Err(FhirPathError::EvaluationError {
                expression: None,
                location: None,
                message: "secondOf() takes no arguments".to_string(),
            });
        }
        Ok(())
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(futures::executor::block_on(self.evaluate(args, context)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
