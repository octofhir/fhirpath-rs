use crate::lexer::{tokenize, TokenType};

pub fn debug_tokenize(input: &str) {
    println!("Tokenizing: '{}'", input);
    match tokenize(input) {
        Ok(tokens) => {
            for (i, token) in tokens.iter().enumerate() {
                println!("  Token {}: {:?} = '{}'", i, token.token_type, token.lexeme);
            }
        }
        Err(e) => {
            println!("  Tokenization error: {:?}", e);
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_datetime_tokenization() {
        // Test cases that work
        debug_tokenize("@2015T.is(DateTime)");
        debug_tokenize("@2015-02T.is(DateTime)");
        debug_tokenize("@2015-02-04T14:34:28Z.is(DateTime)");
        debug_tokenize("@T14:34:28.123.is(Time)");

        // Test cases that fail
        debug_tokenize("@2015-02-04T14:34:28.is(DateTime)");
        debug_tokenize("@T14:34:28.is(Time)");
    }
}
