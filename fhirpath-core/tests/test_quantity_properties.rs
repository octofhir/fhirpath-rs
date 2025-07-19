use fhirpath_core::evaluator::evaluate_expression;
use fhirpath_core::model::FhirPathValue;
use serde_json::json;

#[test]
fn test_quantity_property_access() {
    let observation = json!({
        "resourceType": "Observation",
        "id": "example",
        "status": "final",
        "valueQuantity": {
            "value": 185,
            "unit": "lbs",
            "system": "http://unitsofmeasure.org",
            "code": "[lb_av]"
        }
    });

    // Test accessing unit property
    let unit_result = evaluate_expression("Observation.value.unit", observation.clone()).unwrap();
    println!("Observation.value.unit result: {:?}", unit_result);
    match unit_result {
        FhirPathValue::String(s) => assert_eq!(s, "lbs"),
        _ => panic!("Expected string 'lbs', got {:?}", unit_result),
    }

    // Test accessing value property
    let value_result = evaluate_expression("Observation.value.value", observation.clone()).unwrap();
    println!("Observation.value.value result: {:?}", value_result);
    match value_result {
        FhirPathValue::Decimal(d) => assert_eq!(d, 185.0),
        _ => panic!("Expected decimal 185.0, got {:?}", value_result),
    }

    // Test comparison with value property
    let comparison_result = evaluate_expression("Observation.value.value > 180.0", observation.clone()).unwrap();
    println!("Observation.value.value > 180.0 result: {:?}", comparison_result);
    match comparison_result {
        FhirPathValue::Boolean(b) => assert_eq!(b, true),
        _ => panic!("Expected boolean true, got {:?}", comparison_result),
    }
}
