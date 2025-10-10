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

//! Template-based output formatter

use super::{AnalysisOutput, EvaluationOutput, FormatError, OutputFormatter, ParseOutput};
use std::collections::HashMap;

/// Template formatter that supports custom output formats with placeholders
pub struct TemplateFormatter {
    template: String,
}

impl TemplateFormatter {
    /// Create a new template formatter
    pub fn new(template: String) -> Self {
        Self { template }
    }

    /// Format output using the template with provided variables
    pub fn format(&self, variables: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();

        // Replace all placeholders {key} with their values
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Handle common escape sequences
        result = result
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r");

        result
    }

    /// Get available placeholders from the template
    pub fn get_placeholders(&self) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut chars = self.template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut placeholder = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == '}' {
                        chars.next(); // consume '}'
                        if !placeholder.is_empty() {
                            placeholders.push(placeholder);
                        }
                        break;
                    }
                    placeholder.push(next_ch);
                    chars.next();
                }
            }
        }

        placeholders
    }

    /// Validate that all required placeholders can be satisfied
    pub fn validate(&self, available_keys: &[&str]) -> Result<(), String> {
        let placeholders = self.get_placeholders();
        let missing: Vec<_> = placeholders
            .iter()
            .filter(|p| !available_keys.contains(&p.as_str()))
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Template contains unknown placeholders: {}",
                missing
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        }
    }
}

/// Built-in template presets
pub struct TemplatePresets;

impl TemplatePresets {
    /// CSV format template
    pub fn csv() -> String {
        "{result}".to_string()
    }

    /// Markdown format template
    pub fn markdown() -> String {
        r#"## FHIRPath Evaluation

**Expression**: `{expression}`

**Type**: `{type}`

**Result**:
```json
{result}
```
"#
        .to_string()
    }

    /// HTML format template
    pub fn html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>FHIRPath Result</title>
    <style>
        body {{ font-family: sans-serif; margin: 20px; }}
        .expression {{ background: #f0f0f0; padding: 10px; border-radius: 4px; }}
        .result {{ background: #e8f4f8; padding: 10px; margin-top: 10px; border-radius: 4px; }}
        .type {{ color: #666; font-size: 0.9em; }}
    </style>
</head>
<body>
    <h1>FHIRPath Evaluation Result</h1>
    <div class="expression">
        <strong>Expression:</strong> <code>{expression}</code>
    </div>
    <div class="type">
        <strong>Type:</strong> {type}
    </div>
    <div class="result">
        <strong>Result:</strong>
        <pre>{result}</pre>
    </div>
</body>
</html>"#
            .to_string()
    }

    /// XML format template
    pub fn xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fhirpath-result>
    <expression>{expression}</expression>
    <type>{type}</type>
    <result>{result}</result>
</fhirpath-result>"#
            .to_string()
    }

    /// Simple text format
    pub fn text() -> String {
        "Expression: {expression}\nType: {type}\nResult: {result}".to_string()
    }

    /// Table row format (TSV)
    pub fn tsv() -> String {
        "{expression}\t{type}\t{result}".to_string()
    }

    /// Get all available presets
    pub fn list() -> Vec<(&'static str, &'static str)> {
        vec![
            ("csv", "CSV output (result only)"),
            ("markdown", "Markdown formatted output"),
            ("html", "HTML formatted output"),
            ("xml", "XML formatted output"),
            ("text", "Simple text output"),
            ("tsv", "Tab-separated values"),
        ]
    }

    /// Get preset by name
    pub fn get(name: &str) -> Option<String> {
        match name {
            "csv" => Some(Self::csv()),
            "markdown" => Some(Self::markdown()),
            "html" => Some(Self::html()),
            "xml" => Some(Self::xml()),
            "text" => Some(Self::text()),
            "tsv" => Some(Self::tsv()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_formatter() {
        let template = TemplateFormatter::new("Hello {name}, you are {age} years old".to_string());
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        let result = template.format(&vars);
        assert_eq!(result, "Hello Alice, you are 30 years old");
    }

    #[test]
    fn test_get_placeholders() {
        let template = TemplateFormatter::new("Hello {name}, {greeting}!".to_string());
        let placeholders = template.get_placeholders();
        assert_eq!(placeholders, vec!["name", "greeting"]);
    }

    #[test]
    fn test_escape_sequences() {
        let template = TemplateFormatter::new("Line 1\\nLine 2\\tTabbed".to_string());
        let vars = HashMap::new();
        let result = template.format(&vars);
        assert_eq!(result, "Line 1\nLine 2\tTabbed");
    }

    #[test]
    fn test_output_formatter_evaluation() {
        use super::super::EvaluationOutput;
        use octofhir_fhirpath::Collection;
        use std::time::Duration;

        let template = TemplateFormatter::new("{expression} => {result}".to_string());
        let output = EvaluationOutput {
            success: true,
            result: Some(Collection::from_values(vec![])),
            result_with_metadata: None,
            error: None,
            expression: "Patient.name".to_string(),
            execution_time: Duration::from_millis(10),
            metadata: super::super::OutputMetadata::default(),
        };

        let result = template.format_evaluation(&output).unwrap();
        assert!(result.contains("Patient.name"));
    }
}

impl OutputFormatter for TemplateFormatter {
    fn format_evaluation(&self, output: &EvaluationOutput) -> Result<String, FormatError> {
        let mut vars = HashMap::new();

        // Expression
        vars.insert("expression".to_string(), output.expression.clone());

        // Execution time
        vars.insert(
            "time".to_string(),
            format!("{}ms", output.execution_time.as_millis()),
        );
        vars.insert(
            "time_secs".to_string(),
            format!("{:.3}s", output.execution_time.as_secs_f64()),
        );

        // Result handling
        if output.success {
            if let Some(ref collection) = output.result {
                // Format result as JSON
                let result_json =
                    serde_json::to_string_pretty(&collection).unwrap_or_else(|_| "[]".to_string());
                vars.insert("result".to_string(), result_json.clone());
                vars.insert("result_compact".to_string(), format!("{:?}", collection));

                // Count and type information
                vars.insert("count".to_string(), collection.len().to_string());

                // Try to determine type from collection
                let type_str = if collection.is_empty() {
                    "empty".to_string()
                } else if let Some(first) = collection.first() {
                    format!("{:?}", first)
                        .split('(')
                        .next()
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "unknown".to_string()
                };
                vars.insert("type".to_string(), type_str);
            } else {
                vars.insert("result".to_string(), "null".to_string());
                vars.insert("result_compact".to_string(), "null".to_string());
                vars.insert("count".to_string(), "0".to_string());
                vars.insert("type".to_string(), "empty".to_string());
            }
            vars.insert("status".to_string(), "success".to_string());
        } else {
            // Error case
            if let Some(ref error) = output.error {
                vars.insert("error".to_string(), error.to_string());
            } else {
                vars.insert("error".to_string(), "Unknown error".to_string());
            }
            vars.insert("result".to_string(), "error".to_string());
            vars.insert("result_compact".to_string(), "error".to_string());
            vars.insert("count".to_string(), "0".to_string());
            vars.insert("type".to_string(), "error".to_string());
            vars.insert("status".to_string(), "error".to_string());
        }

        // Metadata
        vars.insert(
            "ast_nodes".to_string(),
            output.metadata.ast_nodes.to_string(),
        );
        vars.insert(
            "cache_hits".to_string(),
            output.metadata.cache_hits.to_string(),
        );
        vars.insert(
            "memory_used".to_string(),
            output.metadata.memory_used.to_string(),
        );

        Ok(self.format(&vars))
    }

    fn format_parse(&self, output: &ParseOutput) -> Result<String, FormatError> {
        let mut vars = HashMap::new();

        vars.insert("expression".to_string(), output.expression.clone());

        if output.success {
            if let Some(ref ast) = output.ast {
                vars.insert("ast".to_string(), format!("{:#?}", ast));
                vars.insert("status".to_string(), "success".to_string());
            } else {
                vars.insert("ast".to_string(), "null".to_string());
                vars.insert("status".to_string(), "success".to_string());
            }
        } else {
            if let Some(ref error) = output.error {
                vars.insert("error".to_string(), error.to_string());
            } else {
                vars.insert("error".to_string(), "Unknown error".to_string());
            }
            vars.insert("ast".to_string(), "error".to_string());
            vars.insert("status".to_string(), "error".to_string());
        }

        // Metadata
        vars.insert(
            "ast_nodes".to_string(),
            output.metadata.ast_nodes.to_string(),
        );

        Ok(self.format(&vars))
    }

    fn format_analysis(&self, output: &AnalysisOutput) -> Result<String, FormatError> {
        let mut vars = HashMap::new();

        vars.insert("expression".to_string(), output.expression.clone());

        if output.success {
            if let Some(ref analysis) = output.analysis {
                // Format analysis result for template
                vars.insert("analysis".to_string(), format!("{:#?}", analysis));
                vars.insert(
                    "type_count".to_string(),
                    analysis.type_annotations.len().to_string(),
                );
                vars.insert(
                    "function_count".to_string(),
                    analysis.function_calls.len().to_string(),
                );
            } else {
                vars.insert("analysis".to_string(), "null".to_string());
                vars.insert("type_count".to_string(), "0".to_string());
                vars.insert("function_count".to_string(), "0".to_string());
            }

            vars.insert(
                "validation_error_count".to_string(),
                output.validation_errors.len().to_string(),
            );
            vars.insert("status".to_string(), "success".to_string());
        } else {
            if let Some(ref error) = output.error {
                vars.insert("error".to_string(), error.to_string());
            } else {
                vars.insert("error".to_string(), "Unknown error".to_string());
            }
            vars.insert("analysis".to_string(), "error".to_string());
            vars.insert("status".to_string(), "error".to_string());
        }

        Ok(self.format(&vars))
    }
}
