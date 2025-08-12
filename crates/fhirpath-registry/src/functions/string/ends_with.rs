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

//! endsWith() function - checks if string ends with suffix

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// endsWith() function - checks if string ends with suffix
pub struct EndsWithFunction;

#[async_trait]
impl AsyncFhirPathFunction for EndsWithFunction {
    fn name(&self) -> &str {
        "endsWith"
    }
    fn human_friendly_name(&self) -> &str {
        "Ends With"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "endsWith",
                vec![ParameterInfo::required("suffix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // endsWith() is a pure string function
    }

    fn documentation(&self) -> &str {
        "Returns `true` when the input string ends with the given `suffix`. If `suffix` is the empty string (''), the result is `true`."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        match (&context.input, &args[0]) {
            (FhirPathValue::String(s), FhirPathValue::String(suffix)) => Ok(
                FhirPathValue::Boolean(s.as_ref().ends_with(suffix.as_ref())),
            ),
            (FhirPathValue::Empty, _) => Ok(FhirPathValue::Empty),
            // Handle empty collections - return empty when any parameter is an empty collection
            (FhirPathValue::Collection(items), _) if items.is_empty() => Ok(FhirPathValue::Empty),
            (_, FhirPathValue::Collection(items)) if items.is_empty() => Ok(FhirPathValue::Empty),
            // Return empty for non-string inputs instead of throwing error (per FHIRPath spec)
            _ => Ok(FhirPathValue::Empty),
        }
    }
}
