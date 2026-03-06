//! Semantic token provider for FHIRPath expressions

use crate::ast::expression::ExpressionNode;
use crate::ast::literal::LiteralValue;
use crate::ast::operator::{BinaryOperator, UnaryOperator};
use crate::parser::parse_with_analysis;

use lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};

/// Standard token types used by the FHIRPath semantic tokens provider
pub const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD, // 0 - $this, $index, $total, as, is, true, false
    SemanticTokenType::FUNCTION, // 1 - function/method names
    SemanticTokenType::PROPERTY, // 2 - property access names
    SemanticTokenType::OPERATOR, // 3 - +, -, *, /, =, !=, and, or, etc.
    SemanticTokenType::NUMBER,  // 4 - integer, decimal literals
    SemanticTokenType::STRING,  // 5 - string literals
    SemanticTokenType::VARIABLE, // 6 - %constants
    SemanticTokenType::TYPE,    // 7 - type names (in is/as/ofType)
];

const TT_KEYWORD: u32 = 0;
const TT_FUNCTION: u32 = 1;
const TT_PROPERTY: u32 = 2;
const TT_OPERATOR: u32 = 3;
const TT_NUMBER: u32 = 4;
const TT_STRING: u32 = 5;
const TT_VARIABLE: u32 = 6;
const TT_TYPE: u32 = 7;

/// Build the semantic tokens legend for capability registration
pub fn build_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKEN_TYPES.to_vec(),
        token_modifiers: vec![],
    }
}

/// Provider for semantic tokens in FHIRPath expressions
pub struct SemanticTokensProvider;

impl SemanticTokensProvider {
    /// Generate semantic tokens for a FHIRPath expression
    pub fn tokenize(expression: &str) -> Vec<SemanticToken> {
        let result = parse_with_analysis(expression);
        let Some(ast) = result.ast else {
            return Vec::new();
        };

        let mut raw_tokens = Vec::new();
        Self::collect_tokens(&ast, expression, &mut raw_tokens);

        // Sort by offset, deduplicate overlapping tokens
        raw_tokens.sort_by_key(|t| (t.0, t.1));
        raw_tokens.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

        // Convert to delta-encoded LSP semantic tokens
        Self::encode_tokens(&raw_tokens, expression)
    }

    /// Generate semantic tokens for a range within a FHIRPath expression
    pub fn tokenize_range(
        expression: &str,
        start_offset: usize,
        end_offset: usize,
    ) -> Vec<SemanticToken> {
        let result = parse_with_analysis(expression);
        let Some(ast) = result.ast else {
            return Vec::new();
        };

        let mut raw_tokens = Vec::new();
        Self::collect_tokens(&ast, expression, &mut raw_tokens);

        // Filter to tokens within the range
        raw_tokens.retain(|&(offset, len, _)| offset + len > start_offset && offset < end_offset);

        raw_tokens.sort_by_key(|t| (t.0, t.1));
        raw_tokens.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

        Self::encode_tokens(&raw_tokens, expression)
    }

    /// Collect raw tokens (offset, length, token_type_index) from AST
    fn collect_tokens(node: &ExpressionNode, source: &str, tokens: &mut Vec<(usize, usize, u32)>) {
        match node {
            ExpressionNode::Literal(lit) => {
                if let Some(ref loc) = lit.location {
                    let token_type = match &lit.value {
                        LiteralValue::Integer(_) | LiteralValue::Decimal(_) => TT_NUMBER,
                        LiteralValue::String(_) => TT_STRING,
                        LiteralValue::Boolean(_) => TT_KEYWORD,
                        LiteralValue::Date(_)
                        | LiteralValue::DateTime(_)
                        | LiteralValue::Time(_) => TT_STRING,
                        LiteralValue::Quantity { .. } | LiteralValue::Long(_) => TT_NUMBER,
                    };
                    tokens.push((loc.offset, loc.length, token_type));
                }
            }
            ExpressionNode::Identifier(id) => {
                if let Some(ref loc) = id.location {
                    let token_type = if id.name.starts_with(|c: char| c.is_uppercase()) {
                        TT_TYPE
                    } else {
                        TT_PROPERTY
                    };
                    tokens.push((loc.offset, loc.length, token_type));
                }
            }
            ExpressionNode::Variable(var) => {
                if let Some(ref loc) = var.location {
                    let token_type = match var.name.as_str() {
                        "this" | "index" | "total" => TT_KEYWORD,
                        _ => TT_VARIABLE,
                    };
                    tokens.push((loc.offset, loc.length, token_type));
                }
            }
            ExpressionNode::FunctionCall(func) => {
                if let Some(ref loc) = func.location {
                    // Token for just the function name
                    tokens.push((loc.offset, func.name.len(), TT_FUNCTION));
                }
                for arg in &func.arguments {
                    Self::collect_tokens(arg, source, tokens);
                }
            }
            ExpressionNode::MethodCall(method) => {
                Self::collect_tokens(&method.object, source, tokens);
                if let Some(ref loc) = method.location {
                    // Find the '.' and emit operator token
                    if let Some(dot_offset) = Self::find_char_in_range(source, loc.offset, '.') {
                        tokens.push((dot_offset, 1, TT_OPERATOR));
                        // Method name right after the dot
                        tokens.push((dot_offset + 1, method.method.len(), TT_FUNCTION));
                    }
                }
                for arg in &method.arguments {
                    Self::collect_tokens(arg, source, tokens);
                }
            }
            ExpressionNode::PropertyAccess(prop) => {
                Self::collect_tokens(&prop.object, source, tokens);
                if let Some(ref loc) = prop.location {
                    // Find the '.' and emit operator token
                    if let Some(dot_offset) = Self::find_char_in_range(source, loc.offset, '.') {
                        tokens.push((dot_offset, 1, TT_OPERATOR));
                        let token_type = if prop.property.starts_with(|c: char| c.is_uppercase()) {
                            TT_TYPE
                        } else {
                            TT_PROPERTY
                        };
                        tokens.push((dot_offset + 1, prop.property.len(), token_type));
                    }
                }
            }
            ExpressionNode::BinaryOperation(binop) => {
                Self::collect_tokens(&binop.left, source, tokens);
                // Find the operator in source between left end and right start
                let op_sym = binop.operator.symbol();
                if let Some(op_offset) = Self::find_operator_between(
                    source,
                    &binop.left,
                    &binop.right,
                    binop.location.as_ref().map(|l| l.offset).unwrap_or(0),
                    op_sym,
                ) {
                    let tt = if is_keyword_operator(&binop.operator) {
                        TT_KEYWORD
                    } else {
                        TT_OPERATOR
                    };
                    tokens.push((op_offset, op_sym.len(), tt));
                }
                Self::collect_tokens(&binop.right, source, tokens);
            }
            ExpressionNode::UnaryOperation(unop) => {
                let op_sym = unop.operator.symbol();
                if let Some(ref loc) = unop.location {
                    let tt = if unop.operator == UnaryOperator::Not {
                        TT_KEYWORD
                    } else {
                        TT_OPERATOR
                    };
                    tokens.push((loc.offset, op_sym.len(), tt));
                }
                Self::collect_tokens(&unop.operand, source, tokens);
            }
            ExpressionNode::TypeCast(cast) => {
                Self::collect_tokens(&cast.expression, source, tokens);
                if let Some(ref loc) = cast.location
                    && let Some(as_offset) =
                        Self::find_keyword_after_expr(source, &cast.expression, loc.offset, "as")
                {
                    tokens.push((as_offset, 2, TT_KEYWORD));
                    let type_start = as_offset + 2;
                    if let Some(type_offset) =
                        Self::find_ident_after(source, type_start, &cast.target_type)
                    {
                        tokens.push((type_offset, cast.target_type.len(), TT_TYPE));
                    }
                }
            }
            ExpressionNode::TypeCheck(check) => {
                Self::collect_tokens(&check.expression, source, tokens);
                if let Some(ref loc) = check.location
                    && let Some(is_offset) =
                        Self::find_keyword_after_expr(source, &check.expression, loc.offset, "is")
                {
                    tokens.push((is_offset, 2, TT_KEYWORD));
                    let type_start = is_offset + 2;
                    if let Some(type_offset) =
                        Self::find_ident_after(source, type_start, &check.target_type)
                    {
                        tokens.push((type_offset, check.target_type.len(), TT_TYPE));
                    }
                }
            }
            ExpressionNode::IndexAccess(idx) => {
                Self::collect_tokens(&idx.object, source, tokens);
                Self::collect_tokens(&idx.index, source, tokens);
            }
            ExpressionNode::Parenthesized(inner) => {
                Self::collect_tokens(inner, source, tokens);
            }
            ExpressionNode::Lambda(lambda) => {
                Self::collect_tokens(&lambda.body, source, tokens);
            }
            ExpressionNode::Collection(coll) => {
                for elem in &coll.elements {
                    Self::collect_tokens(elem, source, tokens);
                }
            }
            ExpressionNode::Filter(filter) => {
                Self::collect_tokens(&filter.base, source, tokens);
                Self::collect_tokens(&filter.condition, source, tokens);
            }
            ExpressionNode::Union(union) => {
                Self::collect_tokens(&union.left, source, tokens);
                // Find '|' operator between left and right
                if let Some(pipe_offset) = Self::find_operator_between(
                    source,
                    &union.left,
                    &union.right,
                    union.location.as_ref().map(|l| l.offset).unwrap_or(0),
                    "|",
                ) {
                    tokens.push((pipe_offset, 1, TT_OPERATOR));
                }
                Self::collect_tokens(&union.right, source, tokens);
            }
            ExpressionNode::Path(path) => {
                Self::collect_tokens(&path.base, source, tokens);
            }
            ExpressionNode::TypeInfo(info) => {
                let full = format!("{}.{}", info.namespace, info.name);
                if let Some(pos) = source.find(&full) {
                    tokens.push((pos, full.len(), TT_TYPE));
                }
            }
        }
    }

    /// Find a character in source starting from `start`
    fn find_char_in_range(source: &str, start: usize, ch: char) -> Option<usize> {
        let region = source.get(start..)?;
        region.find(ch).map(|i| start + i)
    }

    /// Find the operator symbol in source between the left and right child nodes.
    /// Uses child spans to narrow the search region.
    fn find_operator_between(
        source: &str,
        left: &ExpressionNode,
        right: &ExpressionNode,
        node_offset: usize,
        op_sym: &str,
    ) -> Option<usize> {
        // Determine search region: after left child, before right child
        let left_end = Self::node_end(left).unwrap_or(node_offset);
        let right_start = Self::node_start(right).unwrap_or(source.len());

        if left_end > source.len() || right_start > source.len() || left_end > right_start {
            return None;
        }

        let region = &source[left_end..right_start];
        region.find(op_sym).map(|i| left_end + i)
    }

    /// Find a keyword (like 'as', 'is') after an expression's span
    fn find_keyword_after_expr(
        source: &str,
        expr: &ExpressionNode,
        node_offset: usize,
        keyword: &str,
    ) -> Option<usize> {
        let expr_end = Self::node_end(expr).unwrap_or(node_offset);
        let region = source.get(expr_end..)?;
        // Look for the keyword surrounded by non-alphanumeric chars (word boundary)
        let mut search_from = 0;
        while let Some(pos) = region[search_from..].find(keyword) {
            let abs_pos = expr_end + search_from + pos;
            let before_ok = abs_pos == 0 || !source.as_bytes()[abs_pos - 1].is_ascii_alphanumeric();
            let after_pos = abs_pos + keyword.len();
            let after_ok =
                after_pos >= source.len() || !source.as_bytes()[after_pos].is_ascii_alphanumeric();
            if before_ok && after_ok {
                return Some(abs_pos);
            }
            search_from += pos + keyword.len();
            if search_from >= region.len() {
                break;
            }
        }
        None
    }

    /// Find an identifier after a given offset (skipping whitespace)
    fn find_ident_after(source: &str, start: usize, name: &str) -> Option<usize> {
        let region = source.get(start..)?;
        let trimmed_start = region.len() - region.trim_start().len();
        let remaining = &region[trimmed_start..];
        if remaining.starts_with(name) {
            Some(start + trimmed_start)
        } else {
            // Fallback: search for the name
            region.find(name).map(|i| start + i)
        }
    }

    /// Get the start offset of a node from its location
    fn node_start(node: &ExpressionNode) -> Option<usize> {
        Self::node_location(node).map(|l| l.offset)
    }

    /// Get the end offset (offset + length) of a node
    fn node_end(node: &ExpressionNode) -> Option<usize> {
        Self::node_location(node).map(|l| l.offset + l.length)
    }

    /// Extract the SourceLocation from any ExpressionNode
    fn node_location(node: &ExpressionNode) -> Option<&crate::core::SourceLocation> {
        match node {
            ExpressionNode::Literal(n) => n.location.as_ref(),
            ExpressionNode::Identifier(n) => n.location.as_ref(),
            ExpressionNode::Variable(n) => n.location.as_ref(),
            ExpressionNode::FunctionCall(n) => n.location.as_ref(),
            ExpressionNode::MethodCall(n) => n.location.as_ref(),
            ExpressionNode::PropertyAccess(n) => n.location.as_ref(),
            ExpressionNode::BinaryOperation(n) => n.location.as_ref(),
            ExpressionNode::UnaryOperation(n) => n.location.as_ref(),
            ExpressionNode::TypeCast(n) => n.location.as_ref(),
            ExpressionNode::TypeCheck(n) => n.location.as_ref(),
            ExpressionNode::IndexAccess(n) => n.location.as_ref(),
            ExpressionNode::Collection(n) => n.location.as_ref(),
            ExpressionNode::Union(n) => n.location.as_ref(),
            ExpressionNode::Parenthesized(_)
            | ExpressionNode::Lambda(_)
            | ExpressionNode::Filter(_)
            | ExpressionNode::Path(_)
            | ExpressionNode::TypeInfo(_) => None,
        }
    }

    /// Encode raw tokens (offset, length, type) into delta-encoded LSP SemanticTokens
    fn encode_tokens(raw: &[(usize, usize, u32)], source: &str) -> Vec<SemanticToken> {
        let mut result = Vec::with_capacity(raw.len());
        let mut prev_line = 0u32;
        let mut prev_start = 0u32;

        for &(offset, length, token_type) in raw {
            if length == 0 {
                continue;
            }
            let (line, col) = Self::offset_to_line_col(source, offset);

            let delta_line = line - prev_line;
            let delta_start = if delta_line == 0 {
                col - prev_start
            } else {
                col
            };

            result.push(SemanticToken {
                delta_line,
                delta_start,
                length: length as u32,
                token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = line;
            prev_start = col;
        }

        result
    }

    /// Convert byte offset to (line, column) both 0-based
    fn offset_to_line_col(source: &str, offset: usize) -> (u32, u32) {
        let mut line = 0u32;
        let mut col = 0u32;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

/// Check if a binary operator is a keyword (rendered as word rather than symbol)
fn is_keyword_operator(op: &BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Xor
            | BinaryOperator::Implies
            | BinaryOperator::In
            | BinaryOperator::Contains
            | BinaryOperator::Is
            | BinaryOperator::As
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_legend() {
        let legend = build_legend();
        assert!(!legend.token_types.is_empty());
        assert!(legend.token_types.contains(&SemanticTokenType::FUNCTION));
        assert!(legend.token_types.contains(&SemanticTokenType::PROPERTY));
    }

    #[test]
    fn test_offset_to_line_col() {
        assert_eq!(
            SemanticTokensProvider::offset_to_line_col("abc\ndef", 0),
            (0, 0)
        );
        assert_eq!(
            SemanticTokensProvider::offset_to_line_col("abc\ndef", 3),
            (0, 3)
        );
        assert_eq!(
            SemanticTokensProvider::offset_to_line_col("abc\ndef", 4),
            (1, 0)
        );
        assert_eq!(
            SemanticTokensProvider::offset_to_line_col("abc\ndef", 5),
            (1, 1)
        );
    }

    #[test]
    fn test_simple_expression_tokens() {
        let tokens = SemanticTokensProvider::tokenize("1 + 2");
        assert!(
            tokens.len() >= 3,
            "Should have tokens for 1, +, 2; got {}",
            tokens.len()
        );
    }

    #[test]
    fn test_property_access_tokens() {
        let tokens = SemanticTokensProvider::tokenize("Patient.name");
        // Should have: Patient (type), . (operator), name (property)
        assert!(
            tokens.len() >= 3,
            "Should have at least 3 tokens; got {}",
            tokens.len()
        );
    }

    #[test]
    fn test_method_call_tokens() {
        let tokens = SemanticTokensProvider::tokenize("Patient.name.first()");
        // Patient (type), . (op), name (prop), . (op), first (function) — may vary by parser
        assert!(
            tokens.len() >= 4,
            "Should have at least 4 tokens; got {}",
            tokens.len()
        );
    }

    #[test]
    fn test_keyword_operator_tokens() {
        let tokens = SemanticTokensProvider::tokenize("true and false");
        // true (keyword), and (keyword), false (keyword)
        assert!(
            tokens.len() >= 3,
            "Should have at least 3 tokens; got {}",
            tokens.len()
        );
    }

    #[test]
    fn test_type_check_tokens() {
        let tokens = SemanticTokensProvider::tokenize("value is string");
        // value (property), is (keyword), string (type)
        assert!(
            tokens.len() >= 3,
            "Should have at least 3 tokens; got {}",
            tokens.len()
        );
    }

    #[test]
    fn test_union_operator_token() {
        let tokens = SemanticTokensProvider::tokenize("a | b");
        // a (property), | (operator), b (property)
        assert!(
            tokens.len() >= 3,
            "Should have at least 3 tokens; got {}",
            tokens.len()
        );
    }

    #[test]
    fn test_variable_tokens() {
        let tokens = SemanticTokensProvider::tokenize("%context");
        assert!(!tokens.is_empty(), "Should have a token for %context");
        // Should be VARIABLE type
        assert_eq!(tokens[0].token_type, TT_VARIABLE);
    }

    #[test]
    fn test_range_tokenize() {
        let tokens = SemanticTokensProvider::tokenize_range("Patient.name.given", 8, 18);
        // Should include tokens in the .name.given range
        assert!(!tokens.is_empty(), "Should have tokens in range");
    }
}
