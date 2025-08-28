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
use colored::*;
use octofhir_fhirpath_model::FhirPathValue;

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

            if let Some(ref value) = output.result {
                match value {
                    FhirPathValue::Collection(values) => {
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
                                self.colorize(
                                    &(values.len() - 10).to_string(),
                                    colored::Color::Yellow
                                )
                            ));
                        }
                    }
                    single_value => {
                        result.push_str("🎯 Result:\n");
                        let type_name = get_fhir_type_name(single_value);
                        let value_str = format_fhir_value_pretty(single_value);
                        result.push_str(&format!(
                            "   {}: {}\n",
                            self.colorize(&type_name, colored::Color::Green),
                            value_str
                        ));
                    }
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
        } else if let Some(ref error) = output.error {
            result.push_str(&format!(
                "{} Expression: {}\n",
                self.error_icon(),
                self.colorize(&output.expression, colored::Color::Blue)
            ));
            result.push_str(&format!(
                "💥 Error: {}\n",
                self.colorize(&error.to_string(), colored::Color::Red)
            ));
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
            } else if let Some(ref error) = output.error {
                result.push_str(&format!("{} Parse failed\n", self.error_icon()));
                result.push_str(&format!(
                    "📝 Expression: {}\n",
                    self.colorize(&output.expression, colored::Color::Blue)
                ));
                result.push_str(&format!(
                    "💥 Error: {}\n",
                    self.colorize(&error.to_string(), colored::Color::Red)
                ));
            }

            Ok(result)
        }
    }

    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError> {
        {
            let mut result = String::new();

            if !output.validation_errors.is_empty() {
                result.push_str(&format!("{} Validation failed\n", self.error_icon()));
                result.push_str(&format!(
                    "📝 Expression: {}\n",
                    self.colorize(&output.expression, colored::Color::Blue)
                ));
                result.push_str("🔍 Validation Errors:\n");

                for error in &output.validation_errors {
                    let icon = match error.error_type {
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidField => "🔍",
                        octofhir_fhirpath_analyzer::ValidationErrorType::DeprecatedField => "⚠️",
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidResourceType => {
                            "🏥"
                        }
                        octofhir_fhirpath_analyzer::ValidationErrorType::InvalidFunction => "🔧",
                        _ => "❗",
                    };
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
            } else if let Some(ref error) = output.error {
                result.push_str(&format!("{} Analysis failed\n", self.error_icon()));
                result.push_str(&format!(
                    "💥 Error: {}\n",
                    self.colorize(&error.to_string(), colored::Color::Red)
                ));
            }

            Ok(result)
        }
    }
}

fn get_fhir_type_name(value: &FhirPathValue) -> String {
    match value {
        FhirPathValue::String(_) => "String".to_string(),
        FhirPathValue::Integer(_) => "Integer".to_string(),
        FhirPathValue::Decimal(_) => "Decimal".to_string(),
        FhirPathValue::Boolean(_) => "Boolean".to_string(),
        FhirPathValue::Date(_) => "Date".to_string(),
        FhirPathValue::DateTime(_) => "DateTime".to_string(),
        FhirPathValue::Time(_) => "Time".to_string(),
        FhirPathValue::Quantity(_) => "Quantity".to_string(),
        FhirPathValue::Collection(_) => "Collection".to_string(),
        FhirPathValue::Resource(_) => "Resource".to_string(),
        FhirPathValue::JsonValue(_) => "JsonValue".to_string(),
        FhirPathValue::TypeInfoObject { .. } => "TypeInfo".to_string(),
        FhirPathValue::Empty => "Empty".to_string(),
    }
}

fn format_fhir_value_pretty(value: &FhirPathValue) -> String {
    match value {
        FhirPathValue::String(s) => format!("\"{s}\""),
        FhirPathValue::Integer(i) => i.to_string(),
        FhirPathValue::Decimal(d) => d.to_string(),
        FhirPathValue::Boolean(b) => b.to_string(),
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
