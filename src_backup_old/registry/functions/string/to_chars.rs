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

//! toChars() function - converts string to array of single characters

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;
/// toChars() function - converts string to array of single characters
pub struct ToCharsFunction;

#[async_trait]
impl AsyncFhirPathFunction for ToCharsFunction {
    fn name(&self) -> &str {
        "toChars"
    }
    fn human_friendly_name(&self) -> &str {
        "To Chars"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toChars", vec![], TypeInfo::collection(TypeInfo::String))
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // toChars() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match &context.input {
            FhirPathValue::String(s) => {
                let chars: Vec<FhirPathValue> = s
                    .as_ref()
                    .chars()
                    .map(|c| FhirPathValue::String(c.to_string().into()))
                    .collect();
                Ok(FhirPathValue::collection(chars))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
