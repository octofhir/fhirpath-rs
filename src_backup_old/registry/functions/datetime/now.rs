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

//! now() function - returns current date/time

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::FunctionSignature;
use async_trait::async_trait;
use chrono::Utc;

/// now() function - returns current date/time
pub struct NowFunction;

#[async_trait]
impl AsyncFhirPathFunction for NowFunction {
    fn name(&self) -> &str {
        "now"
    }
    fn human_friendly_name(&self) -> &str {
        "Now"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("now", vec![], TypeInfo::DateTime));
        &SIG
    }

    fn documentation(&self) -> &str {
        "Returns the current date and time, including timezone information."
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let now = Utc::now();
        Ok(FhirPathValue::DateTime(
            now.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        ))
    }
}
