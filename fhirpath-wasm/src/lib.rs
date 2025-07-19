use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `extern` block in C.
#[wasm_bindgen]
extern "C" {
    // Bind the `console.log` function from the browser's console
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!(..)`-style syntax for `console.log` logging.
#[allow(dead_code)]
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        log(&format!( $( $t )* ))
    }
}

/// Initialize panic hook for better error messages in the browser
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Evaluate a FHIRPath expression against a FHIR resource
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `resource_json` - The FHIR resource as a JSON string
///
/// # Returns
/// A JSON string containing the evaluation result, or an error message
#[wasm_bindgen]
pub fn evaluate_fhirpath(expression: &str, resource_json: &str) -> String {
    // Parse the JSON resource
    let resource: serde_json::Value = match serde_json::from_str(resource_json) {
        Ok(value) => value,
        Err(e) => {
            return format!(r#"{{"error": "Invalid JSON resource: {}"}}"#, e);
        }
    };

    // Evaluate the FHIRPath expression
    match fhirpath_core::evaluate(expression, resource) {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json_str) => json_str,
            Err(e) => format!(r#"{{"error": "Failed to serialize result: {}"}}"#, e),
        },
        Err(e) => {
            format!(r#"{{"error": "FHIRPath evaluation error: {}"}}"#, e)
        }
    }
}

/// Validate a FHIRPath expression syntax
///
/// # Arguments
/// * `expression` - The FHIRPath expression to validate
///
/// # Returns
/// A JSON string indicating whether the expression is valid
#[wasm_bindgen]
pub fn validate_fhirpath(expression: &str) -> String {
    // Try to parse the expression with an empty resource to check syntax
    let empty_resource = serde_json::Value::Object(serde_json::Map::new());

    match fhirpath_core::evaluate(expression, empty_resource) {
        Ok(_) => r#"{"valid": true}"#.to_string(),
        Err(e) => {
            format!(r#"{{"valid": false, "error": "{}"}}"#, e)
        }
    }
}

/// Get the FHIRPath specification version
#[wasm_bindgen]
pub fn get_fhirpath_version() -> String {
    fhirpath_core::FHIRPATH_SPEC_VERSION.to_string()
}

/// Get the AST (Abstract Syntax Tree) of a FHIRPath expression
///
/// # Arguments
/// * `expression` - The FHIRPath expression to parse
///
/// # Returns
/// A JSON string containing the AST representation, or an error message
#[wasm_bindgen]
pub fn get_expression_ast(expression: &str) -> String {
    // Try to tokenize the expression
    let tokens = match fhirpath_core::lexer::tokenize(expression) {
        Ok(tokens) => tokens,
        Err(error) => {
            return format!(r#"{{"error": "Tokenization error: {}"}}"#, error);
        }
    };

    // Try to parse the tokens
    let ast = match fhirpath_core::parser::parse(&tokens) {
        Ok(ast) => ast,
        Err(error) => {
            return format!(r#"{{"error": "Parse error: {}"}}"#, error);
        }
    };

    // Convert AST to tree representation
    let ast_tree = format_ast_as_tree(&ast, 0);
    format!(
        r#"{{"ast": "{}"}}"#,
        ast_tree.replace('\n', "\\n").replace('"', "\\\"")
    )
}

/// Format AST as a tree structure (similar to CLI implementation)
fn format_ast_as_tree(node: &fhirpath_core::parser::AstNode, indent: usize) -> String {
    use fhirpath_core::parser::{AstNode, BinaryOperator, UnaryOperator};

    let indent_str = "  ".repeat(indent);
    let mut result = String::new();

    match node {
        AstNode::Identifier(name) => {
            result.push_str(&format!("{}Identifier: {}\n", indent_str, name));
        }
        AstNode::StringLiteral(value) => {
            result.push_str(&format!("{}StringLiteral: \"{}\"\n", indent_str, value));
        }
        AstNode::NumberLiteral(value) => {
            result.push_str(&format!("{}NumberLiteral: {}\n", indent_str, value));
        }
        AstNode::BooleanLiteral(value) => {
            result.push_str(&format!("{}BooleanLiteral: {}\n", indent_str, value));
        }
        AstNode::DateTimeLiteral(value) => {
            result.push_str(&format!("{}DateTimeLiteral: {}\n", indent_str, value));
        }
        AstNode::Variable(name) => {
            result.push_str(&format!("{}Variable: %{}\n", indent_str, name));
        }
        AstNode::Path(left, right) => {
            result.push_str(&format!("{}Path:\n", indent_str));
            result.push_str(&format!("{}├─ Left:\n", indent_str));
            result.push_str(&format_ast_as_tree(left, indent + 2));
            result.push_str(&format!("{}└─ Right:\n", indent_str));
            result.push_str(&format_ast_as_tree(right, indent + 2));
        }
        AstNode::FunctionCall { name, arguments } => {
            result.push_str(&format!("{}FunctionCall: {}()\n", indent_str, name));
            if !arguments.is_empty() {
                result.push_str(&format!("{}Arguments:\n", indent_str));
                for (i, arg) in arguments.iter().enumerate() {
                    let prefix = if i == arguments.len() - 1 {
                        "└─"
                    } else {
                        "├─"
                    };
                    result.push_str(&format!("{}{} Arg {}:\n", indent_str, prefix, i + 1));
                    result.push_str(&format_ast_as_tree(arg, indent + 2));
                }
            }
        }
        AstNode::BinaryOp { op, left, right } => {
            result.push_str(&format!(
                "{}BinaryOp: {}\n",
                indent_str,
                format_binary_operator(op)
            ));
            result.push_str(&format!("{}├─ Left:\n", indent_str));
            result.push_str(&format_ast_as_tree(left, indent + 2));
            result.push_str(&format!("{}└─ Right:\n", indent_str));
            result.push_str(&format_ast_as_tree(right, indent + 2));
        }
        AstNode::UnaryOp { op, operand } => {
            result.push_str(&format!(
                "{}UnaryOp: {}\n",
                indent_str,
                format_unary_operator(op)
            ));
            result.push_str(&format!("{}└─ Operand:\n", indent_str));
            result.push_str(&format_ast_as_tree(operand, indent + 2));
        }
        AstNode::Indexer { collection, index } => {
            result.push_str(&format!("{}Indexer:\n", indent_str));
            result.push_str(&format!("{}├─ Collection:\n", indent_str));
            result.push_str(&format_ast_as_tree(collection, indent + 2));
            result.push_str(&format!("{}└─ Index:\n", indent_str));
            result.push_str(&format_ast_as_tree(index, indent + 2));
        }
        AstNode::QuantityLiteral { value, unit } => {
            let unit_str = unit.as_ref().map(|u| format!(" '{}'", u)).unwrap_or_default();
            result.push_str(&format!("{}QuantityLiteral: {}{}\n", indent_str, value, unit_str));
        }
    }

    result
}

/// Format binary operator as string
fn format_binary_operator(op: &fhirpath_core::parser::BinaryOperator) -> &'static str {
    use fhirpath_core::parser::BinaryOperator;

    match op {
        BinaryOperator::Equals => "=",
        BinaryOperator::NotEquals => "!=",
        BinaryOperator::Equivalent => "~",
        BinaryOperator::NotEquivalent => "!~",
        BinaryOperator::LessThan => "<",
        BinaryOperator::LessOrEqual => "<=",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::GreaterOrEqual => ">=",
        BinaryOperator::Addition => "+",
        BinaryOperator::Subtraction => "-",
        BinaryOperator::Multiplication => "*",
        BinaryOperator::Division => "/",
        BinaryOperator::Div => "div",
        BinaryOperator::Mod => "mod",
        BinaryOperator::And => "and",
        BinaryOperator::Or => "or",
        BinaryOperator::Xor => "xor",
        BinaryOperator::Implies => "implies",
        BinaryOperator::In => "in",
        BinaryOperator::Contains => "contains",
        BinaryOperator::Is => "is",
        BinaryOperator::As => "as",
        BinaryOperator::Union => "|",
        BinaryOperator::Concatenation => "&",
    }
}

/// Format unary operator as string
fn format_unary_operator(op: &fhirpath_core::parser::UnaryOperator) -> &'static str {
    use fhirpath_core::parser::UnaryOperator;

    match op {
        UnaryOperator::Positive => "+",
        UnaryOperator::Negate => "-",
        UnaryOperator::Not => "not",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_evaluate_simple_expression() {
        let resource =
            r#"{"resourceType": "Patient", "name": [{"given": ["John"], "family": "Doe"}]}"#;
        let result = evaluate_fhirpath("Patient.name.given", resource);
        assert!(result.contains("John"));
    }

    #[wasm_bindgen_test]
    fn test_validate_expression() {
        let result = validate_fhirpath("Patient.name");
        assert!(result.contains(r#""valid": true"#));
    }

    #[wasm_bindgen_test]
    fn test_invalid_expression() {
        let result = validate_fhirpath("Patient.name.invalid(");
        assert!(result.contains(r#""valid": false"#));
    }
}
