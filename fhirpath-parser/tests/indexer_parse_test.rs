use fhirpath_parser::parse_expression;

#[test]
fn test_patient_name_parsing() {
    // Basic identifier
    let result = parse_expression("Patient");
    println!("Patient: {:?}", result);
    assert!(result.is_ok());

    // Simple path
    let result = parse_expression("Patient.name");
    println!("Patient.name: {:?}", result);
    assert!(result.is_ok());

    // Path with indexer
    let result = parse_expression("Patient.name[0]");
    println!("Patient.name[0]: {:?}", result);
    assert!(result.is_ok());

    // Full expression
    let result = parse_expression("Patient.name[0].given");
    println!("Patient.name[0].given: {:?}", result);
    assert!(result.is_ok());
}
