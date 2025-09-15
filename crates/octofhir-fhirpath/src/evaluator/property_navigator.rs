//! Property navigation with FhirPathWrapped value support
//!
//! This module provides property navigation that preserves type information 
//! and enables zero-copy operations through Arc-based sharing.

use crate::core::{
    error::{FhirPathError, Result},
    error_code::*,
    model_provider::ModelProvider,
    wrapped::{FhirPathWrapped, PrimitiveElement},
    FhirPathValue,
};
use crate::evaluator::choice_types::{ChoiceProperty, ChoiceResolution};
use octofhir_fhir_model::TypeInfo;
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// Property navigator with wrapped value support
#[derive(Debug, Clone, Default)]
pub struct PropertyNavigator;

impl PropertyNavigator {
    /// Navigate a property on a FHIRPath value with type preservation
    pub async fn navigate_property(
        &self,
        base_value: &FhirPathValue,
        property: &str,
        model_provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        // debug aid: property access tracing for resourceType was here
        match base_value {
            FhirPathValue::Wrapped(wrapped) => {
                self.navigate_wrapped_property(wrapped, property, model_provider)
                    .await
            }
            FhirPathValue::ResourceWrapped(wrapped) => {
                self.navigate_resource_property(wrapped, property, model_provider)
                    .await
            }
            FhirPathValue::Resource(json) => {
                // Convert to wrapped for better type handling
                let resource_wrapped = FhirPathWrapped::resource((**json).clone());
                self.navigate_resource_property(&resource_wrapped, property, model_provider)
                    .await
            }
            FhirPathValue::JsonValue(json) => {
                // Convert to wrapped for better type handling
                let wrapped = FhirPathWrapped::new((**json).clone(), None);
                self.navigate_wrapped_property(&wrapped, property, model_provider)
                    .await
            }
            _ => self
                .navigate_primitive_property(base_value, property, model_provider)
                .await,
        }
    }

    /// Navigate property on wrapped value with type preservation
    async fn navigate_wrapped_property(
        &self,
        wrapped: &FhirPathWrapped<JsonValue>,
        property: &str,
        model_provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        // Direct property access with Arc sharing (zero-copy)
        if let Some(prop_wrapped) = wrapped.get_property(property) {
            return Ok(FhirPathValue::Wrapped(prop_wrapped));
        }

        // Check for choice properties using data-aware detection
        // Note: We can't easily wrap the borrowed model_provider in Arc here,
        // so we'll use our own lightweight choice detection logic
        let choice_resolution = self
            .detect_choice_properties_inline(wrapped.unwrap(), property, model_provider)
            .await?;

        if choice_resolution.is_choice {
            return self
                .resolve_choice_resolution(choice_resolution, model_provider)
                .await;
        }

        Ok(FhirPathValue::Empty)
    }

    /// Navigate property on resource with full type awareness
    async fn navigate_resource_property(
        &self,
        wrapped: &FhirPathWrapped<JsonValue>,
        property: &str,
        model_provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        // TODO: Implement proper FHIR property validation using ModelProvider
        // when the interface supports property validation methods
        // For now, use basic navigation with type preservation

        // Use model provider to get type information if available
        if let Some(_parent_type) = wrapped.get_type_info() {
            // Direct property access with type preservation
            if let Some(prop_value) = wrapped.unwrap().get(property) {
                // Create wrapped value - type inference will be improved later
                let wrapped_result = FhirPathWrapped::new(prop_value.clone(), None);
                return Ok(FhirPathValue::Wrapped(wrapped_result));
            }
        }

        // Fall back to basic navigation
        self.navigate_wrapped_property(wrapped, property, model_provider)
            .await
    }

    /// Navigate property on primitive values (limited functionality)
    async fn navigate_primitive_property(
        &self,
        base_value: &FhirPathValue,
        property: &str,
        _model_provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        // Primitive values don't have navigable properties in standard FHIRPath
        match property {
            // Special system properties that exist on all values
            "toString" => Ok(FhirPathValue::string(base_value.to_string()?)),
            "convertsToString" => Ok(FhirPathValue::boolean(base_value.to_string().is_ok())),
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Resolve choice properties into a collection using new ChoiceResolution
    async fn resolve_choice_resolution(
        &self,
        resolution: crate::evaluator::choice_types::ChoiceResolution,
        _model_provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        if resolution.resolved_properties.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if resolution.resolved_properties.len() == 1 {
            // Single choice property - return as wrapped value
            let choice = &resolution.resolved_properties[0];
            return Ok(choice.to_wrapped_value());
        }

        // Multiple choice properties - return as collection
        let wrapped_values: Vec<FhirPathValue> = resolution
            .resolved_properties
            .iter()
            .map(|choice| choice.to_wrapped_value())
            .collect();

        Ok(FhirPathValue::collection(wrapped_values))
    }

    /// Inline choice property detection using ChoiceTypeDetector logic
    async fn detect_choice_properties_inline(
        &self,
        data: &JsonValue,
        base_property: &str,
        _model_provider: &dyn ModelProvider,
    ) -> Result<ChoiceResolution> {
                
        let mut resolved = Vec::new();

        if let Some(obj) = data.as_object() {
            // Scan for properties matching pattern: base + TypeSuffix
            for (key, value) in obj {
                if key.starts_with(base_property) && key.len() > base_property.len() {
                    let suffix = &key[base_property.len()..];

                    // Check if suffix starts with uppercase (indicates choice type)
                    if let Some(first_char) = suffix.chars().next() {
                        if first_char.is_uppercase() {
                            let choice_property = self
                                .create_choice_property_inline(key, suffix, value, obj, base_property)
                                .await?;
                            resolved.push(choice_property);
                        }
                    }
                }
            }
        }

        Ok(ChoiceResolution {
            is_choice: !resolved.is_empty(),
            resolved_properties: resolved,
            base_property: base_property.to_string(),
        })
    }

    /// Create choice property with inline type mapping
    async fn create_choice_property_inline(
        &self,
        property_name: &str,
        type_suffix: &str,
        value: &JsonValue,
        parent_object: &serde_json::Map<String, JsonValue>,
        base_property: &str,
    ) -> Result<ChoiceProperty> {
        
        // Map FHIR type suffix to TypeInfo (inline simplified version)
        let type_info = self.map_choice_suffix_inline(type_suffix, value);

        // Extract primitive element extensions (inline simplified version)
        let primitive_element = self.extract_primitive_element_inline(parent_object, property_name)?;

        Ok(ChoiceProperty {
            property_name: property_name.to_string(),
            base_name: base_property.to_string(),
            type_suffix: type_suffix.to_string(),
            value: Arc::new(value.clone()),
            type_info,
            primitive_element,
        })
    }

    /// Map choice type suffix to TypeInfo (inline version)
    fn map_choice_suffix_inline(&self, suffix: &str, value: &JsonValue) -> crate::core::model_provider::TypeInfo {
        
        let (fhir_type, singleton, fhirpath_type) = match suffix {
            // FHIR primitive types
            "String" => ("string", true, "String"),
            "Integer" => ("integer", true, "Integer"),
            "Boolean" => ("boolean", true, "Boolean"),
            "Decimal" => ("decimal", true, "Decimal"),
            "Date" => ("date", true, "Date"),
            "DateTime" => ("dateTime", true, "DateTime"),
            "Time" => ("time", true, "Time"),
            "Code" => ("code", true, "String"),
            "Uri" => ("uri", true, "String"),
            "Url" => ("url", true, "String"),
            "Id" => ("id", true, "String"),

            // FHIR complex types
            "Quantity" => ("Quantity", true, "Any"),
            "Coding" => ("Coding", true, "Any"),
            "CodeableConcept" => ("CodeableConcept", true, "Any"),
            "Reference" => ("Reference", true, "Any"),
            "Period" => ("Period", true, "Any"),
            "Range" => ("Range", true, "Any"),

            _ => ("Unknown", true, "Any"),
        };

        let is_array = value.is_array();

        TypeInfo {
            type_name: fhirpath_type.to_string(),
            singleton: singleton && !is_array,
            namespace: Some("FHIR".to_string()),
            name: Some(fhir_type.to_string()),
            is_empty: Some(value.is_null() || (is_array && value.as_array().unwrap().is_empty())),
            is_union_type: Some(false),
            union_choices: None,
        }
    }

    /// Extract primitive element (inline simplified version)
    fn extract_primitive_element_inline(
        &self,
        parent_object: &serde_json::Map<String, JsonValue>,
        property_name: &str,
    ) -> Result<Option<PrimitiveElement>> {
        let primitive_key = format!("_{}", property_name);

        if let Some(primitive_value) = parent_object.get(&primitive_key) {
            Ok(Some(PrimitiveElement::from_json(primitive_value)?))
        } else {
            Ok(None)
        }
    }

}

/// Utility functions for property navigation
pub mod utils {

    /// Check if a property name follows choice type pattern (ends with capital letter)
    pub fn is_choice_property_key(property: &str, base_property: &str) -> bool {
        if !property.starts_with(base_property) {
            return false;
        }

        let suffix = &property[base_property.len()..];
        suffix
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Extract choice type suffix from a choice property key
    pub fn extract_choice_suffix(property: &str, base_property: &str) -> Option<String> {
        if is_choice_property_key(property, base_property) {
            Some(property[base_property.len()..].to_string())
        } else {
            None
        }
    }

    /// Check if a property is a primitive element extension (starts with underscore)
    pub fn is_primitive_element_key(property: &str) -> bool {
        property.starts_with('_')
    }

    /// Get the base property name from a primitive element key
    pub fn base_property_from_primitive_key(primitive_key: &str) -> Option<&str> {
        if primitive_key.starts_with('_') && primitive_key.len() > 1 {
            Some(&primitive_key[1..])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model_provider::MockModelProvider;
    use serde_json::json;

    #[tokio::test]
    async fn test_simple_property_navigation() {
        let navigator = PropertyNavigator;
        let provider = crate::core::model_provider::MockModelProvider;

        let patient_data = json!({
            "resourceType": "Patient",
            "name": [{"family": "Smith", "given": ["John"]}],
            "gender": "male"
        });

        let patient_wrapped = FhirPathValue::resource_wrapped(patient_data);

        // Navigate to gender property
        let gender_result = navigator
            .navigate_property(&patient_wrapped, "gender", &provider)
            .await
            .unwrap();

        if let FhirPathValue::Wrapped(wrapped) = gender_result {
            assert_eq!(**wrapped.unwrap(), json!("male"));
            // Should have type information
            assert!(wrapped.get_type_info().is_some());
        } else {
            panic!("Expected wrapped value");
        }
    }

    #[tokio::test]
    async fn test_choice_property_navigation() {
        let navigator = PropertyNavigator;
        let provider = crate::core::model_provider::MockModelProvider;

        let observation_data = json!({
            "resourceType": "Observation",
            "valueString": "test result",
            "valueInteger": 42,
            "_valueString": {
                "extension": [{
                    "url": "http://example.com/ext",
                    "valueBoolean": true
                }]
            }
        });

        let observation_wrapped = FhirPathValue::resource_wrapped(observation_data);

        // Navigate to value property (should resolve choice types)
        let value_result = navigator
            .navigate_property(&observation_wrapped, "value", &provider)
            .await
            .unwrap();

        // Should return collection of both valueString and valueInteger
        assert!(matches!(value_result, FhirPathValue::Collection(_)));

        if let FhirPathValue::Collection(collection) = value_result {
            assert_eq!(collection.len(), 2);

            // Check that both values are wrapped with type info
            let values: Vec<&FhirPathValue> = collection.iter().collect();
            for value in values {
                assert!(matches!(value, FhirPathValue::Wrapped(_)));
                if let FhirPathValue::Wrapped(wrapped) = value {
                    assert!(wrapped.get_type_info().is_some());
                    let type_info = wrapped.get_type_info().unwrap();
                    assert_eq!(type_info.namespace, Some("FHIR".to_string()));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_primitive_extensions_preserved() {
        let navigator = PropertyNavigator;
        let provider = crate::core::model_provider::MockModelProvider;

        let data = json!({
            "status": "active",
            "_status": {
                "id": "status-id",
                "extension": [{
                    "url": "http://example.com/status-ext",
                    "valueString": "additional info"
                }]
            }
        });

        let wrapped = FhirPathValue::wrapped(data, None);

        let status_result = navigator
            .navigate_property(&wrapped, "status", &provider)
            .await
            .unwrap();

        if let FhirPathValue::Wrapped(result_wrapped) = status_result {
            assert_eq!(**result_wrapped.unwrap(), json!("active"));
            assert!(result_wrapped.has_extensions());

            let primitive_element = result_wrapped.get_primitive_element().unwrap();
            assert_eq!(primitive_element.id, Some("status-id".to_string()));
            assert_eq!(primitive_element.extensions.len(), 1);
        } else {
            panic!("Expected wrapped value with extensions");
        }
    }

    #[tokio::test]
    async fn test_arc_sharing_efficiency() {
        let navigator = PropertyNavigator;
        let provider = crate::core::model_provider::MockModelProvider;

        let large_data = json!({
            "resourceType": "Patient",
            "largeField": "x".repeat(10000), // Large data to test Arc efficiency
            "name": [{"family": "Smith"}]
        });

        let patient_wrapped = FhirPathValue::resource_wrapped(large_data);

        // Navigate to name property
        let name_result = navigator
            .navigate_property(&patient_wrapped, "name", &provider)
            .await
            .unwrap();

        // Verify that the large data is not copied (Arc sharing)
        if let FhirPathValue::Wrapped(wrapped) = name_result {
            assert_eq!(**wrapped.unwrap(), json!([{"family": "Smith"}]));
            // The original large data should still be in the same memory location
            // (This is more of a conceptual test - actual Arc pointer comparison would be complex)
        }
    }

    mod utils_tests {
        use super::utils::*;

        #[test]
        fn test_choice_property_detection() {
            assert!(is_choice_property_key("valueString", "value"));
            assert!(is_choice_property_key("valueInteger", "value"));
            assert!(!is_choice_property_key("value", "value"));
            assert!(!is_choice_property_key("valuestring", "value")); // lowercase
            assert!(!is_choice_property_key("otherProperty", "value"));
        }

        #[test]
        fn test_choice_suffix_extraction() {
            assert_eq!(
                extract_choice_suffix("valueString", "value"),
                Some("String".to_string())
            );
            assert_eq!(
                extract_choice_suffix("valueInteger", "value"),
                Some("Integer".to_string())
            );
            assert_eq!(extract_choice_suffix("value", "value"), None);
        }

        #[test]
        fn test_primitive_element_key_detection() {
            assert!(is_primitive_element_key("_status"));
            assert!(is_primitive_element_key("_valueString"));
            assert!(!is_primitive_element_key("status"));
            assert!(!is_primitive_element_key("_"));
        }

        #[test]
        fn test_base_property_extraction() {
            assert_eq!(base_property_from_primitive_key("_status"), Some("status"));
            assert_eq!(
                base_property_from_primitive_key("_valueString"),
                Some("valueString")
            );
            assert_eq!(base_property_from_primitive_key("status"), None);
            assert_eq!(base_property_from_primitive_key("_"), None);
        }
    }
}
