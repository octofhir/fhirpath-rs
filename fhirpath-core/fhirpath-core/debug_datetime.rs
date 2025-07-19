use fhirpath_core::lexer::tokenize;
use fhirpath_core::parser::parse;
use fhirpath_core::evaluator::evaluate_expression;
use serde_json::json;

fn main() {
    let expressions = vec![
        "@2015-02-04T14",
        "@2015-02-04T14:30",
        "@2015-02-04T14:30:45",
    ];

    for expr in expressions {
        println!("\n=== Testing expression: {} ===", expr);

        // Test tokenization
        match tokenize(expr) {
            Ok(tokens) => {
                println!("Tokens:");
                for token in &tokens {
                    println!("  {:?}", token);
                }

                // Test parsing
                match parse(&tokens) {
                    Ok(ast) => {
                        println!("AST: {:?}", ast);

                        // Test evaluation
                        let context = json!({});
                        match evaluate_expression(expr, context) {
                            Ok(result) => {
                                println!("Result: {:?}", result);
                            }
                            Err(e) => {
                                println!("Evaluation error: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("Parse error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("Tokenize error: {:?}", e);
            }
        }
    }
}
