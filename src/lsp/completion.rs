//! LSP completion implementation
//!
//! Provides intelligent, context-aware completions for FHIRPath expressions
//! using the analyzer framework and function registry.

use crate::analyzer::{FhirPathAnalyzer, completion_provider::CompletionContext, completion_provider::ExpressionContext};
use crate::model::provider::ModelProvider;
use crate::registry::FunctionRegistry;
use lsp_types::*;
use std::sync::Arc;

/// Get completions for a position in a document
pub async fn get_completions<P: ModelProvider>(
    text: &str,
    position: Position,
    analyzer: &FhirPathAnalyzer<P>,
    _function_registry: &FunctionRegistry,
) -> Result<Vec<CompletionItem>, Box<dyn std::error::Error + Send + Sync>> {
    // Convert position to byte offset
    let offset = position_to_offset(text, position)?;
    
    // Parse the expression up to the cursor
    let text_up_to_cursor = &text[..offset];
    
    // Try to parse the partial expression
    let expression = match crate::parser::parse(text_up_to_cursor) {
        Ok(expr) => expr,
        Err(_) => {
            // If parsing fails, try to parse just the text before the cursor
            // This handles incomplete expressions during typing
            return get_fallback_completions(text_up_to_cursor, position).await;
        }
    };

    // Create completion context
    let completion_context = create_completion_context(text_up_to_cursor, offset)?;

    // Get completions from the analyzer
    let analysis_result = analyzer
        .analyze_with_completions(&expression, Some("Resource"), offset as u32)
        .await
        .map_err(|e| format!("Analysis error: {}", e))?;

    // Convert analyzer completions to LSP completions
    let mut lsp_completions = Vec::new();
    for completion in analysis_result.completions {
        lsp_completions.push(convert_to_lsp_completion(completion, position));
    }

    // Add context-aware completions based on the current position
    lsp_completions.extend(get_contextual_completions(text_up_to_cursor, position).await?);

    // Sort completions by priority
    lsp_completions.sort_by(|a, b| a.sort_text.as_ref().unwrap_or(&a.label).cmp(b.sort_text.as_ref().unwrap_or(&b.label)));

    Ok(lsp_completions)
}

/// Convert analyzer completion to LSP completion item
fn convert_to_lsp_completion(
    completion: crate::analyzer::completion_provider::Completion,
    _position: Position,
) -> CompletionItem {
    let kind = match completion.kind {
        crate::analyzer::completion_provider::CompletionKind::Property => CompletionItemKind::PROPERTY,
        crate::analyzer::completion_provider::CompletionKind::Method => CompletionItemKind::METHOD,
        crate::analyzer::completion_provider::CompletionKind::Function => CompletionItemKind::FUNCTION,
        crate::analyzer::completion_provider::CompletionKind::Variable => CompletionItemKind::VARIABLE,
        crate::analyzer::completion_provider::CompletionKind::Type => CompletionItemKind::CLASS,
        crate::analyzer::completion_provider::CompletionKind::Keyword => CompletionItemKind::KEYWORD,
        crate::analyzer::completion_provider::CompletionKind::Snippet => CompletionItemKind::SNIPPET,
        crate::analyzer::completion_provider::CompletionKind::Constant => CompletionItemKind::CONSTANT,
        crate::analyzer::completion_provider::CompletionKind::Operator => CompletionItemKind::OPERATOR,
    };

    let insert_text_format = if completion.insert_text.contains("${") {
        Some(InsertTextFormat::SNIPPET)
    } else {
        Some(InsertTextFormat::PLAIN_TEXT)
    };

    CompletionItem {
        label: completion.label,
        kind: Some(kind),
        detail: completion.detail,
        documentation: completion.documentation.map(|doc| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc,
            })
        }),
        deprecated: Some(completion.deprecated),
        preselect: None,
        sort_text: Some(completion.sort_text),
        filter_text: None,
        insert_text: Some(completion.insert_text),
        insert_text_format,
        insert_text_mode: None,
        text_edit: None,
        additional_text_edits: None,
        command: None,
        commit_characters: None,
        data: completion.data,
        tags: None,
    }
}

/// Create completion context from text and cursor position
fn create_completion_context(
    text_up_to_cursor: &str,
    offset: usize,
) -> Result<CompletionContext, Box<dyn std::error::Error + Send + Sync>> {
    // Analyze the text around the cursor to determine context
    let trigger_character = text_up_to_cursor.chars().last();
    
    // Determine if we're in a property access context
    let is_property_access = trigger_character == Some('.');
    
    // Determine if we're in a function call context
    let is_function_call = trigger_character == Some('(') || 
                          text_up_to_cursor.ends_with("(");
    
    // Extract base type if we can determine it
    let base_type = if is_property_access {
        extract_base_type_before_dot(text_up_to_cursor)
    } else {
        None
    };

    let expression_context = if is_property_access || is_function_call || base_type.is_some() {
        Some(ExpressionContext {
            base_type,
            is_collection: false, // TODO: Improve collection detection
            in_function_call: if is_function_call {
                extract_function_name(text_up_to_cursor)
            } else {
                None
            },
            in_method_call: None, // TODO: Detect method calls
            parameter_index: if is_function_call {
                Some(count_parameters_before_cursor(text_up_to_cursor))
            } else {
                None
            },
        })
    } else {
        None
    };

    Ok(CompletionContext {
        position: offset as u32,
        trigger_text: trigger_character.map(|c| c.to_string()),
        trigger_character,
        expression_context,
    })
}

/// Extract the base type before a dot (.)
fn extract_base_type_before_dot(text: &str) -> Option<String> {
    if !text.ends_with('.') {
        return None;
    }
    
    let text_before_dot = &text[..text.len() - 1];
    
    // Simple heuristic: if it starts with a capital letter, it might be a type
    if let Some(word) = text_before_dot.split_whitespace().last() {
        if let Some(first_char) = word.chars().next() {
            if first_char.is_uppercase() {
                return Some(word.to_string());
            }
        }
    }
    
    // Fallback: assume it's a Resource
    Some("Resource".to_string())
}

/// Extract function name from text ending with (
fn extract_function_name(text: &str) -> Option<String> {
    let text = text.trim_end_matches('(').trim();
    text.split_whitespace().last().map(|s| s.to_string())
}

/// Count parameters before cursor in function call
fn count_parameters_before_cursor(text: &str) -> usize {
    let mut count = 0;
    let mut paren_depth = 0;
    
    for ch in text.chars().rev() {
        match ch {
            ')' => paren_depth += 1,
            '(' => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    break;
                }
            }
            ',' if paren_depth == 0 => count += 1,
            _ => {}
        }
    }
    
    count
}

/// Get fallback completions when parsing fails
async fn get_fallback_completions(
    text_up_to_cursor: &str,
    _position: Position,
) -> Result<Vec<CompletionItem>, Box<dyn std::error::Error + Send + Sync>> {
    let mut completions = Vec::new();
    
    // If text is empty or starts fresh, suggest common starting points
    if text_up_to_cursor.trim().is_empty() {
        completions.extend(get_root_level_completions());
    } else if text_up_to_cursor.ends_with('.') {
        // After a dot, suggest common properties
        completions.extend(get_property_completions());
    } else if text_up_to_cursor.ends_with('(') {
        // After opening parenthesis, suggest parameter help
        completions.extend(get_parameter_completions());
    }
    
    Ok(completions)
}

/// Get contextual completions based on cursor position
async fn get_contextual_completions(
    text_up_to_cursor: &str,
    _position: Position,
) -> Result<Vec<CompletionItem>, Box<dyn std::error::Error + Send + Sync>> {
    let mut completions = Vec::new();
    
    // Add operator completions
    if !text_up_to_cursor.trim().is_empty() && 
       !text_up_to_cursor.ends_with('.') && 
       !text_up_to_cursor.ends_with('(') {
        completions.extend(get_operator_completions());
    }
    
    Ok(completions)
}

/// Get root level completions (starting points)
fn get_root_level_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "Patient".to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("FHIR Patient resource".to_string()),
            documentation: Some(Documentation::String("Starting point for Patient resource expressions".to_string())),
            insert_text: Some("Patient".to_string()),
            sort_text: Some("aPatient".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "Observation".to_string(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("FHIR Observation resource".to_string()),
            documentation: Some(Documentation::String("Starting point for Observation resource expressions".to_string())),
            insert_text: Some("Observation".to_string()),
            sort_text: Some("aObservation".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "$this".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Current context".to_string()),
            documentation: Some(Documentation::String("Reference to the current evaluation context".to_string())),
            insert_text: Some("$this".to_string()),
            sort_text: Some("z$this".to_string()),
            ..Default::default()
        },
    ]
}

/// Get property completions (common FHIR properties)
fn get_property_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "id".to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("string".to_string()),
            documentation: Some(Documentation::String("Logical id of this artifact".to_string())),
            insert_text: Some("id".to_string()),
            sort_text: Some("aid".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "name".to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("HumanName[]".to_string()),
            documentation: Some(Documentation::String("A name associated with the resource".to_string())),
            insert_text: Some("name".to_string()),
            sort_text: Some("aname".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "status".to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some("code".to_string()),
            documentation: Some(Documentation::String("Status of the resource".to_string())),
            insert_text: Some("status".to_string()),
            sort_text: Some("astatus".to_string()),
            ..Default::default()
        },
    ]
}

/// Get parameter completions
fn get_parameter_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "$this".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Current context".to_string()),
            insert_text: Some("$this".to_string()),
            sort_text: Some("a$this".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "$index".to_string(),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some("Current iteration index".to_string()),
            insert_text: Some("$index".to_string()),
            sort_text: Some("a$index".to_string()),
            ..Default::default()
        },
    ]
}

/// Get operator completions
fn get_operator_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "where".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Filter collection".to_string()),
            documentation: Some(Documentation::String("Filters a collection based on a condition".to_string())),
            insert_text: Some("where($1)".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some("bwhere".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "select".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Transform collection".to_string()),
            documentation: Some(Documentation::String("Transforms each element in a collection".to_string())),
            insert_text: Some("select($1)".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some("bselect".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "=".to_string(),
            kind: Some(CompletionItemKind::OPERATOR),
            detail: Some("Equals comparison".to_string()),
            insert_text: Some(" = $1".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some("c=".to_string()),
            ..Default::default()
        },
    ]
}

/// Convert LSP position to byte offset in text
fn position_to_offset(text: &str, position: Position) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut offset = 0;
    let mut line = 0u32;
    let mut character = 0u32;

    for ch in text.chars() {
        if line == position.line && character == position.character {
            return Ok(offset);
        }

        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }

        offset += ch.len_utf8();
    }

    // If we've reached the end of the text, return the final offset
    Ok(offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_offset() {
        let text = "line1\nline2\nline3";
        
        assert_eq!(position_to_offset(text, Position::new(0, 0)).unwrap(), 0);
        assert_eq!(position_to_offset(text, Position::new(0, 5)).unwrap(), 5);
        assert_eq!(position_to_offset(text, Position::new(1, 0)).unwrap(), 6);
        assert_eq!(position_to_offset(text, Position::new(2, 5)).unwrap(), 17);
    }

    #[test]
    fn test_extract_base_type_before_dot() {
        assert_eq!(extract_base_type_before_dot("Patient."), Some("Patient".to_string()));
        assert_eq!(extract_base_type_before_dot("Observation."), Some("Observation".to_string()));
        assert_eq!(extract_base_type_before_dot("name."), Some("Resource".to_string()));
        assert_eq!(extract_base_type_before_dot("Patient"), None);
    }

    #[test] 
    fn test_count_parameters_before_cursor() {
        assert_eq!(count_parameters_before_cursor("func("), 0);
        assert_eq!(count_parameters_before_cursor("func(param1, "), 1);
        assert_eq!(count_parameters_before_cursor("func(param1, param2, "), 2);
        assert_eq!(count_parameters_before_cursor("func(nested(a, b), "), 1);
    }
}