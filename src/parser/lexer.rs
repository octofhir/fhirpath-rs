//! Lexical analysis utilities

use super::span::Spanned;
use super::tokenizer::Token;

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
        Self {
            tokens,
            position: 0,
        }
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
            if Self::tokens_match(&token.value, &expected) {
                Ok(self.next().unwrap())
            } else {
                Err(format!("Expected {:?}, found {:?}", expected, token.value))
            }
        } else {
            Err(format!("Expected {expected:?}, found EOF"))
        }
    }

    /// Fast token type matching using direct pattern matching
    #[inline(always)]
    fn tokens_match(token: &Token<'input>, expected: &Token<'input>) -> bool {
        match (token, expected) {
            // Literals
            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Decimal(_), Token::Decimal(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Boolean(_), Token::Boolean(_)) => true,
            (Token::Date(_), Token::Date(_)) => true,
            (Token::DateTime(_), Token::DateTime(_)) => true,
            (Token::Time(_), Token::Time(_)) => true,
            (Token::Quantity { .. }, Token::Quantity { .. }) => true,

            // Identifiers (regular and interned)
            (Token::Identifier(_), Token::Identifier(_)) => true,
            (Token::Identifier(_), Token::InternedIdentifier(_)) => true,
            (Token::InternedIdentifier(_), Token::Identifier(_)) => true,
            (Token::InternedIdentifier(_), Token::InternedIdentifier(_)) => true,

            // Unit tokens (exact matches)
            (Token::Plus, Token::Plus) => true,
            (Token::Minus, Token::Minus) => true,
            (Token::Multiply, Token::Multiply) => true,
            (Token::Divide, Token::Divide) => true,
            (Token::Mod, Token::Mod) => true,
            (Token::Div, Token::Div) => true,
            (Token::Power, Token::Power) => true,
            (Token::Equal, Token::Equal) => true,
            (Token::NotEqual, Token::NotEqual) => true,
            (Token::LessThan, Token::LessThan) => true,
            (Token::LessThanOrEqual, Token::LessThanOrEqual) => true,
            (Token::GreaterThan, Token::GreaterThan) => true,
            (Token::GreaterThanOrEqual, Token::GreaterThanOrEqual) => true,
            (Token::Equivalent, Token::Equivalent) => true,
            (Token::NotEquivalent, Token::NotEquivalent) => true,
            (Token::And, Token::And) => true,
            (Token::Or, Token::Or) => true,
            (Token::Xor, Token::Xor) => true,
            (Token::Implies, Token::Implies) => true,
            (Token::Not, Token::Not) => true,
            (Token::Union, Token::Union) => true,
            (Token::In, Token::In) => true,
            (Token::Contains, Token::Contains) => true,
            (Token::Ampersand, Token::Ampersand) => true,
            (Token::Is, Token::Is) => true,
            (Token::As, Token::As) => true,
            (Token::LeftParen, Token::LeftParen) => true,
            (Token::RightParen, Token::RightParen) => true,
            (Token::LeftBracket, Token::LeftBracket) => true,
            (Token::RightBracket, Token::RightBracket) => true,
            (Token::LeftBrace, Token::LeftBrace) => true,
            (Token::RightBrace, Token::RightBrace) => true,
            (Token::Dot, Token::Dot) => true,
            (Token::Comma, Token::Comma) => true,
            (Token::Colon, Token::Colon) => true,
            (Token::Semicolon, Token::Semicolon) => true,
            (Token::Arrow, Token::Arrow) => true,
            (Token::Dollar, Token::Dollar) => true,
            (Token::Percent, Token::Percent) => true,
            (Token::Backtick, Token::Backtick) => true,
            (Token::DollarThis, Token::DollarThis) => true,
            (Token::DollarIndex, Token::DollarIndex) => true,
            (Token::DollarTotal, Token::DollarTotal) => true,
            (Token::True, Token::True) => true,
            (Token::False, Token::False) => true,
            (Token::Empty, Token::Empty) => true,
            (Token::Define, Token::Define) => true,
            (Token::Where, Token::Where) => true,
            (Token::Select, Token::Select) => true,
            (Token::All, Token::All) => true,
            (Token::First, Token::First) => true,
            (Token::Last, Token::Last) => true,
            (Token::Tail, Token::Tail) => true,
            (Token::Skip, Token::Skip) => true,
            (Token::Take, Token::Take) => true,
            (Token::Distinct, Token::Distinct) => true,
            (Token::Count, Token::Count) => true,
            (Token::OfType, Token::OfType) => true,

            // No match
            _ => false,
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
