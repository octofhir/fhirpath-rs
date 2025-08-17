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

//! Integer conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use rust_decimal::prelude::ToPrimitive;

/// ToInteger function: converts input to Integer
pub struct ToIntegerFunction;

impl Default for ToIntegerFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToIntegerFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toInteger", OperationType::Function)
            .description("Converts input to Integer")
            .example("'1'.toInteger()")
            .example("true.toInteger()")
            .returns(TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn convert_to_integer(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Boolean(b) => {
                if *b {
                    Ok(FhirPathValue::Integer(1))
                } else {
                    Ok(FhirPathValue::Integer(0))
                }
            }
            FhirPathValue::Decimal(d) => {
                // Check if decimal has no fractional part
                if d.fract().is_zero() {
                    // Try to convert to i64
                    if let Some(i) = d.to_i64() {
                        Ok(FhirPathValue::Integer(i))
                    } else {
                        // Cannot represent as i64
                        Ok(FhirPathValue::Empty)
                    }
                } else {
                    // Has fractional part, cannot convert
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::String(s) => {
                // Try to parse as integer
                match s.trim().parse::<i64>() {
                    Ok(i) => Ok(FhirPathValue::Integer(i)),
                    Err(_) => Ok(FhirPathValue::Empty), // Cannot convert
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else if c.len() == 1 {
                    Self::convert_to_integer(c.first().unwrap())
                } else {
                    // Multiple items is an error
                    Err(FhirPathError::EvaluationError {
                        expression: None,
                        location: None,
                        message:
                            "toInteger() requires a single item, but collection has multiple items"
                                .to_string(),
                    })
                }
            }
            _ => Ok(FhirPathValue::Empty), // Cannot convert
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToIntegerFunction {
    fn identifier(&self) -> &str {
        "toInteger"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToIntegerFunction::create_metadata);
        &METADATA
    }

    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(_args, context) {
            return result;
        }

        Self::convert_to_integer(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_integer(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
