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

//! Table output formatter

use super::{OutputFormatter, EvaluationOutput, ParseOutput, AnalysisOutput, FormatError};
use tabled::{Table, Tabled};
use octofhir_fhirpath_model::FhirPathValue;

pub struct TableFormatter {
    _colored: bool,
}

impl TableFormatter {
    pub fn new(colored: bool) -> Self {
        Self { _colored: colored }
    }
}

#[derive(Tabled)]
struct ResultRow {
    #[tabled(rename = "Index")]
    index: String,
    #[tabled(rename = "Type")]
    value_type: String,
    #[tabled(rename = "Value")]
    value: String,
}

#[derive(Tabled)]
struct ValidationErrorRow {
    #[tabled(rename = "Type")]
    error_type: String,
    #[tabled(rename = "Message")]
    message: String,
    #[tabled(rename = "Suggestions")]
    suggestions: String,
}

#[derive(Tabled)]
struct TypeAnnotationRow {
    #[tabled(rename = "Node ID")]
    node_id: String,
    #[tabled(rename = "FHIRPath Type")]
    fhir_path_type: String,
    #[tabled(rename = "Model Type")]
    model_type: String,
    #[tabled(rename = "Cardinality")]
    cardinality: String,
}

impl OutputFormatter for TableFormatter {
    fn format_evaluation(&self, output: &EvaluationOutput) -> Result<String, FormatError> {
        {
            let mut result = String::new();

            if output.success {
                if let Some(ref value) = output.result {
                    // Convert FhirPathValue to table rows
                    let rows = match value {
                        FhirPathValue::Collection(values) => {
                            values.iter().enumerate().map(|(i, v)| ResultRow {
                                index: i.to_string(),
                                value_type: get_fhir_type_name(v),
                                value: format_fhir_value(v),
                            }).collect()
                        },
                        single_value => vec![ResultRow {
                            index: "0".to_string(),
                            value_type: get_fhir_type_name(single_value),
                            value: format_fhir_value(single_value),
                        }],
                    };

                    if !rows.is_empty() {
                        let table = Table::new(rows).to_string();
                        result.push_str(&table);
                        result.push('\n');
                    }

                    result.push_str(&format!("\nExpression: {}\n", output.expression));
                    result.push_str(&format!("Execution time: {:.1}ms\n", 
                                           output.execution_time.as_secs_f64() * 1000.0));
                    if output.metadata.cache_hits > 0 {
                        result.push_str(&format!("Cache hits: {}\n", output.metadata.cache_hits));
                    }
                } else {
                    result.push_str("No results\n");
                }
            } else if let Some(ref error) = output.error {
                result.push_str("❌ Error\n");
                result.push_str(&format!("Expression: {}\n", output.expression));
                result.push_str(&format!("Error: {error}\n"));
            }

            Ok(result)
        }
    }

    fn format_parse(&self, output: &ParseOutput) -> Result<String, FormatError> {
        {
            let mut result = String::new();

            if output.success {
                result.push_str("✅ Parse Result\n");
                result.push_str(&format!("Expression: {}\n", output.expression));
                if let Some(ref ast) = output.ast {
                    result.push_str(&format!("AST: {ast:?}\n"));
                }
                result.push_str(&format!("AST Nodes: {}\n", output.metadata.ast_nodes));
            } else if let Some(ref error) = output.error {
                result.push_str("❌ Parse Error\n");
                result.push_str(&format!("Expression: {}\n", output.expression));
                result.push_str(&format!("Error: {error}\n"));
            }

            Ok(result)
        }
    }

    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError> {
        {
            let mut result = String::new();

            if !output.validation_errors.is_empty() {
                result.push_str("❌ Validation Errors\n");
                
                let error_rows: Vec<ValidationErrorRow> = output.validation_errors.iter()
                    .map(|e| ValidationErrorRow {
                        error_type: format!("{:?}", e.error_type),
                        message: e.message.clone(),
                        suggestions: e.suggestions.join(", "),
                    }).collect();

                let table = Table::new(error_rows).to_string();
                result.push_str(&table);
                result.push('\n');
                return Ok(result);
            }

            if output.success {
                if let Some(ref analysis) = output.analysis {
                    result.push_str(&format!("📊 Analysis Results for: {}\n", output.expression));
                    result.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                    if !analysis.type_annotations.is_empty() {
                        result.push_str("\n🔍 Type Annotations:\n");
                        
                        let type_rows: Vec<TypeAnnotationRow> = analysis.type_annotations.iter()
                            .map(|(node_id, info)| TypeAnnotationRow {
                                node_id: node_id.to_string(),
                                fhir_path_type: info.fhir_path_type.as_ref()
                                    .map(|t| t.to_string()).unwrap_or_else(|| "Unknown".to_string()),
                                model_type: info.model_type.as_ref()
                                    .map(|t| t.to_string()).unwrap_or_else(|| "Unknown".to_string()),
                                cardinality: format!("{:?}", info.cardinality),
                            }).collect();

                        let table = Table::new(type_rows).to_string();
                        result.push_str(&table);
                        result.push('\n');
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

fn format_fhir_value(value: &FhirPathValue) -> String {
    match serde_json::to_string(value) {
        Ok(json) => {
            // Limit display length for readability
            if json.len() > 50 {
                format!("{}...", &json[..47])
            } else {
                json
            }
        },
        Err(_) => format!("{value:?}"),
    }
}