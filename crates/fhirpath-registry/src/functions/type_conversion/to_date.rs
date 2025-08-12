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

//! toDate() function - converts value to date

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use chrono::NaiveDate;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// toDate() function - converts value to date
pub struct ToDateFunction;

#[async_trait]
impl AsyncFhirPathFunction for ToDateFunction {
    fn name(&self) -> &str {
        "toDate"
    }
    fn human_friendly_name(&self) -> &str {
        "To Date"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature::new("toDate", vec![], TypeInfo::Date));
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toDate() is a pure type conversion function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Extract single item from collection according to spec
        let input_item = match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Input collection contains multiple items".to_string(),
                    });
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    items.get(0).unwrap()
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            item => item,
        };

        match input_item {
            FhirPathValue::Date(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Date(*d)])),
            FhirPathValue::DateTime(dt) => {
                // Convert DateTime to Date by extracting just the date part
                let date = dt.naive_local().date();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Date(date)]))
            }
            FhirPathValue::String(s) => {
                // Try to parse string as date using format YYYY-MM-DD
                // Also support partial dates: YYYY and YYYY-MM
                match parse_date_string(s) {
                    Some(date) => Ok(FhirPathValue::collection(vec![FhirPathValue::Date(date)])),
                    None => Ok(FhirPathValue::Empty),
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// Parse a date string supporting YYYY, YYYY-MM, and YYYY-MM-DD formats
fn parse_date_string(s: &str) -> Option<NaiveDate> {
    // Try full format first: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(date);
    }

    // Try year-month format: YYYY-MM (assume first day)
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d") {
        return Some(date);
    }

    // Try year format: YYYY (assume January 1)
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{s}-01-01"), "%Y-%m-%d") {
        return Some(date);
    }

    None
}
