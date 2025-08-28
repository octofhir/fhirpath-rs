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

//! Raw text output formatter (current default)

use super::{OutputFormatter, EvaluationOutput, ParseOutput, AnalysisOutput, FormatError};

pub struct RawFormatter;

impl Default for RawFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl RawFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for RawFormatter {
    fn format_evaluation(&self, output: &EvaluationOutput) -> Result<String, FormatError> {
        let mut result = String::new();

        if output.success {
            if let Some(ref value) = output.result {
                match serde_json::to_string_pretty(value) {
                    Ok(json) => result.push_str(&json),
                    Err(_) => result.push_str(&format!("{value:?}")),
                }
            } else {
                result.push_str("null");
            }
        } else if let Some(ref error) = output.error {
            result.push_str(&format!("Error evaluating expression: {error}"));
        }

        Ok(result)
    }

    fn format_parse(&self, output: &ParseOutput) -> Result<String, FormatError> {
        let mut result = String::new();

        if output.success {
            if let Some(ref ast) = output.ast {
                result.push_str("✓ Expression parsed successfully\n");
                result.push_str(&format!("Expression: {}\n", output.expression));
                result.push_str(&format!("AST: {ast:?}"));
            } else {
                result.push_str("OK");
            }
        } else if let Some(ref error) = output.error {
            result.push_str(&format!("✗ Parse error: {error}"));
        }

        Ok(result)
    }

    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError> {
        let mut result = String::new();

        if !output.validation_errors.is_empty() {
            result.push_str("❌ Validation Errors:\n");
            for error in &output.validation_errors {
                let icon = match error.error_type {
                    octofhir_fhirpath_analyzer::ValidationErrorType::InvalidField => "🔍",
                    octofhir_fhirpath_analyzer::ValidationErrorType::DeprecatedField => "⚠️",
                    octofhir_fhirpath_analyzer::ValidationErrorType::InvalidResourceType => "🏥",
                    octofhir_fhirpath_analyzer::ValidationErrorType::InvalidFunction => "🔧",
                    _ => "❗",
                };
                result.push_str(&format!("  {} {}\n", icon, error.message));
                if !error.suggestions.is_empty() {
                    result.push_str(&format!("    💡 Suggestions: {}\n", error.suggestions.join(", ")));
                }
            }
            return Ok(result);
        }

        if output.success {
            if let Some(ref analysis) = output.analysis {
                result.push_str(&format!("📊 Analysis Results for: {}\n", output.expression));
                result.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                if !analysis.type_annotations.is_empty() {
                    result.push_str("\n🔍 Type Annotations:\n");
                    for (node_id, semantic_info) in &analysis.type_annotations {
                        result.push_str(&format!("  Node {node_id}: \n"));
                        if let Some(ref fhir_type) = semantic_info.fhir_path_type {
                            result.push_str(&format!("    FHIRPath Type: {fhir_type}\n"));
                        }
                        if let Some(ref model_type) = semantic_info.model_type {
                            result.push_str(&format!("    FHIR Model Type: {model_type}\n"));
                        }
                        result.push_str(&format!("    Cardinality: {:?}\n", semantic_info.cardinality));
                        result.push_str(&format!("    Confidence: {:?}\n", semantic_info.confidence));
                    }
                }

                if !analysis.function_calls.is_empty() {
                    result.push_str("\n🔧 Function Calls:\n");
                    for func_analysis in &analysis.function_calls {
                        result.push_str(&format!(
                            "  - {} ({})\n",
                            func_analysis.function_name, 
                            func_analysis.signature.description
                        ));
                        for error in &func_analysis.validation_errors {
                            result.push_str(&format!("    ⚠️  {}\n", error.message));
                        }
                    }
                }

                result.push_str("\n✅ Analysis complete");
            } else {
                result.push_str("OK");
            }
        } else if let Some(ref error) = output.error {
            result.push_str(&format!("❌ Error during analysis: {error}"));
        }

        Ok(result)
    }
}