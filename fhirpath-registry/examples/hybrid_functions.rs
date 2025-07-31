//! Example demonstrating the hybrid function registration system
//! 
//! This example shows how to use both trait-based and closure-based function registration
//! approaches in the FHIRPath registry, providing flexibility for different use cases.

use fhirpath_registry::function::{
    FunctionRegistry, FhirPathFunction, EvaluationContext, FunctionResult, FunctionError
};
use fhirpath_registry::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use rust_decimal::Decimal;

/// Example 1: Trait-based function for complex operations
/// 
/// Use this approach for:
/// - Complex functions with sophisticated logic
/// - Functions that need custom validation
/// - Functions that benefit from structured implementation
#[derive(Debug)]
struct PowerFunction;

impl FhirPathFunction for PowerFunction {
    fn name(&self) -> &str {
        "power"
    }

    fn human_friendly_name(&self) -> &str {
        "Mathematical Power Function"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
        SIGNATURE.get_or_init(|| FunctionSignature {
            name: "power".to_string(),
            min_arity: 2,
            max_arity: Some(2),
            parameters: vec![
                ParameterInfo::required("base", TypeInfo::Decimal),
                ParameterInfo::required("exponent", TypeInfo::Decimal),
            ],
            return_type: TypeInfo::Decimal,
        })
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        match (args.get(0), args.get(1)) {
            (Some(FhirPathValue::Decimal(base)), Some(FhirPathValue::Decimal(exp))) => {
                // Convert to f64 for power calculation, then back to Decimal
                let base_f = base.to_string().parse::<f64>().unwrap_or(0.0);
                let exp_f = exp.to_string().parse::<f64>().unwrap_or(0.0);
                let result = base_f.powf(exp_f);
                Ok(FhirPathValue::Decimal(Decimal::from_f64_retain(result).unwrap_or_default()))
            }
            (Some(FhirPathValue::Integer(base)), Some(FhirPathValue::Integer(exp))) => {
                let base_f = *base as f64;
                let exp_f = *exp as f64;
                let result = base_f.powf(exp_f);
                Ok(FhirPathValue::Decimal(Decimal::from_f64_retain(result).unwrap_or_default()))
            }
            _ => Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "Expected numeric arguments".to_string(),
            })
        }
    }

    fn documentation(&self) -> &str {
        "Raises the first argument to the power of the second argument. power(base, exponent)"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("FHIRPath Hybrid Function Registration Example");
    println!("============================================");

    // Create a function registry
    let mut registry = FunctionRegistry::new();

    // Example 1: Register a trait-based function
    println!("\n1. Registering trait-based function:");
    registry.register(PowerFunction);
    println!("   ✓ Registered 'power' function using trait implementation");

    // Example 2: Register closure-based functions with full signature
    println!("\n2. Registering closure-based functions with full control:");
    
    let double_signature = FunctionSignature {
        name: "double".to_string(),
        min_arity: 1,
        max_arity: Some(1),
        parameters: vec![ParameterInfo::required("value", TypeInfo::Integer)],
        return_type: TypeInfo::Integer,
    };

    registry.register_closure(
        "double",
        "Double Function",
        double_signature,
        "Doubles the input value. double(value)",
        |args, _context| {
            if let Some(FhirPathValue::Integer(n)) = args.first() {
                Ok(FhirPathValue::Integer(n * 2))
            } else {
                Err(FunctionError::EvaluationError {
                    name: "double".to_string(),
                    message: "Expected integer argument".to_string(),
                })
            }
        }
    );
    println!("   ✓ Registered 'double' function using closure with full signature");

    // Example 3: Register simple closure-based functions (minimal boilerplate)
    println!("\n3. Registering simple closure-based functions:");
    
    registry.register_simple(
        "triple",
        1,
        Some(1),
        |args, _context| {
            if let Some(FhirPathValue::Integer(n)) = args.first() {
                Ok(FhirPathValue::Integer(n * 3))
            } else {
                Err(FunctionError::EvaluationError {
                    name: "triple".to_string(),
                    message: "Expected integer argument".to_string(),
                })
            }
        }
    );
    println!("   ✓ Registered 'triple' function using simple closure registration");

    registry.register_simple(
        "greet",
        1,
        Some(1),
        |args, _context| {
            if let Some(FhirPathValue::String(name)) = args.first() {
                Ok(FhirPathValue::String(format!("Hello, {}!", name)))
            } else {
                Ok(FhirPathValue::String("Hello, World!".to_string()))
            }
        }
    );
    println!("   ✓ Registered 'greet' function using simple closure registration");

    // Example 4: Register variadic function
    registry.register_simple(
        "concat",
        1,
        None, // No maximum - variadic function
        |args, _context| {
            let mut result = String::new();
            for arg in args {
                if let FhirPathValue::String(s) = arg {
                    result.push_str(s);
                }
            }
            Ok(FhirPathValue::String(result))
        }
    );
    println!("   ✓ Registered 'concat' variadic function");

    // Display registered functions
    println!("\n4. Registry status:");
    let function_names = registry.function_names();
    println!("   Registered functions: {:?}", function_names);

    // Example 5: Test function evaluation
    println!("\n5. Testing function evaluation:");
    let context = EvaluationContext::new(FhirPathValue::Empty);

    // Test trait-based function
    let args = vec![FhirPathValue::Decimal(Decimal::from(2)), FhirPathValue::Decimal(Decimal::from(3))];
    match registry.evaluate_function("power", &args, &context) {
        Ok(result) => println!("   power(2.0, 3.0) = {:?}", result),
        Err(e) => println!("   Error: {}", e),
    }

    // Test closure-based function
    let args = vec![FhirPathValue::Integer(21)];
    match registry.evaluate_function("double", &args, &context) {
        Ok(result) => println!("   double(21) = {:?}", result),
        Err(e) => println!("   Error: {}", e),
    }

    // Test simple closure function
    let args = vec![FhirPathValue::Integer(14)];
    match registry.evaluate_function("triple", &args, &context) {
        Ok(result) => println!("   triple(14) = {:?}", result),
        Err(e) => println!("   Error: {}", e),
    }

    // Test string function
    let args = vec![FhirPathValue::String("Alice".to_string())];
    match registry.evaluate_function("greet", &args, &context) {
        Ok(result) => println!("   greet('Alice') = {:?}", result),
        Err(e) => println!("   Error: {}", e),
    }

    // Test variadic function
    let args = vec![
        FhirPathValue::String("Hello".to_string()),
        FhirPathValue::String(" ".to_string()),
        FhirPathValue::String("World".to_string()),
        FhirPathValue::String("!".to_string()),
    ];
    match registry.evaluate_function("concat", &args, &context) {
        Ok(result) => println!("   concat('Hello', ' ', 'World', '!') = {:?}", result),
        Err(e) => println!("   Error: {}", e),
    }

    // Example 6: Error handling
    println!("\n6. Testing error handling:");
    
    // Test function not found
    match registry.evaluate_function("nonexistent", &[], &context) {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   nonexistent() -> Error: {}", e),
    }

    // Test invalid arity
    let args = vec![FhirPathValue::Integer(1)]; // Only 1 arg, but power needs 2
    match registry.evaluate_function("power", &args, &context) {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   power(1) -> Error: {}", e),
    }

    println!("\n7. Performance considerations:");
    println!("   • Trait-based functions: Better for complex logic, type safety");
    println!("   • Closure-based functions: Lower overhead, easier to write");
    println!("   • Simple registration: Minimal boilerplate for basic functions");
    println!("   • All approaches support function caching and optimization");

    println!("\nExample completed successfully!");
    Ok(())
}