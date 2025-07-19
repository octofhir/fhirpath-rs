use fhirpath_core::lexer::tokenize;
use fhirpath_core::parser::parse;

#[test]
fn debug_parser_issue() {
    let expression = "@2015-02-04T14:34:28.is(DateTime)";
    println!("Testing expression: {}", expression);

    // First, let's see what tokens are generated
    match tokenize(expression) {
        Ok(tokens) => {
            println!("Tokens generated:");
            for (i, token) in tokens.iter().enumerate() {
                println!("  [{}] {:?} = '{}'", i, token.token_type, token.lexeme);
            }

            // Now let's try to parse
            match parse(&tokens) {
                Ok(ast) => {
                    println!("Successfully parsed: {:?}", ast);
                }
                Err(e) => {
                    println!("Parse error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Tokenization error: {:?}", e);
        }
    }
}

#[test]
fn debug_simpler_case() {
    let expression = "true.is(Boolean)";
    println!("Testing simpler expression: {}", expression);

    // First, let's see what tokens are generated
    match tokenize(expression) {
        Ok(tokens) => {
            println!("Tokens generated:");
            for (i, token) in tokens.iter().enumerate() {
                println!("  [{}] {:?} = '{}'", i, token.token_type, token.lexeme);
            }

            // Now let's try to parse
            match parse(&tokens) {
                Ok(ast) => {
                    println!("Successfully parsed: {:?}", ast);
                }
                Err(e) => {
                    println!("Parse error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Tokenization error: {:?}", e);
        }
    }
}
