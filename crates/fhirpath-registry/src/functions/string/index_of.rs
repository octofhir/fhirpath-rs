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

//! indexOf() function - finds index of substring

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
/// indexOf() function - finds index of substring
pub struct IndexOfFunction;

#[async_trait]
impl AsyncFhirPathFunction for IndexOfFunction {
    fn name(&self) -> &str {
        "indexOf"
    }
    fn human_friendly_name(&self) -> &str {
        "Index Of"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "indexOf",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Integer,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // indexOf() is a pure string function
    }

    fn documentation(&self) -> &str {
        "Returns the 0-based index of the first position `substring` is found in the input string, or -1 if it is not found."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(substring)) => {
                match s.as_ref().find(substring.as_ref()) {
                    Some(index) => Ok(FhirPathValue::Integer(index as i64)),
                    None => Ok(FhirPathValue::Integer(-1)),
                }
            }
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
