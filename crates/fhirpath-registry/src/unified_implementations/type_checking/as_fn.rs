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

//! Unified as() function implementation

use crate::enhanced_metadata::{
    EnhancedFunctionMetadata, PerformanceComplexity,
    TypePattern, MemoryUsage,
};
use crate::function::{FunctionCategory, CompletionVisibility};
use crate::unified_function::ExecutionMode;
use crate::function::{EvaluationContext, FunctionError, FunctionResult};
use crate::metadata_builder::MetadataBuilder;
use crate::unified_function::UnifiedFhirPathFunction;
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;
use rust_decimal::prelude::ToPrimitive;

/// Unified as() function implementation
/// 
/// Casts the input to the given type if possible, otherwise returns empty.
/// Syntax: as(type)
pub struct UnifiedAsFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedAsFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "as",
            vec![ParameterInfo::required("type", TypeInfo::String)],
            TypeInfo::Any,
        );
        
        let metadata = MetadataBuilder::new("as", FunctionCategory::TypeChecking)
            .display_name("Type Cast")
            .description("Casts the input to the given type if possible, otherwise returns empty")
            .example("Patient.name.as(HumanName)")
            .example("'42'.as(integer)")
            .example("someValue.as(string)")
            .signature(signature)
            .execution_mode(ExecutionMode::Async) // Async due to FHIR type resolution
            .input_types(vec![TypePattern::Any])
            .output_type(TypePattern::Any)
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(PerformanceComplexity::Logarithmic) // Type lookup and conversion
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("as(${1:type})")
            .completion_visibility(CompletionVisibility::Always)
            .keywords(vec!["as", "cast", "convert", "type"])
            .usage_pattern(
                "Type casting",
                "value.as(type)",
                "Safe type conversion with fallback to empty"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedAsFunction {
    fn name(&self) -> &str {
        "as"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Async
    }
    
    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required (type name)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let type_name = match &args[0] {
            FhirPathValue::String(name) => name.to_string(),
            FhirPathValue::Collection(items) if items.len() == 1 => {
                match items.get(0) {
                    Some(FhirPathValue::String(name)) => name.to_string(),
                    _ => return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "Type argument must be a string".to_string(),
                    }),
                }
            }
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Type argument must be a string".to_string(),
            }),
        };
        
        // Handle input based on its type
        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut result = Vec::new();
                for item in items.iter() {
                    if let Some(converted) = self.cast_value(item, &type_name, context).await? {
                        result.push(converted);
                    }
                }
                if result.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(result))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            single_item => {
                if let Some(converted) = self.cast_value(single_item, &type_name, context).await? {
                    Ok(converted)
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }
}

impl UnifiedAsFunction {
    /// Attempt to cast a value to the specified type
    async fn cast_value(
        &self,
        value: &FhirPathValue,
        type_name: &str,
        context: &EvaluationContext,
    ) -> FunctionResult<Option<FhirPathValue>> {
        match type_name.to_lowercase().as_str() {
            "string" => Ok(Some(self.to_string_value(value))),
            "integer" => Ok(self.to_integer_value(value)),
            "decimal" => Ok(self.to_decimal_value(value)),
            "boolean" => Ok(self.to_boolean_value(value)),
            "date" => Ok(self.to_date_value(value)),
            "datetime" => Ok(self.to_datetime_value(value)),
            "time" => Ok(self.to_time_value(value)),
            "quantity" => Ok(self.to_quantity_value(value)),
            _ => {
                // For complex FHIR types, check if the value is already of that type
                if let Some(_provider) = context.model_provider.as_ref() {
                    match value {
                        FhirPathValue::Resource(resource) => {
                            // Check if resource's resourceType matches the requested type
                            if let Some(resource_type_value) = resource.as_json().get("resourceType") {
                                if let Some(resource_type_str) = resource_type_value.as_str() {
                                    if resource_type_str == type_name {
                                        Ok(Some(value.clone()))
                                    } else {
                                        Ok(None)
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Ok(None)
                            }
                        }
                        FhirPathValue::JsonValue(json_value) => {
                            // Check if JSON object's resourceType matches the requested type
                            if let Some(resource_type_value) = json_value.get("resourceType") {
                                if let Some(resource_type_str) = resource_type_value.as_str() {
                                    if resource_type_str == type_name {
                                        Ok(Some(value.clone()))
                                    } else {
                                        Ok(None)
                                    }
                                } else {
                                    Ok(None)
                                }
                            } else {
                                // For non-resource objects, we would need more sophisticated type checking
                                // For now, just return None
                                Ok(None)
                            }
                        }
                        _ => Ok(None),
                    }
                } else {
                    // Without ModelProvider, we can't cast to complex types
                    Ok(None)
                }
            }
        }
    }
    
    /// Convert value to string
    fn to_string_value(&self, value: &FhirPathValue) -> FhirPathValue {
        match value {
            FhirPathValue::String(_s) => value.clone(),
            FhirPathValue::Integer(i) => FhirPathValue::String(i.to_string().into()),
            FhirPathValue::Decimal(d) => FhirPathValue::String(d.to_string().into()),
            FhirPathValue::Boolean(b) => FhirPathValue::String(b.to_string().into()),
            FhirPathValue::Date(d) => FhirPathValue::String(d.to_string().into()),
            FhirPathValue::DateTime(dt) => FhirPathValue::String(dt.to_string().into()),
            FhirPathValue::Time(t) => FhirPathValue::String(t.to_string().into()),
            _ => value.clone(), // For complex types, return as-is
        }
    }
    
    /// Convert value to integer
    fn to_integer_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::Integer(_) => Some(value.clone()),
            FhirPathValue::Decimal(d) => {
                if d.fract() == rust_decimal::Decimal::ZERO {
                    // Only convert if it's a whole number
                    d.to_i64().map(FhirPathValue::Integer)
                } else {
                    None
                }
            }
            FhirPathValue::String(s) => {
                s.parse::<i64>().ok().map(FhirPathValue::Integer)
            }
            FhirPathValue::Boolean(b) => {
                Some(FhirPathValue::Integer(if *b { 1 } else { 0 }))
            }
            _ => None,
        }
    }
    
    /// Convert value to decimal
    fn to_decimal_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::Decimal(_) => Some(value.clone()),
            FhirPathValue::Integer(i) => {
                Some(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i)))
            }
            FhirPathValue::String(s) => {
                use std::str::FromStr;
                rust_decimal::Decimal::from_str(s).ok().map(FhirPathValue::Decimal)
            }
            _ => None,
        }
    }
    
    /// Convert value to boolean
    fn to_boolean_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::Boolean(_) => Some(value.clone()),
            FhirPathValue::String(s) => {
                match s.to_lowercase().as_str() {
                    "true" => Some(FhirPathValue::Boolean(true)),
                    "false" => Some(FhirPathValue::Boolean(false)),
                    _ => None,
                }
            }
            FhirPathValue::Integer(i) => {
                match *i {
                    0 => Some(FhirPathValue::Boolean(false)),
                    1 => Some(FhirPathValue::Boolean(true)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    
    /// Convert value to date (placeholder implementation)
    fn to_date_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::Date(_) => Some(value.clone()),
            FhirPathValue::String(_s) => {
                // Would need proper date parsing here
                None
            }
            _ => None,
        }
    }
    
    /// Convert value to datetime (placeholder implementation)
    fn to_datetime_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::DateTime(_) => Some(value.clone()),
            FhirPathValue::String(_s) => {
                // Would need proper datetime parsing here
                None
            }
            _ => None,
        }
    }
    
    /// Convert value to time (placeholder implementation)
    fn to_time_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::Time(_) => Some(value.clone()),
            FhirPathValue::String(_s) => {
                // Would need proper time parsing here
                None
            }
            _ => None,
        }
    }
    
    /// Convert value to quantity (placeholder implementation)
    fn to_quantity_value(&self, value: &FhirPathValue) -> Option<FhirPathValue> {
        match value {
            FhirPathValue::Quantity(_) => Some(value.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use serde_json::json;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_as_string() {
        let func = UnifiedAsFunction::new();
        let context = create_test_context(FhirPathValue::Integer(42));
        
        let args = vec![FhirPathValue::String("string".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::String("42".into()));
    }
    
    #[tokio::test]
    async fn test_as_integer() {
        let func = UnifiedAsFunction::new();
        let context = create_test_context(FhirPathValue::String("123".into()));
        
        let args = vec![FhirPathValue::String("integer".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Integer(123));
    }
    
    #[tokio::test]
    async fn test_as_integer_invalid() {
        let func = UnifiedAsFunction::new();
        let context = create_test_context(FhirPathValue::String("not-a-number".into()));
        
        let args = vec![FhirPathValue::String("integer".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_as_boolean() {
        let func = UnifiedAsFunction::new();
        let context = create_test_context(FhirPathValue::String("true".into()));
        
        let args = vec![FhirPathValue::String("boolean".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
    
    #[tokio::test]
    async fn test_as_decimal() {
        let func = UnifiedAsFunction::new();
        let context = create_test_context(FhirPathValue::Integer(42));
        
        let args = vec![FhirPathValue::String("decimal".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        if let FhirPathValue::Decimal(d) = result {
            assert_eq!(d, rust_decimal::Decimal::from(42));
        } else {
            panic!("Expected decimal result");
        }
    }
    
    #[tokio::test]
    async fn test_as_resource_type() {
        let func = UnifiedAsFunction::new();
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let context = create_test_context(FhirPathValue::JsonValue(patient_json.clone().into()));
        
        let args = vec![FhirPathValue::String("Patient".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::JsonValue(patient_json.into()));
    }
    
    #[tokio::test]
    async fn test_as_wrong_resource_type() {
        let func = UnifiedAsFunction::new();
        let patient_json = json!({
            "resourceType": "Patient",
            "id": "123"
        });
        let context = create_test_context(FhirPathValue::JsonValue(patient_json.into()));
        
        let args = vec![FhirPathValue::String("Observation".into())];
        let result = func.evaluate_async(&args, &context).await.unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedAsFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "as");
        assert_eq!(metadata.execution_mode, ExecutionMode::Async);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::TypeChecking);
    }
}