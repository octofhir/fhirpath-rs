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

//! JSON output formatter

use super::{AnalysisOutput, EvaluationOutput, FormatError, OutputFormatter, ParseOutput};
use serde::Serialize;
use std::collections::HashMap;

pub struct JsonFormatter;

impl Default for JsonFormatter {
    fn default() -> Self {
        Self
    }
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Serialize)]
struct JsonEvaluationResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    type_metadata: Option<Vec<JsonTypeMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonError>,
    expression: String,
    execution_time_ms: f64,
    metadata: JsonMetadata,
}

#[derive(Serialize)]
struct JsonTypeMetadata {
    type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_return_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cardinality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    is_fhir_type: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    constraints: Vec<String>,
}

#[derive(Serialize)]
struct JsonParseResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    ast: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonError>,
    expression: String,
    metadata: JsonMetadata,
}

#[derive(Serialize)]
struct JsonAnalysisResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis: Option<JsonAnalysis>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    validation_errors: Vec<JsonValidationError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonError>,
    expression: String,
    metadata: JsonMetadata,
}

#[derive(Serialize)]
struct JsonError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<JsonLocation>,
}

#[derive(Serialize)]
struct JsonLocation {
    line: usize,
    column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<String>,
}

impl JsonLocation {
    /// Create JsonLocation from FhirPath SourceLocation
    fn from_source_location(loc: &octofhir_fhirpath::core::SourceLocation) -> Self {
        Self {
            line: loc.line,
            column: loc.column,
            span: Some(format!("{}..{}", loc.offset, loc.offset + loc.length)),
        }
    }
}

#[derive(Serialize)]
struct JsonMetadata {
    cache_hits: usize,
    ast_nodes: usize,
    memory_used: usize,
}

#[derive(Serialize)]
struct JsonAnalysis {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    type_annotations: HashMap<String, JsonSemanticInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    function_calls: Vec<JsonFunctionAnalysis>,
}

#[derive(Serialize)]
struct JsonSemanticInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    fhir_path_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_type: Option<String>,
    cardinality: String,
    confidence: String,
}

#[derive(Serialize)]
struct JsonFunctionAnalysis {
    function_name: String,
    signature_description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    validation_errors: Vec<JsonValidationError>,
}

#[derive(Serialize)]
struct JsonValidationError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    suggestions: Vec<String>,
}

impl OutputFormatter for JsonFormatter {
    fn format_evaluation(&self, output: &EvaluationOutput) -> Result<String, FormatError> {
        let (result_value, type_metadata) =
            if let Some(ref collection_with_metadata) = output.result_with_metadata {
                // Use rich metadata when available
                let result_value = Some(collection_with_metadata.to_json_parts());
                let type_metadata = Some(
                    collection_with_metadata
                        .results()
                        .iter()
                        .map(|result| JsonTypeMetadata {
                            type_name: result.type_info.type_name.clone(),
                            expected_return_type: result.type_info.expected_return_type.clone(),
                            cardinality: result.type_info.cardinality.clone(),
                            namespace: result.type_info.namespace.clone(),
                            is_fhir_type: result.type_info.is_fhir_type,
                            constraints: result.type_info.constraints.clone(),
                        })
                        .collect::<Vec<_>>(),
                );
                (result_value, type_metadata)
            } else if let Some(ref value) = output.result {
                // Fall back to basic result without rich type info
                (Some(serde_json::to_value(value)?), None)
            } else {
                (None, None)
            };

        let result = JsonEvaluationResult {
            success: output.success,
            result: result_value,
            type_metadata,
            error: output.error.as_ref().map(|e| JsonError {
                error_type: format!("{e:?}")
                    .split('(')
                    .next()
                    .unwrap_or("Error")
                    .to_string(),
                message: e.to_string(),
                location: e.location().map(JsonLocation::from_source_location),
            }),
            expression: output.expression.clone(),
            execution_time_ms: output.execution_time.as_secs_f64() * 1000.0,
            metadata: JsonMetadata {
                cache_hits: output.metadata.cache_hits,
                ast_nodes: output.metadata.ast_nodes,
                memory_used: output.metadata.memory_used,
            },
        };

        Ok(serde_json::to_string_pretty(&result)?)
    }

    fn format_parse(&self, output: &ParseOutput) -> Result<String, FormatError> {
        let result = JsonParseResult {
            success: output.success,
            ast: if let Some(ref ast) = output.ast {
                Some(serde_json::to_value(format!("{ast:?}"))?)
            } else {
                None
            },
            error: output.error.as_ref().map(|e| JsonError {
                error_type: format!("{e:?}")
                    .split('(')
                    .next()
                    .unwrap_or("Error")
                    .to_string(),
                message: e.to_string(),
                location: e.location().map(JsonLocation::from_source_location),
            }),
            expression: output.expression.clone(),
            metadata: JsonMetadata {
                cache_hits: output.metadata.cache_hits,
                ast_nodes: output.metadata.ast_nodes,
                memory_used: output.metadata.memory_used,
            },
        };

        Ok(serde_json::to_string_pretty(&result)?)
    }

    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError> {
        let result = JsonAnalysisResult {
            success: output.success,
            analysis: if let Some(ref analysis) = output.analysis {
                let mut type_annotations = HashMap::new();
                for (node_id, semantic_info) in &analysis.type_annotations {
                    type_annotations.insert(
                        node_id.to_string(),
                        JsonSemanticInfo {
                            fhir_path_type: semantic_info
                                .fhir_path_type
                                .as_ref()
                                .map(|t| t.to_string()),
                            model_type: semantic_info.model_type.as_ref().map(|t| t.to_string()),
                            cardinality: format!("{:?}", semantic_info.cardinality),
                            confidence: format!("{:?}", semantic_info.confidence),
                        },
                    );
                }

                let function_calls: Vec<JsonFunctionAnalysis> = analysis
                    .function_calls
                    .iter()
                    .map(|func_analysis| JsonFunctionAnalysis {
                        function_name: func_analysis.function_name.clone(),
                        signature_description: func_analysis.signature.description.clone(),
                        validation_errors: func_analysis
                            .validation_errors
                            .iter()
                            .map(|error| JsonValidationError {
                                error_type: format!("{:?}", error.error_type),
                                message: error.message.clone(),
                                suggestions: error.suggestions.clone(),
                            })
                            .collect(),
                    })
                    .collect();

                Some(JsonAnalysis {
                    type_annotations,
                    function_calls,
                })
            } else {
                None
            },
            validation_errors: output
                .validation_errors
                .iter()
                .map(|error| JsonValidationError {
                    error_type: format!("{:?}", error.error_type),
                    message: error.message.clone(),
                    suggestions: error.suggestions.clone(),
                })
                .collect(),
            error: output.error.as_ref().map(|e| JsonError {
                error_type: format!("{e:?}")
                    .split('(')
                    .next()
                    .unwrap_or("Error")
                    .to_string(),
                message: e.to_string(),
                location: e.location().map(JsonLocation::from_source_location),
            }),
            expression: output.expression.clone(),
            metadata: JsonMetadata {
                cache_hits: output.metadata.cache_hits,
                ast_nodes: output.metadata.ast_nodes,
                memory_used: output.metadata.memory_used,
            },
        };

        Ok(serde_json::to_string_pretty(&result)?)
    }
}
