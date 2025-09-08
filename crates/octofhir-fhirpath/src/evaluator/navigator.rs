//! Value navigation implementation for FHIRPath property and index access
//!
//! This module implements the ValueNavigator trait which handles:
//! - Property access on FHIR resources and complex types
//! - Index access for arrays and collections
//! - Path-based navigation through nested structures
//! - Proper handling of FHIR choice types and polymorphic properties

use serde_json::Value as JsonValue;
use std::sync::Arc;

use crate::{
    core::{FhirPathValue, ModelProvider, Result},
    evaluator::metadata_navigator::MetadataNavigator,
    evaluator::traits::MetadataAwareNavigator,
    evaluator::traits::ValueNavigator,
    typing::TypeResolver,
    wrapped::{ValueMetadata, WrappedCollection, WrappedValue},
};

/// Implementation of ValueNavigator for FHIRPath navigation
pub struct Navigator;

impl Navigator {
    /// Create a new standard navigator instance
    pub fn new() -> Self {
        Self
    }

    /// Navigate to a property in a JSON object
    fn navigate_json_property(&self, json: &JsonValue, property: &str) -> Result<FhirPathValue> {
        match json {
            JsonValue::Object(obj) => {
                if let Some(value) = obj.get(property) {
                    Ok(self.json_to_fhir_path_value(value.clone()))
                } else {
                    // Check for choice type properties (e.g., value[x] -> valueString, valueInteger)
                    self.check_choice_type_properties(obj, property)
                }
            }
            JsonValue::Array(arr) => {
                // For arrays, navigate the property in each element and collect results
                let mut results = Vec::new();
                for item in arr {
                    match self.navigate_json_property(item, property)? {
                        FhirPathValue::Empty => {} // Skip empty results
                        FhirPathValue::Collection(vec) => {
                            results.extend(vec);
                        }
                        single_value => {
                            results.push(single_value);
                        }
                    }
                }

                Ok(match results.len() {
                    0 => FhirPathValue::Empty,
                    1 => results.into_iter().next().unwrap(),
                    _ => FhirPathValue::Collection(results),
                })
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }

    /// Check for FHIR choice type properties (e.g., value[x])
    ///
    /// In FHIR, choice types like value[x] are represented as concrete properties
    /// like valueString, valueInteger, etc. This method handles the mapping.
    fn check_choice_type_properties(
        &self,
        obj: &serde_json::Map<String, JsonValue>,
        property: &str,
    ) -> Result<FhirPathValue> {
        // Common choice type patterns in FHIR
        let choice_patterns = [
            "value",         // value[x] -> valueString, valueInteger, etc.
            "effective",     // effective[x] -> effectiveDateTime, effectivePeriod, etc.
            "onset",         // onset[x] -> onsetDateTime, onsetAge, etc.
            "abatement",     // abatement[x] -> abatementDateTime, abatementBoolean, etc.
            "multipleBirth", // multipleBirth[x] -> multipleBirthBoolean, multipleBirthInteger
        ];

        if choice_patterns.contains(&property) {
            // Look for any property that starts with the choice type name
            let prefix = format!("{}", property);
            for (key, value) in obj {
                if key.starts_with(&prefix) && key.len() > prefix.len() {
                    // Found a choice type property
                    return Ok(self.json_to_fhir_path_value(value.clone()));
                }
            }
        }

        // Also check if the requested property is a specific choice type
        // e.g., requesting "valueString" when we have value[x]
        for pattern in &choice_patterns {
            if property.starts_with(pattern) && property.len() > pattern.len() {
                if let Some(value) = obj.get(property) {
                    return Ok(self.json_to_fhir_path_value(value.clone()));
                }
            }
        }

        Ok(FhirPathValue::Empty)
    }

    /// Convert JSON value to FhirPathValue
    fn json_to_fhir_path_value(&self, json: JsonValue) -> FhirPathValue {
        match json {
            JsonValue::Null => FhirPathValue::Empty,
            JsonValue::Bool(b) => FhirPathValue::Boolean(b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    match rust_decimal::Decimal::from_f64_retain(f) {
                        Some(d) => FhirPathValue::Decimal(d),
                        None => FhirPathValue::JsonValue(Arc::new(JsonValue::Number(n))),
                    }
                } else {
                    FhirPathValue::JsonValue(Arc::new(JsonValue::Number(n)))
                }
            }
            JsonValue::String(s) => {
                // For now, just return as string - temporal parsing will be added later
                FhirPathValue::String(s.clone())
            }
            JsonValue::Array(arr) => {
                if arr.is_empty() {
                    FhirPathValue::Empty
                } else {
                    let values: Vec<FhirPathValue> = arr
                        .into_iter()
                        .map(|v| self.json_to_fhir_path_value(v))
                        .collect();

                    if values.len() == 1 {
                        values.into_iter().next().unwrap()
                    } else {
                        FhirPathValue::Collection(values)
                    }
                }
            }
            JsonValue::Object(_) => FhirPathValue::Resource(Arc::new(json)),
        }
    }

    /// Navigate to an indexed element in a JSON array
    fn navigate_json_index(&self, json: &JsonValue, index: usize) -> Result<FhirPathValue> {
        match json {
            JsonValue::Array(arr) => {
                if index < arr.len() {
                    Ok(self.json_to_fhir_path_value(arr[index].clone()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => {
                // For non-array values, index 0 returns the value itself, others return empty
                if index == 0 {
                    Ok(self.json_to_fhir_path_value(json.clone()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    /// Parse and navigate a complex path expression
    ///
    /// This handles paths like "name.family" or "telecom.where(system='phone').value"
    /// For now, we'll implement basic dot-notation paths. Complex expressions
    /// would need to be parsed and evaluated recursively.
    fn navigate_simple_path(
        &self,
        value: &FhirPathValue,
        path: &str,
        _provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current_value = value.clone();

        for part in parts {
            // Skip empty parts (e.g., from leading dots)
            if part.is_empty() {
                continue;
            }

            // Navigate to the next property
            current_value = self.navigate_property(&current_value, part, _provider)?;

            // If we get empty, stop navigation
            if current_value == FhirPathValue::Empty {
                break;
            }
        }

        Ok(current_value)
    }

    /// Bridge method to convert plain navigation to metadata-aware navigation
    pub async fn navigate_property_wrapped(
        &self,
        value: &FhirPathValue,
        property: &str,
        base_path: crate::path::CanonicalPath,
        resolver: &TypeResolver,
    ) -> Result<WrappedCollection> {
        // Create temporary wrapped value
        let metadata = ValueMetadata::unknown(base_path);
        let wrapped_source = WrappedValue::new(value.clone(), metadata);

        // Use metadata-aware navigator
        let metadata_navigator = MetadataNavigator::new();
        metadata_navigator
            .navigate_property_with_metadata(&wrapped_source, property, resolver)
            .await
    }

    /// Bridge method for index navigation with metadata
    pub async fn navigate_index_wrapped(
        &self,
        value: &FhirPathValue,
        index: usize,
        base_path: crate::path::CanonicalPath,
        resolver: &TypeResolver,
    ) -> Result<Option<WrappedValue>> {
        // Create temporary wrapped value
        let metadata = ValueMetadata::unknown(base_path);
        let wrapped_source = WrappedValue::new(value.clone(), metadata);

        // Use metadata-aware navigator
        let metadata_navigator = MetadataNavigator::new();
        metadata_navigator
            .navigate_index_with_metadata(&wrapped_source, index, resolver)
            .await
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::new()
    }
}

impl ValueNavigator for Navigator {
    fn navigate_property(
        &self,
        value: &FhirPathValue,
        property: &str,
        _provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Resource(json) => self.navigate_json_property(json, property),
            FhirPathValue::JsonValue(json) => self.navigate_json_property(json, property),
            FhirPathValue::Collection(vec) => {
                // For collections, navigate property in each element
                let mut results = Vec::new();
                for item in vec {
                    match self.navigate_property(item, property, _provider)? {
                        FhirPathValue::Empty => {} // Skip empty results
                        FhirPathValue::Collection(item_vec) => {
                            results.extend(item_vec);
                        }
                        single_value => {
                            results.push(single_value);
                        }
                    }
                }

                Ok(match results.len() {
                    0 => FhirPathValue::Empty,
                    1 => results.into_iter().next().unwrap(),
                    _ => FhirPathValue::Collection(results),
                })
            }
            _ => {
                // For primitive values, property access returns empty unless it's a type match
                if property == value.type_name() {
                    Ok(value.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    fn navigate_index(&self, value: &FhirPathValue, index: usize) -> Result<FhirPathValue> {
        match value {
            FhirPathValue::Resource(json) => self.navigate_json_index(json, index),
            FhirPathValue::JsonValue(json) => self.navigate_json_index(json, index),
            FhirPathValue::Collection(vec) => {
                if index < vec.len() {
                    Ok(vec[index].clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => {
                // For single values, index 0 returns the value, others return empty
                if index == 0 {
                    Ok(value.clone())
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    fn navigate_path(
        &self,
        value: &FhirPathValue,
        path: &str,
        provider: &dyn ModelProvider,
    ) -> Result<FhirPathValue> {
        // For now, handle simple dot-notation paths
        // More complex path expressions would require parsing and recursive evaluation
        if path.contains('.') {
            self.navigate_simple_path(value, path, provider)
        } else {
            // Simple property access
            self.navigate_property(value, path, provider)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockModelProvider;

    impl ModelProvider for MockModelProvider {
        async fn get_resource_type_properties(&self, _resource_type: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }
        async fn is_choice_type_property(
            &self,
            _resource_type: &str,
            _property: &str,
        ) -> Result<bool> {
            Ok(false)
        }
        async fn resolve_reference(
            &self,
            _reference: &str,
            _current_resource: Option<&FhirPathValue>,
        ) -> Result<Option<FhirPathValue>> {
            Ok(None)
        }
        async fn get_property_type(
            &self,
            _resource_type: &str,
            _property: &str,
        ) -> Result<Option<String>> {
            Ok(None)
        }
    }
    use serde_json::json;

    fn create_test_patient() -> FhirPathValue {
        FhirPathValue::resource(json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [
                {
                    "family": "Smith",
                    "given": ["John", "Q"]
                },
                {
                    "family": "Jones",
                    "given": ["Jane"]
                }
            ],
            "telecom": [
                {
                    "system": "phone",
                    "value": "555-1234"
                },
                {
                    "system": "email",
                    "value": "john@example.com"
                }
            ],
            "birthDate": "1990-01-01",
            "active": true,
            "multipleBirthInteger": 1
        }))
    }

    #[test]
    fn test_simple_property_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;
        let patient = create_test_patient();

        // Test simple property access
        let id_result = navigator
            .navigate_property(&patient, "id", &provider)
            .unwrap();
        assert_eq!(id_result, FhirPathValue::String("example".to_string()));

        let active_result = navigator
            .navigate_property(&patient, "active", &provider)
            .unwrap();
        assert_eq!(active_result, FhirPathValue::Boolean(true));

        let birth_date_result = navigator
            .navigate_property(&patient, "birthDate", &provider)
            .unwrap();
        // Should be a string for now (date parsing comes later)
        assert_eq!(
            birth_date_result,
            FhirPathValue::String("1990-01-01".to_string())
        );
    }

    #[test]
    fn test_array_property_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;
        let patient = create_test_patient();

        // Test array property access
        let name_result = navigator
            .navigate_property(&patient, "name", &provider)
            .unwrap();
        match name_result {
            FhirPathValue::Collection(names) => {
                assert_eq!(names.len(), 2);
                // Each should be a Resource (JSON object)
                for name in names {
                    match name {
                        FhirPathValue::Resource(json) => {
                            assert!(json.get("family").is_some());
                        }
                        _ => panic!("Expected Resource in name collection"),
                    }
                }
            }
            _ => panic!("Expected Collection for name array"),
        }

        let telecom_result = navigator
            .navigate_property(&patient, "telecom", &provider)
            .unwrap();
        match telecom_result {
            FhirPathValue::Collection(telecoms) => {
                assert_eq!(telecoms.len(), 2);
            }
            _ => panic!("Expected Collection for telecom array"),
        }
    }

    #[test]
    fn test_choice_type_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;
        let patient = create_test_patient();

        // Test choice type property (multipleBirth[x] -> multipleBirthInteger)
        let multiple_birth_result = navigator
            .navigate_property(&patient, "multipleBirth", &provider)
            .unwrap();
        assert_eq!(multiple_birth_result, FhirPathValue::Integer(1));

        // Test direct access to choice type property
        let multiple_birth_int_result = navigator
            .navigate_property(&patient, "multipleBirthInteger", &provider)
            .unwrap();
        assert_eq!(multiple_birth_int_result, FhirPathValue::Integer(1));
    }

    #[test]
    fn test_missing_property_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;
        let patient = create_test_patient();

        // Test non-existent property
        let missing_result = navigator
            .navigate_property(&patient, "nonexistent", &provider)
            .unwrap();
        assert_eq!(missing_result, FhirPathValue::Empty);
    }

    #[test]
    fn test_index_navigation() {
        let navigator = Navigator::new();
        let patient = create_test_patient();

        // First get the name array
        let provider = MockModelProvider;
        let name_result = navigator
            .navigate_property(&patient, "name", &provider)
            .unwrap();

        // Test index access on the name collection
        let first_name = navigator.navigate_index(&name_result, 0).unwrap();
        match first_name {
            FhirPathValue::Resource(json) => {
                assert_eq!(json.get("family").unwrap().as_str().unwrap(), "Smith");
            }
            _ => panic!("Expected Resource for first name"),
        }

        let second_name = navigator.navigate_index(&name_result, 1).unwrap();
        match second_name {
            FhirPathValue::Resource(json) => {
                assert_eq!(json.get("family").unwrap().as_str().unwrap(), "Jones");
            }
            _ => panic!("Expected Resource for second name"),
        }

        // Test out-of-bounds index
        let out_of_bounds = navigator.navigate_index(&name_result, 10).unwrap();
        assert_eq!(out_of_bounds, FhirPathValue::Empty);
    }

    #[test]
    fn test_index_on_single_value() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;
        let patient = create_test_patient();

        // Get a single value property
        let id_result = navigator
            .navigate_property(&patient, "id", &provider)
            .unwrap();

        // Index 0 should return the value itself
        let indexed_id = navigator.navigate_index(&id_result, 0).unwrap();
        assert_eq!(indexed_id, FhirPathValue::String("example".to_string()));

        // Other indices should return empty
        let out_of_bounds = navigator.navigate_index(&id_result, 1).unwrap();
        assert_eq!(out_of_bounds, FhirPathValue::Empty);
    }

    #[test]
    fn test_path_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;
        let patient = create_test_patient();

        // Test simple path navigation
        let simple_path_result = navigator.navigate_path(&patient, "id", &provider).unwrap();
        assert_eq!(
            simple_path_result,
            FhirPathValue::String("example".to_string())
        );

        // Test complex path navigation (this would typically require recursive evaluation)
        // For now, our simple implementation can't handle this fully, but let's test basic cases

        // Note: Full path navigation like "name.family" would require the evaluator
        // to handle the intermediate steps properly. This is typically done by the
        // main expression evaluator, not just the navigator.
    }

    #[test]
    fn test_collection_property_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;

        // Create a collection of patients
        let patient1 = FhirPathValue::resource(json!({
            "resourceType": "Patient",
            "id": "patient1",
            "active": true
        }));
        let patient2 = FhirPathValue::resource(json!({
            "resourceType": "Patient",
            "id": "patient2",
            "active": false
        }));
        let collection = FhirPathValue::Collection(vec![patient1, patient2]);

        // Navigate property on collection - should collect results from all items
        let ids_result = navigator
            .navigate_property(&collection, "id", &provider)
            .unwrap();
        match ids_result {
            FhirPathValue::Collection(ids) => {
                assert_eq!(ids.len(), 2);
                assert_eq!(ids[0], FhirPathValue::String("patient1".to_string()));
                assert_eq!(ids[1], FhirPathValue::String("patient2".to_string()));
            }
            _ => panic!("Expected Collection for ids from collection"),
        }

        let active_result = navigator
            .navigate_property(&collection, "active", &provider)
            .unwrap();
        match active_result {
            FhirPathValue::Collection(actives) => {
                assert_eq!(actives.len(), 2);
                assert_eq!(actives[0], FhirPathValue::Boolean(true));
                assert_eq!(actives[1], FhirPathValue::Boolean(false));
            }
            _ => panic!("Expected Collection for active values from collection"),
        }
    }

    #[test]
    fn test_primitive_value_navigation() {
        let navigator = Navigator::new();
        let provider = MockModelProvider;

        // Test property access on primitive values
        let string_value = FhirPathValue::String("test".to_string());

        // Type name matches should return the value itself
        let string_result = navigator
            .navigate_property(&string_value, "String", &provider)
            .unwrap();
        assert_eq!(string_result, string_value);

        // Non-matching property should return empty
        let empty_result = navigator
            .navigate_property(&string_value, "Integer", &provider)
            .unwrap();
        assert_eq!(empty_result, FhirPathValue::Empty);

        let empty_result2 = navigator
            .navigate_property(&string_value, "someProperty", &provider)
            .unwrap();
        assert_eq!(empty_result2, FhirPathValue::Empty);
    }
}
