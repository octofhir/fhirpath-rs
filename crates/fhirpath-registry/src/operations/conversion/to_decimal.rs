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

//! Decimal conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;
use regex::Regex;
use rust_decimal::Decimal;
use std::sync::OnceLock;

/// ToDecimal function: converts input to Decimal
pub struct ToDecimalFunction;

impl Default for ToDecimalFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl ToDecimalFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toDecimal", OperationType::Function)
            .description("If the input collection contains a single item, this function will return a single decimal if the item is convertible")
            .example("'1.5'.toDecimal()")
            .example("true.toDecimal()")
            .example("42.toDecimal()")
            .returns(TypeConstraint::Specific(FhirPathType::Decimal))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn decimal_regex() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            // Match: optional sign, digits, optional decimal point and digits
            Regex::new(r"^(\+|-)?\d+(\.\d+)?$").unwrap()
        })
    }

    fn convert_to_decimal(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            // Already a decimal
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),

            // Integer conversion
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(Decimal::from(*i))),

            // Boolean conversion
            FhirPathValue::Boolean(b) => {
                if *b {
                    Ok(FhirPathValue::Decimal(Decimal::from(1)))
                } else {
                    Ok(FhirPathValue::Decimal(Decimal::from(0)))
                }
            }

            // String conversion with validation
            FhirPathValue::String(s) => {
                let trimmed = s.trim();
                if Self::decimal_regex().is_match(trimmed) {
                    match trimmed.parse::<Decimal>() {
                        Ok(d) => Ok(FhirPathValue::Decimal(d)),
                        Err(_) => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection on parse error
                    }
                } else {
                    Ok(FhirPathValue::Collection(vec![].into())) // Empty collection for invalid format
                }
            }

            // Empty input
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),

            // Collection handling
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Collection(vec![].into()))
                } else if c.len() == 1 {
                    Self::convert_to_decimal(c.first().unwrap())
                } else {
                    // Multiple items - return empty collection per FHIRPath spec
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }

            // Unsupported types
            _ => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection for unsupported types
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToDecimalFunction {
    fn identifier(&self) -> &str {
        "toDecimal"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(ToDecimalFunction::create_metadata);
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

        Self::convert_to_decimal(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_decimal(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
