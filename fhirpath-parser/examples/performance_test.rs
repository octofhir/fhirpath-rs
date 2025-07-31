use std::time::Instant;
use fhirpath_parser::tokenizer::Tokenizer;
use fhirpath_parser::pratt::parse_expression_pratt;

fn main() {
    println!("FHIRPath Performance Measurements");
    println!("=================================");
    
    let test_expressions = vec![
        ("Simple", "Patient.name"),
        ("Medium", "Patient.name.given"),
        ("Complex", "Patient.name.where(use = 'official').given.first()"),
        ("Arithmetic", "2 + 3 * 4 - 1"),
        ("Mixed", "Patient.age > 18 and Patient.active = true"),
    ];
    
    for (name, expression) in test_expressions {
        println!("\n{}: {}", name, expression);
        
        // Tokenizer Performance
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            let mut tokenizer = Tokenizer::new(expression);
            let mut token_count = 0;
            while let Ok(Some(_)) = tokenizer.next_token() {
                token_count += 1;
            }
        }
        let tokenizer_duration = start.elapsed();
        let tokenizer_per_sec = (iterations as f64) / tokenizer_duration.as_secs_f64();
        
        println!("  Tokenizer: {:?} total, {:.0} ops/sec", tokenizer_duration, tokenizer_per_sec);
        
        // Parser Performance
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parse_expression_pratt(expression);
        }
        let parser_duration = start.elapsed();
        let parser_per_sec = (iterations as f64) / parser_duration.as_secs_f64();
        
        println!("  Parser:    {:?} total, {:.0} ops/sec", parser_duration, parser_per_sec);
        
        // Check if parsing succeeded
        match parse_expression_pratt(expression) {
            Ok(_) => println!("  ✓ Parsing successful"),
            Err(e) => println!("  ✗ Parsing failed: {:?}", e),
        }
    }
    
    println!("\n=================================");
    println!("Performance test completed!");
}