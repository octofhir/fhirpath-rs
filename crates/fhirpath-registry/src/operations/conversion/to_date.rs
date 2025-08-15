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

//! Date conversion functions implementation

use crate::metadata::{
    FhirPathType, MetadataBuilder, OperationMetadata, OperationType, PerformanceComplexity,
    TypeConstraint,
};
use crate::operation::FhirPathOperation;
use crate::operations::EvaluationContext;
use async_trait::async_trait;
use chrono::NaiveDate;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;
use regex::Regex;
use std::sync::OnceLock;

/// ToDate function: converts input to Date
pub struct ToDateFunction;

impl ToDateFunction {
    pub fn new() -> Self {
        Self
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("toDate", OperationType::Function)
            .description("If the input collection contains a single item, this function will return a single date if the item is convertible")
            .example("'2023-01-01'.toDate()")
            .example("'2023-01'.toDate()")  // Partial date
            .example("'2023'.toDate()")      // Year only
            .returns(TypeConstraint::Specific(FhirPathType::Date))
            .performance(PerformanceComplexity::Constant, true)
            .build()
    }

    fn date_regex() -> &'static Regex {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| {
            // Support full dates and partial dates (YYYY, YYYY-MM, YYYY-MM-DD)
            Regex::new(r"^(\d{4})(-(\d{2})(-(\d{2}))?)?$").unwrap()
        })
    }

    fn convert_to_date(value: &FhirPathValue) -> Result<FhirPathValue> {
        match value {
            // Already a date
            FhirPathValue::Date(d) => Ok(FhirPathValue::Date(*d)),
            
            // DateTime conversion - extract date part
            FhirPathValue::DateTime(dt) => {
                Ok(FhirPathValue::Date(dt.date_naive()))
            }
            
            // String conversion with partial date support
            FhirPathValue::String(s) => {
                let trimmed = s.trim();
                if Self::date_regex().is_match(trimmed) {
                    if let Some(date) = Self::parse_partial_date(trimmed) {
                        Ok(FhirPathValue::Date(date))
                    } else {
                        Ok(FhirPathValue::Collection(vec![].into())) // Empty collection for invalid date
                    }
                } else {
                    Ok(FhirPathValue::Collection(vec![].into())) // Empty collection for invalid format
                }
            }
            
            // Empty input
            FhirPathValue::Empty => Ok(FhirPathValue::Collection(vec![].into())),
            
            // Collection handling
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Ok(FhirPathValue::Collection(vec![].into()))
                } else if c.len() == 1 {
                    Self::convert_to_date(c.first().unwrap())
                } else {
                    // Multiple items - return empty collection per FHIRPath spec
                    Ok(FhirPathValue::Collection(vec![].into()))
                }
            }
            
            // Unsupported types
            _ => Ok(FhirPathValue::Collection(vec![].into())), // Empty collection for unsupported types
        }
    }

    fn parse_partial_date(date_str: &str) -> Option<NaiveDate> {
        let parts: Vec<&str> = date_str.split('-').collect();
        
        match parts.len() {
            1 => {
                // Year only (YYYY) - use January 1st
                if let Ok(year) = parts[0].parse::<i32>() {
                    NaiveDate::from_ymd_opt(year, 1, 1)
                } else {
                    None
                }
            }
            2 => {
                // Year-month (YYYY-MM) - use 1st of month
                if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                    if month >= 1 && month <= 12 {
                        NaiveDate::from_ymd_opt(year, month, 1)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            3 => {
                // Full date (YYYY-MM-DD)
                if let (Ok(year), Ok(month), Ok(day)) = (
                    parts[0].parse::<i32>(), 
                    parts[1].parse::<u32>(), 
                    parts[2].parse::<u32>()
                ) {
                    if month >= 1 && month <= 12 && day >= 1 && day <= 31 {
                        NaiveDate::from_ymd_opt(year, month, day)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[async_trait]
impl FhirPathOperation for ToDateFunction {
    fn identifier(&self) -> &str {
        "toDate"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        static METADATA: std::sync::LazyLock<OperationMetadata> =
            std::sync::LazyLock::new(|| ToDateFunction::create_metadata());
        &METADATA
    }

    async fn evaluate(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if let Some(result) = self.try_evaluate_sync(_args, context) {
            return result;
        }

        Self::convert_to_date(&context.input)
    }

    fn try_evaluate_sync(
        &self,
        _args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        Some(Self::convert_to_date(&context.input))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        use std::sync::Arc;
        use octofhir_fhirpath_model::provider::MockModelProvider;
        use octofhir_fhirpath_registry::FhirPathRegistry;
        
        let registry = Arc::new(FhirPathRegistry::new());
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, registry, model_provider)
    }

    #[tokio::test]
    async fn test_to_date() {
        let func = ToDateFunction::new();

        // Test with date
        let ctx = create_test_context(FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));

        // Test with string that can be parsed as date
        let ctx = create_test_context(FhirPathValue::String("2023-01-01".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));

        // Test with string that cannot be parsed as date
        let ctx = create_test_context(FhirPathValue::String("invalid-date".into()));
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);

        // Test with empty
        let ctx = create_test_context(FhirPathValue::Empty);
        let result = func.evaluate(&[], &ctx).await.unwrap();
        assert_eq!(result, FhirPathValue::Empty);
    }

    #[tokio::test]
    async fn test_to_date_sync() {
        let func = ToDateFunction::new();
        let ctx = create_test_context(FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));
        let result = func.try_evaluate_sync(&[], &ctx).unwrap().unwrap();
        assert_eq!(result, FhirPathValue::Date(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()));
        assert!(func.supports_sync());
    }
}
