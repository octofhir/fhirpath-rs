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

//! today() function - returns current date

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use chrono::Local;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// today() function - returns current date
pub struct TodayFunction;

#[async_trait]
impl AsyncFhirPathFunction for TodayFunction {
    fn name(&self) -> &str {
        "today"
    }
    fn human_friendly_name(&self) -> &str {
        "Today"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("today", vec![], TypeInfo::Date));
        &SIG
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let today = Local::now().date_naive();
        Ok(FhirPathValue::Date(today))
    }
}
