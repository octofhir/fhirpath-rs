// FHIRPath integration tests
//
// This file contains integration tests for the FHIRPath engine.

use std::fs;
use std::path::Path;

#[test]
fn test_load_fixture() {
    // Test that we can load a fixture file
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("patient-example.json");

    assert!(fixture_path.exists(), "Fixture file does not exist");

    let fixture_content = fs::read_to_string(fixture_path).expect("Failed to read fixture file");
    let json: serde_json::Value =
        serde_json::from_str(&fixture_content).expect("Failed to parse JSON");

    assert_eq!(json["resourceType"], "Patient", "Expected Patient resource");
}

// This test will be enabled when the FHIRPath engine is implemented
#[test]
fn test_simple_path_expression() {
    // Test a simple path expression
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("patient-example.json");

    let fixture_content = fs::read_to_string(fixture_path).expect("Failed to read fixture file");
    let resource: serde_json::Value =
        serde_json::from_str(&fixture_content).expect("Failed to parse JSON");

    // Now that the FHIRPath engine is implemented, this should work
    let result = fhirpath_core::evaluate("Patient.name.given", resource);
    assert!(
        result.is_ok(),
        "FHIRPath evaluation failed: {:?}",
        result.err()
    );

    let value = result.unwrap();
    println!("DEBUG: Actual result from Patient.name.given: {:?}", value);
    println!("DEBUG: Result type: {:?}", value);
    println!("DEBUG: Is array? {}", value.is_array());
    assert!(
        value.is_array(),
        "Expected array result for Patient.name.given"
    );

    // The patient example should have at least one given name
    let array = value.as_array().unwrap();
    assert!(!array.is_empty(), "Expected at least one given name");
}
