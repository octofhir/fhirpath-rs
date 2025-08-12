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

//! replaceMatches() function - regex replacement

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
use regex::Regex;

/// replaceMatches() function - regex replacement
pub struct ReplaceMatchesFunction;

#[async_trait]
impl AsyncFhirPathFunction for ReplaceMatchesFunction {
    fn name(&self) -> &str {
        "replaceMatches"
    }
    fn human_friendly_name(&self) -> &str {
        "Replace Matches"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replaceMatches",
                vec![
                    ParameterInfo::required("pattern", TypeInfo::String),
                    ParameterInfo::required("substitution", TypeInfo::String),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // replaceMatches() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match (&context.input, &args[0], &args[1]) {
            (
                FhirPathValue::String(s),
                FhirPathValue::String(pattern),
                FhirPathValue::String(substitution),
            ) => {
                // Handle empty pattern - return original string unchanged
                if pattern.as_ref().is_empty() {
                    return Ok(FhirPathValue::String(s.clone()));
                }

                match Regex::new(pattern.as_ref()) {
                    Ok(re) => Ok(FhirPathValue::String(
                        re.replace_all(s.as_ref(), substitution.as_ref())
                            .to_string()
                            .into(),
                    )),
                    Err(e) => Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Invalid regex pattern: {e}"),
                    }),
                }
            }
            (FhirPathValue::Empty, _, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _, _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            (_, _, FhirPathValue::Empty) => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items), _) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            (_, _, FhirPathValue::Collection(items)) if items.is_empty() => {
                Ok(FhirPathValue::Empty)
            }
            _ => Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", context.input),
            }),
        }
    }
}
