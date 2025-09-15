//! Data-aware choice type resolution for polymorphic FHIR properties
//!
//! This module implements intelligent choice type resolution using actual JSON data
//! to detect polymorphic properties (value[x], effective[x]) and map them to proper
//! FHIR types with namespace support and primitive element extension handling.

use crate::core::{
    error::{FhirPathError, Result},
    error_code::*,
    model_provider::{ModelProvider, TypeInfo},
    wrapped::{FhirPathWrapped, PrimitiveElement},
    Collection, FhirPathValue,
};
use serde_json::Value as JsonValue;
use std::sync::Arc;

/// Individual choice property detected in FHIR data
#[derive(Debug, Clone)]
pub struct ChoiceProperty {
    /// Full property name (e.g., "valueString", "valueInteger")
    pub property_name: String,
    /// Base property name (e.g., "value")
    pub base_name: String,
    /// Type suffix from property name (e.g., "String", "Integer")
    pub type_suffix: String,
    /// Actual JSON value for this property
    pub value: Arc<JsonValue>,
    /// Resolved FHIR type information
    pub type_info: TypeInfo,
    /// Extensions from primitive element (e.g., _valueString)
    pub primitive_element: Option<PrimitiveElement>,
}

/// Result of choice type detection for a base property
#[derive(Debug, Clone)]
pub struct ChoiceResolution {
    /// All resolved choice properties found
    pub resolved_properties: Vec<ChoiceProperty>,
    /// Whether any choice properties were detected
    pub is_choice: bool,
    /// The base property name that was searched
    pub base_property: String,
}

/// Detector for data-aware choice type resolution
#[derive(Debug)]
pub struct ChoiceTypeDetector {
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
}

impl ChoiceTypeDetector {
    /// Create new choice type detector
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self { model_provider }
    }

    /// Detect choice properties in JSON data
    ///
    /// Scans the JSON object for properties matching the pattern: base + TypeSuffix
    /// where TypeSuffix starts with an uppercase letter (indicating choice type).
    ///
    /// # Arguments
    /// * `data` - JSON object to scan for choice properties
    /// * `base_property` - Base property name to search for (e.g., "value", "effective")
    pub async fn detect_choice_properties(
        &self,
        data: &JsonValue,
        base_property: &str,
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
                                .resolve_choice_property(key, suffix, value, obj, base_property)
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

    /// Resolve individual choice property with type mapping and extensions
    async fn resolve_choice_property(
        &self,
        property_name: &str,
        type_suffix: &str,
        value: &JsonValue,
        parent_object: &serde_json::Map<String, JsonValue>,
        base_property: &str,
    ) -> Result<ChoiceProperty> {
        // Map FHIR type suffix to proper type info
        let type_info = self.map_choice_type_suffix(type_suffix, value).await?;

        // Extract primitive element extensions
        let primitive_element = self.extract_primitive_element(parent_object, property_name)?;

        Ok(ChoiceProperty {
            property_name: property_name.to_string(),
            base_name: base_property.to_string(),
            type_suffix: type_suffix.to_string(),
            value: Arc::new(value.clone()),
            type_info,
            primitive_element,
        })
    }

    /// Map FHIR choice type suffix to proper TypeInfo with dynamic ModelProvider resolution
    async fn map_choice_type_suffix(&self, suffix: &str, value: &JsonValue) -> Result<TypeInfo> {
        let is_array = value.is_array();

        // First, try to get type information from ModelProvider
        if let Ok(Some(provider_type_info)) = self.model_provider.get_type(suffix).await {
            // ModelProvider knows about this type - use its information
            return Ok(TypeInfo {
                type_name: provider_type_info.type_name,
                singleton: provider_type_info.singleton && !is_array,
                namespace: provider_type_info.namespace,
                name: provider_type_info.name,
                is_empty: Some(value.is_null() || (is_array && value.as_array().unwrap().is_empty())),
                is_union_type: provider_type_info.is_union_type,
                union_choices: provider_type_info.union_choices,
            });
        }

        // ModelProvider doesn't know this type - graceful fallback
        let (fhir_name, fhirpath_type) = self.infer_type_from_suffix(suffix);

        Ok(TypeInfo {
            type_name: fhirpath_type,
            singleton: !is_array, // Default singleton unless it's an array
            namespace: Some("FHIR".to_string()),
            name: Some(fhir_name),
            is_empty: Some(value.is_null() || (is_array && value.as_array().unwrap().is_empty())),
            is_union_type: Some(false), // Fallback types are not unions
            union_choices: None,
        })
    }

    /// Infer type information from choice type suffix when ModelProvider doesn't know the type
    fn infer_type_from_suffix(&self, suffix: &str) -> (String, String) {
        // Common FHIR primitive types - map to their FHIRPath equivalents
        match suffix {
            // FHIR primitive types that have direct FHIRPath equivalents
            "String" => ("string".to_string(), "String".to_string()),
            "Integer" => ("integer".to_string(), "Integer".to_string()),
            "Boolean" => ("boolean".to_string(), "Boolean".to_string()),
            "Decimal" => ("decimal".to_string(), "Decimal".to_string()),
            "Date" => ("date".to_string(), "Date".to_string()),
            "DateTime" => ("dateTime".to_string(), "DateTime".to_string()),
            "Time" => ("time".to_string(), "Time".to_string()),

            // FHIR string-like primitives that map to String in FHIRPath
            "Code" | "Uri" | "Url" | "Id" | "Oid" | "Uuid" | "Canonical" | "Markdown" => {
                (suffix.to_lowercase(), "String".to_string())
            },

            // For any unknown type, use the suffix as FHIR name, "Any" for FHIRPath type
            _ => {
                (suffix.to_string(), "Any".to_string())
            }
        }
    }

    /// Extract primitive element extensions from parent object
    ///
    /// FHIR primitive elements can have extensions stored in a parallel property
    /// with an underscore prefix (e.g., _valueString for valueString extensions).
    fn extract_primitive_element(
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

    /// Convert choice resolution to wrapped FHIRPath values
    pub fn choice_resolution_to_wrapped_values(
        &self,
        resolution: &ChoiceResolution,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        for choice_prop in &resolution.resolved_properties {
            let wrapped_value = FhirPathValue::Wrapped(FhirPathWrapped {
                value: choice_prop.value.clone(),
                type_info: Some(choice_prop.type_info.clone()),
                primitive_element: choice_prop.primitive_element.clone(),
            });
            results.push(wrapped_value);
        }

        Ok(results)
    }

    /// Check if a property name follows choice type pattern
    pub fn is_choice_property_name(&self, property_name: &str, base_property: &str) -> bool {
        if !property_name.starts_with(base_property) {
            return false;
        }

        let suffix = &property_name[base_property.len()..];
        suffix
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Get choice type suffix from property name
    pub fn extract_choice_suffix(&self, property_name: &str, base_property: &str) -> Option<String> {
        if self.is_choice_property_name(property_name, base_property) {
            Some(property_name[base_property.len()..].to_string())
        } else {
            None
        }
    }

}

impl ChoiceProperty {
    /// Convert choice property to wrapped FHIRPath value
    pub fn to_wrapped_value(&self) -> FhirPathValue {
        FhirPathValue::Wrapped(FhirPathWrapped {
            value: self.value.clone(),
            type_info: Some(self.type_info.clone()),
            primitive_element: self.primitive_element.clone(),
        })
    }

    /// Check if this choice property has primitive extensions
    pub fn has_extensions(&self) -> bool {
        self.primitive_element
            .as_ref()
            .map(|pe| pe.has_extensions())
            .unwrap_or(false)
    }

    /// Get the FHIRPath type name for this choice property
    pub fn get_fhirpath_type(&self) -> &str {
        &self.type_info.type_name
    }

    /// Get the FHIR type name for this choice property
    pub fn get_fhir_type(&self) -> Option<&str> {
        self.type_info.name.as_deref()
    }

    /// Check if this is a primitive choice type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self.type_info.type_name.as_str(),
            "String" | "Integer" | "Boolean" | "Decimal" | "Date" | "DateTime" | "Time"
        )
    }

    /// Check if this is a complex choice type
    pub fn is_complex(&self) -> bool {
        self.type_info.type_name == "Any"
    }
}

impl ChoiceResolution {
    /// Create empty choice resolution
    pub fn empty(base_property: String) -> Self {
        Self {
            resolved_properties: Vec::new(),
            is_choice: false,
            base_property,
        }
    }

    /// Get choice property by type suffix
    pub fn get_by_suffix(&self, suffix: &str) -> Option<&ChoiceProperty> {
        self.resolved_properties
            .iter()
            .find(|prop| prop.type_suffix == suffix)
    }

    /// Get choice property by property name
    pub fn get_by_property_name(&self, property_name: &str) -> Option<&ChoiceProperty> {
        self.resolved_properties
            .iter()
            .find(|prop| prop.property_name == property_name)
    }

    /// Get all type suffixes found
    pub fn get_type_suffixes(&self) -> Vec<&str> {
        self.resolved_properties
            .iter()
            .map(|prop| prop.type_suffix.as_str())
            .collect()
    }

    /// Convert to collection of wrapped values
    pub fn to_wrapped_collection(&self) -> Result<Collection> {
        let values: Vec<FhirPathValue> = self
            .resolved_properties
            .iter()
            .map(|prop| prop.to_wrapped_value())
            .collect();

        Ok(Collection::from_values(values))
    }

    /// Get the first choice property (for single property case)
    pub fn first(&self) -> Option<&ChoiceProperty> {
        self.resolved_properties.first()
    }

    /// Check if only primitive choice types were found
    pub fn is_all_primitive(&self) -> bool {
        !self.resolved_properties.is_empty()
            && self.resolved_properties.iter().all(|prop| prop.is_primitive())
    }

    /// Check if any complex choice types were found
    pub fn has_complex_types(&self) -> bool {
        self.resolved_properties.iter().any(|prop| prop.is_complex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model_provider::MockModelProvider;
    use serde_json::json;

    #[tokio::test]
    async fn test_choice_property_detection() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let observation = json!({
            "resourceType": "Observation",
            "valueString": "test value",
            "valueInteger": 42,
            "_valueString": {
                "extension": [{
                    "url": "http://example.com/ext",
                    "valueString": "extension value"
                }]
            }
        });

        let resolution = detector
            .detect_choice_properties(&observation, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 2);

        // Check valueString resolution
        let string_prop = resolution
            .resolved_properties
            .iter()
            .find(|p| p.type_suffix == "String")
            .unwrap();
        assert_eq!(string_prop.property_name, "valueString");
        assert_eq!(string_prop.base_name, "value");
        assert_eq!(string_prop.type_info.type_name, "String");
        assert_eq!(string_prop.type_info.namespace, Some("FHIR".to_string()));
        assert_eq!(string_prop.type_info.name, Some("string".to_string()));
        assert!(string_prop.primitive_element.is_some());
        assert!(string_prop.has_extensions());

        // Check valueInteger resolution
        let integer_prop = resolution
            .resolved_properties
            .iter()
            .find(|p| p.type_suffix == "Integer")
            .unwrap();
        assert_eq!(integer_prop.property_name, "valueInteger");
        assert_eq!(integer_prop.type_info.type_name, "Integer");
        assert_eq!(integer_prop.type_info.namespace, Some("FHIR".to_string()));
        assert_eq!(integer_prop.type_info.name, Some("integer".to_string()));
        assert!(integer_prop.primitive_element.is_none());
    }

    #[tokio::test]
    async fn test_complex_choice_types() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let observation = json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 123.45,
                "unit": "mg",
                "system": "http://unitsofmeasure.org",
                "code": "mg"
            },
            "valueCodeableConcept": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "LA6113-0",
                    "display": "Positive"
                }]
            }
        });

        let resolution = detector
            .detect_choice_properties(&observation, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 2);
        assert!(resolution.has_complex_types());

        // Complex types should map to "Any" in FHIRPath
        let quantity_prop = resolution
            .resolved_properties
            .iter()
            .find(|p| p.type_suffix == "Quantity")
            .unwrap();
        assert_eq!(quantity_prop.type_info.type_name, "Any");
        assert_eq!(quantity_prop.type_info.name, Some("Quantity".to_string()));
        assert!(quantity_prop.is_complex());

        let concept_prop = resolution
            .resolved_properties
            .iter()
            .find(|p| p.type_suffix == "CodeableConcept")
            .unwrap();
        assert_eq!(concept_prop.type_info.type_name, "Any");
        assert_eq!(concept_prop.type_info.name, Some("CodeableConcept".to_string()));
    }

    #[tokio::test]
    async fn test_no_choice_properties() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let observation = json!({
            "resourceType": "Observation",
            "status": "final",
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "15074-8"
                }]
            }
        });

        let resolution = detector
            .detect_choice_properties(&observation, "value")
            .await
            .unwrap();

        assert!(!resolution.is_choice);
        assert!(resolution.resolved_properties.is_empty());
    }

    #[tokio::test]
    async fn test_array_choice_properties() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let data = json!({
            "effectiveString": ["2023-01-01", "2023-01-02"],
            "effectiveDateTime": ["2023-01-01T10:00:00Z"]
        });

        let resolution = detector
            .detect_choice_properties(&data, "effective")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 2);

        // Array properties should not be singleton
        let string_prop = resolution.get_by_suffix("String").unwrap();
        assert!(!string_prop.type_info.singleton);

        let datetime_prop = resolution.get_by_suffix("DateTime").unwrap();
        assert!(!datetime_prop.type_info.singleton);
    }

    #[test]
    fn test_choice_property_utilities() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Test choice property name detection
        assert!(detector.is_choice_property_name("valueString", "value"));
        assert!(detector.is_choice_property_name("effectiveDateTime", "effective"));
        assert!(!detector.is_choice_property_name("value", "value"));
        assert!(!detector.is_choice_property_name("valuestring", "value")); // lowercase

        // Test suffix extraction
        assert_eq!(
            detector.extract_choice_suffix("valueString", "value"),
            Some("String".to_string())
        );
        assert_eq!(
            detector.extract_choice_suffix("effectiveDate", "effective"),
            Some("Date".to_string())
        );
        assert_eq!(detector.extract_choice_suffix("value", "value"), None);

        // Test choice property name detection
        assert!(detector.is_choice_property_name("valueString", "value"));
        assert!(detector.is_choice_property_name("effectiveDateTime", "effective"));
        assert!(!detector.is_choice_property_name("status", "value"));
        assert!(!detector.is_choice_property_name("valuestring", "value")); // lowercase
    }

    #[tokio::test]
    async fn test_wrapped_value_conversion() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let observation = json!({
            "valueString": "test result",
            "valueInteger": 100
        });

        let resolution = detector
            .detect_choice_properties(&observation, "value")
            .await
            .unwrap();

        let wrapped_values = detector
            .choice_resolution_to_wrapped_values(&resolution)
            .unwrap();

        assert_eq!(wrapped_values.len(), 2);

        // Verify wrapped values
        for value in wrapped_values {
            match value {
                FhirPathValue::Wrapped(wrapped) => {
                    assert!(wrapped.get_type_info().is_some());
                    let type_info = wrapped.get_type_info().unwrap();
                    assert_eq!(type_info.namespace, Some("FHIR".to_string()));
                    assert!(matches!(
                        type_info.type_name.as_str(),
                        "String" | "Integer"
                    ));
                }
                _ => panic!("Expected wrapped value"),
            }
        }
    }

    #[tokio::test]
    async fn test_medication_effective_choice_types() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let medication = json!({
            "resourceType": "Medication",
            "effectiveDateTime": "2023-12-01T10:00:00Z",
            "effectivePeriod": {
                "start": "2023-12-01",
                "end": "2023-12-31"
            },
            "_effectiveDateTime": {
                "extension": [{
                    "url": "http://example.com/precision",
                    "valueString": "minute"
                }]
            }
        });

        let resolution = detector
            .detect_choice_properties(&medication, "effective")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 2);

        // Check effectiveDateTime
        let datetime_prop = resolution.get_by_suffix("DateTime").unwrap();
        assert_eq!(datetime_prop.property_name, "effectiveDateTime");
        assert_eq!(datetime_prop.type_info.type_name, "DateTime");
        assert!(datetime_prop.has_extensions());

        // Check effectivePeriod
        let period_prop = resolution.get_by_suffix("Period").unwrap();
        assert_eq!(period_prop.property_name, "effectivePeriod");
        assert_eq!(period_prop.type_info.type_name, "Any");
        assert_eq!(period_prop.type_info.name, Some("Period".to_string()));
        assert!(!period_prop.has_extensions());
    }

    #[tokio::test]
    async fn test_all_primitive_choice_types() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "valueString": "text value",
            "valueInteger": 42,
            "valueBoolean": true,
            "valueDecimal": 123.45,
            "valueDate": "2023-12-01",
            "valueDateTime": "2023-12-01T10:00:00Z",
            "valueTime": "10:00:00",
            "valueCode": "active",
            "valueUri": "http://example.com",
            "valueUrl": "https://example.com/resource",
            "valueId": "patient-123",
            "valueOid": "1.2.3.4.5",
            "valueUuid": "550e8400-e29b-41d4-a716-446655440000"
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 13);

        // Verify all primitive types map to correct FHIRPath types
        let expected_mappings = vec![
            ("String", "String"),
            ("Integer", "Integer"),
            ("Boolean", "Boolean"),
            ("Decimal", "Decimal"),
            ("Date", "Date"),
            ("DateTime", "DateTime"),
            ("Time", "Time"),
            ("Code", "String"), // Code maps to String in FHIRPath
            ("Uri", "String"),  // URI maps to String in FHIRPath
            ("Url", "String"),  // URL maps to String in FHIRPath
            ("Id", "String"),   // ID maps to String in FHIRPath
            ("Oid", "String"),  // OID maps to String in FHIRPath
            ("Uuid", "String"), // UUID maps to String in FHIRPath
        ];

        for (suffix, expected_type) in expected_mappings {
            let prop = resolution.get_by_suffix(suffix).unwrap();
            assert_eq!(prop.type_info.type_name, expected_type);
            assert_eq!(prop.type_info.namespace, Some("FHIR".to_string()));
            assert!(prop.type_info.singleton);
            assert!(prop.is_primitive() || expected_type == "String");
        }
    }

    #[tokio::test]
    async fn test_all_complex_choice_types() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "valueQuantity": {
                "value": 123.45,
                "unit": "mg",
                "system": "http://unitsofmeasure.org",
                "code": "mg"
            },
            "valueCoding": {
                "system": "http://loinc.org",
                "code": "LA6113-0",
                "display": "Positive"
            },
            "valueCodeableConcept": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "LA6113-0"
                }]
            },
            "valueReference": {
                "reference": "Patient/123"
            },
            "valuePeriod": {
                "start": "2023-01-01",
                "end": "2023-12-31"
            },
            "valueRange": {
                "low": {"value": 10, "unit": "mg"},
                "high": {"value": 20, "unit": "mg"}
            },
            "valueAttachment": {
                "contentType": "image/png",
                "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg=="
            }
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 7);
        assert!(resolution.has_complex_types());

        // Verify all complex types map to "Any" in FHIRPath
        let expected_complex_types = vec![
            "Quantity", "Coding", "CodeableConcept", "Reference", "Period", "Range", "Attachment"
        ];

        for complex_type in expected_complex_types {
            let prop = resolution.get_by_suffix(complex_type).unwrap();
            assert_eq!(prop.type_info.type_name, "Any");
            assert_eq!(prop.type_info.name, Some(complex_type.to_string()));
            assert_eq!(prop.type_info.namespace, Some("FHIR".to_string()));
            assert!(prop.type_info.singleton);
            assert!(prop.is_complex());
        }
    }

    #[tokio::test]
    async fn test_unknown_choice_type_suffix_graceful_fallback() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "valueUnknownType": "some value"
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await;

        // Should succeed with graceful fallback (not error like before)
        assert!(resolution.is_ok());
        let resolution = resolution.unwrap();
        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 1);

        let prop = resolution.first().unwrap();
        assert_eq!(prop.property_name, "valueUnknownType");
        assert_eq!(prop.type_suffix, "UnknownType");
        assert_eq!(prop.type_info.type_name, "Any"); // Fallback to "Any"
        assert_eq!(prop.type_info.namespace, Some("FHIR".to_string()));
        assert_eq!(prop.type_info.name, Some("UnknownType".to_string()));
    }

    #[tokio::test]
    async fn test_empty_choice_resolution() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "status": "active",
            "code": {
                "coding": [{"system": "http://loinc.org", "code": "15074-8"}]
            }
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();

        assert!(!resolution.is_choice);
        assert!(resolution.resolved_properties.is_empty());
        assert_eq!(resolution.base_property, "value");

        // Test empty resolution methods
        assert!(resolution.first().is_none());
        assert!(resolution.get_type_suffixes().is_empty());
        assert!(!resolution.is_all_primitive());
        assert!(!resolution.has_complex_types());

        let collection = resolution.to_wrapped_collection().unwrap();
        assert!(collection.is_empty());
    }

    #[tokio::test]
    async fn test_single_choice_property_resolution() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "valueString": "single choice value"
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 1);
        assert!(resolution.is_all_primitive());
        assert!(!resolution.has_complex_types());

        let first_prop = resolution.first().unwrap();
        assert_eq!(first_prop.property_name, "valueString");
        assert_eq!(first_prop.type_suffix, "String");

        let suffixes = resolution.get_type_suffixes();
        assert_eq!(suffixes, vec!["String"]);
    }

    #[tokio::test]
    async fn test_mixed_primitive_and_complex_choice_types() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "valueString": "text value",
            "valueQuantity": {
                "value": 100,
                "unit": "mg"
            },
            "valueBoolean": false
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 3);
        assert!(!resolution.is_all_primitive()); // Mix of primitive and complex
        assert!(resolution.has_complex_types());

        // Should find both primitive and complex types
        let primitive_count = resolution.resolved_properties.iter()
            .filter(|prop| prop.is_primitive())
            .count();
        let complex_count = resolution.resolved_properties.iter()
            .filter(|prop| prop.is_complex())
            .count();

        assert_eq!(primitive_count, 2); // String and Boolean
        assert_eq!(complex_count, 1);   // Quantity
    }

    #[tokio::test]
    async fn test_nested_choice_property_detection() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let bundle_entry = json!({
            "resource": {
                "resourceType": "Observation",
                "valueString": "nested value",
                "valueInteger": 42
            },
            "effectiveDateTime": "2023-12-01T10:00:00Z"
        });

        // Test detecting choice properties in nested resource
        let resource_resolution = detector
            .detect_choice_properties(&bundle_entry["resource"], "value")
            .await
            .unwrap();

        assert!(resource_resolution.is_choice);
        assert_eq!(resource_resolution.resolved_properties.len(), 2);

        // Test detecting choice properties at bundle entry level
        let entry_resolution = detector
            .detect_choice_properties(&bundle_entry, "effective")
            .await
            .unwrap();

        assert!(entry_resolution.is_choice);
        assert_eq!(entry_resolution.resolved_properties.len(), 1);
        assert_eq!(entry_resolution.first().unwrap().property_name, "effectiveDateTime");
    }

    #[tokio::test]
    async fn test_choice_property_with_null_values() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let test_data = json!({
            "valueString": null,
            "valueInteger": 42
        });

        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 2);

        // Check that null value is properly handled
        let string_prop = resolution.get_by_suffix("String").unwrap();
        assert!(string_prop.type_info.is_empty == Some(true));

        let integer_prop = resolution.get_by_suffix("Integer").unwrap();
        assert!(integer_prop.type_info.is_empty == Some(false));
    }

    #[tokio::test]
    async fn test_choice_property_utility_methods() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Test choice property name detection
        assert!(detector.is_choice_property_name("valueString", "value"));
        assert!(detector.is_choice_property_name("effectiveDateTime", "effective"));
        assert!(!detector.is_choice_property_name("status", "value"));
        assert!(!detector.is_choice_property_name("valuestring", "value")); // lowercase

        // Test suffix extraction
        assert_eq!(
            detector.extract_choice_suffix("valueString", "value"),
            Some("String".to_string())
        );
        assert_eq!(
            detector.extract_choice_suffix("effectivePeriod", "effective"),
            Some("Period".to_string())
        );
        assert_eq!(
            detector.extract_choice_suffix("status", "value"),
            None
        );
    }

    #[tokio::test]
    async fn test_performance_with_large_choice_object() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Create a large object with many properties
        let mut large_object = serde_json::Map::new();
        
        // Add choice properties
        large_object.insert("valueString".to_string(), json!("test"));
        large_object.insert("valueInteger".to_string(), json!(42));
        large_object.insert("valueBoolean".to_string(), json!(true));
        
        // Add many non-choice properties to test performance
        for i in 0..1000 {
            large_object.insert(format!("property{}", i), json!(format!("value{}", i)));
        }

        let test_data = JsonValue::Object(large_object);

        // Should still efficiently find choice properties
        let start = std::time::Instant::now();
        let resolution = detector
            .detect_choice_properties(&test_data, "value")
            .await
            .unwrap();
        let duration = start.elapsed();

        assert!(resolution.is_choice);
        assert_eq!(resolution.resolved_properties.len(), 3);
        
        // Should complete quickly even with large object
        assert!(duration.as_millis() < 100, "Detection took too long: {:?}", duration);
    }
}

// Additional integration test module
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::model_provider::MockModelProvider;
    use serde_json::json;

    #[tokio::test]
    async fn test_real_observation_example() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Real FHIR Observation example with multiple choice types
        let observation = json!({
            "resourceType": "Observation",
            "id": "example",
            "status": "final",
            "category": [{
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                    "code": "vital-signs",
                    "display": "Vital Signs"
                }]
            }],
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "29463-7",
                    "display": "Body Weight"
                }]
            },
            "subject": {
                "reference": "Patient/example"
            },
            "effectiveDateTime": "2023-12-01T10:00:00Z",
            "valueQuantity": {
                "value": 72.5,
                "unit": "kg",
                "system": "http://unitsofmeasure.org",
                "code": "kg"
            },
            "_effectiveDateTime": {
                "extension": [{
                    "url": "http://hl7.org/fhir/StructureDefinition/observation-effectiveTime-precision",
                    "valueString": "minute"
                }]
            }
        });

        // Test value choice detection
        let value_resolution = detector
            .detect_choice_properties(&observation, "value")
            .await
            .unwrap();

        assert!(value_resolution.is_choice);
        assert_eq!(value_resolution.resolved_properties.len(), 1);
        
        let quantity_prop = value_resolution.first().unwrap();
        assert_eq!(quantity_prop.property_name, "valueQuantity");
        assert_eq!(quantity_prop.type_suffix, "Quantity");
        assert_eq!(quantity_prop.type_info.type_name, "Any");
        assert_eq!(quantity_prop.type_info.name, Some("Quantity".to_string()));

        // Test effective choice detection
        let effective_resolution = detector
            .detect_choice_properties(&observation, "effective")
            .await
            .unwrap();

        assert!(effective_resolution.is_choice);
        assert_eq!(effective_resolution.resolved_properties.len(), 1);

        let datetime_prop = effective_resolution.first().unwrap();
        assert_eq!(datetime_prop.property_name, "effectiveDateTime");
        assert_eq!(datetime_prop.type_suffix, "DateTime");
        assert_eq!(datetime_prop.type_info.type_name, "DateTime");
        assert!(datetime_prop.has_extensions());
    }

    #[tokio::test]
    async fn test_real_medication_example() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        // Real FHIR Medication example
        let medication = json!({
            "resourceType": "Medication",
            "id": "med0301",
            "code": {
                "coding": [{
                    "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                    "code": "1594660",
                    "display": "Tylenol PM"
                }]
            },
            "manufacturer": {
                "reference": "Organization/mmanu"
            },
            "form": {
                "coding": [{
                    "system": "http://snomed.info/sct",
                    "code": "385055001",
                    "display": "Tablet dose form"
                }]
            },
            "ingredient": [{
                "itemCodeableConcept": {
                    "coding": [{
                        "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                        "code": "1389",
                        "display": "Acetaminophen"
                    }]
                },
                "strengthQuantity": {
                    "value": 500,
                    "unit": "mg",
                    "system": "http://unitsofmeasure.org",
                    "code": "mg"
                }
            }]
        });

        // Test ingredient choice detection (itemCodeableConcept vs itemReference)
        for ingredient in medication["ingredient"].as_array().unwrap() {
            let item_resolution = detector
                .detect_choice_properties(ingredient, "item")
                .await
                .unwrap();

            if item_resolution.is_choice {
                assert_eq!(item_resolution.resolved_properties.len(), 1);
                
                let item_prop = item_resolution.first().unwrap();
                assert_eq!(item_prop.property_name, "itemCodeableConcept");
                assert_eq!(item_prop.type_suffix, "CodeableConcept");
                assert_eq!(item_prop.type_info.type_name, "Any");
                assert_eq!(item_prop.type_info.name, Some("CodeableConcept".to_string()));
            }
        }
    }

    #[tokio::test]
    async fn test_bundle_with_multiple_resources() {
        let model_provider = Arc::new(MockModelProvider);
        let detector = ChoiceTypeDetector::new(model_provider);

        let bundle = json!({
            "resourceType": "Bundle",
            "id": "example",
            "type": "collection",
            "entry": [
                {
                    "resource": {
                        "resourceType": "Observation",
                        "valueString": "Normal",
                        "valueQuantity": {
                            "value": 120,
                            "unit": "mmHg"
                        }
                    }
                },
                {
                    "resource": {
                        "resourceType": "Patient",
                        "name": [{
                            "family": "Smith",
                            "given": ["John"]
                        }]
                    }
                }
            ]
        });

        // Test choice detection in bundle entries
        for entry in bundle["entry"].as_array().unwrap() {
            let resource = &entry["resource"];
            
            if resource["resourceType"] == "Observation" {
                let value_resolution = detector
                    .detect_choice_properties(resource, "value")
                    .await
                    .unwrap();

                assert!(value_resolution.is_choice);
                assert_eq!(value_resolution.resolved_properties.len(), 2);

                // Should find both valueString and valueQuantity
                assert!(value_resolution.get_by_suffix("String").is_some());
                assert!(value_resolution.get_by_suffix("Quantity").is_some());
            }
        }
    }
}