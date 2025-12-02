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

//! Pretty output formatter with colors and symbols

use super::{AnalysisOutput, EvaluationOutput, FormatError, OutputFormatter, ParseOutput};
use colored::Colorize;
use octofhir_fhirpath::FhirPathValue;

pub struct PrettyFormatter {
    colored: bool,
}

impl PrettyFormatter {
    pub fn new(colored: bool) -> Self {
        Self { colored }
    }

    fn colorize(&self, text: &str, color: colored::Color) -> String {
        if self.colored {
            text.color(color).to_string()
        } else {
            text.to_string()
        }
    }

    fn success_icon(&self) -> String {
        if self.colored {
            "✅".to_string()
        } else {
            "[OK]".to_string()
        }
    }

    fn error_icon(&self) -> String {
        if self.colored {
            "❌".to_string()
        } else {
            "[ERROR]".to_string()
        }
    }

    fn warning_icon(&self) -> String {
        if self.colored {
            "⚠️".to_string()
        } else {
            "[WARNING]".to_string()
        }
    }

    fn info_icon(&self) -> String {
        if self.colored {
            "ℹ️".to_string()
        } else {
            "[INFO]".to_string()
        }
    }
}

impl OutputFormatter for PrettyFormatter {
    fn format_evaluation(&self, output: &EvaluationOutput) -> Result<String, FormatError> {
        let mut result = String::new();

        if output.success {
            result.push_str(&format!(
                "{} Expression: {}\n",
                self.success_icon(),
                self.colorize(&output.expression, colored::Color::Blue)
            ));

            if let Some(ref collection_with_metadata) = output.result_with_metadata {
                // Use rich metadata when available
                let results = collection_with_metadata.results();
                result.push_str(&format!(
                    "🎯 Results ({} items):\n",
                    self.colorize(&results.len().to_string(), colored::Color::Yellow)
                ));
                for (i, result_metadata) in results.iter().enumerate().take(10) {
                    // Use the expected return type if available, otherwise use the type name
                    let type_name = result_metadata
                        .type_info
                        .expected_return_type
                        .as_ref()
                        .unwrap_or(&result_metadata.type_info.type_name);

                    let value_str = format_fhir_value_pretty(&result_metadata.value);

                    // Show additional type information when available
                    let type_display = if let Some(namespace) = &result_metadata.type_info.namespace
                    {
                        format!("{namespace}.{type_name}")
                    } else {
                        type_name.clone()
                    };

                    result.push_str(&format!(
                        "   [{}] {}: {}\n",
                        self.colorize(&i.to_string(), colored::Color::Cyan),
                        self.colorize(&type_display, colored::Color::Green),
                        value_str
                    ));
                }
                if results.len() > 10 {
                    result.push_str(&format!(
                        "   ... and {} more items\n",
                        self.colorize(&(results.len() - 10).to_string(), colored::Color::Yellow)
                    ));
                }
            } else if let Some(ref collection) = output.result {
                // Fall back to basic formatting for backward compatibility
                let values: Vec<&FhirPathValue> = collection.iter().collect();
                result.push_str(&format!(
                    "🎯 Results ({} items):\n",
                    self.colorize(&values.len().to_string(), colored::Color::Yellow)
                ));
                for (i, item) in values.iter().enumerate().take(10) {
                    let type_name = get_fhir_type_name(item);
                    let value_str = format_fhir_value_pretty(item);
                    result.push_str(&format!(
                        "   [{}] {}: {}\n",
                        self.colorize(&i.to_string(), colored::Color::Cyan),
                        self.colorize(&type_name, colored::Color::Green),
                        value_str
                    ));
                }
                if values.len() > 10 {
                    result.push_str(&format!(
                        "   ... and {} more items\n",
                        self.colorize(&(values.len() - 10).to_string(), colored::Color::Yellow)
                    ));
                }
            } else {
                result.push_str(&format!(
                    "🎯 Result: {}\n",
                    self.colorize("null", colored::Color::Red)
                ));
            }

            // Add performance info
            let exec_time = output.execution_time.as_secs_f64() * 1000.0;
            result.push_str(&format!(
                "⏱️  Execution: {}ms",
                self.colorize(&format!("{exec_time:.1}"), colored::Color::Magenta)
            ));

            if output.metadata.cache_hits > 0 {
                result.push_str(&format!(
                    " | 🎯 Cache hits: {}",
                    self.colorize(
                        &output.metadata.cache_hits.to_string(),
                        colored::Color::Cyan
                    )
                ));
            }
            result.push('\n');
        } else if output.error.is_some() {
            // Error details are already shown via diagnostic handler to stderr
            // No need to duplicate the error message here
        }

        Ok(result)
    }

    fn format_parse(&self, output: &ParseOutput) -> Result<String, FormatError> {
        {
            let mut result = String::new();

            if output.success {
                result.push_str(&format!("{} Parse successful\n", self.success_icon()));
                result.push_str(&format!(
                    "📝 Expression: {}\n",
                    self.colorize(&output.expression, colored::Color::Blue)
                ));
                if let Some(ref ast) = output.ast {
                    result.push_str(&format!("🌳 AST: {ast:?}\n"));
                }
                result.push_str(&format!(
                    "📊 AST nodes: {}\n",
                    self.colorize(&output.metadata.ast_nodes.to_string(), colored::Color::Cyan)
                ));
            } else if output.error.is_some() {
                result.push_str(&format!("{} Parse failed\n", self.error_icon()));
                // Error details are already shown via diagnostic handler to stderr
                // No need to duplicate the error message here
            }

            Ok(result)
        }
    }

    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError> {
        {
            let mut result = String::new();

            if !output.validation_errors.is_empty() {
                result.push_str(&format!("{} Validation failed\n", self.error_icon()));
                result.push_str("🔍 Validation Errors:\n");

                for error in &output.validation_errors {
                    let icon = "❗";
                    result.push_str(&format!(
                        "  {} {}\n",
                        icon,
                        self.colorize(&error.message, colored::Color::Red)
                    ));
                    if !error.suggestions.is_empty() {
                        result.push_str(&format!(
                            "    💡 Suggestions: {}\n",
                            self.colorize(&error.suggestions.join(", "), colored::Color::Yellow)
                        ));
                    }
                }
                return Ok(result);
            }

            if output.success {
                if let Some(ref analysis) = output.analysis {
                    result.push_str(&format!("{} Analysis complete\n", self.success_icon()));
                    result.push_str(&format!(
                        "📊 Expression: {}\n",
                        self.colorize(&output.expression, colored::Color::Blue)
                    ));

                    if !analysis.type_annotations.is_empty() {
                        result.push_str("🔍 Type Annotations:\n");
                        for (node_id, semantic_info) in &analysis.type_annotations {
                            result.push_str(&format!(
                                "  📍 Node {}: ",
                                self.colorize(&node_id.to_string(), colored::Color::Cyan)
                            ));
                            if let Some(ref fhir_type) = semantic_info.fhir_path_type {
                                result.push_str(&format!(
                                    "{} ",
                                    self.colorize(fhir_type, colored::Color::Green)
                                ));
                            }
                            if let Some(ref model_type) = semantic_info.model_type {
                                result.push_str(&format!(
                                    "({})",
                                    self.colorize(model_type, colored::Color::Magenta)
                                ));
                            }
                            result.push('\n');
                        }
                    }

                    if !analysis.function_calls.is_empty() {
                        result.push_str("🔧 Function Calls:\n");
                        for func_analysis in &analysis.function_calls {
                            result.push_str(&format!(
                                "  🔹 {} ({})\n",
                                self.colorize(&func_analysis.function_name, colored::Color::Blue),
                                self.colorize(
                                    &func_analysis.signature.description,
                                    colored::Color::Yellow
                                )
                            ));
                            for error in &func_analysis.validation_errors {
                                result.push_str(&format!(
                                    "    {} {}\n",
                                    self.warning_icon(),
                                    self.colorize(&error.message, colored::Color::Red)
                                ));
                            }
                        }
                    }

                    result.push_str(&format!("\n{} Analysis successful\n", self.success_icon()));
                } else {
                    result.push_str(&format!(
                        "{} Analysis complete (no analyzer)\n",
                        self.info_icon()
                    ));
                }
            } else if output.error.is_some() {
                result.push_str(&format!("{} Analysis failed\n", self.error_icon()));
                // Error details are already shown via diagnostic handler to stderr
                // No need to duplicate the error message here
            }

            Ok(result)
        }
    }
}

fn get_fhir_type_name(value: &FhirPathValue) -> String {
    // Prefer the embedded TypeInfo's type_name for all variants, and include namespace when available
    let type_name = value.display_type_name();
    match value {
        FhirPathValue::Boolean(_, type_info, _)
        | FhirPathValue::Integer(_, type_info, _)
        | FhirPathValue::Decimal(_, type_info, _)
        | FhirPathValue::String(_, type_info, _)
        | FhirPathValue::Date(_, type_info, _)
        | FhirPathValue::DateTime(_, type_info, _)
        | FhirPathValue::Time(_, type_info, _) => {
            if let Some(ns) = &type_info.namespace {
                format!("{ns}.{type_name}")
            } else {
                type_name
            }
        }
        FhirPathValue::Quantity { type_info, .. } => {
            let name = type_info.name.as_deref().unwrap_or(&type_info.type_name);
            if let Some(ns) = &type_info.namespace {
                format!("{ns}.{name}")
            } else {
                name.to_string()
            }
        }
        FhirPathValue::Resource(_json, type_info, _) => {
            let name = type_info.name.as_deref().unwrap_or(&type_info.type_name);
            if let Some(ns) = &type_info.namespace {
                format!("{ns}.{name}")
            } else {
                name.to_string()
            }
        }
        FhirPathValue::Collection(_) => "Collection".to_string(),
        FhirPathValue::Empty => "Empty".to_string(),
    }
}

fn format_fhir_value_pretty(value: &FhirPathValue) -> String {
    match value {
        FhirPathValue::String(s, _, _) => format!("\"{s}\""),
        FhirPathValue::Integer(i, _, _) => i.to_string(),
        FhirPathValue::Decimal(d, _, _) => d.to_string(),
        FhirPathValue::Boolean(b, _, _) => b.to_string(),
        other => {
            match serde_json::to_string(other) {
                Ok(json) => {
                    // Limit display length for readability
                    if json.len() > 50 {
                        format!("{}...", &json[..47])
                    } else {
                        json
                    }
                }
                Err(_) => format!("{other:?}"),
            }
        }
    }
}
