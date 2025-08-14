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

//! Unified join() function implementation

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

/// Unified join() function implementation
/// 
/// Joins a collection of strings into a single string using a separator.
/// Syntax: join(separator)
pub struct UnifiedJoinFunction {
    metadata: EnhancedFunctionMetadata,
}

impl UnifiedJoinFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature::new(
            "join",
            vec![ParameterInfo::required("separator", TypeInfo::String)],
            TypeInfo::String,
        );
        
        let metadata = MetadataBuilder::new("join", FunctionCategory::StringOperations)
            .display_name("Join")
            .description("Joins a collection of strings into a single string using a separator")
            .example("Patient.name.given.join(' ')")
            .example("Bundle.entry.resource.ofType(Patient).id.join(',')")
            .signature(signature)
            .execution_mode(ExecutionMode::Sync)
            .input_types(vec![TypePattern::CollectionOf(Box::new(TypePattern::StringLike))])
            .output_type(TypePattern::StringLike)
            .supports_collections(true)
            .requires_collection(true)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .memory_usage(MemoryUsage::Linear)
            .lsp_snippet("join('${1:separator}')")
            .completion_visibility(CompletionVisibility::Contextual)
            .keywords(vec!["join", "combine", "concatenate", "merge"])
            .usage_pattern(
                "String joining",
                "collection.join(separator)",
                "Combining collections of strings with separators"
            )
            .build();
        
        Self { metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for UnifiedJoinFunction {
    fn name(&self) -> &str {
        "join"
    }
    
    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }
    
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sync
    }
    
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Validate arguments - exactly 1 required (separator)
        if args.len() != 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: args.len(),
            });
        }
        
        let separator = match &args[0] {
            FhirPathValue::String(s) => s.to_string(),
            _ => return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "join() requires a string separator argument".to_string(),
            }),
        };
        
        let input_collection = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => {
                return Ok(FhirPathValue::Empty);
            }
            single_item => {
                // Treat single item as a collection of one
                return self.join_single_value(single_item, &separator);
            }
        };
        
        if input_collection.is_empty() {
            return Ok(FhirPathValue::Empty);
        }
        
        // Convert all items to strings
        let mut string_parts = Vec::new();
        for item in input_collection.iter() {
            match item {
                FhirPathValue::String(s) => {
                    string_parts.push(s.to_string());
                }
                FhirPathValue::Integer(i) => {
                    string_parts.push(i.to_string());
                }
                FhirPathValue::Decimal(d) => {
                    string_parts.push(d.to_string());
                }
                FhirPathValue::Boolean(b) => {
                    string_parts.push(b.to_string());
                }
                _ => {
                    return Err(FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: "join() can only be applied to collections containing strings or convertible values".to_string(),
                    });
                }
            }
        }
        
        let result = string_parts.join(&separator);
        Ok(FhirPathValue::String(result.into()))
    }
}

impl UnifiedJoinFunction {
    /// Handle join operation on a single value
    fn join_single_value(&self, value: &FhirPathValue, _separator: &str) -> FunctionResult<FhirPathValue> {
        // For single values, join just returns the string representation
        match value {
            FhirPathValue::String(s) => Ok(FhirPathValue::String(s.clone())),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::String(i.to_string().into())),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::String(d.to_string().into())),
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::String(b.to_string().into())),
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "join() can only be applied to strings or convertible values".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::FhirPathValue;
    use rust_decimal::Decimal;
    
    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        EvaluationContext::new(input)
    }
    
    #[tokio::test]
    async fn test_join_basic() {
        let func = UnifiedJoinFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("one".into()),
            FhirPathValue::String("two".into()),
            FhirPathValue::String("three".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("one,two,three".into()));
    }
    
    #[tokio::test]
    async fn test_join_with_space() {
        let func = UnifiedJoinFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("John".into()),
            FhirPathValue::String("Doe".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(" ".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("John Doe".into()));
    }
    
    #[tokio::test]
    async fn test_join_mixed_types() {
        let func = UnifiedJoinFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("Value:".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::Decimal(Decimal::new(314, 2)), // 3.14
            FhirPathValue::Boolean(true),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(" ".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("Value: 42 3.14 true".into()));
    }
    
    #[tokio::test]
    async fn test_join_single_item() {
        let func = UnifiedJoinFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("only".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("only".into()));
    }
    
    #[tokio::test]
    async fn test_join_empty_collection() {
        let func = UnifiedJoinFunction::new();
        
        let context = create_test_context(FhirPathValue::Empty);
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::Empty);
    }
    
    #[tokio::test]
    async fn test_join_empty_separator() {
        let func = UnifiedJoinFunction::new();
        
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::String("a".into()),
            FhirPathValue::String("b".into()),
            FhirPathValue::String("c".into()),
        ]);
        let context = create_test_context(collection);
        let args = vec![FhirPathValue::String("".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("abc".into()));
    }
    
    #[tokio::test]
    async fn test_join_single_value_input() {
        let func = UnifiedJoinFunction::new();
        
        let context = create_test_context(FhirPathValue::String("single".into()));
        let args = vec![FhirPathValue::String(",".into())];
        let result = func.evaluate_sync(&args, &context).unwrap();
        
        assert_eq!(result, FhirPathValue::String("single".into()));
    }
    
    #[tokio::test]
    async fn test_function_metadata() {
        let func = UnifiedJoinFunction::new();
        let metadata = func.metadata();
        
        assert_eq!(metadata.basic.name, "join");
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.basic.category, FunctionCategory::StringOperations);
    }
}