//! High-performance FHIRPath parser implementation using the optimized tokenizer

use crate::error::{ParseError, ParseResult};
use crate::tokenizer::{Token, Tokenizer};
use fhirpath_ast::{ExpressionNode, LiteralValue, BinaryOperator, UnaryOperator};

/// High-performance FHIRPath parser using the optimized tokenizer
pub struct Parser<'input> {
    tokenizer: Tokenizer<'input>,
    current_token: Option<Token<'input>>,
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
            current_token: None,
        }
    }

    fn advance(&mut self) -> ParseResult<()> {
        self.current_token = self.tokenizer.next_token()?;
        Ok(())
    }

    fn peek(&self) -> Option<&Token<'input>> {
        self.current_token.as_ref()
    }

    /// Look ahead at the next token without consuming it
    fn peek_ahead(&mut self) -> ParseResult<Option<Token<'input>>> {
        // Save current state
        let saved_token = self.current_token.clone();
        let saved_tokenizer = self.tokenizer.clone();

        // Advance to get next token
        self.advance()?;
        let next_token = self.current_token.clone();

        // Restore state
        self.current_token = saved_token;
        self.tokenizer = saved_tokenizer;

        Ok(next_token)
    }

    /// Parse a standalone function call (e.g., count(), exists())
    fn parse_function_call(&mut self, function_name: &str) -> ParseResult<ExpressionNode> {
        self.advance()?; // consume function name

        // Expect left parenthesis
        if let Some(Token::LeftParen) = self.peek() {
            self.advance()?; // consume '('
        } else {
            return Err(ParseError::UnexpectedToken {
                token: "Expected '(' after function name".to_string(),
                position: 0,
            });
        }

        // Parse arguments
        let mut args = Vec::new();

        // Handle empty argument list
        if let Some(Token::RightParen) = self.peek() {
            self.advance()?; // consume ')'
            return Ok(ExpressionNode::function_call(function_name, args));
        }

        // Parse argument list
        loop {
            args.push(self.parse_expression()?);

            match self.peek() {
                Some(Token::Comma) => {
                    self.advance()?; // consume ','
                    continue;
                }
                Some(Token::RightParen) => {
                    self.advance()?; // consume ')'
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        token: "Expected ',' or ')' in function arguments".to_string(),
                        position: 0,
                    });
                }
            }
        }

        Ok(ExpressionNode::function_call(function_name, args))
    }

    /// Parse a method call on an expression (e.g., Patient.name.count())
    fn parse_method_call(&mut self, base: ExpressionNode, method_name: &str) -> ParseResult<ExpressionNode> {
        self.advance()?; // consume method name

        // Expect left parenthesis
        if let Some(Token::LeftParen) = self.peek() {
            self.advance()?; // consume '('
        } else {
            return Err(ParseError::UnexpectedToken {
                token: "Expected '(' after method name".to_string(),
                position: 0,
            });
        }

        // Parse arguments
        let mut args = Vec::new();

        // Handle empty argument list
        if let Some(Token::RightParen) = self.peek() {
            self.advance()?; // consume ')'
            return Ok(ExpressionNode::method_call(base, method_name, args));
        }

        // Parse argument list
        loop {
            args.push(self.parse_expression()?);

            match self.peek() {
                Some(Token::Comma) => {
                    self.advance()?; // consume ','
                    continue;
                }
                Some(Token::RightParen) => {
                    self.advance()?; // consume ')'
                    break;
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        token: "Expected ',' or ')' in method arguments".to_string(),
                        position: 0,
                    });
                }
            }
        }

        Ok(ExpressionNode::method_call(base, method_name, args))
    }

    /// Parse unary expressions: -, +, not
    fn parse_unary(&mut self) -> ParseResult<ExpressionNode> {
        if self.current_token.is_none() {
            self.advance()?;
        }

        match self.peek() {
            Some(Token::Minus) => {
                self.advance()?;
                let operand = self.parse_unary()?;
                Ok(ExpressionNode::unary_op(UnaryOperator::Minus, operand))
            }
            Some(Token::Plus) => {
                self.advance()?;
                self.parse_unary() // Unary plus is essentially a no-op
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse primary expressions: identifiers, literals, parenthesized expressions
    fn parse_primary(&mut self) -> ParseResult<ExpressionNode> {
        if self.current_token.is_none() {
            self.advance()?;
        }

        match self.peek() {
            Some(Token::Identifier(name)) => {
                let identifier_name = *name;

                // Check if this is a function call (identifier followed by parentheses)
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call(identifier_name)
                } else {
                    let expr = ExpressionNode::identifier(identifier_name);
                    self.advance()?;

                    // Handle method calls and path expressions
                    self.parse_method_chain(expr)
                }
            }
            Some(Token::Integer(value)) => {
                let int_value = *value;
                self.advance()?;

                // Check if this is a quantity (integer followed by unit string)
                if let Some(Token::String(unit)) = self.peek() {
                    let unit_str = *unit;
                    self.advance()?; // consume unit string

                    // Create a quantity literal
                    let expr = ExpressionNode::literal(LiteralValue::Quantity {
                        value: int_value.to_string(),
                        unit: unit_str.to_string(),
                    });
                    self.parse_method_chain(expr)
                } else {
                    let expr = ExpressionNode::literal(LiteralValue::Integer(int_value));
                    // Handle method calls on integer literals
                    self.parse_method_chain(expr)
                }
            }
            Some(Token::Decimal(value)) => {
                let decimal_str = *value;
                self.advance()?;

                // Check if this is a quantity (decimal followed by unit string)
                if let Some(Token::String(unit)) = self.peek() {
                    let unit_str = *unit;
                    self.advance()?; // consume unit string

                    // Create a quantity literal
                    let expr = ExpressionNode::literal(LiteralValue::Quantity {
                        value: decimal_str.to_string(),
                        unit: unit_str.to_string(),
                    });
                    self.parse_method_chain(expr)
                } else {
                    let expr = ExpressionNode::literal(LiteralValue::Decimal(decimal_str.to_string()));
                    // Handle method calls on decimal literals
                    self.parse_method_chain(expr)
                }
            }
            Some(Token::String(value)) => {
                let expr = ExpressionNode::literal(LiteralValue::String(value.to_string()));
                self.advance()?;

                // Handle method calls on string literals
                self.parse_method_chain(expr)
            }
            Some(Token::True) => {
                let expr = ExpressionNode::literal(LiteralValue::Boolean(true));
                self.advance()?;

                // Handle method calls on boolean literals
                self.parse_method_chain(expr)
            }
            Some(Token::False) => {
                let expr = ExpressionNode::literal(LiteralValue::Boolean(false));
                self.advance()?;

                // Handle method calls on boolean literals
                self.parse_method_chain(expr)
            }
            Some(Token::Date(value)) => {
                let expr = ExpressionNode::literal(LiteralValue::Date(value.to_string()));
                self.advance()?;

                // Handle method calls on date literals
                self.parse_method_chain(expr)
            }
            Some(Token::DateTime(value)) => {
                let expr = ExpressionNode::literal(LiteralValue::DateTime(value.to_string()));
                self.advance()?;

                // Handle method calls on datetime literals
                self.parse_method_chain(expr)
            }
            Some(Token::Time(value)) => {
                let expr = ExpressionNode::literal(LiteralValue::Time(value.to_string()));
                self.advance()?;

                // Handle method calls on time literals
                self.parse_method_chain(expr)
            }
            Some(Token::LeftParen) => {
                self.advance()?; // consume '('
                let expr = self.parse_expression()?;

                if let Some(Token::RightParen) = self.peek() {
                    self.advance()?; // consume ')'

                    // Handle method calls on parenthesized expressions
                    self.parse_method_chain(expr)
                } else {
                    Err(ParseError::UnexpectedToken {
                        token: "Expected ')'".to_string(),
                        position: 0,
                    })
                }
            }
            Some(Token::LeftBrace) => {
                self.advance()?; // consume '{'

                // Handle empty collections
                if let Some(Token::RightBrace) = self.peek() {
                    self.advance()?; // consume '}'
                    let expr = ExpressionNode::literal(LiteralValue::Null);
                    self.parse_method_chain(expr)
                } else {
                    Err(ParseError::UnexpectedToken {
                        token: "Non-empty collections not yet supported".to_string(),
                        position: 0,
                    })
                }
            }
            Some(Token::Dollar) => {
                self.advance()?; // consume '$'

                // Expect identifier after dollar sign
                match self.peek() {
                    Some(Token::Identifier(var_name)) => {
                        let expr = ExpressionNode::variable(*var_name);
                        self.advance()?;

                        // Handle method calls on variables
                        self.parse_method_chain(expr)
                    }
                    _ => {
                        Err(ParseError::UnexpectedToken {
                            token: "Expected variable name after '$'".to_string(),
                            position: 0,
                        })
                    }
                }
            }
            // Handle standalone function calls (e.g., count(), exists(), where())
            Some(Token::Count) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("count")
                } else {
                    let expr = ExpressionNode::identifier("count");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::Where) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("where")
                } else {
                    let expr = ExpressionNode::identifier("where");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::Select) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("select")
                } else {
                    let expr = ExpressionNode::identifier("select");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::All) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("all")
                } else {
                    let expr = ExpressionNode::identifier("all");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::First) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("first")
                } else {
                    let expr = ExpressionNode::identifier("first");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::Last) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("last")
                } else {
                    let expr = ExpressionNode::identifier("last");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::Empty) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("empty")
                } else {
                    let expr = ExpressionNode::identifier("empty");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::Tail) => {
                if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                    self.parse_function_call("tail")
                } else {
                    let expr = ExpressionNode::identifier("tail");
                    self.advance()?;
                    Ok(expr)
                }
            }
            Some(Token::Backtick) => {
                self.advance()?; // consume '`'

                // Expect identifier after backtick
                match self.peek() {
                    Some(Token::Identifier(name)) => {
                        let identifier_name = *name;
                        self.advance()?; // consume identifier

                        // Expect closing backtick
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier(identifier_name);
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    // Allow keywords as identifiers within backticks
                    Some(Token::Where) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("where");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::Select) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("select");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::All) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("all");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::First) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("first");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::Last) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("last");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::Count) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("count");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::Empty) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("empty");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::Tail) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("tail");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    // Add common FHIRPath keywords that might appear as identifiers
                    Some(Token::True) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("true");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    Some(Token::False) => {
                        self.advance()?; // consume keyword
                        if let Some(Token::Backtick) = self.peek() {
                            self.advance()?; // consume closing '`'
                            let expr = ExpressionNode::identifier("false");
                            self.parse_method_chain(expr)
                        } else {
                            Err(ParseError::UnexpectedToken {
                                token: "Expected closing backtick".to_string(),
                                position: 0,
                            })
                        }
                    }
                    _ => {
                        Err(ParseError::UnexpectedToken {
                            token: "Expected identifier after backtick".to_string(),
                            position: 0,
                        })
                    }
                }
            }
            _ => {
                Err(ParseError::UnexpectedToken {
                    token: format!("{:?}", self.peek()),
                    position: 0,
                })
            }
        }
    }

    /// Helper method to parse method chain on any expression
    fn parse_method_chain(&mut self, mut expr: ExpressionNode) -> ParseResult<ExpressionNode> {
        // Handle method calls and path expressions
        while let Some(Token::Dot) = self.peek() {
            self.advance()?; // consume dot

            match self.peek() {
                Some(Token::Identifier(path)) => {
                    let path_name = *path;
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, path_name)?;
                    } else {
                        expr = ExpressionNode::path(expr, path_name);
                        self.advance()?;
                    }
                }
                Some(Token::Where) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "where")?;
                    } else {
                        expr = ExpressionNode::path(expr, "where");
                        self.advance()?;
                    }
                }
                Some(Token::Select) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "select")?;
                    } else {
                        expr = ExpressionNode::path(expr, "select");
                        self.advance()?;
                    }
                }
                Some(Token::All) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "all")?;
                    } else {
                        expr = ExpressionNode::path(expr, "all");
                        self.advance()?;
                    }
                }
                Some(Token::First) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "first")?;
                    } else {
                        expr = ExpressionNode::path(expr, "first");
                        self.advance()?;
                    }
                }
                Some(Token::Last) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "last")?;
                    } else {
                        expr = ExpressionNode::path(expr, "last");
                        self.advance()?;
                    }
                }
                Some(Token::Count) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "count")?;
                    } else {
                        expr = ExpressionNode::path(expr, "count");
                        self.advance()?;
                    }
                }
                Some(Token::Tail) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "tail")?;
                    } else {
                        expr = ExpressionNode::path(expr, "tail");
                        self.advance()?;
                    }
                }
                Some(Token::Empty) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "empty")?;
                    } else {
                        expr = ExpressionNode::path(expr, "empty");
                        self.advance()?;
                    }
                }
                Some(Token::Take) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "take")?;
                    } else {
                        expr = ExpressionNode::path(expr, "take");
                        self.advance()?;
                    }
                }
                Some(Token::Skip) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "skip")?;
                    } else {
                        expr = ExpressionNode::path(expr, "skip");
                        self.advance()?;
                    }
                }
                Some(Token::Distinct) => {
                    if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                        expr = self.parse_method_call(expr, "distinct")?;
                    } else {
                        expr = ExpressionNode::path(expr, "distinct");
                        self.advance()?;
                    }
                }
                Some(Token::Backtick) => {
                    self.advance()?; // consume '`'

                    // Parse the backtick identifier
                    let path_name = match self.peek() {
                        Some(Token::Identifier(name)) => {
                            let id = *name;
                            self.advance()?; // consume identifier
                            id
                        }
                        Some(Token::Where) => {
                            self.advance()?; // consume keyword
                            "where"
                        }
                        Some(Token::Select) => {
                            self.advance()?; // consume keyword
                            "select"
                        }
                        Some(Token::All) => {
                            self.advance()?; // consume keyword
                            "all"
                        }
                        Some(Token::First) => {
                            self.advance()?; // consume keyword
                            "first"
                        }
                        Some(Token::Last) => {
                            self.advance()?; // consume keyword
                            "last"
                        }
                        Some(Token::Count) => {
                            self.advance()?; // consume keyword
                            "count"
                        }
                        Some(Token::Empty) => {
                            self.advance()?; // consume keyword
                            "empty"
                        }
                        Some(Token::Tail) => {
                            self.advance()?; // consume keyword
                            "tail"
                        }
                        Some(Token::Distinct) => {
                            self.advance()?; // consume keyword
                            "distinct"
                        }
                        Some(Token::True) => {
                            self.advance()?; // consume keyword
                            "true"
                        }
                        Some(Token::False) => {
                            self.advance()?; // consume keyword
                            "false"
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                token: "Expected identifier after backtick".to_string(),
                                position: 0,
                            });
                        }
                    };

                    // Expect closing backtick
                    if let Some(Token::Backtick) = self.peek() {
                        self.advance()?; // consume closing '`'

                        // Check if this is a method call
                        if let Ok(Some(Token::LeftParen)) = self.peek_ahead() {
                            expr = self.parse_method_call(expr, path_name)?;
                        } else {
                            expr = ExpressionNode::path(expr, path_name);
                        }
                    } else {
                        return Err(ParseError::UnexpectedToken {
                            token: "Expected closing backtick".to_string(),
                            position: 0,
                        });
                    }
                }
                // Add more keyword handling as needed
                _ => break,
            }
        }

        // Handle indexer expressions: expr[0]
        while let Some(Token::LeftBracket) = self.peek() {
            self.advance()?; // consume '['
            let index_expr = self.parse_expression()?;
            if let Some(Token::RightBracket) = self.peek() {
                self.advance()?; // consume ']'
                expr = ExpressionNode::index(expr, index_expr);
            } else {
                return Err(ParseError::UnexpectedToken {
                    token: "Expected ']'".to_string(),
                    position: 0,
                });
            }
        }

        Ok(expr)
    }

    /// Parse equality expressions - lower precedence than comparison
    fn parse_equality(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_comparison()?;

        loop {
            match self.peek() {
                Some(Token::Equal) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Equal, left, right);
                }
                Some(Token::NotEqual) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = ExpressionNode::binary_op(BinaryOperator::NotEqual, left, right);
                }
                Some(Token::Equivalent) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Equivalent, left, right);
                }
                Some(Token::NotEquivalent) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = ExpressionNode::binary_op(BinaryOperator::NotEquivalent, left, right);
                }
                Some(Token::Contains) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Contains, left, right);
                }
                Some(Token::In) => {
                    self.advance()?;
                    let right = self.parse_comparison()?;
                    left = ExpressionNode::binary_op(BinaryOperator::In, left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// Parse arithmetic expressions (addition and subtraction)
    fn parse_arithmetic(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_multiplicative()?;

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance()?;
                    let right = self.parse_multiplicative()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Add, left, right);
                }
                Some(Token::Minus) => {
                    self.advance()?;
                    let right = self.parse_multiplicative()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Subtract, left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// Parse multiplicative expressions (multiplication, division, modulo)
    fn parse_multiplicative(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_unary()?;

        loop {
            match self.peek() {
                Some(Token::Multiply) => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Multiply, left, right);
                }
                Some(Token::Divide) => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Divide, left, right);
                }
                Some(Token::Div) => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    left = ExpressionNode::binary_op(BinaryOperator::IntegerDivide, left, right);
                }
                Some(Token::Mod) => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Modulo, left, right);
                }
                Some(Token::Ampersand) => {
                    self.advance()?;
                    let right = self.parse_unary()?;
                    left = ExpressionNode::binary_op(BinaryOperator::Concatenate, left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// Parse union expressions (|) - higher precedence than equality
    fn parse_union(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_arithmetic()?;

        while let Some(Token::Union) = self.peek() {
            self.advance()?;
            let right = self.parse_arithmetic()?;
            left = ExpressionNode::union(left, right);
        }

        Ok(left)
    }

    /// Parse comparison expressions
    fn parse_comparison(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_union()?;

        loop {
            match self.peek() {
                Some(Token::LessThan) => {
                    self.advance()?;
                    let right = self.parse_union()?;
                    left = ExpressionNode::binary_op(BinaryOperator::LessThan, left, right);
                }
                Some(Token::LessThanOrEqual) => {
                    self.advance()?;
                    let right = self.parse_union()?;
                    left = ExpressionNode::binary_op(BinaryOperator::LessThanOrEqual, left, right);
                }
                Some(Token::GreaterThan) => {
                    self.advance()?;
                    let right = self.parse_union()?;
                    left = ExpressionNode::binary_op(BinaryOperator::GreaterThan, left, right);
                }
                Some(Token::GreaterThanOrEqual) => {
                    self.advance()?;
                    let right = self.parse_union()?;
                    left = ExpressionNode::binary_op(BinaryOperator::GreaterThanOrEqual, left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// Parse logical AND expressions
    fn parse_and(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_equality()?;

        while let Some(Token::And) = self.peek() {
            self.advance()?;
            let right = self.parse_equality()?;
            left = ExpressionNode::binary_op(BinaryOperator::And, left, right);
        }

        Ok(left)
    }

    /// Parse logical XOR expressions
    fn parse_xor(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_and()?;

        while let Some(Token::Xor) = self.peek() {
            self.advance()?;
            let right = self.parse_and()?;
            left = ExpressionNode::binary_op(BinaryOperator::Xor, left, right);
        }

        Ok(left)
    }

    /// Parse logical OR expressions
    fn parse_or(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_xor()?;

        while let Some(Token::Or) = self.peek() {
            self.advance()?;
            let right = self.parse_xor()?;
            left = ExpressionNode::binary_op(BinaryOperator::Or, left, right);
        }

        Ok(left)
    }

    /// Parse logical IMPLIES expressions (lowest precedence)
    fn parse_implies(&mut self) -> ParseResult<ExpressionNode> {
        let mut left = self.parse_or()?;

        while let Some(Token::Implies) = self.peek() {
            self.advance()?;
            let right = self.parse_or()?;
            left = ExpressionNode::binary_op(BinaryOperator::Implies, left, right);
        }

        Ok(left)
    }

    /// Parse complete expression
    pub fn parse_expression(&mut self) -> ParseResult<ExpressionNode> {
        self.parse_implies()
    }

    /// Parse the entire input
    pub fn parse(&mut self) -> ParseResult<ExpressionNode> {
        let expr = self.parse_expression()?;

        // Check if we consumed all tokens
        if self.current_token.is_some() {
            return Err(ParseError::UnexpectedToken {
                token: format!("Unexpected token: {:?}", self.current_token),
                position: 0,
            });
        }

        Ok(expr)
    }
}

/// Parse FHIRPath expression using the high-performance parser
pub fn parse_expression(input: &str) -> ParseResult<ExpressionNode> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let result = parse_expression("Patient").unwrap();
        assert!(matches!(result, ExpressionNode::Identifier { .. }));
    }

    #[test]
    fn test_path_expression() {
        let result = parse_expression("Patient.name").unwrap();
        assert!(matches!(result, ExpressionNode::Path { .. }));
    }

    #[test]
    fn test_complex_path() {
        let result = parse_expression("Patient.name.given").unwrap();
        assert!(matches!(result, ExpressionNode::Path { .. }));
    }

    #[test]
    fn test_function_call() {
        // For now, this will parse as path until we implement function calls
        let result = parse_expression("Patient.name.where");
        assert!(result.is_ok());
    }
}
