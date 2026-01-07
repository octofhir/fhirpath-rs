//! Completion provider for FHIRPath expressions

use crate::core::ModelProvider;
use crate::evaluator::FunctionRegistry;
use crate::lsp::LspContext;
use lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, MarkupContent, MarkupKind,
};
use std::sync::Arc;

/// Type of completion expected based on context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    /// FHIR resource properties (after `.`)
    Property,
    /// FHIRPath functions (after `(` or at start)
    Function,
    /// Constants (`%variable`)
    Constant,
    /// Keywords (`$this`, `$index`, `$total`)
    Keyword,
    /// Resource types (for `is`, `as`, `ofType`)
    ResourceType,
    /// Unknown/general
    Unknown,
}

/// Context for completion at a specific position
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// The trigger character that initiated completion
    pub trigger_char: Option<char>,
    /// Current expression text
    pub expression: String,
    /// Cursor position (byte offset)
    pub cursor_offset: usize,
    /// Partial token at cursor (for filtering)
    pub partial_token: Option<String>,
    /// Expected completion kind based on analysis
    pub kind: CompletionKind,
    /// Inferred or explicit resource type
    pub resource_type: Option<String>,
}

impl CompletionContext {
    /// Analyze expression and cursor position to determine completion context
    pub fn analyze(expression: &str, cursor_offset: usize, trigger_char: Option<char>) -> Self {
        let kind = Self::detect_kind(expression, cursor_offset, trigger_char);
        let partial_token = Self::extract_partial_token(expression, cursor_offset);
        let resource_type = Self::infer_resource_type(expression);

        Self {
            trigger_char,
            expression: expression.to_string(),
            cursor_offset,
            partial_token,
            kind,
            resource_type,
        }
    }

    /// Detect completion kind from expression context
    fn detect_kind(
        expression: &str,
        cursor_offset: usize,
        trigger_char: Option<char>,
    ) -> CompletionKind {
        match trigger_char {
            Some('.') => CompletionKind::Property,
            Some('%') => CompletionKind::Constant,
            Some('$') => CompletionKind::Keyword,
            Some('(') => CompletionKind::Function,
            _ => {
                // Analyze preceding text
                let before = &expression[..cursor_offset.min(expression.len())];
                let trimmed = before.trim_end();

                if trimmed.ends_with('.') {
                    CompletionKind::Property
                } else if trimmed.ends_with('%') {
                    CompletionKind::Constant
                } else if trimmed.ends_with('$') {
                    CompletionKind::Keyword
                } else if Self::is_after_type_operator(trimmed) {
                    CompletionKind::ResourceType
                } else {
                    CompletionKind::Function
                }
            }
        }
    }

    /// Check if cursor is after a type operator (is, as, ofType)
    fn is_after_type_operator(text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        if let Some(last) = words.last() {
            matches!(*last, "is" | "as" | "ofType(" | "ofType")
        } else {
            false
        }
    }

    /// Extract partial token before cursor for filtering
    fn extract_partial_token(expression: &str, cursor_offset: usize) -> Option<String> {
        let before = &expression[..cursor_offset.min(expression.len())];

        // Find start of current token
        let token_start = before
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let partial = &before[token_start..];
        if partial.is_empty() {
            None
        } else {
            Some(partial.to_string())
        }
    }

    /// Infer resource type from expression prefix
    fn infer_resource_type(expression: &str) -> Option<String> {
        // Simple heuristic: first identifier that starts with uppercase
        let trimmed = expression.trim_start();
        let first_part: String = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();

        if !first_part.is_empty() && first_part.chars().next()?.is_uppercase() {
            Some(first_part)
        } else {
            None
        }
    }

    /// Extract the property chain from expression up to cursor position
    /// Returns (base_type, property_chain) where property_chain is the list of property names
    /// Example: "Patient.name.given." -> (Some("Patient"), ["name", "given"])
    pub fn extract_property_chain(
        expression: &str,
        cursor_offset: usize,
    ) -> (Option<String>, Vec<String>) {
        let before = &expression[..cursor_offset.min(expression.len())];

        // Find the start of the current navigation chain
        // We need to handle nested parentheses to find the correct chain start
        let chain_text = Self::extract_chain_text(before);

        if chain_text.is_empty() {
            return (None, Vec::new());
        }

        // Split by '.' to get segments
        let segments: Vec<&str> = chain_text.split('.').collect();

        if segments.is_empty() {
            return (None, Vec::new());
        }

        // First segment is the base type (if it starts with uppercase)
        let first = segments[0].trim();
        let base_type =
            if !first.is_empty() && first.chars().next().is_some_and(|c| c.is_uppercase()) {
                Some(first.to_string())
            } else {
                None
            };

        // Remaining segments are the property chain
        // Skip the last segment if it's empty (means we're completing after a dot)
        // or if it's a partial token being typed
        let properties: Vec<String> = segments[1..]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        (base_type, properties)
    }

    /// Extract the chain text from before cursor, handling parentheses
    fn extract_chain_text(before: &str) -> &str {
        // Find the last position that could start a navigation chain
        // This handles cases like: someFunc(Patient.name.)
        let mut paren_depth = 0;
        let mut chain_start = 0;

        for (i, c) in before.char_indices().rev() {
            match c {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    } else {
                        // We've gone past a function call, start after this
                        chain_start = i + 1;
                        break;
                    }
                }
                ' ' | '\t' | '\n' | '|' | '+' | '-' | '*' | '/' | '=' | '<' | '>' | ','
                    if paren_depth == 0 =>
                {
                    // Found a delimiter, chain starts after this
                    chain_start = i + 1;
                    break;
                }
                _ => {}
            }
        }

        before[chain_start..].trim_start()
    }
}

/// Provider for FHIRPath completions
pub struct CompletionProvider {
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    function_registry: Arc<FunctionRegistry>,
}

impl CompletionProvider {
    /// Create a new completion provider
    pub fn new(
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        function_registry: Arc<FunctionRegistry>,
    ) -> Self {
        Self {
            model_provider,
            function_registry,
        }
    }

    /// Provide completions for the given context
    pub async fn provide(
        &self,
        context: &CompletionContext,
        lsp_context: &LspContext,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();

        match context.kind {
            CompletionKind::Property => {
                // Properties first (sortText "0_"), then functions (sortText "1_")
                items.extend(self.property_completions(context, lsp_context).await);
                items.extend(self.function_completions_with_priority("1_"));
            }
            CompletionKind::Function => {
                // When we have explicit resource type context (e.g., ViewDefinition columns),
                // also provide property completions since user might be typing a property name
                // without the resource prefix
                if lsp_context.resource_type.is_some() {
                    items.extend(self.property_completions(context, lsp_context).await);
                    items.extend(self.function_completions_with_priority("1_"));
                } else {
                    items.extend(self.function_completions_with_priority("0_"));
                }
            }
            CompletionKind::Constant => {
                items.extend(self.constant_completions(lsp_context));
            }
            CompletionKind::Keyword => {
                items.extend(Self::keyword_completions());
            }
            CompletionKind::ResourceType => {
                items.extend(self.resource_type_completions().await);
            }
            CompletionKind::Unknown => {
                // Provide all applicable completions
                if lsp_context.resource_type.is_some() {
                    items.extend(self.property_completions(context, lsp_context).await);
                }
                items.extend(self.function_completions_with_priority("1_"));
                items.extend(Self::keyword_completions());
            }
        }

        // Filter by partial token
        if let Some(ref partial) = context.partial_token {
            let partial_lower = partial.to_lowercase();
            items.retain(|item| item.label.to_lowercase().starts_with(&partial_lower));
        }

        items
    }

    /// Generate property completions based on resource type
    async fn property_completions(
        &self,
        context: &CompletionContext,
        lsp_context: &LspContext,
    ) -> Vec<CompletionItem> {
        // Resolve the type at cursor position by traversing the property chain
        let resolved_type = self.resolve_type_at_cursor(context, lsp_context).await;

        let Some(type_name) = resolved_type else {
            return Vec::new();
        };

        // Get elements from model provider using the resolved type
        let elements = match self.model_provider.get_elements(&type_name).await {
            Ok(elements) => elements,
            Err(_) => return Vec::new(),
        };

        // Convert ElementInfo to CompletionItem with sortText for priority
        elements
            .into_iter()
            .map(|element| {
                CompletionItem {
                    label: element.name.clone(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(element.element_type.clone()),
                    documentation: element.documentation.map(|doc| {
                        Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: doc,
                        })
                    }),
                    // Properties get "0_" prefix for highest priority
                    sort_text: Some(format!("0_{}", element.name)),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Resolve the type at cursor position by traversing the property chain
    async fn resolve_type_at_cursor(
        &self,
        context: &CompletionContext,
        lsp_context: &LspContext,
    ) -> Option<String> {
        // Extract the property chain from the expression
        let (base_type, properties) =
            CompletionContext::extract_property_chain(&context.expression, context.cursor_offset);

        // Determine the starting type
        let start_type = lsp_context.resource_type.as_ref().cloned().or(base_type)?;

        // Check if the expression before cursor ends with "."
        // If not and we have properties, the last one is a partial token being typed
        // and should NOT be traversed (it's what we're completing, not a path segment)
        let before_cursor =
            &context.expression[..context.cursor_offset.min(context.expression.len())];
        let has_trailing_dot = before_cursor.trim_end().ends_with('.');

        // Determine which properties to actually traverse
        let properties_to_traverse: &[String] = if !has_trailing_dot && !properties.is_empty() {
            // Last property is partial token being typed - skip it
            &properties[..properties.len() - 1]
        } else {
            &properties
        };

        // If no properties to traverse, return the starting type
        if properties_to_traverse.is_empty() {
            return Some(start_type);
        }

        // Traverse the property chain to resolve the final type
        let mut current_type = start_type;

        for property in properties_to_traverse {
            // Get the type info for the current type
            let type_info = match self.model_provider.get_type(&current_type).await {
                Ok(Some(info)) => info,
                Ok(None) | Err(_) => return None,
            };

            // Get the element type for this property
            match self
                .model_provider
                .get_element_type(&type_info, property)
                .await
            {
                Ok(Some(element_type)) => {
                    // Update current type to the element's type
                    current_type = element_type.type_name;
                }
                Ok(None) | Err(_) => {
                    // Property not found, can't continue chain resolution
                    return None;
                }
            }
        }

        Some(current_type)
    }

    /// Generate function completions from registry with sort priority prefix
    fn function_completions_with_priority(&self, sort_prefix: &str) -> Vec<CompletionItem> {
        self.function_registry
            .list_functions()
            .iter()
            .filter_map(|name| {
                self.function_registry.get_metadata(name).map(|meta| {
                    let insert_text = if meta.signature.min_params > 0 {
                        format!("{}($0)", name)
                    } else {
                        format!("{}()", name)
                    };

                    CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(format!(
                            "({} args) -> {}",
                            meta.signature.min_params, meta.signature.return_type
                        )),
                        documentation: if meta.description.is_empty() {
                            None
                        } else {
                            Some(Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: meta.description.clone(),
                            }))
                        },
                        insert_text: Some(insert_text),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        sort_text: Some(format!("{}{}", sort_prefix, name)),
                        ..Default::default()
                    }
                })
            })
            .collect()
    }

    /// Generate constant completions (built-in + external)
    fn constant_completions(&self, lsp_context: &LspContext) -> Vec<CompletionItem> {
        // Built-in FHIRPath constants
        let mut items = vec![
            Self::create_constant_item(
                "%resource",
                "Resource",
                "The root resource being evaluated",
            ),
            Self::create_constant_item("%context", "Element", "The evaluation context element"),
            Self::create_constant_item("%ucum", "string", "UCUM unit system URL"),
            Self::create_constant_item("%sct", "string", "SNOMED CT system URL"),
            Self::create_constant_item("%loinc", "string", "LOINC system URL"),
        ];

        // External constants from context
        for (name, info) in &lsp_context.constants {
            items.push(Self::create_constant_item(
                &format!("%{}", name),
                &info.type_name,
                info.description.as_deref().unwrap_or("External constant"),
            ));
        }

        items
    }

    /// Create a constant completion item
    fn create_constant_item(name: &str, type_name: &str, description: &str) -> CompletionItem {
        CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some(type_name.to_string()),
            documentation: Some(Documentation::String(description.to_string())),
            insert_text: Some(name.to_string()),
            ..Default::default()
        }
    }

    /// Generate keyword completions
    fn keyword_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "$this".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Current focus item".to_string()),
                documentation: Some(Documentation::String(
                    "References the current item in iteration contexts like where() or select()"
                        .to_string(),
                )),
                ..Default::default()
            },
            CompletionItem {
                label: "$index".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Current iteration index".to_string()),
                documentation: Some(Documentation::String(
                    "The zero-based index of the current item in iteration".to_string(),
                )),
                ..Default::default()
            },
            CompletionItem {
                label: "$total".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Aggregate accumulator".to_string()),
                documentation: Some(Documentation::String(
                    "The running total in aggregate() function".to_string(),
                )),
                ..Default::default()
            },
        ]
    }

    /// Generate resource type completions
    async fn resource_type_completions(&self) -> Vec<CompletionItem> {
        // Common FHIR resource types
        // In a full implementation, this would query the ModelProvider
        let common_types = [
            "Patient",
            "Observation",
            "Condition",
            "Encounter",
            "Procedure",
            "MedicationRequest",
            "DiagnosticReport",
            "Practitioner",
            "Organization",
            "Location",
            "Device",
            "Immunization",
            "AllergyIntolerance",
            "CarePlan",
            "Goal",
            "Bundle",
            "Composition",
            "DocumentReference",
        ];

        common_types
            .iter()
            .map(|name| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("FHIR Resource Type".to_string()),
                ..Default::default()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_kind_property() {
        let ctx = CompletionContext::analyze("Patient.", 8, Some('.'));
        assert_eq!(ctx.kind, CompletionKind::Property);
    }

    #[test]
    fn test_detect_kind_constant() {
        let ctx = CompletionContext::analyze("Patient.name | %", 16, Some('%'));
        assert_eq!(ctx.kind, CompletionKind::Constant);
    }

    #[test]
    fn test_detect_kind_keyword() {
        let ctx = CompletionContext::analyze("where($", 7, Some('$'));
        assert_eq!(ctx.kind, CompletionKind::Keyword);
    }

    #[test]
    fn test_infer_resource_type() {
        let ctx = CompletionContext::analyze("Patient.name", 12, None);
        assert_eq!(ctx.resource_type, Some("Patient".to_string()));
    }

    #[test]
    fn test_infer_resource_type_observation() {
        let ctx = CompletionContext::analyze("Observation.value", 17, None);
        assert_eq!(ctx.resource_type, Some("Observation".to_string()));
    }

    #[test]
    fn test_partial_token() {
        let ctx = CompletionContext::analyze("Patient.na", 10, None);
        assert_eq!(ctx.partial_token, Some("na".to_string()));
    }

    #[test]
    fn test_extract_property_chain_simple() {
        let (base, props) = CompletionContext::extract_property_chain("Patient.", 8);
        assert_eq!(base, Some("Patient".to_string()));
        assert!(props.is_empty());
    }

    #[test]
    fn test_extract_property_chain_one_level() {
        let (base, props) = CompletionContext::extract_property_chain("Patient.name.", 13);
        assert_eq!(base, Some("Patient".to_string()));
        assert_eq!(props, vec!["name"]);
    }

    #[test]
    fn test_extract_property_chain_two_levels() {
        let (base, props) = CompletionContext::extract_property_chain("Patient.name.given.", 19);
        assert_eq!(base, Some("Patient".to_string()));
        assert_eq!(props, vec!["name", "given"]);
    }

    #[test]
    fn test_extract_property_chain_partial_token() {
        // When typing "Patient.name.gi" cursor at position 16
        let (base, props) = CompletionContext::extract_property_chain("Patient.name.gi", 15);
        assert_eq!(base, Some("Patient".to_string()));
        // "gi" is partial token, "name" is complete
        assert_eq!(props, vec!["name", "gi"]);
    }

    #[test]
    fn test_extract_property_chain_in_function() {
        // After function call: where(Patient.name.)
        let (base, props) = CompletionContext::extract_property_chain("where(Patient.name.", 19);
        assert_eq!(base, Some("Patient".to_string()));
        assert_eq!(props, vec!["name"]);
    }

    #[test]
    fn test_extract_property_chain_after_pipe() {
        // After pipe operator: something | Patient.name.
        let (base, props) =
            CompletionContext::extract_property_chain("something | Patient.name.", 25);
        assert_eq!(base, Some("Patient".to_string()));
        assert_eq!(props, vec!["name"]);
    }
}
