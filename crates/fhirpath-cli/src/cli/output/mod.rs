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

//! Output formatting for CLI commands

mod json;
mod pretty;
mod raw;

use clap::ValueEnum;
use octofhir_fhirpath::{Collection, ExpressionNode, FhirPathError, FhirPathValue};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

pub use json::JsonFormatter;
pub use pretty::PrettyFormatter;
pub use raw::RawFormatter;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum OutputFormat {
    /// Pretty Ariadne-formatted output with colors and diagnostics (default)
    Pretty,
    /// JSON structured output for tooling
    Json,
    /// Raw text output
    Raw,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Raw => write!(f, "raw"),
            OutputFormat::Pretty => write!(f, "pretty"),
        }
    }
}

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Format error: {0}")]
    FormatError(String),
}

#[derive(Debug, Clone)]
pub struct EvaluationOutput {
    pub success: bool,
    pub result: Option<Collection>,
    pub error: Option<FhirPathError>,
    pub expression: String,
    pub execution_time: Duration,
    pub metadata: OutputMetadata,
}

impl EvaluationOutput {
    /// Create evaluation output from FhirPathValue result
    pub fn from_fhir_path_value(
        value: FhirPathValue,
        expression: String,
        execution_time: Duration,
    ) -> Self {
        let collection = Collection::from_values(value.to_collection());
        Self {
            success: true,
            result: Some(collection),
            error: None,
            expression,
            execution_time,
            metadata: OutputMetadata {
                cache_hits: 0,
                ast_nodes: 0,
                memory_used: 0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseOutput {
    pub success: bool,
    pub ast: Option<ExpressionNode>,
    pub error: Option<FhirPathError>,
    pub expression: String,
    pub metadata: OutputMetadata,
}

#[derive(Debug, Clone)]
pub struct AnalysisOutput {
    pub success: bool,
    pub analysis: Option<AnalysisResult>,
    pub validation_errors: Vec<ValidationError>,
    pub error: Option<FhirPathError>,
    pub expression: String,
    pub metadata: OutputMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct OutputMetadata {
    pub cache_hits: usize,
    pub ast_nodes: usize,
    pub memory_used: usize,
}

pub trait OutputFormatter {
    fn format_evaluation(&self, output: &EvaluationOutput) -> Result<String, FormatError>;
    fn format_parse(&self, output: &ParseOutput) -> Result<String, FormatError>;
    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError>;
}

pub struct FormatterFactory {
    no_color: bool,
}

impl FormatterFactory {
    pub fn new(no_color: bool) -> Self {
        Self { no_color }
    }

    pub fn create_formatter(&self, format: OutputFormat) -> Box<dyn OutputFormatter> {
        match format {
            OutputFormat::Json => Box::new(JsonFormatter::new()),
            OutputFormat::Raw => Box::new(RawFormatter::new()),
            OutputFormat::Pretty => Box::new(PrettyFormatter::new(!self.no_color)),
        }
    }
}

// -------- Minimal analyzer-friendly types for CLI formatting --------

#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    pub type_annotations: HashMap<String, SemanticInfo>,
    pub function_calls: Vec<FunctionAnalysis>,
}

#[derive(Debug, Clone, Default)]
pub struct SemanticInfo {
    pub fhir_path_type: Option<String>,
    pub model_type: Option<String>,
    pub cardinality: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionSignature {
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct FunctionAnalysis {
    pub function_name: String,
    pub signature: FunctionSignature,
    pub validation_errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationError {
    pub error_type: String,
    pub message: String,
    pub suggestions: Vec<String>,
}

impl Default for FormatterFactory {
    fn default() -> Self {
        Self::new(false)
    }
}
