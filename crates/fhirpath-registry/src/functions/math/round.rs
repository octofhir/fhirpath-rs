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

//! round() function - rounds to nearest integer

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// round() function - rounds to nearest integer
pub struct RoundFunction;

#[async_trait]
impl AsyncFhirPathFunction for RoundFunction {
    fn name(&self) -> &str {
        "round"
    }
    fn human_friendly_name(&self) -> &str {
        "Round"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "round",
                vec![ParameterInfo::optional("precision", TypeInfo::Integer)],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // round() is a pure mathematical function
    }

    fn documentation(&self) -> &str {
        "Rounds the decimal to the nearest whole number using a traditional round (i.e. 0.5 or higher will round to 1). If specified, the precision argument determines the decimal place at which the rounding will occur."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle single-element collections (common in method calls like (1.5).round())
        let input_value = match &context.input {
            FhirPathValue::Collection(items) if items.len() == 1 => items.get(0).unwrap(),
            other => other,
        };

        match input_value {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => {
                if let Some(FhirPathValue::Integer(precision)) = args.first() {
                    Ok(FhirPathValue::Decimal(d.round_dp(*precision as u32)))
                } else {
                    Ok(FhirPathValue::Integer(d.round().to_i64().unwrap_or(0)))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Number".to_string(),
                actual: format!("{input_value:?}"),
            }),
        }
    }
}
