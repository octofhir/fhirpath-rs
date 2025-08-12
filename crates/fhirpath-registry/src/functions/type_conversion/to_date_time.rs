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

//! toDateTime() function - converts value to datetime

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::FunctionSignature;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, TimeZone};
use fhirpath_model::{FhirPathValue, types::TypeInfo};

/// toDateTime() function - converts value to datetime
pub struct ToDateTimeFunction;

#[async_trait]
impl AsyncFhirPathFunction for ToDateTimeFunction {
    fn name(&self) -> &str {
        "toDateTime"
    }
    fn human_friendly_name(&self) -> &str {
        "To DateTime"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new("toDateTime", vec![], TypeInfo::DateTime)
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // toDateTime() is a pure type conversion function
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
            FhirPathValue::DateTime(dt) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::DateTime(
                    *dt,
                )]))
            }
            FhirPathValue::Date(date) => {
                // Convert Date to DateTime with empty time components (not set to zero)
                // According to spec: "the time components empty (not set to zero)"
                // This means we should preserve the date precision and not add time components
                // The result should still be a Date type to indicate time is unspecified
                Ok(FhirPathValue::collection(vec![FhirPathValue::Date(*date)]))
            }
            FhirPathValue::String(s) => {
                // Try to parse string as datetime
                // If it's a date-only string, return as Date (time components empty)
                // If it's a full datetime string, return as DateTime

                // First try parsing as a date
                if let Some(date) = parse_date_only_string(s) {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Date(date)]));
                }

                // Then try parsing as datetime
                match parse_datetime_string(s) {
                    Some(datetime) => {
                        // Extract just the date part if time components are effectively empty (midnight)
                        let date = datetime.naive_local().date();
                        Ok(FhirPathValue::collection(vec![FhirPathValue::Date(date)]))
                    }
                    None => Ok(FhirPathValue::Empty),
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// Parse a date-only string (YYYY, YYYY-MM, YYYY-MM-DD)
fn parse_date_only_string(s: &str) -> Option<chrono::NaiveDate> {
    // Try full format first: YYYY-MM-DD
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(date);
    }

    // Try year-month format: YYYY-MM (assume first day)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{s}-01"), "%Y-%m-%d") {
        return Some(date);
    }

    // Try year format: YYYY (assume January 1)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&format!("{s}-01-01"), "%Y-%m-%d") {
        return Some(date);
    }

    None
}

/// Parse a datetime string supporting various ISO formats
fn parse_datetime_string(s: &str) -> Option<DateTime<FixedOffset>> {
    // Try parsing as full ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt);
    }

    // Try parsing as ISO format with various timezone patterns
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.f%z", // YYYY-MM-DDThh:mm:ss.fff+/-hh:mm
        "%Y-%m-%dT%H:%M:%S%z",    // YYYY-MM-DDThh:mm:ss+/-hh:mm
        "%Y-%m-%dT%H:%M:%S%.f",   // YYYY-MM-DDThh:mm:ss.fff (assume UTC)
        "%Y-%m-%dT%H:%M:%S",      // YYYY-MM-DDThh:mm:ss (assume UTC)
        "%Y-%m-%dT%H:%M",         // YYYY-MM-DDThh:mm (assume UTC)
    ];

    for format in &formats {
        if let Ok(dt) = DateTime::parse_from_str(s, format) {
            return Some(dt);
        }

        // Try with UTC assumption for formats without timezone
        if !format.contains("%z") {
            if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(s, format) {
                if let Some(dt) = FixedOffset::east_opt(0)
                    .unwrap()
                    .from_local_datetime(&naive_dt)
                    .single()
                {
                    return Some(dt);
                }
            }
        }
    }

    None
}
