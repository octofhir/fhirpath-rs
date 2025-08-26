#[cfg(test)]
mod tests {
    use crate::FhirPathValue;
    use crate::fhirschema_provider::FhirSchemaModelProvider;
    use crate::json_value::JsonValue;
    use crate::provider::ModelProvider;
    use sonic_rs::json;

    #[tokio::test]
    async fn test_schema_based_choice_property_detection() {
        let provider = FhirSchemaModelProvider::new()
            .await
            .expect("Failed to create provider");

        // Test basic choice property detection (should work with fallback even without schema)
        let is_choice = provider.is_choice_property("Observation", "value").await;
        println!("Observation.value is choice property: {is_choice}");
        assert!(
            is_choice,
            "Observation.value should be detected as a choice property"
        );

        let is_choice_patient = provider.is_choice_property("Patient", "deceased").await;
        println!("Patient.deceased is choice property: {is_choice_patient}");
        assert!(
            is_choice_patient,
            "Patient.deceased should be detected as a choice property"
        );

        // Test non-choice properties
        let is_not_choice = provider.is_choice_property("Patient", "name").await;
        println!("Patient.name is choice property: {is_not_choice}");
        assert!(
            !is_not_choice,
            "Patient.name should not be a choice property"
        );
    }

    #[tokio::test]
    async fn test_choice_variant_retrieval() {
        let provider = FhirSchemaModelProvider::new()
            .await
            .expect("Failed to create provider");

        // Test getting choice variants
        let variants = provider.get_choice_variants("Observation", "value").await;
        println!(
            "Found {} choice variants for Observation.value",
            variants.len()
        );
        assert!(
            !variants.is_empty(),
            "Should find choice variants for Observation.value"
        );

        // Verify some expected variants exist
        let has_quantity = variants.iter().any(|v| v.property_name == "valueQuantity");
        let has_string = variants.iter().any(|v| v.property_name == "valueString");
        let has_boolean = variants.iter().any(|v| v.property_name == "valueBoolean");

        println!("Has valueQuantity: {has_quantity}");
        println!("Has valueString: {has_string}");
        println!("Has valueBoolean: {has_boolean}");

        assert!(has_quantity, "Should find valueQuantity variant");
        assert!(has_string, "Should find valueString variant");
        assert!(has_boolean, "Should find valueBoolean variant");
    }

    #[tokio::test]
    async fn test_choice_property_resolution() {
        let provider = FhirSchemaModelProvider::new()
            .await
            .expect("Failed to create provider");

        // Test resolving choice property from data
        let observation_with_quantity = json!({
            "resourceType": "Observation",
            "valueQuantity": {
                "value": 185,
                "unit": "lbs"
            }
        });

        let observation_value =
            FhirPathValue::JsonValue(JsonValue::from(observation_with_quantity));
        let resolved = provider
            .resolve_choice_property("Observation", "value", &observation_value)
            .await;

        println!("Resolved choice property: {resolved:?}");
        assert_eq!(resolved, Some("valueQuantity".to_string()));

        // Test with string value
        let observation_with_string = json!({
            "resourceType": "Observation",
            "valueString": "Normal"
        });

        let observation_string_value =
            FhirPathValue::JsonValue(JsonValue::from(observation_with_string));
        let resolved_string = provider
            .resolve_choice_property("Observation", "value", &observation_string_value)
            .await;

        println!("Resolved string choice property: {resolved_string:?}");
        assert_eq!(resolved_string, Some("valueString".to_string()));
    }

    #[tokio::test]
    async fn test_get_choice_base_property() {
        let provider = FhirSchemaModelProvider::new()
            .await
            .expect("Failed to create provider");

        // Test reverse lookup from concrete property to base
        let base = provider
            .get_choice_base_property("Observation", "valueQuantity")
            .await;
        println!("Base property for valueQuantity: {base:?}");
        assert_eq!(base, Some("value".to_string()));

        let base_deceased = provider
            .get_choice_base_property("Patient", "deceasedBoolean")
            .await;
        println!("Base property for deceasedBoolean: {base_deceased:?}");
        assert_eq!(base_deceased, Some("deceased".to_string()));

        // Test non-choice property
        let base_none = provider.get_choice_base_property("Patient", "name").await;
        println!("Base property for name: {base_none:?}");
        assert_eq!(base_none, None);
    }
}
