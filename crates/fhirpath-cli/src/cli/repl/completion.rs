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

use rustyline::Helper;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Result as RlResult};
use std::sync::Arc;

use crate::model::provider::ModelProvider;
use crate::registry::FunctionRegistry;

/// FHIRPath completer for rustyline
pub struct FhirPathCompleter {
    commands: Vec<String>,
    cached_functions: std::sync::RwLock<Option<Vec<String>>>,
    cached_resource_types: std::sync::RwLock<Option<Vec<String>>>,
    model_provider: Arc<dyn ModelProvider>,
    registry: std::sync::RwLock<Option<Arc<FunctionRegistry>>>,
}

impl FhirPathCompleter {
    /// Create a new completer
    pub fn new(model_provider: Arc<dyn ModelProvider>) -> Self {
        Self::with_registry(model_provider, None)
    }

    /// Create a new completer with function registry
    pub fn with_registry(
        model_provider: Arc<dyn ModelProvider>,
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
            ":quit".to_string(),
            ":exit".to_string(),
        ];

        Self {
            commands,
            cached_functions: std::sync::RwLock::new(None),
            cached_resource_types: std::sync::RwLock::new(None),
            model_provider,
            registry: std::sync::RwLock::new(registry),
        }
    }

    /// Get completions for function names with enhanced descriptions
    fn complete_function(&self, word: &str, _context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Get function names from cache or fallback to common functions
        let function_names = self.get_cached_function_names();

        // Create enhanced completions with descriptions
        for name in function_names {
            if name.starts_with(word) {
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
        }

        // Sort by relevance: exact matches first, then by length, then alphabetically
        candidates.sort_by(|a, b| {
            let a_name = &a.replacement;
            let b_name = &b.replacement;

            // Exact match comes first
            if a_name == word && b_name != word {
                return std::cmp::Ordering::Less;
            }
            if b_name == word && a_name != word {
                return std::cmp::Ordering::Greater;
            }

            // Shorter names come first (more likely to be what user wants)
            let len_cmp = a_name.len().cmp(&b_name.len());
            if len_cmp != std::cmp::Ordering::Equal {
                return len_cmp;
            }

            // Alphabetical order
            a_name.cmp(b_name)
        });

        candidates
    }

    /// Get description for a function/operation
    fn get_function_description(&self, name: &str) -> String {
        match name {
            // Core collection operations
            "first" => "Returns first item in collection".to_string(),
            "last" => "Returns last item in collection".to_string(),
            "count" => "Returns number of items".to_string(),
            "length" => "Returns string length or collection size".to_string(),
            "empty" => "True if collection is empty".to_string(),
            "exists" => "True if collection is not empty".to_string(),
            "single" => "Returns single item (error if not exactly one)".to_string(),
            "distinct" => "Returns unique items".to_string(),

            // Lambda operations (evaluator-handled)
            "where" => "Filters collection by condition".to_string(),
            "select" => "Transforms each item using expression".to_string(),
            "all" => "True if all items satisfy condition".to_string(),
            "any" => "True if any item satisfies condition".to_string(),
            "repeat" => "Repeatedly applies expression".to_string(),
            "aggregate" => "Aggregates collection into single value".to_string(),
            "iif" => "Conditional expression (if-then-else)".to_string(),

            // String operations
            "substring" => "Extracts substring".to_string(),
            "contains" => "True if string contains substring".to_string(),
            "startsWith" => "True if string starts with prefix".to_string(),
            "endsWith" => "True if string ends with suffix".to_string(),
            "upper" => "Converts to uppercase".to_string(),
            "lower" => "Converts to lowercase".to_string(),
            "replace" => "Replaces substring with new value".to_string(),

            // Type operations
            "ofType" => "Filters by specific type".to_string(),
            "as" => "Casts to specific type".to_string(),
            "is" => "Checks if value is of type".to_string(),
            "toString" => "Converts to string".to_string(),
            "toInteger" => "Converts to integer".to_string(),

            // Collection operations
            "union" => "Combines collections, removes duplicates".to_string(),
            "intersect" => "Items that exist in both collections".to_string(),
            "exclude" => "Items in first but not second collection".to_string(),
            "skip" => "Skips first N items".to_string(),
            "take" => "Takes first N items".to_string(),

            // DateTime operations
            "today" => "Current date".to_string(),
            "now" => "Current date and time".to_string(),

            // Common FHIR properties
            "id" => "Resource identifier".to_string(),
            "meta" => "Resource metadata".to_string(),
            "resourceType" => "Type of FHIR resource".to_string(),
            "identifier" => "Business identifier".to_string(),
            "active" => "Whether record is active".to_string(),
            "name" => "Human name".to_string(),
            "telecom" => "Contact details".to_string(),
            "gender" => "Administrative gender".to_string(),
            "birthDate" => "Date of birth".to_string(),
            "address" => "Address information".to_string(),
            "status" => "Status of the resource".to_string(),
            "subject" => "Subject of the resource".to_string(),
            "code" => "Code/classification".to_string(),
            "value" => "Value of the element".to_string(),
            "text" => "Human readable text".to_string(),
            "extension" => "Additional content defined by implementations".to_string(),

            _ => String::new(), // No description available
        }
    }

    /// Get cached function names or return common FHIR properties as fallback
    fn get_cached_function_names(&self) -> Vec<String> {
        // Try to read from cache first
        let mut function_names = if let Ok(cache) = self.cached_functions.read() {
            if let Some(ref names) = *cache {
                names.clone()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // If no cached functions, try to get from registry
        if function_names.is_empty() {
            function_names.extend(self.get_functions_from_registry());
        }

        // Always add lambda functions since they're not in the registry
        function_names.extend(self.get_lambda_functions());

        // If still no functions, add fallback common FHIR properties and basic functions
        if function_names.is_empty() {
            function_names.extend(vec![
                // Common FHIR properties for property navigation
                "id".to_string(),
                "meta".to_string(),
                "text".to_string(),
                "identifier".to_string(),
                "active".to_string(),
                "name".to_string(),
                "telecom".to_string(),
                "gender".to_string(),
                "birthDate".to_string(),
                "address".to_string(),
                "resourceType".to_string(),
                // Basic functions that are commonly used
                "first".to_string(),
                "last".to_string(),
                "count".to_string(),
                "exists".to_string(),
                "empty".to_string(),
                "single".to_string(),
            ]);
        }

        // Remove duplicates and sort
        function_names.sort();
        function_names.dedup();
        function_names
    }

    /// Get lambda functions that are handled directly by the evaluator
    fn get_lambda_functions(&self) -> Vec<String> {
        vec![
            // Collection lambda functions
            "where".to_string(),
            "select".to_string(),
            "all".to_string(),
            "any".to_string(),
            "repeat".to_string(),
            // Aggregate lambda functions
            "aggregate".to_string(),
            // Conditional lambda functions
            "iif".to_string(),
        ]
    }

    /// Cache function names from registry (to be called when needed)
    pub fn cache_function_names(&self, function_names: Vec<String>) {
        if let Ok(mut cache) = self.cached_functions.write() {
            *cache = Some(function_names);
        }
    }

    /// Update the registry reference
    pub fn set_registry(&self, registry: Arc<FunctionRegistry>) {
        if let Ok(mut reg) = self.registry.write() {
            *reg = Some(registry);
        }
        // Clear function cache when registry changes
        if let Ok(mut cache) = self.cached_functions.write() {
            *cache = None;
        }
    }

    /// Get function names from registry if available
    fn get_functions_from_registry(&self) -> Vec<String> {
        if let Ok(registry_guard) = self.registry.read() {
            if let Some(ref registry) = *registry_guard {
                return registry.get_all_function_names();
            }
        }
        Vec::new()
    }

    /// Get resource types from model provider
    fn get_resource_types_from_provider(&self) -> Vec<String> {
        // Try cache first
        if let Ok(cache) = self.cached_resource_types.read() {
            if let Some(ref types) = *cache {
                return types.clone();
            }
        }

        // Get from model provider (this would be async in real implementation)
        let resource_types = vec![
            "Patient".to_string(),
            "Bundle".to_string(),
            "Observation".to_string(),
            "Condition".to_string(),
            "Organization".to_string(),
            "Practitioner".to_string(),
            "Encounter".to_string(),
            "Procedure".to_string(),
            "MedicationRequest".to_string(),
            "DiagnosticReport".to_string(),
            "AllergyIntolerance".to_string(),
        ];

        // Cache the result
        if let Ok(mut cache) = self.cached_resource_types.write() {
            *cache = Some(resource_types.clone());
        }

        resource_types
    }

    /// Complete FHIR properties with descriptions
    fn complete_properties(&self, word: &str, _context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Common FHIR properties with descriptions
        let properties = [
            ("id", "Resource identifier"),
            ("meta", "Resource metadata"),
            ("text", "Human readable text"),
            ("contained", "Contained resources"),
            ("extension", "Additional content"),
            ("modifierExtension", "Extensions that cannot be ignored"),
            ("identifier", "Business identifier"),
            ("active", "Whether record is active"),
            ("name", "Human name"),
            ("telecom", "Contact details"),
            ("gender", "Administrative gender"),
            ("birthDate", "Date of birth"),
            ("address", "Address information"),
            ("maritalStatus", "Marital status"),
            ("photo", "Image of the person"),
            ("contact", "Contact party"),
            ("communication", "Language preferences"),
            ("generalPractitioner", "Primary care provider"),
            ("managingOrganization", "Organization responsible"),
            ("resourceType", "Type of FHIR resource"),
            ("status", "Status of the resource"),
            ("category", "Classification"),
            ("code", "Code/classification"),
            ("subject", "Subject of the resource"),
            ("encounter", "Healthcare event"),
            ("effectiveDateTime", "When performed"),
            ("valueQuantity", "Quantity value"),
            ("valueCodeableConcept", "Coded value"),
            ("valueString", "String value"),
            ("component", "Component results"),
            ("system", "Identity of the terminology system"),
            ("value", "Contact point details"),
            ("use", "Purpose of contact point"),
            ("given", "Given names"),
            ("family", "Family name"),
            ("prefix", "Name prefix"),
            ("suffix", "Name suffix"),
            ("period", "Time period when active"),
            ("line", "Street address line"),
            ("city", "City name"),
            ("state", "State/Province"),
            ("postalCode", "Postal code"),
            ("country", "Country"),
        ];

        for (prop, desc) in properties {
            if prop.starts_with(word) {
                candidates.push(Pair {
                    display: format!("{prop} - {desc}"),
                    replacement: prop.to_string(),
                });
            }
        }

        candidates
    }

    /// Get context-aware suggestions based on what the user is typing
    fn get_context_suggestions(&self, word: &str, context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // Don't suggest anything for command contexts (after :load, :set, etc.)
        if self.is_command_context(context) {
            return candidates;
        }

        // Suggest common patterns based on context
        if context.ends_with(".where(") && word.is_empty() {
            let suggestions = [
                ("system = 'email'", "Filter by email system"),
                ("use = 'official'", "Filter by official use"),
                ("active = true", "Filter for active items"),
                ("exists()", "Filter for non-empty items"),
            ];

            for (suggestion, desc) in suggestions {
                candidates.push(Pair {
                    display: format!("{suggestion} - {desc}"),
                    replacement: suggestion.to_string(),
                });
            }
        }

        // Suggest operations after dot only when user starts typing and not in command context
        if context.ends_with('.') && !word.is_empty() && !self.is_after_command_word(context) {
            let common_ops = [
                ("first()", "Get first item"),
                ("last()", "Get last item"),
                ("count()", "Count items"),
                ("exists()", "Check if not empty"),
                ("where(...)", "Filter by condition"),
                ("select(...)", "Transform items"),
                ("empty()", "Check if empty"),
            ];

            for (op, desc) in common_ops {
                if op.starts_with(word) {
                    candidates.push(Pair {
                        display: format!("{op} - {desc}"),
                        replacement: op.to_string(),
                    });
                }
            }
        }

        // Suggest comparison operators only in appropriate contexts
        if self.is_expression_context(context)
            && context.contains("where(")
            && (word.is_empty() || word.ends_with(' '))
        {
            let operators = [
                ("=", "Equal to"),
                ("!=", "Not equal to"),
                (">=", "Greater than or equal"),
                ("<=", "Less than or equal"),
                (">", "Greater than"),
                ("<", "Less than"),
                ("and", "Logical AND"),
                ("or", "Logical OR"),
                ("contains", "String contains"),
                ("startsWith", "String starts with"),
            ];

            for (op, desc) in operators {
                if op.starts_with(word.trim()) {
                    candidates.push(Pair {
                        display: format!("{op} - {desc}"),
                        replacement: op.to_string(),
                    });
                }
            }
        }

        candidates
    }

    /// Check if we're in a command context where we shouldn't suggest FHIRPath expressions
    fn is_command_context(&self, context: &str) -> bool {
        // Special handling for :set command - allow expressions in value part
        if let Some(set_pos) = context.find(":set ") {
            let after_set = &context[set_pos + 5..];
            let parts: Vec<&str> = after_set.split_whitespace().collect();
            // If we have variable name and are on the value, allow expressions
            if parts.len() >= 2 {
                return false; // Allow expressions for the value part
            }
            return true; // Still in variable name part
        }

        // Check for other command patterns
        context.starts_with(":load ") ||
        context.starts_with(":unset ") ||
        context.starts_with(":help ") ||
        // :type and :explain should allow expressions as their arguments
        // Also check for partial command contexts
        (context.starts_with(':') && !context.contains(' ') && context.len() < 8)
    }

    /// Check if we're after a command word but before the expression part
    fn is_after_command_word(&self, context: &str) -> bool {
        // For :set command, we want to allow expressions after the variable name
        if let Some(set_pos) = context.find(":set ") {
            let after_set = &context[set_pos + 5..];
            let parts: Vec<&str> = after_set.split_whitespace().collect();
            // If we have the variable name and are typing the value, allow expressions
            return parts.len() < 2;
        }

        // For other commands, check if we're in a file path context
        context.starts_with(":load ") && !context.contains('.')
    }

    /// Check if we're in an expression context (not a command context)
    fn is_expression_context(&self, context: &str) -> bool {
        !self.is_command_context(context) && !context.starts_with(':')
    }

    /// Get command-specific completions
    fn get_command_specific_completions(&self, word: &str, context: &str) -> Vec<Pair> {
        let mut candidates = Vec::new();

        // For :set command, after variable name, allow expressions
        if let Some(set_pos) = context.find(":set ") {
            let after_set = &context[set_pos + 5..];
            let parts: Vec<&str> = after_set.split_whitespace().collect();

            if parts.len() >= 2 {
                // We're typing the value part - suggest common expression patterns
                let suggestions = [
                    ("Patient.name.first().given.first()", "First given name"),
                    (
                        "Patient.telecom.where(use='work').value",
                        "Work contact value",
                    ),
                    (
                        "Patient.telecom.where(system='email').value",
                        "Email address",
                    ),
                    ("Patient.active", "Patient active status"),
                    ("'simple string'", "String literal"),
                    ("today()", "Current date"),
                ];

                for (suggestion, desc) in suggestions {
                    if suggestion.starts_with(word) {
                        candidates.push(Pair {
                            display: format!("{suggestion} - {desc}"),
                            replacement: suggestion.to_string(),
                        });
                    }
                }
            }
        }

        // For :load command, suggest file extensions
        if context.starts_with(":load ") && !word.is_empty() {
            if word.ends_with('.') {
                candidates.push(Pair {
                    display: "json - JSON file".to_string(),
                    replacement: "json".to_string(),
                });
            }
        }

        // For :help command, suggest function names
        if context.starts_with(":help ") {
            let function_names = self.get_cached_function_names();
            for name in function_names {
                if name.starts_with(word) {
                    candidates.push(Pair {
                        display: format!("{name} - Function help"),
                        replacement: name,
                    });
                }
            }
        }

        candidates
    }
}

impl Completer for FhirPathCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context) -> RlResult<(usize, Vec<Pair>)> {
        let line = &line[..pos];

        // Find the word being completed
        let (start, word) = if let Some(last_space) = line.rfind(' ') {
            (last_space + 1, &line[last_space + 1..])
        } else {
            (0, line)
        };

        let mut candidates = Vec::new();

        // Complete commands (starting with :)
        if word.starts_with(':') {
            if word == ":" {
                // User typed just ':' - show all commands, start replacement after the colon
                let colon_start = start + 1; // Position after the ':'
                candidates.extend(
                    self.commands
                        .iter()
                        .map(|cmd| Pair {
                            display: cmd.clone(),
                            replacement: cmd[1..].to_string(), // Remove the ':' from replacement since it's already typed
                        })
                        .collect::<Vec<_>>(),
                );
                return Ok((colon_start, candidates));
            } else {
                // User typed partial command like ':l' or ':load' - replace from after the colon
                let colon_start = start + 1; // Position after the ':'
                candidates.extend(
                    self.commands
                        .iter()
                        .filter(|cmd| cmd.starts_with(word))
                        .map(|cmd| Pair {
                            display: cmd.clone(),
                            replacement: cmd[1..].to_string(), // Always remove the ':' prefix
                        })
                        .collect::<Vec<_>>(),
                );
                return Ok((colon_start, candidates));
            }
        } else if word.is_empty() && line.trim_end().ends_with(':') {
            // Handle edge case where word parsing might miss the colon
            candidates.extend(
                self.commands
                    .iter()
                    .map(|cmd| Pair {
                        display: cmd.clone(),
                        replacement: cmd[1..].to_string(),
                    })
                    .collect::<Vec<_>>(),
            );
            return Ok((pos, candidates)); // Start after the current position
        } else if !self.is_command_context(line) {
            // Only provide FHIRPath completions when not in command context

            // Complete function names with enhanced descriptions
            candidates.extend(self.complete_function(word, line));

            // Add property completions with descriptions based on context
            if !word.is_empty() {
                candidates.extend(self.complete_properties(word, line));
            }

            // Add context-aware suggestions only when user is actively typing
            if line.contains('.') && !word.is_empty() {
                candidates.extend(self.get_context_suggestions(word, line));
            }
        } else {
            // In command context - provide context-specific completions
            candidates.extend(self.get_command_specific_completions(word, line));
        }

        Ok((start, candidates))
    }
}

impl Hinter for FhirPathCompleter {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context) -> Option<Self::Hint> {
        if line.len() < pos {
            return None;
        }

        let line = &line[..pos];

        // Enhanced command hints
        if line == ":" {
            return Some(
                "load <file> | set <name> <value> | vars | help | quit | type | explain"
                    .to_string(),
            );
        }

        // Partial command hints with examples
        match line {
            ":l" | ":lo" | ":loa" => Some("oad patient.json".to_string()),
            ":s" | ":se" => Some("et myVar 'value' or Patient.name.first()".to_string()),
            ":h" | ":he" | ":hel" => Some("elp [operation]".to_string()),
            ":t" | ":ty" | ":typ" => Some("ype Patient.name".to_string()),
            ":e" | ":ex" | ":exp" | ":expl" | ":expla" | ":explai" => {
                Some("xplain Patient.telecom.where(system='email')".to_string())
            }
            ":q" | ":qu" | ":qui" => Some("uit".to_string()),
            ":r" | ":re" | ":res" | ":reso" | ":resou" | ":resour" | ":resourc" => {
                Some("esource".to_string())
            }
            ":v" | ":va" | ":var" => Some("ars".to_string()),
            ":u" | ":un" | ":uns" | ":unse" => Some("nset varName".to_string()),
            _ => {
                if self.is_command_context(line) {
                    None // Don't provide expression hints in command context
                } else {
                    self.get_expression_hint(line)
                }
            }
        }
    }
}

impl FhirPathCompleter {
    /// Get intelligent hints for FHIRPath expressions
    fn get_expression_hint(&self, line: &str) -> Option<String> {
        // Hint for dot operations
        if line.ends_with(".") {
            return Some(
                "first() | count() | where(condition) | select(expression) | exists()".to_string(),
            );
        }

        // Hint for where clauses
        if line.ends_with(".where(") {
            return Some("system = 'email' | use = 'official' | active = true)".to_string());
        }

        // Hint for select clauses
        if line.ends_with(".select(") {
            return Some("given.first() | value | id)".to_string());
        }

        // Hint for string operations
        if line.contains("'") && !line.ends_with("'") {
            return Some("' (close string)".to_string());
        }

        // Hint for comparison operators
        if line.contains(" ") && !line.contains("=") && !line.contains(">") && !line.contains("<") {
            let words: Vec<&str> = line.split_whitespace().collect();
            if !words.is_empty() && !words.last().unwrap().starts_with(':') {
                return Some("= | != | > | < | >= | <= | contains".to_string());
            }
        }

        // Only suggest resource types when user starts typing something that looks like a resource
        if !line.is_empty() && line.len() > 2 && !line.contains('.') && !line.starts_with(':') {
            let common_resources = [
                "Patient",
                "Bundle",
                "Observation",
                "Condition",
                "Organization",
            ];
            for resource in common_resources {
                if resource.to_lowercase().starts_with(&line.to_lowercase()) {
                    return Some(format!(
                        ".name | .id | .<property> (for {resource} resource)"
                    ));
                }
            }
        }

        // Hint for resource properties
        if line == "Patient" {
            return Some(".name | .telecom | .identifier | .active | .gender".to_string());
        }

        if line == "Bundle" {
            return Some(".entry | .total | .type | .timestamp".to_string());
        }

        if line == "Observation" {
            return Some(".code | .value | .status | .subject | .effectiveDateTime".to_string());
        }

        // No specific hint
        None
    }

    /// Highlight REPL commands
    fn highlight_command<'l>(&self, line: &'l str) -> std::borrow::Cow<'l, str> {
        if !line.trim_start().starts_with(':') {
            return std::borrow::Cow::Borrowed(line);
        }

        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut current_word = String::new();
        let mut in_command = false;

        while let Some(ch) = chars.next() {
            if ch == ':' && !in_command {
                // Start of command - color it cyan
                result.push_str("\x1b[36m:"); // Cyan
                in_command = true;
                current_word.clear();
            } else if in_command && (ch.is_whitespace() || chars.peek().is_none()) {
                // End of command word
                if !ch.is_whitespace() {
                    current_word.push(ch);
                }

                // Color known commands differently
                if self
                    .commands
                    .iter()
                    .any(|cmd| cmd == &format!(":{}", current_word))
                {
                    result.push_str(&format!("\x1b[1;36m{}\x1b[0m", current_word)); // Bold cyan
                } else {
                    result.push_str(&format!("\x1b[36m{}\x1b[0m", current_word)); // Regular cyan
                }

                if ch.is_whitespace() {
                    result.push(ch);
                }
                in_command = false;
                current_word.clear();
            } else if in_command {
                current_word.push(ch);
            } else {
                // Regular text after command
                result.push(ch);
            }
        }

        // Handle case where command is at end of line
        if in_command && !current_word.is_empty() {
            if self
                .commands
                .iter()
                .any(|cmd| cmd == &format!(":{}", current_word))
            {
                result.push_str(&format!("\x1b[1;36m{}\x1b[0m", current_word));
            } else {
                result.push_str(&format!("\x1b[36m{}\x1b[0m", current_word));
            }
        }

        std::borrow::Cow::Owned(result)
    }

    /// Highlight FHIRPath expressions with syntax coloring
    fn highlight_fhirpath<'l>(&self, line: &'l str) -> std::borrow::Cow<'l, str> {
        if line.trim().is_empty() {
            return std::borrow::Cow::Borrowed(line);
        }

        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut current_token = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                // String literals
                '\'' => {
                    if !current_token.is_empty() {
                        self.append_highlighted_token(&mut result, &current_token);
                        current_token.clear();
                    }

                    result.push_str("\x1b[32m'"); // Green for strings

                    // Read until closing quote or end of line
                    let mut string_content = String::new();
                    let mut escaped = false;

                    while let Some(inner_ch) = chars.next() {
                        if escaped {
                            string_content.push(inner_ch);
                            escaped = false;
                        } else if inner_ch == '\\' {
                            string_content.push(inner_ch);
                            escaped = true;
                        } else if inner_ch == '\'' {
                            string_content.push(inner_ch);
                            break;
                        } else {
                            string_content.push(inner_ch);
                        }
                    }

                    result.push_str(&string_content);
                    result.push_str("\x1b[0m"); // Reset color
                }

                // Operators and punctuation
                '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' => {
                    if !current_token.is_empty() {
                        self.append_highlighted_token(&mut result, &current_token);
                        current_token.clear();
                    }

                    // Look ahead for multi-character operators
                    let mut operator = String::from(ch);
                    if let Some(&next_ch) = chars.peek() {
                        match (ch, next_ch) {
                            ('=', '=') | ('!', '=') | ('>', '=') | ('<', '=') => {
                                operator.push(chars.next().unwrap());
                            }
                            _ => {}
                        }
                    }

                    result.push_str(&format!("\x1b[33m{}\x1b[0m", operator)); // Yellow for operators
                }

                // Parentheses and brackets
                '(' | ')' | '[' | ']' => {
                    if !current_token.is_empty() {
                        self.append_highlighted_token(&mut result, &current_token);
                        current_token.clear();
                    }
                    result.push_str(&format!("\x1b[37m{}\x1b[0m", ch)); // White for brackets
                }

                // Dot notation
                '.' => {
                    if !current_token.is_empty() {
                        self.append_highlighted_token(&mut result, &current_token);
                        current_token.clear();
                    }
                    result.push_str(&format!("\x1b[37m{}\x1b[0m", ch)); // White for dots
                }

                // Comma
                ',' => {
                    if !current_token.is_empty() {
                        self.append_highlighted_token(&mut result, &current_token);
                        current_token.clear();
                    }
                    result.push_str(&format!("\x1b[37m{}\x1b[0m", ch)); // White for commas
                }

                // Whitespace
                ch if ch.is_whitespace() => {
                    if !current_token.is_empty() {
                        self.append_highlighted_token(&mut result, &current_token);
                        current_token.clear();
                    }
                    result.push(ch);
                }

                // Regular characters - accumulate into token
                _ => {
                    current_token.push(ch);
                }
            }
        }

        // Handle final token
        if !current_token.is_empty() {
            self.append_highlighted_token(&mut result, &current_token);
        }

        std::borrow::Cow::Owned(result)
    }

    /// Helper to append a highlighted token based on its type
    fn append_highlighted_token(&self, result: &mut String, token: &str) {
        // Check if it's a number
        if token.parse::<f64>().is_ok() || token.parse::<i64>().is_ok() {
            result.push_str(&format!("\x1b[35m{}\x1b[0m", token)); // Magenta for numbers
            return;
        }

        // Check if it's a boolean
        if matches!(token, "true" | "false") {
            result.push_str(&format!("\x1b[35m{}\x1b[0m", token)); // Magenta for booleans
            return;
        }

        // Check if it's a logical operator/keyword
        if matches!(
            token,
            "and" | "or" | "xor" | "implies" | "mod" | "div" | "in" | "contains"
        ) {
            result.push_str(&format!("\x1b[33m{}\x1b[0m", token)); // Yellow for keywords/operators
            return;
        }

        // Check if it's a function from registry or common functions
        let function_names = self.get_cached_function_names();
        if function_names.iter().any(|f| f == token) {
            result.push_str(&format!("\x1b[34m{}\x1b[0m", token)); // Blue for functions
            return;
        }

        // Check if it's a FHIR resource type from model provider
        let resource_types = self.get_resource_types_from_provider();
        if resource_types.iter().any(|r| r == token) {
            result.push_str(&format!("\x1b[1;32m{}\x1b[0m", token)); // Bold green for resource types
            return;
        }

        // Check if it starts with uppercase (likely a resource type or property)
        if token.chars().next().map_or(false, |c| c.is_uppercase()) {
            result.push_str(&format!("\x1b[32m{}\x1b[0m", token)); // Green for properties/types
            return;
        }

        // Default: no highlighting
        result.push_str(token);
    }
}

impl Highlighter for FhirPathCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        // Skip highlighting for commands
        if line.trim_start().starts_with(':') {
            return self.highlight_command(line);
        }

        self.highlight_fhirpath(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        // Return true to trigger highlighting on most actions for responsive syntax coloring
        matches!(_kind, CmdKind::MoveCursor | CmdKind::Other)
    }
}

impl Validator for FhirPathCompleter {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> RlResult<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

impl Helper for FhirPathCompleter {}
