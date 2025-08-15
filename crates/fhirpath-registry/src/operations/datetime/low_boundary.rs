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

//! LowBoundary function implementation

use crate::operation::FhirPathOperation;
use crate::metadata::{
    MetadataBuilder, OperationMetadata, OperationType, TypeConstraint, PerformanceComplexity, FhirPathType
};
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::{FhirPathValue, Collection};
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime, DateTime, FixedOffset, TimeZone, Utc};

/// LowBoundary function - returns the lower boundary of a partial date/time value
#[derive(Debug, Clone)]
pub struct LowBoundaryFunction;

impl LowBoundaryFunction {
    // Helper method to convert NaiveDate to DateTime<FixedOffset>
    fn calculate_date_low_boundary_typed(&self, date: &NaiveDate) -> Result<DateTime<FixedOffset>> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let boundary_str = self.calculate_date_low_boundary(&date_str);
        
        // Parse the result back to DateTime<FixedOffset>
        match DateTime::parse_from_str(&boundary_str, "%Y-%m-%dT%H:%M:%S%.3f") {
            Ok(dt) => Ok(dt),
            Err(_) => {
                // Try with timezone
                match DateTime::parse_from_str(&format!("{}+00:00", boundary_str), "%Y-%m-%dT%H:%M:%S%.3f%z") {
                    Ok(dt) => Ok(dt),
                    Err(_) => {
                        // Fallback: create datetime at UTC
                        let naive_datetime = date.and_hms_opt(0, 0, 0).unwrap_or_else(|| date.and_hms(0, 0, 0));
                        Ok(Utc.from_utc_datetime(&naive_datetime).with_timezone(&FixedOffset::east(0)))
                    }
                }
            }
        }
    }
    
    // Helper method to handle DateTime<FixedOffset>
    fn calculate_datetime_low_boundary_typed(&self, datetime: &DateTime<FixedOffset>) -> Result<DateTime<FixedOffset>> {
        let datetime_str = datetime.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string();
        let boundary_str = self.calculate_datetime_low_boundary(&datetime_str);
        
        // Parse the result back to DateTime<FixedOffset>
        match DateTime::parse_from_str(&boundary_str, "%Y-%m-%dT%H:%M:%S%.3f%z") {
            Ok(dt) => Ok(dt),
            Err(_) => {
                // Try without timezone
                match DateTime::parse_from_str(&format!("{}+00:00", boundary_str), "%Y-%m-%dT%H:%M:%S%.3f%z") {
                    Ok(dt) => Ok(dt),
                    Err(_) => Ok(*datetime) // Fallback to original
                }
            }
        }
    }
    
    // Helper method to handle NaiveTime
    fn calculate_time_low_boundary_typed(&self, time: &NaiveTime) -> Result<NaiveTime> {
        let time_str = time.format("%H:%M:%S%.3f").to_string();
        let boundary_str = self.calculate_time_low_boundary(&time_str);
        
        // Parse the result back to NaiveTime
        match NaiveTime::parse_from_str(&boundary_str, "%H:%M:%S%.3f") {
            Ok(t) => Ok(t),
            Err(_) => {
                // Try other formats
                if let Ok(t) = NaiveTime::parse_from_str(&boundary_str, "%H:%M:%S") {
                    Ok(t)
                } else if let Ok(t) = NaiveTime::parse_from_str(&boundary_str, "%H:%M") {
                    Ok(t)
                } else {
                    Ok(*time) // Fallback to original
                }
            }
        }
    }
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("lowBoundary", OperationType::Function)
            .description("Returns the lower boundary of a partial date/time value")
            .example("@2023-01.lowBoundary()")
            .example("@T12:30.lowBoundary()")
            .returns(TypeConstraint::Specific(FhirPathType::DateTime))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn calculate_date_low_boundary(&self, date: &str) -> String {
        // Convert partial date to full datetime at earliest possible moment
        let parts: Vec<&str> = date.split('-').collect();

        match parts.len() {
            1 => {
                // Year only -> YYYY-01-01T00:00:00.000
                format!("{}-01-01T00:00:00.000", parts[0])
            }
            2 => {
                // Year-month -> YYYY-MM-01T00:00:00.000
                format!("{}-01T00:00:00.000", date)
            }
            3 => {
                // Full date -> YYYY-MM-DDT00:00:00.000
                format!("{}T00:00:00.000", date)
            }
            _ => date.to_string(), // Invalid format, return as-is
        }
    }

    fn calculate_datetime_low_boundary(&self, datetime: &str) -> String {
        // Fill in missing precision with earliest values
        if datetime.contains('T') {
            let parts: Vec<&str> = datetime.split('T').collect();
            let date_part = parts[0];
            let time_part = if parts.len() > 1 { parts[1] } else { "" };

            // Expand date part if needed
            let full_date = self.expand_date_to_low_boundary(date_part);
            // Expand time part to full precision
            let full_time = self.expand_time_to_low_boundary(time_part);

            // Remove 'T00:00:00.000' from full_date if it was added, since we're adding our own time
            let clean_date = if full_date.contains('T') {
                full_date.split('T').next().unwrap_or(date_part)
            } else {
                &full_date
            };

            format!("{}T{}", clean_date, full_time)
        } else {
            // No time part, add minimum time
            let full_date = self.expand_date_to_low_boundary(datetime);
            if full_date.contains('T') {
                full_date // Already has time
            } else {
                format!("{}T00:00:00.000", full_date)
            }
        }
    }

    fn calculate_time_low_boundary(&self, time: &str) -> String {
        self.expand_time_to_low_boundary(time)
    }

    fn expand_date_to_low_boundary(&self, date: &str) -> String {
        let parts: Vec<&str> = date.split('-').collect();

        match parts.len() {
            1 => {
                // Year only -> YYYY-01-01
                format!("{}-01-01", parts[0])
            }
            2 => {
                // Year-month -> YYYY-MM-01
                format!("{}-01", date)
            }
            _ => date.to_string(), // Already full or invalid
        }
    }

    fn expand_time_to_low_boundary(&self, time: &str) -> String {
        if time.is_empty() {
            return "00:00:00.000".to_string();
        }

        // Handle timezone offset
        let (time_part, tz_part) = if time.contains('+') {
            let parts: Vec<&str> = time.split('+').collect();
            (parts[0], Some(format!("+{}", parts[1])))
        } else if time.contains('Z') {
            (time.trim_end_matches('Z'), Some("Z".to_string()))
        } else if time.rfind('-').map_or(false, |pos| pos > 2) {
            // Find last '-' that could be timezone (not in date part)
            let pos = time.rfind('-').unwrap();
            (&time[..pos], Some(time[pos..].to_string()))
        } else {
            (time, None)
        };

        // Expand time part to full precision with minimum values
        let parts: Vec<&str> = time_part.split(':').collect();

        let expanded_time = match parts.len() {
            1 => {
                // Hour only -> HH:00:00.000
                format!("{}:00:00.000", parts[0])
            }
            2 => {
                // Hour:minute -> HH:MM:00.000
                format!("{}:00.000", time_part)
            }
            3 => {
                // Hour:minute:second -> check if has milliseconds
                if parts[2].contains('.') {
                    time_part.to_string() // Already has milliseconds
                } else {
                    format!("{}.000", time_part) // Add milliseconds
                }
            }
            _ => time_part.to_string(), // Invalid format, return as-is
        };

        // Add timezone back if it existed
        if let Some(tz) = tz_part {
            format!("{}{}", expanded_time, tz)
        } else {
            expanded_time
        }
    }
}

#[async_trait]
impl FhirPathOperation for LowBoundaryFunction {
    fn identifier(&self) -> &str {
        "lowBoundary"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> = std::sync::LazyLock::new(|| {
            LowBoundaryFunction::create_metadata()
        });
        &METADATA
    }

    async fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            });
        }

        let input = &context.input;

        match input {
            FhirPathValue::Date(d) => {
                let low_boundary = self.calculate_date_low_boundary_typed(d)?;
                Ok(FhirPathValue::DateTime(low_boundary))
            }
            FhirPathValue::DateTime(dt) => {
                let low_boundary = self.calculate_datetime_low_boundary_typed(dt)?;
                Ok(FhirPathValue::DateTime(low_boundary))
            }
            FhirPathValue::Time(t) => {
                let low_boundary = self.calculate_time_low_boundary_typed(t)?;
                Ok(FhirPathValue::Time(low_boundary))
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    let item_context = context.with_focus(items.get(0).unwrap().clone());
                    self.evaluate(args, &item_context).await
                } else if items.is_empty() {
                    Ok(FhirPathValue::Collection(Collection::new()))
                } else {
                    Err(FhirPathError::InvalidArguments {
                        message: "lowBoundary() requires a single date/time value".to_string()
                    })
                }
            }
            _ => Err(FhirPathError::InvalidArguments {
                message: "lowBoundary() requires a date, datetime, or time value".to_string()
            }),
        }
    }

    fn try_evaluate_sync(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Option<Result<FhirPathValue>> {
        // Validate no arguments
        if !args.is_empty() {
            return Some(Err(FhirPathError::InvalidArgumentCount {
                function_name: self.identifier().to_string(),
                expected: 0,
                actual: args.len()
            }));
        }

        let input = &context.input;

        let result = match input {
            FhirPathValue::Date(d) => {
                match self.calculate_date_low_boundary_typed(d) {
                    Ok(low_boundary) => Ok(FhirPathValue::DateTime(low_boundary)),
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::DateTime(dt) => {
                match self.calculate_datetime_low_boundary_typed(dt) {
                    Ok(low_boundary) => Ok(FhirPathValue::DateTime(low_boundary)),
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Time(t) => {
                match self.calculate_time_low_boundary_typed(t) {
                    Ok(low_boundary) => Ok(FhirPathValue::Time(low_boundary)),
                    Err(e) => Err(e),
                }
            }
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    let item_context = context.with_focus(items.get(0).unwrap().clone());
                    return self.try_evaluate_sync(args, &item_context);
                } else if items.is_empty() {
                    Ok(FhirPathValue::Collection(Collection::new()))
                } else {
                    Err(FhirPathError::EvaluationError {
                        message: "lowBoundary() requires a single date/time value".to_string()
                    })
                }
            }
            _ => Err(FhirPathError::EvaluationError {
                message: "lowBoundary() requires a date, datetime, or time value".to_string()
            }),
        };

        Some(result)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::EvaluationContext;
    use octofhir_fhirpath_model::FhirPathValue;

    #[tokio::test]
    async fn test_low_boundary_date() -> Result<()> {
        let function = LowBoundaryFunction::new();

        // Test year only
        let context = EvaluationContext::new(FhirPathValue::Date("2023".to_string()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::DateTime(dt) => {
                assert_eq!(dt, "2023-01-01T00:00:00.000");
            }
            _ => panic!("Expected DateTime value"),
        }

        // Test year-month
        let context = EvaluationContext::new(FhirPathValue::Date("2023-06".to_string()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::DateTime(dt) => {
                assert_eq!(dt, "2023-06-01T00:00:00.000");
            }
            _ => panic!("Expected DateTime value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_low_boundary_time() -> Result<()> {
        let function = LowBoundaryFunction::new();

        // Test hour only
        let context = EvaluationContext::new(FhirPathValue::Time("14".to_string()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Time(t) => {
                assert_eq!(t, "14:00:00.000");
            }
            _ => panic!("Expected Time value"),
        }

        // Test hour:minute
        let context = EvaluationContext::new(FhirPathValue::Time("14:30".to_string()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::Time(t) => {
                assert_eq!(t, "14:30:00.000");
            }
            _ => panic!("Expected Time value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_low_boundary_datetime() -> Result<()> {
        let function = LowBoundaryFunction::new();

        // Test partial datetime
        let context = EvaluationContext::new(FhirPathValue::DateTime("2023-06-15T14:30".to_string()));
        let result = function.evaluate(&[], &context).await?;

        match result {
            FhirPathValue::DateTime(dt) => {
                assert_eq!(dt, "2023-06-15T14:30:00.000");
            }
            _ => panic!("Expected DateTime value"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_low_boundary_invalid_args() -> () {
        let function = LowBoundaryFunction::new();
        let context = EvaluationContext::new(FhirPathValue::Date("2023".to_string()));

        let result = function.evaluate(&[FhirPathValue::String("invalid".into())], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = result {
            assert_eq!(expected, 0);
            assert_eq!(actual, 1);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }

    #[tokio::test]
    async fn test_low_boundary_invalid_input() -> () {
        let function = LowBoundaryFunction::new();
        let context = EvaluationContext::new(FhirPathValue::String("not a date".into()));

        let result = function.evaluate(&[], &context).await;

        assert!(result.is_err());
        if let Err(FhirPathError::EvaluationError { message: _ }) = result {
            // Expected
        } else {
            panic!("Expected InvalidOperation error");
        }
    }
}
