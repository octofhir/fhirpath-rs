//! Debug test for all function

use fhirpath_evaluator::FhirPathEngine;
use fhirpath_model::{FhirPathValue, FhirResource};
use fhirpath_parser::parse_expression;
use serde_json::Value;
use std::fs;

#[test]
fn test_all_function_debug() {
    // Load patient example
    let patient_json = fs::read_to_string("../specs/fhirpath/tests/input/patient-example.json")
        .expect("Failed to read patient example");
    let patient: Value = serde_json::from_str(&patient_json)
        .expect("Failed to parse patient JSON");

    let patient_resource = FhirPathValue::Resource(FhirResource::from_json(patient));

    let engine = FhirPathEngine::new();

    // Test simple all function without criteria
    let expr = parse_expression("Patient.name.all()").expect("Failed to parse expression");
    let result = engine.evaluate(&expr, patient_resource.clone()).expect("Failed to evaluate");
    println!("Patient.name.all() = {:?}", result);

    // Test all function with criteria (this should use lambda evaluation)
    let expr = parse_expression("Patient.name.all(given.exists())").expect("Failed to parse expression");
    match engine.evaluate(&expr, patient_resource.clone()) {
        Ok(result) => println!("Patient.name.all(given.exists()) = {:?}", result),
        Err(e) => println!("Error evaluating Patient.name.all(given.exists()): {:?}", e),
    }

    // Test select function (this should also use lambda evaluation)
    let expr = parse_expression("Patient.name.select(given.exists())").expect("Failed to parse expression");
    match engine.evaluate(&expr, patient_resource.clone()) {
        Ok(result) => println!("Patient.name.select(given.exists()) = {:?}", result),
        Err(e) => println!("Error evaluating Patient.name.select(given.exists()): {:?}", e),
    }
}
