//! Lexical analysis utilities

use crate::tokenizer::Token;
use crate::span::Spanned;

/// Check if a character can start an identifier
pub fn is_identifier_start(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_start(c) || c == '_'
}

/// Check if a character can continue an identifier
pub fn is_identifier_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}

/// Token stream with lookahead capability
#[derive(Debug)]
pub struct TokenStream<'input> {
    tokens: Vec<Spanned<Token<'input>>>,
    position: usize,
}

impl<'input> TokenStream<'input> {
    /// Create a new token stream
    pub fn new(tokens: Vec<Spanned<Token<'input>>>) -> Self {
        Self { tokens, position: 0 }
    }
    
    /// Peek at the current token without consuming
    pub fn peek(&self) -> Option<&Spanned<Token<'input>>> {
        self.tokens.get(self.position)
    }
    
    /// Peek at a token n positions ahead
    pub fn peek_ahead(&self, n: usize) -> Option<&Spanned<Token<'input>>> {
        self.tokens.get(self.position + n)
    }
    
    /// Consume and return the current token
    pub fn next(&mut self) -> Option<Spanned<Token<'input>>> {
        if self.position < self.tokens.len() {
            let token = self.tokens[self.position].clone();
            self.position += 1;
            Some(token)
        } else {
            None
        }
    }
    
    /// Check if we're at the end of the stream
    pub fn is_eof(&self) -> bool {
        self.position >= self.tokens.len()
    }
    
    /// Get the current position in the stream
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Reset to a previous position
    pub fn reset_to(&mut self, position: usize) {
        self.position = position.min(self.tokens.len());
    }
    
    /// Consume a token if it matches the predicate
    pub fn consume_if<F>(&mut self, predicate: F) -> Option<Spanned<Token<'input>>>
    where
        F: FnOnce(&Token<'input>) -> bool,
    {
        if let Some(token) = self.peek() {
            if predicate(&token.value) {
                return self.next();
            }
        }
        None
    }
    
    /// Expect a specific token type
    pub fn expect(&mut self, expected: Token<'input>) -> Result<Spanned<Token<'input>>, String> {
        if let Some(token) = self.peek() {
            if std::mem::discriminant(&token.value) == std::mem::discriminant(&expected) {
                Ok(self.next().unwrap())
            } else {
                Err(format!("Expected {:?}, found {:?}", expected, token.value))
            }
        } else {
            Err(format!("Expected {:?}, found EOF", expected))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_identifier_chars() {
        assert!(is_identifier_start('a'));
        assert!(is_identifier_start('Z'));
        assert!(is_identifier_start('_'));
        assert!(!is_identifier_start('0'));
        assert!(!is_identifier_start('-'));
        
        assert!(is_identifier_continue('a'));
        assert!(is_identifier_continue('0'));
        assert!(is_identifier_continue('_'));
        assert!(!is_identifier_continue('-'));
    }
}