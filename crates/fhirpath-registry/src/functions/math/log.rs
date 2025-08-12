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

//! log() function - logarithm with base

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use rust_decimal::prelude::*;

/// log() function - logarithm with base
pub struct LogFunction;

#[async_trait]
impl AsyncFhirPathFunction for LogFunction {
    fn name(&self) -> &str {
        "log"
    }
    fn human_friendly_name(&self) -> &str {
        "Logarithm"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "log",
                vec![ParameterInfo::required("base", TypeInfo::Any)],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // log() is a pure mathematical function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let base = match &args[0] {
            FhirPathValue::Integer(i) => *i as f64,
            FhirPathValue::Decimal(d) => d.to_f64().unwrap_or(0.0),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Number".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        if base <= 0.0 || base == 1.0 {
            return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Logarithm base must be positive and not equal to 1".to_string(),
            });
        }

        match &context.input {
            FhirPathValue::Integer(i) => {
                if *i <= 0 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot take logarithm of non-positive number".to_string(),
                    });
                }
                let result = (*i as f64).log(base);
                Ok(FhirPathValue::Decimal(
                    Decimal::from_f64(result).unwrap_or_default(),
                ))
            }
            FhirPathValue::Decimal(d) => {
                if d.is_sign_negative() || d.is_zero() {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Cannot take logarithm of non-positive number".to_string(),
                    });
                }
                let result = d.to_f64().unwrap_or(0.0).log(base);
                Ok(FhirPathValue::Decimal(
                    Decimal::from_f64(result).unwrap_or_default(),
                ))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Number".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
