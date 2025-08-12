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

//! escape() function - escapes special characters

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
/// escape() function - escapes special characters
pub struct EscapeFunction;

#[async_trait]
impl AsyncFhirPathFunction for EscapeFunction {
    fn name(&self) -> &str {
        "escape"
    }
    fn human_friendly_name(&self) -> &str {
        "Escape"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "escape",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // escape() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(escape_type)) => {
                match escape_type.as_ref() {
                    "json" => {
                        let escaped = s
                            .chars()
                            .map(|c| match c {
                                '"' => r#"\""#.to_string(),
                                '\\' => r"\\".to_string(),
                                '\n' => r"\n".to_string(),
                                '\r' => r"\r".to_string(),
                                '\t' => r"\t".to_string(),
                                _ => c.to_string(),
                            })
                            .collect::<String>();
                        Ok(FhirPathValue::String(escaped.into()))
                    }
                    "html" => {
                        let escaped = s
                            .chars()
                            .map(|c| match c {
                                '<' => "&lt;".to_string(),
                                '>' => "&gt;".to_string(),
                                '&' => "&amp;".to_string(),
                                '"' => "&quot;".to_string(),
                                '\'' => "&#39;".to_string(),
                                _ => c.to_string(),
                            })
                            .collect::<String>();
                        Ok(FhirPathValue::String(escaped.into()))
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Unsupported escape type: {escape_type}"),
                    }),
                }
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
