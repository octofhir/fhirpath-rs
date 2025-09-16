use octofhir_fhirpath::registry::{
    FunctionRegistry, create_comprehensive_operator_registry, export_functions_json,
    export_operators_json,
};

fn main() {
    // Test function registry export
    let function_registry = FunctionRegistry::new();
    let functions_json = export_functions_json(&function_registry);
    println!("Functions JSON structure:");
    println!("{}", serde_json::to_string_pretty(&functions_json).unwrap());
    println!("\n{}\n", "=".repeat(50));

    // Test operator registry export
    let operator_registry = create_comprehensive_operator_registry();
    let operators_json = export_operators_json(&operator_registry);
    println!("Operators JSON structure:");
    println!("{}", serde_json::to_string_pretty(&operators_json).unwrap());

    // Basic validation
    assert!(functions_json.get("functions").unwrap().is_array());

    let ops = operators_json.as_object().unwrap();
    assert!(ops.get("binary_operators").unwrap().is_array());
    assert!(ops.get("unary_operators").unwrap().is_array());

    // Check for specific operators
    let binary_ops = ops.get("binary_operators").unwrap().as_array().unwrap();
    let unary_ops = ops.get("unary_operators").unwrap().as_array().unwrap();

    // Should have some operators
    assert!(!binary_ops.is_empty());
    assert!(!unary_ops.is_empty());

    println!("\n✅ Export functionality verification passed!");
    println!(
        "✅ Functions export: {} functions",
        functions_json
            .get("functions")
            .unwrap()
            .as_array()
            .unwrap()
            .len()
    );
    println!("✅ Binary operators: {} operators", binary_ops.len());
    println!("✅ Unary operators: {} operators", unary_ops.len());
}
