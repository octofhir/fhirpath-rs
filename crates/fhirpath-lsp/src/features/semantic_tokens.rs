//! Semantic token generation for rich syntax highlighting

use lsp_types::{SemanticToken, SemanticTokens, SemanticTokensResult};
use octofhir_fhirpath::ast::ExpressionNode;
use octofhir_fhirpath::parser;

use crate::document::FhirPathDocument;

/// Token types (indices into legend)
pub mod token_types {
    /// Keyword token type (e.g., and, or, true, false)
    pub const KEYWORD: u32 = 0;
    /// Function token type (e.g., first, count, where)
    pub const FUNCTION: u32 = 1;
    /// Operator token type (e.g., +, -, *, /)
    pub const OPERATOR: u32 = 2;
    /// Variable token type (e.g., $this, $index)
    pub const VARIABLE: u32 = 3;
    /// Property token type (e.g., name, family)
    pub const PROPERTY: u32 = 4;
    /// Number literal token type
    pub const NUMBER: u32 = 5;
    /// String literal token type
    pub const STRING: u32 = 6;
    /// Comment token type
    pub const COMMENT: u32 = 7;
}

/// Token modifiers (bit flags)
pub mod token_modifiers {
    /// Readonly modifier (for immutable variables)
    pub const READONLY: u32 = 1 << 0;
    /// Deprecated modifier (for deprecated functions/properties)
    pub const DEPRECATED: u32 = 1 << 1;
    /// Definition modifier (for definitions)
    pub const DEFINITION: u32 = 1 << 2;
}

/// Token collector for AST traversal
struct TokenCollector {
    /// Text being analyzed (for position calculations)
    text: String,
    /// Collected tokens with absolute positions
    tokens: Vec<TokenWithPosition>,
}

/// Token with absolute position (before delta encoding)
#[derive(Debug, Clone)]
struct TokenWithPosition {
    line: u32,
    start_char: u32,
    length: u32,
    token_type: u32,
    token_modifiers_bitset: u32,
}

impl TokenCollector {
    fn new(text: String) -> Self {
        Self {
            text,
            tokens: Vec::new(),
        }
    }

    /// Add a token at the given text position
    fn add_token(&mut self, text: &str, token_type: u32, token_modifiers: u32) {
        // Find position of this text in the document
        if let Some(offset) = self.text.find(text) {
            let position = self.offset_to_position(offset);
            self.tokens.push(TokenWithPosition {
                line: position.0,
                start_char: position.1,
                length: text.len() as u32,
                token_type,
                token_modifiers_bitset: token_modifiers,
            });
        }
    }

    /// Convert byte offset to line/character position
    fn offset_to_position(&self, offset: usize) -> (u32, u32) {
        let mut line = 0u32;
        let mut line_start = 0;

        for (i, ch) in self.text.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                line_start = i + 1;
            }
        }

        let character = self.text[line_start..offset].chars().count() as u32;
        (line, character)
    }

    /// Walk the AST and collect tokens
    fn walk_ast(&mut self, node: &ExpressionNode) {
        use octofhir_fhirpath::ast::*;

        match node {
            ExpressionNode::Literal(literal) => {
                match &literal.value {
                    LiteralValue::String(s) => {
                        // String literals
                        self.add_token(&format!("'{}'", s), token_types::STRING, 0);
                    }
                    LiteralValue::Integer(_) | LiteralValue::Decimal(_) => {
                        // Number literals - find in source
                        if let Some(loc) = &literal.location
                            && let Some(text) = self.extract_text_from_location(loc)
                        {
                            self.add_token(&text, token_types::NUMBER, 0);
                        }
                    }
                    LiteralValue::Boolean(b) => {
                        // Boolean keywords
                        self.add_token(&b.to_string(), token_types::KEYWORD, 0);
                    }
                    _ => {}
                }
            }

            ExpressionNode::Identifier(id) => {
                // Check if it's a keyword
                if is_keyword(&id.name) {
                    self.add_token(&id.name, token_types::KEYWORD, 0);
                } else {
                    // Property or resource type
                    self.add_token(&id.name, token_types::PROPERTY, 0);
                }
            }

            ExpressionNode::FunctionCall(func) => {
                // Function name
                self.add_token(&func.name, token_types::FUNCTION, 0);

                // Process arguments
                for arg in &func.arguments {
                    self.walk_ast(arg);
                }
            }

            ExpressionNode::MethodCall(method) => {
                // Method object
                self.walk_ast(&method.object);

                // Method name
                self.add_token(&method.method, token_types::FUNCTION, 0);

                // Method arguments
                for arg in &method.arguments {
                    self.walk_ast(arg);
                }
            }

            ExpressionNode::PropertyAccess(prop) => {
                // Object
                self.walk_ast(&prop.object);

                // Property name
                self.add_token(&prop.property, token_types::PROPERTY, 0);
            }

            ExpressionNode::IndexAccess(idx) => {
                self.walk_ast(&idx.object);
                self.walk_ast(&idx.index);
            }

            ExpressionNode::BinaryOperation(bin_op) => {
                self.walk_ast(&bin_op.left);

                // Operator - use symbol() method to get actual operator text
                let op_str = bin_op.operator.symbol();
                // Logical operators are keywords, not operators
                let token_type = if bin_op.operator.is_logical() {
                    token_types::KEYWORD
                } else {
                    token_types::OPERATOR
                };
                self.add_token(op_str, token_type, 0);

                self.walk_ast(&bin_op.right);
            }

            ExpressionNode::UnaryOperation(un_op) => {
                // Operator - use symbol() method
                let op_str = un_op.operator.symbol();
                let token_type =
                    if matches!(un_op.operator, octofhir_fhirpath::ast::UnaryOperator::Not) {
                        token_types::KEYWORD
                    } else {
                        token_types::OPERATOR
                    };
                self.add_token(op_str, token_type, 0);

                self.walk_ast(&un_op.operand);
            }

            ExpressionNode::Lambda(lambda) => {
                if let Some(param) = &lambda.parameter {
                    self.add_token(param, token_types::VARIABLE, token_modifiers::READONLY);
                }
                self.walk_ast(&lambda.body);
            }

            ExpressionNode::Collection(coll) => {
                for elem in &coll.elements {
                    self.walk_ast(elem);
                }
            }

            ExpressionNode::Parenthesized(expr) => {
                self.walk_ast(expr);
            }

            ExpressionNode::TypeCast(cast) => {
                self.walk_ast(&cast.expression);
                self.add_token("as", token_types::KEYWORD, 0);
                self.add_token(&cast.target_type, token_types::KEYWORD, 0);
            }

            ExpressionNode::Filter(filter) => {
                self.walk_ast(&filter.base);
                self.walk_ast(&filter.condition);
            }

            ExpressionNode::Union(union) => {
                self.walk_ast(&union.left);
                self.walk_ast(&union.right);
            }

            ExpressionNode::TypeCheck(type_check) => {
                self.walk_ast(&type_check.expression);
                self.add_token("is", token_types::KEYWORD, 0);
                self.add_token(&type_check.target_type, token_types::KEYWORD, 0);
            }

            ExpressionNode::Variable(var) => {
                self.add_token(&var.name, token_types::VARIABLE, token_modifiers::READONLY);
            }

            ExpressionNode::Path(path) => {
                self.walk_ast(&path.base);
                // Path string could be tokenized further if needed
                self.add_token(&path.path, token_types::PROPERTY, 0);
            }

            ExpressionNode::TypeInfo(type_info) => {
                // Format: namespace.name (e.g., "System.Integer")
                let full_type = format!("{}.{}", type_info.namespace, type_info.name);
                self.add_token(&full_type, token_types::KEYWORD, 0);
            }
        }
    }

    /// Extract text from source location
    fn extract_text_from_location(
        &self,
        _loc: &octofhir_fhirpath::core::SourceLocation,
    ) -> Option<String> {
        // For now, we can't easily extract text from source location
        // This would require storing span information in the AST
        None
    }

    /// Convert collected tokens to delta-encoded LSP format
    fn into_lsp_tokens(mut self) -> Vec<SemanticToken> {
        // Sort tokens by position (line, then character)
        self.tokens.sort_by_key(|t| (t.line, t.start_char));

        let mut lsp_tokens = Vec::new();
        let mut prev_line = 0;
        let mut prev_char = 0;

        for token in self.tokens {
            let delta_line = token.line.saturating_sub(prev_line);
            let delta_start = if delta_line == 0 {
                token.start_char.saturating_sub(prev_char)
            } else {
                token.start_char
            };

            lsp_tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifiers_bitset,
            });

            prev_line = token.line;
            prev_char = if delta_line == 0 {
                prev_char + delta_start
            } else {
                token.start_char
            };
        }

        lsp_tokens
    }
}

/// Check if an identifier is a FHIRPath keyword
fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "true"
            | "false"
            | "and"
            | "or"
            | "xor"
            | "implies"
            | "div"
            | "mod"
            | "in"
            | "contains"
            | "is"
            | "as"
    )
}

/// Generate semantic tokens for document
pub fn generate_semantic_tokens(document: &FhirPathDocument) -> SemanticTokensResult {
    let mut all_tokens = Vec::new();

    // Process each expression in the document
    for expr in &document.expressions {
        // Parse expression to AST
        let parse_result = parser::parse(&expr.text);

        if let Some(ast) = parse_result.ast {
            // Walk AST and collect tokens
            let mut collector = TokenCollector::new(document.text.clone());
            collector.walk_ast(&ast);
            let tokens = collector.into_lsp_tokens();
            all_tokens.extend(tokens);
        }
    }

    // Remove duplicates and sort
    all_tokens.sort_by_key(|t| (t.delta_line, t.delta_start));
    all_tokens.dedup_by_key(|t| (t.delta_line, t.delta_start, t.length));

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: all_tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_generate_semantic_tokens_simple() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.family".to_string(),
            1,
        );

        let result = generate_semantic_tokens(&doc);

        if let SemanticTokensResult::Tokens(tokens) = result {
            // Should have tokens for Patient, name, family
            assert!(!tokens.data.is_empty());
        }
    }

    #[test]
    fn test_generate_semantic_tokens_function() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.first()".to_string(),
            1,
        );

        let result = generate_semantic_tokens(&doc);

        if let SemanticTokensResult::Tokens(tokens) = result {
            // Should have function token
            let has_function = tokens
                .data
                .iter()
                .any(|t| t.token_type == token_types::FUNCTION);
            assert!(has_function, "Should have function token");
        }
    }

    #[test]
    fn test_generate_semantic_tokens_keyword() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.active and Patient.deceased".to_string(),
            1,
        );

        let result = generate_semantic_tokens(&doc);

        if let SemanticTokensResult::Tokens(tokens) = result {
            // Should have keyword token for 'and'
            let has_keyword = tokens
                .data
                .iter()
                .any(|t| t.token_type == token_types::KEYWORD);
            assert!(has_keyword, "Should have keyword token");
        }
    }
}
