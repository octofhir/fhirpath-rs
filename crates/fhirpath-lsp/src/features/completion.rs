//! Completion feature implementation
//!
//! Provides context-aware autocomplete for:
//! - FHIRPath functions with signatures and documentation
//! - FHIR resource properties
//! - Keywords and operators
//! - Variables ($this, $index, etc.)

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, CompletionResponse, Documentation,
    InsertTextFormat, MarkupContent, MarkupKind,
};
use octofhir_fhirpath::evaluator::create_function_registry;
use walkdir::WalkDir;

use crate::document::FhirPathDocument;

/// Completion context to determine what kind of completion to provide
#[derive(Debug, Clone, PartialEq, Eq)]
enum CompletionContext {
    /// General context - provide functions and keywords
    General,
    /// After `.` - provide properties
    Property,
    /// After `(` - provide function parameters
    FunctionParameters,
    /// After `$` - provide variables
    Variable,
}

/// Generate completion items for the given document and position
pub fn generate_completions(
    document: &FhirPathDocument,
    position: lsp_types::Position,
) -> Option<CompletionResponse> {
    let mut items = Vec::new();

    // Get text before cursor
    let offset = document.position_to_offset(position);
    let text_before = &document.text[..offset];

    // Check if we're in an @input-file directive
    if text_before.contains("@input-file") {
        return Some(generate_file_path_completions(document));
    }

    // Determine completion context
    let context = determine_context(text_before);

    match context {
        CompletionContext::General => {
            items.extend(get_function_completions());
            items.extend(get_keyword_completions());
        }
        CompletionContext::Property => {
            items.extend(get_property_completions());
        }
        CompletionContext::FunctionParameters => {
            // Could provide parameter hints here
            items.extend(get_variable_completions());
        }
        CompletionContext::Variable => {
            items.extend(get_variable_completions());
        }
    }

    Some(CompletionResponse::List(CompletionList {
        is_incomplete: false,
        items,
    }))
}

/// Determine completion context from text before cursor
fn determine_context(text_before: &str) -> CompletionContext {
    let trimmed = text_before.trim_end();

    if trimmed.ends_with('$') {
        return CompletionContext::Variable;
    }

    if trimmed.ends_with('.') {
        return CompletionContext::Property;
    }

    if trimmed.ends_with('(') {
        return CompletionContext::FunctionParameters;
    }

    CompletionContext::General
}

/// Get function completions from the function registry
fn get_function_completions() -> Vec<CompletionItem> {
    let registry = create_function_registry();
    let mut items = Vec::new();

    for (name, metadata) in registry.all_metadata() {
        // Build signature string
        let params_str = metadata
            .signature
            .parameters
            .iter()
            .map(|p| {
                let opt = if p.optional { "?" } else { "" };
                format!("{}{}", p.name, opt)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let signature = format!(
            "{}({}) -> {}",
            name, params_str, metadata.signature.return_type
        );

        // Build detailed documentation
        let mut doc_parts = vec![
            format!("**{}**", name),
            String::new(),
            metadata.description.clone(),
            String::new(),
            format!("```fhirpath\n{}\n```", signature),
        ];

        // Add parameter details
        if !metadata.signature.parameters.is_empty() {
            doc_parts.push(String::new());
            doc_parts.push("**Parameters:**".to_string());
            for param in &metadata.signature.parameters {
                let opt = if param.optional { " (optional)" } else { "" };
                doc_parts.push(format!(
                    "- `{}`: {}{}{}",
                    param.name,
                    param.parameter_type.join(" | "),
                    opt,
                    if !param.description.is_empty() {
                        format!(" - {}", param.description)
                    } else {
                        String::new()
                    }
                ));
            }
        }

        // Add category
        doc_parts.push(String::new());
        doc_parts.push(format!("*Category: {:?}*", metadata.category));

        let documentation = MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc_parts.join("\n"),
        };

        items.push(CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(signature),
            documentation: Some(Documentation::MarkupContent(documentation)),
            insert_text: Some(format!("{}($0)", name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }

    // Sort by name
    items.sort_by(|a, b| a.label.cmp(&b.label));

    items
}

/// Get property completions (common FHIR properties)
fn get_property_completions() -> Vec<CompletionItem> {
    vec![
        // Common FHIR properties
        create_property_item("name", "HumanName[]", "Patient or Practitioner name"),
        create_property_item("given", "string[]", "Given names"),
        create_property_item("family", "string", "Family name"),
        create_property_item("birthDate", "date", "Date of birth"),
        create_property_item("gender", "code", "Administrative gender"),
        create_property_item("active", "boolean", "Whether record is in active use"),
        create_property_item(
            "identifier",
            "Identifier[]",
            "Identifiers for this resource",
        ),
        create_property_item("telecom", "ContactPoint[]", "Contact details"),
        create_property_item("address", "Address[]", "Addresses"),
        create_property_item("id", "string", "Logical resource identifier"),
        create_property_item("resourceType", "string", "Resource type name"),
        create_property_item("meta", "Meta", "Metadata about the resource"),
        create_property_item("text", "Narrative", "Human-readable narrative"),
        create_property_item("contained", "Resource[]", "Contained resources"),
        create_property_item("extension", "Extension[]", "Additional content"),
        create_property_item(
            "modifierExtension",
            "Extension[]",
            "Extensions that modify meaning",
        ),
        // ContactPoint properties
        create_property_item("system", "code", "Contact point system"),
        create_property_item("value", "string", "The actual contact point value"),
        create_property_item("use", "code", "Purpose of this contact point"),
        create_property_item("rank", "positiveInt", "Preference order"),
        create_property_item("period", "Period", "Time period when valid"),
        // Address properties
        create_property_item("line", "string[]", "Street address lines"),
        create_property_item("city", "string", "City name"),
        create_property_item("state", "string", "State/province"),
        create_property_item("postalCode", "string", "Postal code"),
        create_property_item("country", "string", "Country"),
        create_property_item("type", "code", "Address type"),
        // Identifier properties
        create_property_item("assigner", "Reference", "Organization that issued id"),
        // Common choice types
        create_property_item("valueString", "string", "Value as string"),
        create_property_item("valueBoolean", "boolean", "Value as boolean"),
        create_property_item("valueInteger", "integer", "Value as integer"),
        create_property_item("valueDecimal", "decimal", "Value as decimal"),
        create_property_item("valueDate", "date", "Value as date"),
        create_property_item("valueDateTime", "dateTime", "Value as dateTime"),
        create_property_item("valueCode", "code", "Value as code"),
        create_property_item("valueCoding", "Coding", "Value as Coding"),
        create_property_item(
            "valueCodeableConcept",
            "CodeableConcept",
            "Value as CodeableConcept",
        ),
        create_property_item("valueQuantity", "Quantity", "Value as Quantity"),
        create_property_item("valueReference", "Reference", "Value as Reference"),
    ]
}

/// Helper to create a property completion item
fn create_property_item(name: &str, type_name: &str, description: &str) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(type_name.to_string()),
        documentation: Some(Documentation::String(description.to_string())),
        ..Default::default()
    }
}

/// Get keyword completions
fn get_keyword_completions() -> Vec<CompletionItem> {
    vec![
        create_keyword_item("and", "Logical AND operator"),
        create_keyword_item("or", "Logical OR operator"),
        create_keyword_item("xor", "Logical XOR operator"),
        create_keyword_item("implies", "Logical implication operator"),
        create_keyword_item("div", "Integer division operator"),
        create_keyword_item("mod", "Modulo operator"),
        create_keyword_item("in", "Collection membership test"),
        create_keyword_item("contains", "Collection containment test"),
        create_keyword_item("is", "Type checking operator"),
        create_keyword_item("as", "Type casting operator"),
        create_keyword_item("true", "Boolean true literal"),
        create_keyword_item("false", "Boolean false literal"),
    ]
}

/// Helper to create a keyword completion item
fn create_keyword_item(keyword: &str, description: &str) -> CompletionItem {
    CompletionItem {
        label: keyword.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        documentation: Some(Documentation::String(description.to_string())),
        ..Default::default()
    }
}

/// Get variable completions
fn get_variable_completions() -> Vec<CompletionItem> {
    vec![
        create_variable_item(
            "$this",
            "Current context item in iteration",
            "The current item when iterating in where(), select(), etc.",
        ),
        create_variable_item(
            "$index",
            "Current index in iteration",
            "Zero-based index of current item in iteration",
        ),
        create_variable_item(
            "$total",
            "Total items in iteration",
            "Total number of items being iterated",
        ),
        create_variable_item(
            "$context",
            "Root context",
            "The root context of the evaluation",
        ),
    ]
}

/// Helper to create a variable completion item
fn create_variable_item(name: &str, detail: &str, description: &str) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::String(description.to_string())),
        ..Default::default()
    }
}

/// Generate file path completions for @input-file directive
fn generate_file_path_completions(document: &FhirPathDocument) -> CompletionResponse {
    let mut items = Vec::new();

    // Get workspace root from document URI
    if let Ok(file_path) = document.uri.to_file_path()
        && let Some(workspace_root) = file_path.parent()
    {
        // Scan workspace for .json files (limit depth to avoid performance issues)
        for entry in WalkDir::new(workspace_root)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Only include JSON files
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                // Create relative path from workspace root
                let relative_path = path
                    .strip_prefix(workspace_root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                // Create absolute path for detail
                let absolute_path = path.display().to_string();

                // Read file size for additional info
                let detail = if let Ok(metadata) = std::fs::metadata(path) {
                    let size = metadata.len();
                    if size < 1024 {
                        format!("{} bytes", size)
                    } else if size < 1024 * 1024 {
                        format!("{:.1} KB", size as f64 / 1024.0)
                    } else {
                        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                    }
                } else {
                    "JSON file".to_string()
                };

                items.push(CompletionItem {
                    label: relative_path.clone(),
                    kind: Some(CompletionItemKind::FILE),
                    detail: Some(detail),
                    documentation: Some(Documentation::String(format!(
                        "**Path:** `{}`\n\nFHIR resource file",
                        absolute_path
                    ))),
                    insert_text: Some(relative_path),
                    ..Default::default()
                });
            }
        }
    }

    // Sort by label
    items.sort_by(|a, b| a.label.cmp(&b.label));

    CompletionResponse::List(CompletionList {
        is_incomplete: false,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_determine_context_general() {
        assert_eq!(
            determine_context("Patient.name"),
            CompletionContext::General
        );
    }

    #[test]
    fn test_determine_context_property() {
        assert_eq!(determine_context("Patient."), CompletionContext::Property);
    }

    #[test]
    fn test_determine_context_variable() {
        assert_eq!(determine_context("where($"), CompletionContext::Variable);
    }

    #[test]
    fn test_determine_context_function_params() {
        assert_eq!(
            determine_context("where("),
            CompletionContext::FunctionParameters
        );
    }

    #[test]
    fn test_get_function_completions() {
        let items = get_function_completions();
        assert!(!items.is_empty());

        // Check that common functions are present
        let function_names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(function_names.contains(&"where"));
        assert!(function_names.contains(&"select"));
        assert!(function_names.contains(&"first"));
        assert!(function_names.contains(&"count"));
    }

    #[test]
    fn test_get_property_completions() {
        let items = get_property_completions();
        assert!(!items.is_empty());

        // Check that common properties are present
        let property_names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(property_names.contains(&"name"));
        assert!(property_names.contains(&"family"));
        assert!(property_names.contains(&"given"));
    }

    #[test]
    fn test_get_keyword_completions() {
        let items = get_keyword_completions();
        assert!(!items.is_empty());

        let keyword_names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(keyword_names.contains(&"and"));
        assert!(keyword_names.contains(&"or"));
        assert!(keyword_names.contains(&"true"));
    }

    #[test]
    fn test_get_variable_completions() {
        let items = get_variable_completions();
        assert!(!items.is_empty());

        let var_names: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(var_names.contains(&"$this"));
        assert!(var_names.contains(&"$index"));
    }

    #[test]
    fn test_generate_completions_general() {
        let doc = FhirPathDocument::new(
            Url::parse("file:///test.fhirpath").unwrap(),
            "Patient.name.".to_string(),
            1,
        );

        let result = generate_completions(&doc, lsp_types::Position::new(0, 13));
        assert!(result.is_some());

        if let Some(CompletionResponse::List(list)) = result {
            assert!(!list.items.is_empty());
        }
    }
}
