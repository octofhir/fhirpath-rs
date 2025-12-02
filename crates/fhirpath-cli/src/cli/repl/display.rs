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

//! Display formatting for REPL output

use anyhow::Error;
use colored::Colorize;
use octofhir_fhirpath::FhirPathValue;
use octofhir_fhirpath::core::JsonValueExt;

/// Handles formatting of REPL output
pub struct DisplayFormatter {
    use_colors: bool,
}

impl DisplayFormatter {
    /// Create a new display formatter
    pub fn new(use_colors: bool) -> Self {
        Self { use_colors }
    }

    /// Format a FHIRPath evaluation result
    pub fn format_result(&self, result: &FhirPathValue, show_types: bool) -> String {
        match result {
            FhirPathValue::Collection(collection) => {
                if collection.is_empty() {
                    self.format_empty_collection()
                } else if collection.len() == 1 {
                    self.format_single_value(collection.iter().next().unwrap(), show_types)
                } else {
                    self.format_collection_items(collection.values(), show_types)
                }
            }
            FhirPathValue::Empty => self.format_empty_collection(),
            single => self.format_single_value(single, show_types),
        }
    }

    /// Format a single FHIRPath value
    pub fn format_value(&self, value: &FhirPathValue, show_types: bool) -> String {
        self.format_single_value(value, show_types)
    }

    /// Format an error message
    pub fn format_error(&self, error: &Error) -> String {
        let error_message = error.to_string();
        let suggestion = self.get_error_suggestion(&error_message);

        if self.use_colors {
            if suggestion.is_empty() {
                format!("🚨 {}: {}", self.red("Error"), error_message)
            } else {
                format!(
                    "🚨 {}: {}\n💡 {}: {}",
                    self.red("Error"),
                    error_message,
                    self.cyan("Suggestion"),
                    suggestion
                )
            }
        } else if suggestion.is_empty() {
            format!("Error: {error_message}")
        } else {
            format!("Error: {error_message}\nSuggestion: {suggestion}")
        }
    }

    /// Get helpful suggestions based on error messages
    fn get_error_suggestion(&self, error_message: &str) -> String {
        let error_lower = error_message.to_lowercase();

        // Common FHIRPath syntax errors
        if error_lower.contains("expected") && error_lower.contains("')'") {
            return "Check for missing closing parenthesis ')'. Try using ':help' for syntax guidance.".to_string();
        }

        if error_lower.contains("unexpected") && error_lower.contains("token") {
            return "Check your expression syntax. Common issues: missing quotes for strings, incorrect operators.".to_string();
        }

        if error_lower.contains("parse") {
            return "Syntax error in expression. Try ':help' for FHIRPath syntax examples."
                .to_string();
        }

        // Function/operation errors
        if error_lower.contains("function") && error_lower.contains("not found") {
            return "Function not recognized. Use ':help' to see available operations or check spelling.".to_string();
        }

        if error_lower.contains("cannot resolve") {
            return "Property or path not found. Check the resource structure or try ':type <expression>' for type info.".to_string();
        }

        // Type errors
        if error_lower.contains("type") && error_lower.contains("mismatch") {
            return "Type mismatch. Use '.ofType()' or '.as()' for type casting, or check expected types.".to_string();
        }

        // Collection errors
        if error_lower.contains("single") && error_lower.contains("multiple") {
            return "Expression returned multiple values. Use '.first()', '.last()', or add filters with '.where()'.".to_string();
        }

        if error_lower.contains("empty") && error_lower.contains("collection") {
            return "Empty result. Check if the resource has the expected properties or use '.exists()' to verify.".to_string();
        }

        // Resource/model errors
        if error_lower.contains("no resource") {
            return "No resource loaded. Use ':load <file>' to load a FHIR resource first."
                .to_string();
        }

        if error_lower.contains("model provider") {
            return "Model provider issue. Try restarting the REPL or check your FHIR version setting.".to_string();
        }

        // Variable errors
        if error_lower.contains("variable") && error_lower.contains("not defined") {
            return "Variable not found. Use ':set <name> <value>' to define variables or ':vars' to list them.".to_string();
        }

        // File/IO errors
        if error_lower.contains("file") && error_lower.contains("not found") {
            return "File not found. Check the file path and ensure the file exists.".to_string();
        }

        if error_lower.contains("json") {
            return "JSON parsing error. Ensure the file contains valid JSON and is a FHIR resource.".to_string();
        }

        // Network/timeout errors
        if error_lower.contains("timeout") {
            return "Operation timed out. Try simpler expressions or check your network connection.".to_string();
        }

        // Generic help for unknown errors
        if !error_lower.is_empty() {
            return "Try ':help' for general assistance or ':explain <expression>' to understand expression evaluation.".to_string();
        }

        String::new()
    }

    fn format_empty_collection(&self) -> String {
        if self.use_colors {
            self.dim("{}")
        } else {
            "{}".to_string()
        }
    }

    fn format_single_value(&self, value: &FhirPathValue, show_types: bool) -> String {
        let value_str = match value {
            FhirPathValue::String(s, _, _) => {
                if self.use_colors {
                    self.green(&format!("\"{s}\""))
                } else {
                    format!("\"{s}\"")
                }
            }
            FhirPathValue::Integer(i, _, _) => {
                if self.use_colors {
                    self.blue(&i.to_string())
                } else {
                    i.to_string()
                }
            }
            FhirPathValue::Decimal(d, _, _) => {
                if self.use_colors {
                    self.blue(&d.to_string())
                } else {
                    d.to_string()
                }
            }
            FhirPathValue::Boolean(b, _, _) => {
                let bool_str = b.to_string();
                if self.use_colors {
                    self.yellow(&bool_str)
                } else {
                    bool_str
                }
            }
            FhirPathValue::Date(d, _, _) => {
                if self.use_colors {
                    self.magenta(&d.to_string())
                } else {
                    d.to_string()
                }
            }
            FhirPathValue::DateTime(dt, _, _) => {
                if self.use_colors {
                    self.magenta(&dt.to_string())
                } else {
                    dt.to_string()
                }
            }
            FhirPathValue::Time(t, _, _) => {
                if self.use_colors {
                    self.magenta(&t.to_string())
                } else {
                    t.to_string()
                }
            }
            FhirPathValue::Quantity { value, .. } => {
                if self.use_colors {
                    self.cyan(&value.to_string())
                } else {
                    value.to_string()
                }
            }
            FhirPathValue::Collection(_) => {
                // This shouldn't happen for single values, but handle it
                "Collection".to_string()
            }
            FhirPathValue::Resource(json_value, _, _) => {
                // Format JSON values (both JsonValue and Resource are now consolidated into Json)
                match json_value.as_inner() {
                    serde_json::Value::String(s) => {
                        if self.use_colors {
                            self.green(&format!("\"{s}\""))
                        } else {
                            format!("\"{s}\"")
                        }
                    }
                    serde_json::Value::Number(n) => {
                        if self.use_colors {
                            self.blue(&n.to_string())
                        } else {
                            n.to_string()
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        let bool_str = b.to_string();
                        if self.use_colors {
                            self.yellow(&bool_str)
                        } else {
                            bool_str
                        }
                    }
                    serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                        // For complex JSON objects (including FHIR resources), show formatted JSON
                        if let Ok(pretty) = serde_json::to_string_pretty(json_value.as_inner()) {
                            if self.use_colors {
                                self.cyan(&pretty)
                            } else {
                                pretty
                            }
                        } else {
                            json_value.as_inner().to_string()
                        }
                    }
                    serde_json::Value::Null => {
                        if self.use_colors {
                            self.dim("null")
                        } else {
                            "null".to_string()
                        }
                    }
                }
            }
            _ => {
                // Handle other value types with clean display
                format!("{value:?}")
            }
        };

        if show_types {
            let type_name = value.type_name();
            if self.use_colors {
                format!("{value_str}: {}", self.dim(type_name))
            } else {
                format!("{value_str}: {type_name}")
            }
        } else {
            value_str
        }
    }

    fn format_collection_items(&self, collection: &[FhirPathValue], show_types: bool) -> String {
        let items_str: Vec<String> = collection
            .iter()
            .map(|item| self.format_single_value(item, false))
            .collect();

        let result = if collection.len() <= 5 {
            format!("[{}]", items_str.join(", "))
        } else {
            // Show first few items and count
            let displayed = items_str.iter().take(3).cloned().collect::<Vec<_>>();
            format!(
                "[{}, ... ({} items)]",
                displayed.join(", "),
                collection.len()
            )
        };

        if show_types {
            let type_name = if collection.is_empty() {
                "Collection<Any>".to_string()
            } else {
                let first_item = collection.iter().next().unwrap();
                let item_type = first_item.type_name();
                // Check if all items have the same type
                let all_same = collection.iter().all(|item| item.type_name() == item_type);
                if all_same {
                    format!("Collection<{item_type}>")
                } else {
                    "Collection<Any>".to_string()
                }
            };

            if self.use_colors {
                format!("{}: {}", result, self.dim(&type_name))
            } else {
                format!("{result}: {type_name}")
            }
        } else {
            result
        }
    }

    // Color helper methods
    fn red(&self, text: &str) -> String {
        if self.use_colors {
            text.red().to_string()
        } else {
            text.to_string()
        }
    }

    fn green(&self, text: &str) -> String {
        if self.use_colors {
            text.green().to_string()
        } else {
            text.to_string()
        }
    }

    fn yellow(&self, text: &str) -> String {
        if self.use_colors {
            text.yellow().to_string()
        } else {
            text.to_string()
        }
    }

    fn blue(&self, text: &str) -> String {
        if self.use_colors {
            format!("\x1b[34m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn magenta(&self, text: &str) -> String {
        if self.use_colors {
            format!("\x1b[35m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn cyan(&self, text: &str) -> String {
        if self.use_colors {
            text.cyan().to_string()
        } else {
            text.to_string()
        }
    }

    fn dim(&self, text: &str) -> String {
        if self.use_colors {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }
}
