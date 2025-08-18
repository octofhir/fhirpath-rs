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

//! toString() implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// toString(): Converts input to String where possible
pub struct ToStringFunction;

impl Default for ToStringFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToStringFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toString", OperationType::Function)
            .description("If the input collection contains a single item, this function will return a single String representation")
            .example("true.toString() // returns 'true'")
            .example("123.toString() // returns '123'")
            .example("123.45.toString() // returns '123.45'")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn convert_to_string(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            // Already a string
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),

            // Integer conversion
            FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string().into())),

            // Decimal conversion with proper formatting
            FhirPathValue::Decimal(d) => {
                // Format decimal according to FHIRPath specification
                let formatted = Self::format_decimal(*d);
                Ok(FhirPathValue::String(formatted.into()))
            }

            // Boolean conversion
            FhirPathValue::Boolean(b) => {
                let s = if *b { "true" } else { "false" };
                Ok(FhirPathValue::String(s.into()))
            }

            // Date conversion
            FhirPathValue::Date(d) => Ok(FhirPathValue::String(d.to_string().into())),

            // DateTime conversion
            FhirPathValue::DateTime(dt) => Ok(FhirPathValue::String(dt.to_string().into())),

            // Time conversion
            FhirPathValue::Time(t) => Ok(FhirPathValue::String(t.to_string().into())),

            // Quantity conversion
            FhirPathValue::Quantity(q) => {
                let formatted_value = Self::format_decimal(q.value);
                let result = if let Some(unit) = &q.unit {
                    // Only quote UCUM units, leave standard units unquoted
                    match unit.as_str() {
                        "wk" | "mo" | "a" | "d" => format!("{formatted_value} '{unit}'"),
                        _ => format!("{formatted_value} {unit}"),
                    }
                } else {
                    formatted_value
                };
                Ok(FhirPathValue::String(result.into()))
            }

            // Collection handling
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Collection(vec![].into()))
                } else if c.len() == 1 {
                    Self::convert_to_string(c.first().unwrap())
                } else {
                    // Multiple items - return empty collection per FHIRPath spec
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }

            // Empty input
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

            // Unsupported types
            _ => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection for unsupported types
        }
    }

    fn format_decimal(decimal: rust_decimal::Decimal) -> String {
        // Format decimal according to FHIRPath specification
        let s = decimal.to_string();

        // Remove trailing zeros after decimal point
        if s.contains('.') {
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            if trimmed.is_empty() {
                "0".to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            s
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToStringFunction {
    fn identifier(&self) -> &str {
        "toString"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToStringFunction::create_metadata);
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
        Self::convert_to_string(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_string(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
