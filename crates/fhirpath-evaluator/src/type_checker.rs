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

//! Type checking functions for FHIRPath expressions
//!
//! This module provides async-compatible type checking capabilities including
//! the `is`, `as`, and `ofType` operators with ModelProvider integration.

use super::context::EvaluationContext;
use octofhir_fhirpath_core::EvaluationResult;
use octofhir_fhirpath_model::{
    Collection, FhirPathValue,
    provider::{ModelProvider, TypeReflectionInfo},
};
use std::sync::Arc;

/// Type checker that uses async ModelProvider for advanced type operations
pub struct TypeChecker {
    /// Reference to the async ModelProvider
    provider: Arc<dyn ModelProvider>,
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new(provider: Arc<dyn ModelProvider>) -> Self {
        Self { provider }
    }

    /// Implement the `is` operator with async type checking
    pub async fn is_operator(
        &self,
        context: &EvaluationContext,
        input: &FhirPathValue,
        type_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // Handle collections by checking each element
        match input {
            FhirPathValue::Collection(collection) => {
                let results = self
                    .check_collection_types(context, collection, type_name, CheckOperation::Is)
                    .await?;
                Ok(FhirPathValue::Collection(Collection::from_vec(results)))
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Boolean(false)),
            _ => {
                let is_match = self
                    .check_single_value_type(context, input, type_name)
                    .await?;
                Ok(FhirPathValue::Boolean(is_match))
            }
        }
    }

    /// Implement the `as` operator with async type checking
    pub async fn as_operator(
        &self,
        context: &EvaluationContext,
        input: &FhirPathValue,
        type_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        match input {
            FhirPathValue::Collection(collection) => {
                let results = self
                    .filter_collection_by_type(context, collection, type_name)
                    .await?;
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Collection(Collection::from_vec(results)))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                let is_match = self
                    .check_single_value_type(context, input, type_name)
                    .await?;
                if is_match {
                    Ok(input.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    /// Implement the `ofType` function with async type checking
    pub async fn of_type_function(
        &self,
        context: &EvaluationContext,
        input: &FhirPathValue,
        type_name: &str,
    ) -> EvaluationResult<FhirPathValue> {
        // ofType is similar to `as` but specifically for filtering collections
        match input {
            FhirPathValue::Collection(collection) => {
                let results = self
                    .filter_collection_by_type(context, collection, type_name)
                    .await?;
                Ok(FhirPathValue::Collection(Collection::from_vec(results)))
            }
            _ => {
                // For non-collections, ofType behaves like `as`
                self.as_operator(context, input, type_name).await
            }
        }
    }

    /// Check if a collection of values matches a type (for `is` operator)
    async fn check_collection_types(
        &self,
        context: &EvaluationContext,
        collection: &Collection,
        type_name: &str,
        operation: CheckOperation,
    ) -> EvaluationResult<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        for item in collection.iter() {
            match operation {
                CheckOperation::Is => {
                    let is_match = self
                        .check_single_value_type(context, item, type_name)
                        .await?;
                    results.push(FhirPathValue::Boolean(is_match));
                }
            }
        }

        Ok(results)
    }

    /// Filter a collection by type (for `as` and `ofType` operators)
    async fn filter_collection_by_type(
        &self,
        context: &EvaluationContext,
        collection: &Collection,
        type_name: &str,
    ) -> EvaluationResult<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        for item in collection.iter() {
            let is_match = self
                .check_single_value_type(context, item, type_name)
                .await?;
            if is_match {
                results.push(item.clone());
            }
        }

        Ok(results)
    }

    /// Check if a single value matches a given type using async ModelProvider
    async fn check_single_value_type(
        &self,
        context: &EvaluationContext,
        value: &FhirPathValue,
        target_type: &str,
    ) -> EvaluationResult<bool> {
        // First check against built-in FHIR primitive types (fast path)
        if self.check_primitive_type(value, target_type) {
            return Ok(true);
        }

        // Always use ModelProvider for advanced type checking
        self.check_complex_type_async(context, value, target_type)
            .await
    }

    /// Check primitive FHIR types without async provider
    fn check_primitive_type(&self, value: &FhirPathValue, target_type: &str) -> bool {
        let normalized_target = target_type.to_lowercase();

        match value {
            FhirPathValue::Boolean(_) => {
                matches!(normalized_target.as_str(), "boolean" | "bool")
            }
            FhirPathValue::Integer(_) => {
                matches!(normalized_target.as_str(), "integer" | "int" | "number")
            }
            FhirPathValue::Decimal(_) => {
                matches!(normalized_target.as_str(), "decimal" | "number")
            }
            FhirPathValue::String(_) => {
                matches!(normalized_target.as_str(), "string" | "str")
            }
            FhirPathValue::Date(_) => {
                matches!(normalized_target.as_str(), "date")
            }
            FhirPathValue::DateTime(_) => {
                matches!(normalized_target.as_str(), "datetime" | "instant")
            }
            FhirPathValue::Time(_) => {
                matches!(normalized_target.as_str(), "time")
            }
            FhirPathValue::Quantity(_) => {
                matches!(normalized_target.as_str(), "quantity")
            }
            _ => false,
        }
    }

    /// Advanced type checking using async ModelProvider
    async fn check_complex_type_async(
        &self,
        _context: &EvaluationContext,
        value: &FhirPathValue,
        target_type: &str,
    ) -> EvaluationResult<bool> {
        // Infer the actual type of the value
        let actual_type = self.infer_value_type(value);

        if let Some(actual) = actual_type {
            // Check type cache first - for now, skip caching boolean results
            // TODO: Implement proper boolean result caching

            // Use ModelProvider to check subtype relationship
            let is_subtype = self.provider.is_subtype_of(&actual, target_type).await;

            // TODO: Implement proper caching for boolean results
            // For now, return result directly without caching

            Ok(is_subtype)
        } else {
            // Unknown type, fallback to basic checking
            Ok(self.check_basic_type_compatibility(value, target_type))
        }
    }

    /// Basic type compatibility checking (fallback)
    fn check_basic_type_compatibility(&self, value: &FhirPathValue, target_type: &str) -> bool {
        // Handle special FHIR system types
        match target_type.to_lowercase().as_str() {
            "any" => true, // Any type matches anything
            "element" => {
                // Element is base type for all FHIR elements
                matches!(
                    value,
                    FhirPathValue::Resource(_) | FhirPathValue::JsonValue(_)
                )
            }
            "resource" => {
                matches!(value, FhirPathValue::Resource(_))
            }
            "domainresource" => {
                // Most FHIR resources extend DomainResource
                if let FhirPathValue::Resource(resource) = value {
                    // Basic heuristic: if it has resourceType, it's likely a DomainResource
                    resource.resource_type().is_some()
                } else {
                    false
                }
            }
            "backboneelement" => {
                // BackboneElement is used for complex elements within resources
                matches!(value, FhirPathValue::JsonValue(_))
            }
            _ => {
                // Try primitive type matching as fallback
                self.check_primitive_type(value, target_type)
            }
        }
    }

    /// Infer FHIR type name from FhirPathValue
    fn infer_value_type(&self, value: &FhirPathValue) -> Option<String> {
        match value {
            FhirPathValue::Resource(resource) => resource.resource_type().map(|rt| rt.to_string()),
            FhirPathValue::JsonValue(json) => {
                // Try to get resourceType or infer from structure
                json.get("resourceType")
                    .and_then(|rt| rt.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        // Fallback: assume it's a complex element
                        Some("Element".to_string())
                    })
            }
            FhirPathValue::String(_) => Some("string".to_string()),
            FhirPathValue::Integer(_) => Some("integer".to_string()),
            FhirPathValue::Decimal(_) => Some("decimal".to_string()),
            FhirPathValue::Boolean(_) => Some("boolean".to_string()),
            FhirPathValue::Date(_) => Some("date".to_string()),
            FhirPathValue::DateTime(_) => Some("dateTime".to_string()),
            FhirPathValue::Time(_) => Some("time".to_string()),
            FhirPathValue::Quantity(_) => Some("Quantity".to_string()),
            _ => None,
        }
    }

    /// Handle polymorphic type checking (choice types)
    pub async fn handle_choice_type_checking(
        &self,
        context: &EvaluationContext,
        value: &FhirPathValue,
        choice_type_pattern: &str,
    ) -> EvaluationResult<bool> {
        // Handle FHIR choice types like value[x] -> valueString, valueInteger, etc.
        if choice_type_pattern.contains("[x]") {
            let base_pattern = choice_type_pattern.replace("[x]", "");

            // Check if value matches any of the common choice type suffixes
            let common_suffixes = [
                "String",
                "Integer",
                "Boolean",
                "Decimal",
                "Date",
                "DateTime",
                "Time",
                "Code",
                "Uri",
                "Canonical",
                "Oid",
                "Uuid",
                "Url",
                "Quantity",
                "CodeableConcept",
                "Coding",
                "Reference",
                "Period",
            ];

            for suffix in &common_suffixes {
                let full_type = format!("{base_pattern}{suffix}");
                if self
                    .check_single_value_type(context, value, &full_type)
                    .await?
                {
                    return Ok(true);
                }
            }

            Ok(false)
        } else {
            // Not a choice type, use regular checking
            self.check_single_value_type(context, value, choice_type_pattern)
                .await
        }
    }

    /// Get enhanced type information using async ModelProvider
    pub async fn get_enhanced_type_info(&self, type_name: &str) -> Option<TypeReflectionInfo> {
        self.provider.get_type_reflection(type_name).await
    }
}

/// Operation types for collection checking
enum CheckOperation {
    Is,
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_model::{MockModelProvider, json_arc::ArcJsonValue};
    use tokio;

    #[tokio::test]
    async fn test_primitive_type_checking() {
        let provider = Arc::new(MockModelProvider::empty());
        let checker = TypeChecker::new(provider);

        // Test basic primitive type checking
        assert!(checker.check_primitive_type(&FhirPathValue::Boolean(true), "boolean"));
        assert!(checker.check_primitive_type(&FhirPathValue::Integer(42), "integer"));
        assert!(checker.check_primitive_type(&FhirPathValue::String("test".into()), "string"));

        // Test case insensitivity
        assert!(checker.check_primitive_type(&FhirPathValue::Boolean(true), "Boolean"));
        assert!(checker.check_primitive_type(&FhirPathValue::Integer(42), "Integer"));

        // Test negative cases
        assert!(!checker.check_primitive_type(&FhirPathValue::Boolean(true), "string"));
        assert!(!checker.check_primitive_type(&FhirPathValue::Integer(42), "boolean"));
    }

    #[tokio::test]
    async fn test_basic_compatibility() {
        let provider = Arc::new(MockModelProvider::empty());
        let checker = TypeChecker::new(provider);

        // Test special system types
        let json_val =
            FhirPathValue::JsonValue(ArcJsonValue::new(serde_json::json!({"test": "value"})));

        assert!(checker.check_basic_type_compatibility(&json_val, "any"));
        assert!(checker.check_basic_type_compatibility(&json_val, "element"));

        // Test resource type checking would require actual resource
        assert!(checker.check_basic_type_compatibility(&FhirPathValue::Boolean(true), "any"));
    }

    #[tokio::test]
    async fn test_type_inference() {
        let provider = Arc::new(MockModelProvider::empty());
        let checker = TypeChecker::new(provider);

        assert_eq!(
            checker.infer_value_type(&FhirPathValue::String("test".into())),
            Some("string".to_string())
        );
        assert_eq!(
            checker.infer_value_type(&FhirPathValue::Integer(42)),
            Some("integer".to_string())
        );
        assert_eq!(
            checker.infer_value_type(&FhirPathValue::Boolean(true)),
            Some("boolean".to_string())
        );

        // Test JSON value with resourceType
        let patient_json = serde_json::json!({
            "resourceType": "Patient",
            "id": "example"
        });
        let patient_val = FhirPathValue::JsonValue(ArcJsonValue::new(patient_json));

        assert_eq!(
            checker.infer_value_type(&patient_val),
            Some("Patient".to_string())
        );
    }
}
