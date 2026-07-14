// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Auto-completion support for the REPL

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use reedline::{Completer, Span, Suggestion};
use std::collections::HashMap;
use std::sync::Arc;

use octofhir_fhirpath::FunctionRegistry;
use octofhir_fhirpath::ModelProvider;

/// Completion candidate
#[derive(Debug, Clone)]
pub struct Pair {
    pub display: String,
    pub replacement: String,
}

#[derive(Debug, Clone)]
struct PropertyCompletion {
    name: String,
    detail: String,
}

/// FHIRPath completer for reedline
pub struct FhirPathCompleter {
    commands: Vec<String>,
    cached_functions: std::sync::RwLock<Option<Vec<String>>>,
    cached_resource_types: std::sync::RwLock<Option<Vec<String>>>,
    cached_properties: std::sync::RwLock<HashMap<String, Vec<PropertyCompletion>>>,
    model_provider: Arc<dyn ModelProvider + Send + Sync>,
    registry: std::sync::RwLock<Option<Arc<FunctionRegistry>>>,
    fuzzy_matcher: SkimMatcherV2,
}

impl FhirPathCompleter {
    /// Create a new completer
    pub fn new(model_provider: Arc<dyn ModelProvider + Send + Sync>) -> Self {
        Self::with_registry(model_provider, None)
    }

    /// Create a new completer with function registry
    pub fn with_registry(
        model_provider: Arc<dyn ModelProvider + Send + Sync>,
        registry: Option<Arc<FunctionRegistry>>,
    ) -> Self {
        let commands = vec![
            ":load".to_string(),
            ":set".to_string(),
            ":unset".to_string(),
            ":vars".to_string(),
            ":resource".to_string(),
            ":type".to_string(),
            ":explain".to_string(),
            ":help".to_string(),
            ":history".to_string(),
            ":analyze".to_string(),
            ":validate".to_string(),
            ":quit".to_string(),
            ":exit".to_string(),
        ];

        Self {
            commands,
            cached_functions: std::sync::RwLock::new(None),
            cached_resource_types: std::sync::RwLock::new(None),
            cached_properties: std::sync::RwLock::new(HashMap::new()),
            model_provider,
            registry: std::sync::RwLock::new(registry),
            fuzzy_matcher: SkimMatcherV2::default(),
        }
    }

    /// Get completions for function names with enhanced descriptions and fuzzy matching
    fn complete_function(&self, word: &str, _context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Get function names from cache or fallback to common functions
        let function_names = self.get_cached_function_names();

        // Use fuzzy matching for better completion experience
        let mut scored_matches: Vec<(i64, String)> = Vec::new();

        for name in function_names {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(&name, word) {
                scored_matches.push((score, name));
            } else if name.starts_with(word) {
                // Fallback to prefix matching with high score
                scored_matches.push((1000, name));
            }
        }

        // Sort by fuzzy match score (higher is better)
        scored_matches.sort_by_key(|m| std::cmp::Reverse(m.0));

        // Create enhanced completions with descriptions
        for (_, name) in scored_matches.into_iter().take(10) {
            // Limit to top 10 matches
            let description = self.get_function_description(&name);
            let display = if description.is_empty() {
                name.clone()
            } else {
                format!("{name} - {description}")
            };

            candidates.push(Pair {
                display,
                replacement: name,
            });
        }

        candidates
    }

    /// Get description for a function/operation from registry
    fn get_function_description(&self, name: &str) -> String {
        // Get description from registry metadata
        if let Ok(registry_guard) = self.registry.read()
            && let Some(ref registry) = *registry_guard
            && let Some(metadata) = registry.get_metadata(name)
        {
            return metadata.description.clone();
        }

        // Fallback descriptions for common functions
        match name {
            "first" => "Returns the first item in a collection".to_string(),
            "last" => "Returns the last item in a collection".to_string(),
            "count" => "Returns the number of items in a collection".to_string(),
            "where" => "Filters a collection based on a boolean expression".to_string(),
            "select" => "Transforms each item in a collection using an expression".to_string(),
            "exists" => "Returns true if the collection is not empty".to_string(),
            "empty" => "Returns true if the collection is empty".to_string(),
            _ => "FHIRPath function".to_string(),
        }
    }

    /// Get cached function names from registry or cache
    fn get_cached_function_names(&self) -> Vec<String> {
        // First check cache
        if let Ok(guard) = self.cached_functions.read()
            && let Some(ref cached) = *guard
        {
            return cached.clone();
        }

        // Get from registry (registry is always available as core feature)
        if let Ok(registry_guard) = self.registry.read()
            && let Some(ref registry) = *registry_guard
        {
            // Get all function names directly
            let function_names: Vec<String> =
                registry.list_functions().into_iter().cloned().collect();

            // Cache the results
            if let Ok(mut cache_guard) = self.cached_functions.write() {
                *cache_guard = Some(function_names.clone());
            }

            return function_names;
        }

        // This should never happen since registry is always available
        vec![]
    }

    /// Check if we're in a command context (line starts with :)
    fn is_command_context(&self, line: &str) -> bool {
        line.trim_start().starts_with(':')
    }

    /// Pre-fetch resource types from model provider asynchronously
    /// This should be called during REPL initialization to populate the cache
    pub async fn prefetch_resource_types(&self) {
        let resource_types = self
            .model_provider
            .get_resource_types()
            .await
            .unwrap_or_default();

        if let Ok(mut cache_guard) = self.cached_resource_types.write() {
            *cache_guard = Some(resource_types.clone());
        }

        let mut type_names = resource_types;
        if let Ok(complex_types) = self.model_provider.get_complex_types().await {
            type_names.extend(complex_types);
        }

        type_names.sort();
        type_names.dedup();

        let mut properties_by_type = HashMap::new();
        for type_name in type_names {
            if let Ok(elements) = self.model_provider.get_elements(&type_name).await
                && !elements.is_empty()
            {
                let completions = elements
                    .into_iter()
                    .map(|element| PropertyCompletion {
                        name: element.name,
                        detail: element.documentation.unwrap_or(element.element_type),
                    })
                    .collect();
                properties_by_type.insert(type_name, completions);
            }
        }

        if let Ok(mut cache_guard) = self.cached_properties.write() {
            *cache_guard = properties_by_type;
        }
    }

    /// Get cached resource types (synchronous)
    /// Returns cached resource types.
    fn get_cached_resource_types(&self) -> Vec<String> {
        if let Ok(guard) = self.cached_resource_types.read()
            && let Some(ref cached) = *guard
        {
            return cached.clone();
        }

        Vec::new()
    }

    /// Extract the most likely resource type from a FHIRPath context
    fn extract_resource_type_from_context(&self, context: &str) -> String {
        // Handle common FHIRPath patterns to determine the current resource type

        // Remove the current word being typed (after the last space)
        let context = if let Some(last_space) = context.rfind(' ') {
            &context[..last_space]
        } else {
            context
        };

        // Case 1: Simple resource type like "Patient."
        if let Some(dot_pos) = context.find('.') {
            let first_part = &context[..dot_pos];
            if self.is_resource_type(first_part) {
                // Handle complex paths like "Bundle.entry.resource."
                if context.contains("Bundle.entry.resource") {
                    // This could be any resource type, default to generic
                    return "Resource".to_string();
                } else if context.contains("Bundle.entry") {
                    return "BundleEntry".to_string();
                } else {
                    return first_part.to_string();
                }
            }
        }

        // Case 2: No dots, might be typing a resource type
        if context.is_empty() || !context.contains('.') {
            return "Resource".to_string(); // Generic fallback
        }

        // Case 3: Complex expression - infer from resource type mentions available in cache.
        if context.contains("Bundle.entry.resource") {
            "Resource".to_string() // Generic resource in Bundle
        } else if context.contains("Bundle") {
            "Bundle".to_string()
        } else if let Some(resource_type) = self
            .get_cached_resource_types()
            .into_iter()
            .find(|resource_type| context.contains(resource_type))
        {
            resource_type
        } else {
            "Resource".to_string() // Generic fallback
        }
    }

    /// Check if a string is a known FHIR resource type
    fn is_resource_type(&self, candidate: &str) -> bool {
        let resource_types = self.get_cached_resource_types();
        resource_types.contains(&candidate.to_string())
    }

    fn complete_properties(&self, word: &str, context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Extract resource type from context - handle complex expressions
        let resource_type = self.extract_resource_type_from_context(context);

        let properties = if let Ok(guard) = self.cached_properties.read() {
            guard.get(&resource_type).cloned().unwrap_or_default()
        } else {
            Vec::new()
        };

        // Use fuzzy matching for property completion, but show all if word is empty
        let mut scored_matches: Vec<(i64, PropertyCompletion)> = Vec::new();

        for property in properties {
            if word.is_empty() {
                // Show all properties when word is empty (user just typed a dot)
                scored_matches.push((1000, property));
            } else if let Some(score) = self.fuzzy_matcher.fuzzy_match(&property.name, word) {
                scored_matches.push((score, property));
            } else if property.name.starts_with(word) {
                // Fallback to prefix matching with high score
                scored_matches.push((1000, property));
            }
        }

        // Sort by fuzzy match score (higher is better)
        scored_matches.sort_by_key(|m| std::cmp::Reverse(m.0));

        // Create completion candidates
        for (_, property) in scored_matches.into_iter().take(8) {
            // Limit to top 8 properties
            candidates.push(Pair {
                display: format!("{} - {}", property.name, property.detail),
                replacement: property.name,
            });
        }

        candidates
    }

    /// Get context-aware suggestions based on what the user is typing
    fn get_context_suggestions(&self, word: &str, context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Suggest resource types if context looks like it's starting
        if !context.contains('.') && !context.starts_with(':') {
            let resource_types = self.get_cached_resource_types();
            for resource_type in resource_types {
                if let Some(_score) = self.fuzzy_matcher.fuzzy_match(&resource_type, word) {
                    candidates.push(Pair {
                        display: format!("{} - FHIR resource type", resource_type),
                        replacement: resource_type,
                    });
                }
            }
        }

        candidates
    }

    /// Get command-specific completions
    fn get_command_specific_completions(&self, word: &str, line: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Parse command to provide appropriate completions
        if line.starts_with(":help") && !word.is_empty() {
            // Complete function names for help command
            let function_names = self.get_cached_function_names();
            for name in function_names {
                if name.starts_with(word) {
                    candidates.push(Pair {
                        display: format!("{} - get help for this function", name),
                        replacement: name,
                    });
                }
            }
        }

        candidates
    }

    /// Cache function names for future use
    pub fn cache_function_names(&self, function_names: Vec<String>) {
        if let Ok(mut guard) = self.cached_functions.write() {
            *guard = Some(function_names);
        }
    }
}

// Reedline completer implementation
impl Completer for FhirPathCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let line = &line[..pos];

        // Find the word being completed
        let (start, word) = if let Some(last_space) = line.rfind(' ') {
            (last_space + 1, &line[last_space + 1..])
        } else {
            (0, line)
        };

        let mut suggestions = Vec::new();

        // Complete commands (starting with :)
        if let Some(command_word) = word.strip_prefix(':') {
            for cmd in &self.commands {
                if cmd.starts_with(word)
                    || self.fuzzy_matcher.fuzzy_match(cmd, command_word).is_some()
                {
                    suggestions.push(Suggestion {
                        value: cmd[1..].to_string(), // Remove ':' for replacement
                        description: Some(format!("Command: {}", cmd)),
                        extra: None,
                        display_override: None,
                        span: Span::new(start + 1, pos), // Start after the ':'
                        append_whitespace: true,
                        style: None,
                        match_indices: None,
                    });
                }
            }
        } else if !self.is_command_context(line) {
            // FHIRPath completion

            // Function completions with fuzzy matching
            let function_suggestions = self.complete_function(word, line);
            for pair in function_suggestions.into_iter().take(8) {
                suggestions.push(Suggestion {
                    value: pair.replacement,
                    description: Some(pair.display),
                    extra: None,
                    display_override: None,
                    span: Span::new(start, pos),
                    append_whitespace: false,
                    style: None,
                    match_indices: None,
                });
            }

            // Property completions - trigger when we have a dot in the line
            if line.contains('.') {
                // For property completion, we need to extract the word after the last dot
                let (property_start, property_word) = if let Some(last_dot) = line.rfind('.') {
                    (last_dot + 1, &line[last_dot + 1..])
                } else {
                    (start, word)
                };

                let property_suggestions = self.complete_properties(property_word, line);
                for pair in property_suggestions.into_iter().take(8) {
                    suggestions.push(Suggestion {
                        value: pair.replacement,
                        description: Some(pair.display),
                        extra: None,
                        display_override: None,
                        span: Span::new(property_start, pos),
                        append_whitespace: false,
                        style: None,
                        match_indices: None,
                    });
                }
            }

            // Context suggestions
            if !word.is_empty() {
                let context_suggestions = self.get_context_suggestions(word, line);
                for pair in context_suggestions.into_iter().take(3) {
                    suggestions.push(Suggestion {
                        value: pair.replacement,
                        description: Some(pair.display),
                        extra: None,
                        display_override: None,
                        span: Span::new(start, pos),
                        append_whitespace: false,
                        style: None,
                        match_indices: None,
                    });
                }
            }
        } else {
            // Command context completions
            let command_suggestions = self.get_command_specific_completions(word, line);
            for pair in command_suggestions.into_iter().take(5) {
                suggestions.push(Suggestion {
                    value: pair.replacement,
                    description: Some(pair.display),
                    extra: None,
                    display_override: None,
                    span: Span::new(start, pos),
                    append_whitespace: false,
                    style: None,
                    match_indices: None,
                });
            }
        }

        // Limit total suggestions for better UX
        suggestions.truncate(15);

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhir_model::EmptyModelProvider;

    #[tokio::test]
    async fn property_completion_uses_model_provider_metadata() {
        let completer = FhirPathCompleter::new(Arc::new(EmptyModelProvider));
        completer.prefetch_resource_types().await;

        let completions = completer.complete_properties("", "Patient.");
        let names: Vec<_> = completions
            .iter()
            .map(|completion| completion.replacement.as_str())
            .collect();

        assert!(names.contains(&"id"));
        assert!(names.contains(&"name"));
    }
}
