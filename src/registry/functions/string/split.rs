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

//! split() function - splits string by separator

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
/// split() function - splits string by separator
pub struct SplitFunction;

#[async_trait]
impl AsyncFhirPathFunction for SplitFunction {
    fn name(&self) -> &str {
        "split"
    }
    fn human_friendly_name(&self) -> &str {
        "Split"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "split",
                vec![ParameterInfo::required("separator", TypeInfo::String)],
                TypeInfo::collection(TypeInfo::String),
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // split() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(separator)) => {
                let parts: Vec<FhirPathValue> = s
                    .as_ref()
                    .split(separator.as_ref())
                    .map(|part| FhirPathValue::String(part.to_string().into()))
                    .collect();
                Ok(FhirPathValue::collection(parts))
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
