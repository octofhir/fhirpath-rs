//! Example demonstrating documentation generation from function metadata
//! 
//! This example shows how to leverage rich function metadata for automatic documentation
//! generation, implementing section 6.1 of the FHIRPath improvement plan.

use fhirpath_registry::function::{FunctionRegistry, register_builtin_functions};

fn main() {
    println!("FHIRPath Documentation Generation Example");
    println!("==========================================\n");

    // Create registry and register built-in functions
    let mut registry = FunctionRegistry::new();
    register_builtin_functions(&mut registry);

    // Example 1: Generate complete markdown documentation
    println!("1. Complete Function Documentation (first 10 functions):\n");
    let docs = registry.generate_function_docs();
    let lines: Vec<&str> = docs.lines().take(50).collect(); // Show first 50 lines
    for line in lines {
        println!("{}", line);
    }
    println!("... (truncated)\n");

    // Example 2: Generate documentation for a specific function
    println!("2. Documentation for 'count' function:\n");
    if let Some(count_docs) = registry.generate_function_doc("count") {
        println!("{}", count_docs);
    }

    // Example 3: Generate JSON documentation for programmatic use
    println!("3. JSON Documentation Sample:\n");
    let json_docs = registry.generate_function_docs_json();
    
    // Pretty print a subset of the JSON
    if let Some(functions) = json_docs["functions"].as_object() {
        if let Some(count_function) = functions.get("count") {
            println!("Count function metadata:");
            println!("{}", serde_json::to_string_pretty(count_function).unwrap());
        }
    }

    // Example 4: Statistics
    println!("\n4. Documentation Statistics:");
    println!("Total functions documented: {}", json_docs["total_count"]);
    if let Some(generated_at) = json_docs["generated_at"].as_str() {
        println!("Generated at: {}", generated_at);
    }

    // Example 5: List all available functions
    println!("\n5. All Available Functions:");
    let function_names = registry.function_names();
    for (i, name) in function_names.iter().enumerate() {
        if i % 10 == 0 && i > 0 {
            println!(); // New line every 10 functions
        }
        print!("{:<15}", name);
    }
    println!("\n\nTotal: {} functions", function_names.len());
}