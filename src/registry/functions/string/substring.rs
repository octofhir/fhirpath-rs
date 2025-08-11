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

//! substring() function - extracts a substring

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
/// substring() function - extracts a substring
pub struct SubstringFunction;

#[async_trait]
impl AsyncFhirPathFunction for SubstringFunction {
    fn name(&self) -> &str {
        "substring"
    }
    fn human_friendly_name(&self) -> &str {
        "Substring"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "substring",
                vec![
                    ParameterInfo::required("start", TypeInfo::Integer),
                    ParameterInfo::optional("length", TypeInfo::Integer),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // substring() is a pure string function
    }

    fn documentation(&self) -> &str {
        "Returns the part of the string starting at position `start` (zero-based). If `length` is given, will return at most `length` number of characters from the input string. If `start` lies outside the length of the string, the function returns empty (`{ }`). If there are less remaining characters in the string than indicated by `length`, the function returns just the remaining characters."
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty collections in arguments
        if let Some(FhirPathValue::Collection(items)) = args.first() {
            if items.is_empty() {
                return Ok(FhirPathValue::Empty);
            }
        }
        if let Some(FhirPathValue::Collection(items)) = args.get(1) {
            if items.is_empty() {
                return Ok(FhirPathValue::Empty);
            }
        }

        let input_string = match &context.input {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            FhirPathValue::Resource(r) => {
                // Try to extract string value from FhirResource
                match r.as_json() {
                    serde_json::Value::String(s) => s.clone(),
                    _ => return Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) if items.is_empty() => {
                return Ok(FhirPathValue::Empty);
            }
            _ => return Ok(FhirPathValue::Empty),
        };

        let start_int = match &args[0] {
            FhirPathValue::Integer(i) => *i,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Handle negative indices and out of bounds - return empty string
        if start_int < 0 {
            return Ok(FhirPathValue::Empty);
        }

        let start = start_int as usize;
        let chars: Vec<char> = input_string.chars().collect();

        if start >= chars.len() {
            return Ok(FhirPathValue::Empty);
        }

        let result = if let Some(length_arg) = args.get(1) {
            match length_arg {
                FhirPathValue::Integer(len_int) => {
                    if *len_int < 0 {
                        return Ok(FhirPathValue::Empty);
                    }
                    let len = *len_int as usize;
                    chars.iter().skip(start).take(len).collect::<String>()
                }
                FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
                _ => {
                    return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: 1,
                        expected: "Integer".to_string(),
                        actual: format!("{length_arg:?}"),
                    });
                }
            }
        } else {
            chars.iter().skip(start).collect::<String>()
        };

        Ok(FhirPathValue::String(result.into()))
    }
}
