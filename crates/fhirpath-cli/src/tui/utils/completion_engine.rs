//! Context-aware completion engine for FHIRPath expressions

use std::collections::HashSet;
use std::sync::Arc;

use octofhir_fhirpath::FhirPathValue;
use octofhir_fhirpath::FunctionRegistry;
use octofhir_fhirpath::core::ModelProvider;
use serde_json::Value as JsonValue;

use crate::tui::app::{AppState, CompletionItem, CompletionKind};

/// Context-aware completion engine
pub struct CompletionEngine {
    _model_provider: Arc<dyn ModelProvider + Send + Sync>,
    function_registry: Option<Arc<FunctionRegistry>>,
    // analyzer: Arc<StaticAnalyzer>, // Removed
    cache: CompletionCache,
}

/// Cache for completion results
#[derive(Default)]
struct CompletionCache {
    functions: Vec<CompletionItem>,
    properties: std::collections::HashMap<String, Vec<CompletionItem>>,
    keywords: Vec<CompletionItem>,
}

/// Completion context information
#[derive(Debug)]
pub struct CompletionContext {
    pub expression: String,
    pub cursor_position: usize,
    pub current_resource_type: Option<String>,
    pub preceding_path: Vec<String>,
}

impl CompletionEngine {
    /// Create a new completion engine
    pub async fn new(
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        function_registry: Arc<FunctionRegistry>,
        // analyzer: Arc<StaticAnalyzer>, // Removed
    ) -> anyhow::Result<Self> {
        let mut engine = Self {
            _model_provider: model_provider,
            function_registry: Some(function_registry),
            // analyzer,
            cache: CompletionCache::default(),
        };

        // Pre-populate cache
        engine.populate_function_cache().await?;
        engine.populate_keyword_cache();

        Ok(engine)
    }

    /// Get completions for the given context
    pub async fn get_completions(
        &mut self,
        context: CompletionContext,
        state: &AppState,
    ) -> anyhow::Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();

        // Determine what kind of completion is needed based on context
        let completion_type = self.analyze_completion_context(&context)?;

        match completion_type {
            CompletionType::Function => {
                completions.extend(self.get_function_completions(&context).await?);
            }
            CompletionType::Property => {
                completions.extend(self.get_property_completions(&context, state).await?);
            }
            CompletionType::ResourceType => {
                completions.extend(self.get_resource_type_completions(&context).await?);
            }
            CompletionType::Variable => {
                completions.extend(self.get_variable_completions(&context, state));
            }
            CompletionType::Keyword => {
                completions.extend(self.get_keyword_completions(&context));
            }
            CompletionType::Mixed => {
                // Provide all types of completions, ranked by relevance
                completions.extend(self.get_function_completions(&context).await?);
                completions.extend(self.get_property_completions(&context, state).await?);
                completions.extend(self.get_variable_completions(&context, state));
            }
        }

        // Sort by relevance
        self.rank_completions(&mut completions, &context);

        Ok(completions)
    }

    /// Analyze context to determine completion type
    fn analyze_completion_context(
        &self,
        context: &CompletionContext,
    ) -> anyhow::Result<CompletionType> {
        let text_before_cursor = &context.expression[..context.cursor_position];
        let trimmed = text_before_cursor.trim_end();

        if trimmed.is_empty() {
            return Ok(CompletionType::ResourceType);
        }

        if trimmed.ends_with('.') {
            Ok(CompletionType::Property)
        } else if trimmed.contains('(') && !trimmed.ends_with(')') {
            Ok(CompletionType::Function)
        } else if trimmed.ends_with('%') {
            Ok(CompletionType::Variable)
        } else if text_before_cursor.ends_with(' ') {
            Ok(CompletionType::Keyword)
        } else if self
            .get_partial_word(context)
            .chars()
            .all(|c| c.is_ascii_uppercase())
        {
            Ok(CompletionType::ResourceType)
        } else {
            Ok(CompletionType::Mixed)
        }
    }

    /// Get function completions
    async fn get_function_completions(
        &self,
        _context: &CompletionContext,
    ) -> anyhow::Result<Vec<CompletionItem>> {
        Ok(self.cache.functions.clone())
    }

    /// Get property completions
    async fn get_property_completions(
        &mut self,
        context: &CompletionContext,
        state: &AppState,
    ) -> anyhow::Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();
        let mut seen = HashSet::new();
        let partial = self.get_partial_word(context).to_lowercase();

        if let Some(resource) = state.current_resource.as_ref() {
            Self::collect_properties_from_value(resource, &partial, &mut completions, &mut seen);

            if let FhirPathValue::Resource(_, type_info, _) = resource {
                if !completions.is_empty() {
                    self.cache
                        .properties
                        .insert(type_info.type_name.clone(), completions.clone());
                }
            }
        }

        if completions.is_empty() {
            if let Some(resource_type) = context.current_resource_type.as_deref() {
                if let Some(cached) = self.cache.properties.get(resource_type) {
                    completions.extend(
                        cached
                            .iter()
                            .filter(|item| {
                                partial.is_empty() || item.text.to_lowercase().starts_with(&partial)
                            })
                            .cloned(),
                    );
                }
            }
        }

        Ok(completions)
    }

    /// Get resource type completions
    async fn get_resource_type_completions(
        &self,
        _context: &CompletionContext,
    ) -> anyhow::Result<Vec<CompletionItem>> {
        // This would get resource types from ModelProvider
        Ok(vec![
            CompletionItem {
                text: "Patient".to_string(),
                display: "Patient".to_string(),
                kind: CompletionKind::ResourceType,
                documentation: Some("Patient resource type".to_string()),
                insert_range: None,
            },
            CompletionItem {
                text: "Observation".to_string(),
                display: "Observation".to_string(),
                kind: CompletionKind::ResourceType,
                documentation: Some("Observation resource type".to_string()),
                insert_range: None,
            },
        ])
    }

    /// Get variable completions
    fn get_variable_completions(
        &self,
        _context: &CompletionContext,
        state: &AppState,
    ) -> Vec<CompletionItem> {
        state
            .variables
            .keys()
            .map(|name| CompletionItem {
                text: format!("%{}", name),
                display: format!("%{}", name),
                kind: CompletionKind::Variable,
                documentation: Some(format!("Variable: {}", name)),
                insert_range: None,
            })
            .collect()
    }

    /// Get keyword completions
    fn get_keyword_completions(&self, _context: &CompletionContext) -> Vec<CompletionItem> {
        self.cache.keywords.clone()
    }

    /// Rank completions by relevance
    fn rank_completions(&self, completions: &mut [CompletionItem], context: &CompletionContext) {
        let partial = self.get_partial_word(context);

        completions.sort_by(|a, b| {
            let a_score = self.completion_score(a, &partial);
            let b_score = self.completion_score(b, &partial);
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get partial word being completed
    fn get_partial_word(&self, context: &CompletionContext) -> String {
        let text_before_cursor = &context.expression[..context.cursor_position];
        text_before_cursor
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .next_back()
            .unwrap_or("")
            .to_string()
    }

    /// Calculate relevance score for a completion
    fn completion_score(&self, completion: &CompletionItem, partial: &str) -> f64 {
        if partial.is_empty() {
            return match completion.kind {
                CompletionKind::Function => 1.0,
                CompletionKind::Property => 0.9,
                CompletionKind::Variable => 0.8,
                CompletionKind::Keyword => 0.7,
                CompletionKind::ResourceType => 0.6,
                CompletionKind::Operator => 0.5,
            };
        }

        let text = completion.text.to_lowercase();
        let partial = partial.to_lowercase();

        if text.starts_with(&partial) {
            1.0 + (partial.len() as f64 / text.len() as f64)
        } else if text.contains(&partial) {
            0.5 + (partial.len() as f64 / text.len() as f64)
        } else {
            0.0
        }
    }

    /// Pre-populate function cache
    async fn populate_function_cache(&mut self) -> anyhow::Result<()> {
        // Use the actual function registry to get all available functions
        if let Some(registry) = &self.function_registry {
            let functions = registry.list_functions();

            for name in functions {
                let completion = CompletionItem {
                    text: format!("{}()", name),
                    display: format!("{}()", name),
                    kind: CompletionKind::Function,
                    documentation: None,
                    insert_range: None,
                };
                self.cache.functions.push(completion);
            }
        } else {
            // Fallback to basic function list when registry is not available
            let basic_functions = vec![
                "first", "last", "count", "empty", "exists", "where", "select", "single",
            ];

            for name in basic_functions {
                let completion = CompletionItem {
                    text: format!("{}()", name),
                    display: format!("{}() - FHIRPath function", name),
                    kind: CompletionKind::Function,
                    documentation: Some(format!("FHIRPath {} function", name)),
                    insert_range: None,
                };
                self.cache.functions.push(completion);
            }
        }

        Ok(())
    }

    /// Pre-populate keyword cache
    fn populate_keyword_cache(&mut self) {
        let keywords = vec![
            ("and", "Logical AND operator"),
            ("or", "Logical OR operator"),
            ("xor", "Logical XOR operator"),
            ("implies", "Logical implication operator"),
            ("is", "Type checking operator"),
            ("as", "Type casting operator"),
            ("div", "Integer division operator"),
            ("mod", "Modulo operator"),
            ("in", "Membership test operator"),
            ("contains", "String/collection contains operator"),
            ("true", "Boolean true literal"),
            ("false", "Boolean false literal"),
            ("null", "Null literal"),
        ];

        for (keyword, description) in keywords {
            let completion = CompletionItem {
                text: keyword.to_string(),
                display: format!("{} - {}", keyword, description),
                kind: CompletionKind::Keyword,
                documentation: Some(description.to_string()),
                insert_range: None,
            };
            self.cache.keywords.push(completion);
        }
    }

    fn collect_properties_from_value(
        value: &FhirPathValue,
        partial: &str,
        completions: &mut Vec<CompletionItem>,
        seen: &mut HashSet<String>,
    ) {
        match value {
            FhirPathValue::Resource(json, type_info, _) => {
                if let JsonValue::Object(map) = json.as_ref() {
                    for (key, child) in map.iter() {
                        let key_lower = key.to_lowercase();
                        if !partial.is_empty() && !key_lower.starts_with(partial) {
                            continue;
                        }
                        if !seen.insert(key.clone()) {
                            continue;
                        }

                        let documentation = format!(
                            "Property '{}' on {} ({})",
                            key,
                            type_info.type_name,
                            Self::describe_json_value(child)
                        );

                        completions.push(CompletionItem {
                            text: key.clone(),
                            display: format!("{} · {}", key, type_info.type_name),
                            kind: CompletionKind::Property,
                            documentation: Some(documentation),
                            insert_range: None,
                        });
                    }
                }
            }
            FhirPathValue::Collection(collection) => {
                for nested in collection.values() {
                    Self::collect_properties_from_value(nested, partial, completions, seen);
                }
            }
            _ => {}
        }
    }

    fn describe_json_value(value: &JsonValue) -> &'static str {
        match value {
            JsonValue::Null => "null",
            JsonValue::Bool(_) => "boolean",
            JsonValue::Number(_) => "number",
            JsonValue::String(_) => "string",
            JsonValue::Array(_) => "array",
            JsonValue::Object(obj) => {
                if obj.contains_key("resourceType") {
                    "resource"
                } else {
                    "object"
                }
            }
        }
    }
}

/// Type of completion being requested
#[derive(Debug, Clone, PartialEq)]
enum CompletionType {
    Function,
    Property,
    ResourceType,
    Variable,
    Keyword,
    Mixed,
}
