use fhirpath_core::evaluator::evaluate_expression;
use serde_json::json;

#[test]
fn debug_quantity_issue() {
    // Create a simplified observation JSON structure based on what we expect
    let json_data = json!({
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

    println!("Test JSON structure:");
    println!("{}", serde_json::to_string_pretty(&json_data).unwrap());

    // Test what Observation.value resolves to
    match evaluate_expression("Observation.value", json_data.clone()) {
        Ok(result) => {
            println!("Observation.value result: {:?}", result);
        }
        Err(e) => {
            println!("Error evaluating Observation.value: {:?}", e);
        }
    }

    // Test what Observation.valueQuantity resolves to
    match evaluate_expression("Observation.valueQuantity", json_data.clone()) {
        Ok(result) => {
            println!("Observation.valueQuantity result: {:?}", result);
        }
        Err(e) => {
            println!("Error evaluating Observation.valueQuantity: {:?}", e);
        }
    }

    // Test the is() function directly
    match evaluate_expression("Observation.value.is(Quantity)", json_data.clone()) {
        Ok(result) => {
            println!("Observation.value.is(Quantity) result: {:?}", result);
        }
        Err(e) => {
            println!("Error evaluating Observation.value.is(Quantity): {:?}", e);
        }
    }
}
