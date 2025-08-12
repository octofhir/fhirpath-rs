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

//! truncate() function - truncates decimal places

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;

/// truncate() function - truncates decimal places
pub struct TruncateFunction;

#[async_trait]
impl AsyncFhirPathFunction for TruncateFunction {
    fn name(&self) -> &str {
        "truncate"
    }
    fn human_friendly_name(&self) -> &str {
        "Truncate"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("truncate", vec![], TypeInfo::Integer)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // truncate() is a pure mathematical function
    }

    fn documentation(&self) -> &str {
        "Returns the integer portion of the input."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::Decimal(d) => {
                Ok(FhirPathValue::Integer(d.trunc().to_i64().unwrap_or(0)))
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
