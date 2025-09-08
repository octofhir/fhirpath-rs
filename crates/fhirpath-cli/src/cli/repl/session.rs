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

//! REPL session management and main loop

use std::collections::HashMap;

use anyhow::{Context, Result};
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::{Editor, history::FileHistory};
use serde_json::Value as JsonValue;

use super::completion::FhirPathCompleter;
use super::display::DisplayFormatter;
use super::help::HelpSystem;
use super::{ReplCommand, ReplConfig};
use octofhir_fhirpath::analyzer::StaticAnalyzer;
use octofhir_fhirpath::core::JsonValueExt;
use octofhir_fhirpath::diagnostics::{ColorScheme, DiagnosticEngine, DiagnosticSeverity};
use octofhir_fhirpath::parser::{parse, parse_with_analysis};
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};

/// Main REPL session that handles user interaction
pub struct ReplSession {
    engine: FhirPathEngine,
    analyzer: StaticAnalyzer,
    diagnostic_engine: DiagnosticEngine,
    editor: Editor<FhirPathCompleter, FileHistory>,
    current_resource: Option<FhirPathValue>,
    variables: HashMap<String, FhirPathValue>,
    config: ReplConfig,
    formatter: DisplayFormatter,
    help_system: HelpSystem,
    interrupt_count: u32,
}

impl ReplSession {
    /// Create a new REPL session with a pre-created engine
    pub async fn new(engine: FhirPathEngine, config: ReplConfig) -> Result<Self> {
        // Create analyzer
        let analyzer = StaticAnalyzer::new(
            engine.get_function_registry().clone(),
            engine.get_model_provider().clone(),
        );

        // Initialize editor with history
        let mut editor = Editor::<FhirPathCompleter, FileHistory>::new()
            .context("Failed to create readline editor")?;

        // Set up completer
        let completer = FhirPathCompleter::with_registry(
            engine.get_model_provider().clone(),
            Some(engine.get_function_registry().clone()),
        );
        editor.set_helper(Some(completer));

        // Load history if file is specified
        if let Some(history_path) = &config.history_file {
            if history_path.exists() {
                let _ = editor.load_history(history_path);
            }
        }

        let formatter = DisplayFormatter::new(config.color_output);
        let help_system = HelpSystem::with_registry(engine.get_function_registry().clone());

        // Create diagnostic engine for beautiful error reports
        let diagnostic_engine = if config.color_output {
            DiagnosticEngine::with_colors(ColorScheme::default())
        } else {
            DiagnosticEngine::new()
        };

        Ok(Self {
            engine,
            analyzer,
            diagnostic_engine,
            editor,
            current_resource: None,
            variables: HashMap::new(),
            config,
            formatter,
            help_system,
            interrupt_count: 0,
        })
    }

    /// Start the main REPL loop
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();

        // Cache function names for autocomplete
        self.cache_function_names().await;

        loop {
            match self.editor.readline(&self.config.prompt) {
                Ok(line) => {
                    let line = line.trim();

                    // Handle empty lines
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history and process normally
                    self.editor
                        .add_history_entry(line)
                        .context("Failed to add history entry")?;

                    match self.process_input(line).await {
                        Ok(Some(output)) => {
                            println!("{}", output);
                            self.interrupt_count = 0; // Reset interrupt count after successful command
                        }
                        Ok(None) => {
                            // Command handled, no output
                        }
                        Err(e) => {
                            println!("{}", self.formatter.format_error(&e));
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    self.interrupt_count += 1;
                    if self.interrupt_count == 1 {
                        if self.config.color_output {
                            println!("🚫 Use ':quit' or press Ctrl+C again to exit");
                        } else {
                            println!("Use ':quit' or press Ctrl+C again to exit");
                        }
                        continue;
                    } else {
                        if self.config.color_output {
                            println!("\n👋 Goodbye!");
                        } else {
                            println!("\nGoodbye!");
                        }
                        break;
                    }
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }

        // Save history before exit
        if let Some(history_path) = &self.config.history_file {
            if let Err(e) = self.editor.save_history(history_path) {
                eprintln!("Warning: Failed to save history: {}", e);
            }
        }

        Ok(())
    }

    /// Process a line of input from the user
    async fn process_input(&mut self, line: &str) -> Result<Option<String>> {
        if let Some(command) = self.parse_command(line)? {
            self.execute_command(command).await
        } else {
            // Treat as FHIRPath expression
            self.evaluate_expression(line).await.map(Some)
        }
    }

    /// Parse input line as a REPL command or None if it's an expression
    fn parse_command(&self, line: &str) -> Result<Option<ReplCommand>> {
        if !line.starts_with(':') {
            return Ok(None);
        }

        ReplCommand::parse(line)
    }

    /// Execute a REPL command
    async fn execute_command(&mut self, command: ReplCommand) -> Result<Option<String>> {
        match command {
            ReplCommand::Load { path } => match self.load_resource_from_file(&path) {
                Ok(()) => {
                    let success_msg = if self.config.color_output {
                        format!("✅ Successfully loaded resource from '{path}'")
                    } else {
                        format!("Successfully loaded resource from '{path}'")
                    };
                    Ok(Some(success_msg))
                }
                Err(e) => {
                    let error_msg = if self.config.color_output {
                        format!(
                            "❌ Failed to load '{path}': {e}\n💡 Check that the file exists and contains valid FHIR JSON"
                        )
                    } else {
                        format!(
                            "Failed to load '{path}': {e}\nTip: Check that the file exists and contains valid FHIR JSON"
                        )
                    };
                    Err(anyhow::anyhow!(error_msg))
                }
            },
            ReplCommand::Set { name, value } => match self.set_variable(&name, &value).await {
                Ok(display_value) => {
                    let success_msg = if self.config.color_output {
                        format!("✅ Variable '{name}' set to {display_value}")
                    } else {
                        format!("Variable '{name}' set to {display_value}")
                    };
                    Ok(Some(success_msg))
                }
                Err(e) => {
                    let error_msg = if self.config.color_output {
                        format!(
                            "❌ Failed to set variable '{name}': {e}\n💡 Use simple values like 'text' or FHIRPath expressions like Patient.name.first()"
                        )
                    } else {
                        format!(
                            "Failed to set variable '{name}': {e}\nTip: Use simple values like 'text' or FHIRPath expressions like Patient.name.first()"
                        )
                    };
                    Err(anyhow::anyhow!(error_msg))
                }
            },
            ReplCommand::Unset { name } => {
                if self.variables.contains_key(&name) {
                    self.unset_variable(&name);
                    let success_msg = if self.config.color_output {
                        format!("✅ Variable '{name}' removed")
                    } else {
                        format!("Variable '{name}' removed")
                    };
                    Ok(Some(success_msg))
                } else {
                    let warning_msg = if self.config.color_output {
                        format!(
                            "⚠️ Variable '{name}' not found. Use ':vars' to see defined variables."
                        )
                    } else {
                        format!(
                            "Warning: Variable '{name}' not found. Use ':vars' to see defined variables."
                        )
                    };
                    Ok(Some(warning_msg))
                }
            }
            ReplCommand::Vars => Ok(Some(self.list_variables())),
            ReplCommand::Resource => match &self.current_resource {
                Some(_) => Ok(Some(self.show_current_resource())),
                None => {
                    let msg = if self.config.color_output {
                        "ℹ️ No resource loaded. Use ':load <file>' to load a FHIR resource."
                            .to_string()
                    } else {
                        "No resource loaded. Use ':load <file>' to load a FHIR resource."
                            .to_string()
                    };
                    Ok(Some(msg))
                }
            },
            ReplCommand::Type { expression } => {
                let type_info = self.get_expression_type(&expression).await?;
                Ok(Some(type_info))
            }
            ReplCommand::Explain { expression } => {
                let explanation = self.explain_expression(&expression).await?;
                Ok(Some(explanation))
            }
            ReplCommand::Help { function } => {
                let help_text = self.get_help(function.as_deref()).await?;
                Ok(Some(help_text))
            }
            ReplCommand::History => Ok(Some(self.show_history()?)),
            ReplCommand::Analyze { expression } => {
                let analysis = self.analyze_expression(&expression).await?;
                Ok(Some(analysis))
            }
            ReplCommand::Validate { expression } => {
                let validation = self.validate_expression(&expression).await?;
                Ok(Some(validation))
            }
            ReplCommand::Quit => {
                std::process::exit(0);
            }
        }
    }

    /// Evaluate a FHIRPath expression with enhanced feedback
    async fn evaluate_expression(&mut self, expression: &str) -> Result<String> {
        // Pre-validation
        if expression.trim().is_empty() {
            return Ok(
                "Empty expression. Try typing a FHIRPath expression or ':help' for assistance."
                    .to_string(),
            );
        }

        // Check if resource is needed
        let needs_resource = !expression.starts_with("'")
            && !expression.chars().all(|c| c.is_numeric() || c == '.')
            && !expression.starts_with("today")
            && !expression.starts_with("now");

        let input_json = if let Some(resource) = &self.current_resource {
            match resource {
                FhirPathValue::Resource(res) => res.clone(),
                FhirPathValue::JsonValue(json) => std::sync::Arc::new(json.as_ref().clone()),
                _ => std::sync::Arc::new(serde_json::json!({})),
            }
        } else if needs_resource {
            // Provide helpful message when no resource is loaded but expression likely needs one
            let suggestion = if self.config.color_output {
                "ℹ️ No resource loaded. Some expressions may need a resource context.\n💡 Use ':load <file>' to load a FHIR resource, or try literal values like 'text' or numbers."
            } else {
                "No resource loaded. Some expressions may need a resource context.\nTip: Use ':load <file>' to load a FHIR resource, or try literal values like 'text' or numbers."
            };
            return Ok(suggestion.to_string());
        } else {
            std::sync::Arc::new(serde_json::json!({}))
        };

        // Add timing information for complex expressions
        let start = std::time::Instant::now();

        // For now, use a simple evaluation approach
        // TODO: Integrate variables and full engine support properly
        use octofhir_fhirpath::{Collection, FhirPathValue};
        use std::collections::HashMap;
        let input_value = FhirPathValue::resource(input_json.as_ref().clone());
        let collection = Collection::single(input_value);
        let variables = HashMap::new();
        let result = self
            .engine
            .evaluate_with_variables(expression, &collection, variables, None, None)
            .await
            .with_context(|| format!("Failed to evaluate expression: '{}'", expression))?;

        // Convert FhirPathValue to Vec<FhirPathValue>
        let values: Vec<FhirPathValue> = result.iter().cloned().collect();

        // Convert back to single FhirPathValue for formatting
        let result_value = if values.is_empty() {
            FhirPathValue::Empty
        } else if values.len() == 1 {
            values.into_iter().next().unwrap()
        } else {
            FhirPathValue::Collection(values)
        };

        let duration = start.elapsed();
        let mut output = self
            .formatter
            .format_result(&result_value, self.config.show_types);

        // Add performance information for longer evaluations
        if duration.as_millis() > 100 {
            let timing_info = if self.config.color_output {
                format!(
                    "\n⏱️ Evaluation took {:.2}ms",
                    duration.as_secs_f64() * 1000.0
                )
            } else {
                format!("\nEvaluation took {:.2}ms", duration.as_secs_f64() * 1000.0)
            };
            output.push_str(&timing_info);
        }

        Ok(output)
    }

    /// Load resource from file
    fn load_resource_from_file(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;

        let json_value: JsonValue = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON from file: {}", path))?;

        self.load_resource_from_json(json_value)
    }

    /// Load resource from JSON value
    pub fn load_resource_from_json(&mut self, json: JsonValue) -> Result<()> {
        let resource = FhirPathValue::resource(json);
        self.current_resource = Some(resource);
        Ok(())
    }

    /// Set a variable value (supports both literal values and FHIRPath expressions)
    pub async fn set_variable(&mut self, name: &str, value: &str) -> Result<String> {
        let (parsed_value, display_value) = self.parse_variable_value(value).await?;
        self.variables.insert(name.to_string(), parsed_value);
        Ok(display_value)
    }

    /// Parse a string value into a FHIRPath value (supports both literal values and expressions)
    async fn parse_variable_value(&mut self, value: &str) -> Result<(FhirPathValue, String)> {
        let value = value.trim();

        // Check if it looks like a FHIRPath expression (contains dots, function calls, etc.)
        let looks_like_expression = value.contains('.')
            || value.contains('(')
            || value.starts_with("Patient")
            || value.starts_with("Bundle")
            || value.starts_with("Observation")
            || value.starts_with("Condition")
            || value.starts_with("today")
            || value.starts_with("now")
            || (value.len() > 1
                && !value.starts_with('"')
                && !value.starts_with('\'')
                && !value.parse::<i64>().is_ok()
                && !value.parse::<f64>().is_ok());

        if looks_like_expression {
            // Try to evaluate as FHIRPath expression
            match self.try_evaluate_as_expression(value).await {
                Ok(result) => {
                    let display = self.formatter.format_value(&result, false);
                    Ok((result, display))
                }
                Err(e) => {
                    // If expression evaluation fails, check if it's a simple literal that we should treat as string
                    if value.starts_with('"') && value.ends_with('"') {
                        // JSON string
                        let string_val = value[1..value.len() - 1].to_string();
                        let fhir_val = FhirPathValue::string(string_val.clone());
                        Ok((fhir_val, format!("\"{}\"", string_val)))
                    } else if value.starts_with('\'') && value.ends_with('\'') {
                        // FHIRPath string
                        let string_val = value[1..value.len() - 1].to_string();
                        let fhir_val = FhirPathValue::string(string_val.clone());
                        Ok((fhir_val, format!("'{}'", string_val)))
                    } else {
                        // Expression evaluation failed
                        return Err(anyhow::anyhow!(
                            "Failed to evaluate expression '{}': {}",
                            value,
                            e
                        ));
                    }
                }
            }
        } else {
            // Try to parse as JSON first
            if let Ok(json_val) = serde_json::from_str::<JsonValue>(value) {
                let fhir_val = FhirPathValue::resource(json_val.clone());
                let display =
                    serde_json::to_string(&json_val).unwrap_or_else(|_| value.to_string());
                Ok((fhir_val, display))
            } else {
                // Treat as string literal
                let fhir_val = FhirPathValue::string(value.to_string());
                Ok((fhir_val, format!("'{}'", value)))
            }
        }
    }

    /// Try to evaluate a string as a FHIRPath expression
    async fn try_evaluate_as_expression(&mut self, expression: &str) -> Result<FhirPathValue> {
        let input_json = if let Some(resource) = &self.current_resource {
            match resource {
                FhirPathValue::Resource(res) => res.clone(),
                FhirPathValue::JsonValue(json) => std::sync::Arc::new(json.as_ref().clone()),
                _ => std::sync::Arc::new(serde_json::json!({})),
            }
        } else {
            // For expressions that don't need context like literals or today/now
            std::sync::Arc::new(serde_json::json!({}))
        };

        // For now, use simple evaluation without variables
        // TODO: Add variables support back
        use octofhir_fhirpath::Collection;
        use std::collections::HashMap;
        let input_value = FhirPathValue::resource(input_json.as_ref().clone());
        let collection = Collection::single(input_value);
        let variables = HashMap::new();
        let result = self
            .engine
            .evaluate_with_variables(expression, &collection, variables, None, None)
            .await
            .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))?;

        // Convert to the format expected by the rest of the function
        let values: Vec<FhirPathValue> = result.iter().cloned().collect();
        if let Some(first_value) = values.first() {
            Ok(first_value.clone())
        } else {
            Ok(FhirPathValue::Empty)
        }
    }

    /// Remove a variable
    fn unset_variable(&mut self, name: &str) {
        self.variables.remove(name);
    }

    /// List all current variables
    fn list_variables(&self) -> String {
        if self.variables.is_empty() && self.current_resource.is_none() {
            "No variables set".to_string()
        } else {
            let mut output = Vec::new();

            // Show context resource
            if let Some(resource) = &self.current_resource {
                let resource_type = match resource {
                    FhirPathValue::Resource(res) => res.resource_type().unwrap_or("Unknown"),
                    _ => "Unknown",
                };
                output.push(format!("%context = {} resource", resource_type));
            }

            // Show variables
            for (name, value) in &self.variables {
                let value_str = self.formatter.format_value(value, false);
                output.push(format!("{} = {}", name, value_str));
            }

            output.join("\n")
        }
    }

    /// Show current resource information
    fn show_current_resource(&self) -> String {
        if let Some(resource) = &self.current_resource {
            let resource_type = match resource {
                FhirPathValue::Resource(res) => res.resource_type().unwrap_or("Unknown"),
                _ => "Unknown",
            };
            format!("Current resource: {}", resource_type)
        } else {
            "No resource loaded".to_string()
        }
    }

    /// Get type information for an expression
    async fn get_expression_type(&self, expression: &str) -> Result<String> {
        let parse_result = parse(expression);
        let parsed_expr = parse_result.into_result()?;
        match self.analyzer.analyze(&parsed_expr).await {
            Ok(analysis) => {
                if analysis.type_info.is_empty() {
                    Ok(format!(
                        "No type information available for '{}'",
                        expression
                    ))
                } else {
                    let types: Vec<String> = analysis
                        .type_info
                        .values()
                        .map(|info| format!("{:?}", info))
                        .collect();
                    if types.is_empty() {
                        Ok(format!(
                            "Type information available but no FHIRPath types resolved for '{}'",
                            expression
                        ))
                    } else if types.len() == 1 {
                        Ok(format!("{}: {}", expression, types[0]))
                    } else {
                        Ok(format!("{}: {}", expression, types.join(" | ")))
                    }
                }
            }
            Err(e) => Ok(format!("Type analysis failed for '{}': {}", expression, e)),
        }
    }

    /// Explain expression evaluation steps
    async fn explain_expression(&mut self, expression: &str) -> Result<String> {
        // Try to parse and analyze the expression to provide insights
        let parse_result = parse(expression);
        let parsed_expr = match parse_result.into_result() {
            Ok(expr) => expr,
            Err(e) => return Ok(format!("Parse error: {}", e)),
        };
        match self.analyzer.analyze(&parsed_expr).await {
            Ok(analysis) => {
                let mut explanation = vec![
                    format!("Expression analysis for: {}", expression),
                    "─".repeat(50),
                ];

                if !analysis.type_info.is_empty() {
                    let types: Vec<String> = analysis
                        .type_info
                        .values()
                        .map(|info| format!("{:?}", info))
                        .collect();
                    if !types.is_empty() {
                        explanation.push(format!("Return type(s): {}", types.join(" | ")));
                    }
                }

                if !analysis.diagnostics.is_empty() {
                    explanation.push("Validation issues:".to_string());
                    for error in &analysis.diagnostics {
                        explanation.push(format!("  ⚠️  {}", error.message));
                    }
                }

                if analysis.complexity_metrics.function_calls > 0 {
                    explanation.push(format!(
                        "Function calls: {}",
                        analysis.complexity_metrics.function_calls
                    ));
                }

                // Try to evaluate and show result
                if let Ok(result) = self.evaluate_expression(expression).await {
                    explanation.push("Evaluation result:".to_string());
                    explanation.push(format!("  ▶️  {}", result));
                }

                Ok(explanation.join("\n"))
            }
            Err(e) => Ok(format!("Expression explanation failed: {}", e)),
        }
    }

    /// Analyze expression with full diagnostics using unified diagnostic system
    async fn analyze_expression(&mut self, expression: &str) -> Result<String> {
        use crate::cli::diagnostics::CliDiagnosticHandler;
        use crate::cli::output::OutputFormat;
        use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};

        // Create diagnostic handler using the same system as CLI
        let mut handler = CliDiagnosticHandler::new(if self.config.color_output {
            OutputFormat::Pretty
        } else {
            OutputFormat::Raw
        });
        let source_id = handler.add_source("expression".to_string(), expression.to_string());

        // Parse expression with analysis mode (same as CLI)
        let parse_result = parse_with_mode(expression, ParsingMode::Analysis);

        let mut all_diagnostics: Vec<octofhir_fhirpath::diagnostics::AriadneDiagnostic> =
            Vec::new();

        // Convert parser diagnostics to AriadneDiagnostic format (same as CLI)
        if !parse_result.diagnostics.is_empty() {
            let parser_diagnostics: Vec<_> = parse_result
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    use octofhir_fhirpath::core::error_code::ErrorCode;
                    use octofhir_fhirpath::diagnostics::AriadneDiagnostic;

                    // Convert location to span
                    let span = if let Some(location) = &diagnostic.location {
                        location.offset..(location.offset + location.length)
                    } else {
                        0..0
                    };

                    // Parse error code
                    let error_code = if diagnostic.code.code.starts_with("FP") {
                        if let Ok(num) = diagnostic.code.code[2..].parse::<u16>() {
                            ErrorCode::new(num)
                        } else {
                            ErrorCode::new(1)
                        }
                    } else if let Ok(num) = diagnostic.code.code.parse::<u16>() {
                        ErrorCode::new(num)
                    } else {
                        ErrorCode::new(1)
                    };

                    AriadneDiagnostic {
                        severity: diagnostic.severity.clone(),
                        error_code,
                        message: diagnostic.message.clone(),
                        span,
                        help: None,
                        note: None,
                        related: Vec::new(),
                    }
                })
                .collect();
            all_diagnostics.extend(parser_diagnostics);
        }

        // Run static analysis if parsing succeeded (same logic as CLI)
        if parse_result.success && parse_result.ast.is_some() {
            match self
                .analyzer
                .analyze(parse_result.ast.as_ref().unwrap())
                .await
            {
                Ok(analysis_result) => {
                    // Add static analysis diagnostics (using Ariadne diagnostics from analyzer)
                    let mut static_diagnostics = analysis_result.ariadne_diagnostics.clone();

                    // Fix missing spans by calculating them from expression text (same as CLI)
                    for diagnostic in &mut static_diagnostics {
                        if diagnostic.span == (0..0) {
                            if let Some(span) =
                                self.calculate_span_from_message(&diagnostic.message, expression)
                            {
                                diagnostic.span = span;
                            }
                        }
                    }

                    all_diagnostics.extend(static_diagnostics);
                }
                Err(e) => {
                    // Convert analysis error to diagnostic
                    let error_diagnostic = handler.create_diagnostic_from_error(
                        octofhir_fhirpath::core::error_code::FP0001,
                        format!("Static analysis failed: {}", e),
                        0..expression.len(),
                        None,
                    );
                    all_diagnostics.push(error_diagnostic);
                }
            }
        }

        // Sort and deduplicate diagnostics (same as CLI)
        all_diagnostics.sort_by(|a, b| {
            a.span
                .start
                .cmp(&b.span.start)
                .then(a.error_code.code.cmp(&b.error_code.code))
                .then(a.message.cmp(&b.message))
        });

        all_diagnostics.dedup_by(|a, b| {
            a.message == b.message && a.error_code == b.error_code && a.span == b.span
        });

        // Format diagnostics using the unified system
        let mut output = Vec::new();
        match handler.report_diagnostics(&all_diagnostics, source_id, &mut output) {
            Ok(_) => {
                let diagnostic_output = String::from_utf8(output)
                    .unwrap_or_else(|_| "Encoding error in diagnostics".to_string());

                // Add analysis summary if successful
                if all_diagnostics.is_empty() {
                    let success_msg = if self.config.color_output {
                        format!(
                            "✅ {}",
                            "Expression analysis passed with no issues".bright_green()
                        )
                    } else {
                        "✓ Expression analysis passed with no issues".to_string()
                    };
                    Ok(success_msg)
                } else {
                    // Check if there are any errors
                    let has_errors = all_diagnostics.iter().any(|d| {
                        matches!(
                            d.severity,
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
                        )
                    });

                    let summary = if has_errors {
                        if self.config.color_output {
                            format!("❌ {}", "Analysis completed with errors".bright_red())
                        } else {
                            "✗ Analysis completed with errors".to_string()
                        }
                    } else {
                        if self.config.color_output {
                            format!("⚠️  {}", "Analysis completed with warnings".bright_yellow())
                        } else {
                            "⚠ Analysis completed with warnings".to_string()
                        }
                    };

                    Ok(format!("{}\n\n{}", summary, diagnostic_output))
                }
            }
            Err(e) => Ok(format!("Failed to format diagnostics: {}", e)),
        }
    }

    /// Validate expression syntax only
    async fn validate_expression(&mut self, expression: &str) -> Result<String> {
        let parse_result = parse_with_analysis(expression);

        if parse_result.has_errors() {
            // Use Ariadne diagnostics for syntax errors
            Ok(self.format_parser_diagnostics(expression, &parse_result.diagnostics))
        } else {
            if self.config.color_output {
                Ok(format!(
                    "{} Expression syntax is valid",
                    "✅".bright_green()
                ))
            } else {
                Ok("✓ Expression syntax is valid".to_string())
            }
        }
    }

    /// Get help text
    async fn get_help(&self, function: Option<&str>) -> Result<String> {
        if let Some(func) = function {
            // Function-specific help
            if let Some(help) = self.help_system.get_function_help(func) {
                let mut output = vec![
                    format!("📚 Help for function '{}'", help.name),
                    "─".repeat(50),
                    format!("Description: {}", help.description),
                    format!("Usage: {}", help.usage),
                    format!("Returns: {}", help.returns),
                ];

                if !help.examples.is_empty() {
                    output.push("Examples:".to_string());
                    for example in &help.examples {
                        output.push(format!("  • {}", example));
                    }
                }

                Ok(output.join("\n"))
            } else if self.help_system.function_exists(func).await {
                // Function exists in registry but no detailed help available
                Ok(format!(
                    "Function '{}' is available but detailed help is not yet implemented.\nThis function is registered in the system. Try using it in an expression to see how it works.",
                    func
                ))
            } else {
                Ok(format!(
                    "No help available for function '{}'. Try ':help' for a list of available commands.",
                    func
                ))
            }
        } else {
            // General help
            Ok(self.get_general_help())
        }
    }

    /// Get command history
    fn show_history(&self) -> Result<String> {
        let history = self.editor.history();
        let entries: Vec<String> = history
            .iter()
            .enumerate()
            .map(|(i, entry)| format!("{:3}: {}", i + 1, entry))
            .collect();

        if entries.is_empty() {
            Ok("No command history".to_string())
        } else {
            Ok(entries.join("\n"))
        }
    }

    /// Print welcome message
    fn print_welcome(&self) {
        if self.config.color_output {
            println!("\n🔥 {}", self.cyan("FHIRPath Interactive REPL"));
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!(
                "📚 {} Evaluate FHIRPath expressions against FHIR resources",
                self.bold("Usage:")
            );
            println!(
                "   • Type expressions: {}, {}, {}",
                self.green("Patient.name.first()"),
                self.green("Bundle.entry.count()"),
                self.green("Observation.value")
            );
            println!(
                "   • Use commands: {}, {}, {}",
                self.blue(":load patient.json"),
                self.blue(":help"),
                self.blue(":vars")
            );
            println!();
            println!(
                "🚀 {} Start by loading a FHIR resource: {}",
                self.bold("Quick Start:"),
                self.blue(":load <your-file.json>")
            );
            println!(
                "📖 {} Use {} for available commands and {} for syntax help",
                self.bold("Help:"),
                self.blue(":help"),
                self.blue(":help <operation>")
            );
            println!(
                "🏃 {} Press {} or use {} to exit",
                self.bold("Exit:"),
                self.yellow("Ctrl+C twice"),
                self.blue(":quit")
            );
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        } else {
            println!("\nFHIRPath Interactive REPL");
            println!("========================");
            println!("Usage: Evaluate FHIRPath expressions against FHIR resources");
            println!("  • Type expressions: Patient.name.first(), Bundle.entry.count()");
            println!("  • Use commands: :load patient.json, :help, :vars");
            println!();
            println!("Quick Start: Start by loading a FHIR resource: :load <your-file.json>");
            println!(
                "Help: Use :help for available commands and :help <operation> for syntax help"
            );
            println!("Exit: Press Ctrl+C twice or use :quit to exit");
            println!("========================\n");
        }
    }

    // Helper methods for colored output in welcome message
    fn cyan(&self, text: &str) -> String {
        if self.config.color_output {
            text.cyan().to_string()
        } else {
            text.to_string()
        }
    }

    fn bold(&self, text: &str) -> String {
        if self.config.color_output {
            text.bold().to_string()
        } else {
            text.to_string()
        }
    }

    fn green(&self, text: &str) -> String {
        if self.config.color_output {
            text.green().to_string()
        } else {
            text.to_string()
        }
    }

    fn blue(&self, text: &str) -> String {
        if self.config.color_output {
            text.blue().to_string()
        } else {
            text.to_string()
        }
    }

    fn yellow(&self, text: &str) -> String {
        if self.config.color_output {
            text.yellow().to_string()
        } else {
            text.to_string()
        }
    }

    /// Cache function names from registry for autocomplete
    async fn cache_function_names(&mut self) {
        let function_names: Vec<String> = self
            .engine
            .get_function_registry()
            .list_functions()
            .iter()
            .map(|f| f.name.clone())
            .collect();
        if let Some(helper) = self.editor.helper_mut() {
            helper.cache_function_names(function_names);
        }
    }

    /// Get general help text
    fn get_general_help(&self) -> String {
        r#"Available commands:
  <expression>          Evaluate FHIRPath expression
  :load <file>         Load FHIR resource from file  
  :set <name> <value>  Set variable (supports expressions and literals)
  :unset <name>        Remove variable
  :vars                List all variables
  :resource            Show current resource
  :type <expression>   Show type information
  :explain <expr>      Show evaluation steps
  :help [function]     Show help (for specific function if provided)
  :history             Show command history
  :quit                Exit REPL

Multi-line expressions:
  - End lines with backslash (\) to continue on next line
  - Press Enter on empty line to evaluate multi-line expression
  - Use Ctrl+C to cancel multi-line mode

Examples:
  Patient.name.given.first()
  telecom.where(system = 'email').value
  Bundle.entry \
    .where(resource.resourceType = 'Patient') \
    .select(resource.name.first())
  :load examples/patient.json
  :set myVar "hello world"
  :set workPhone Patient.telecom.where(use='work').value
  :set patientName Patient.name.first().given.first()"#
            .to_string()
    }

    // Multi-line expression support methods

    /// Check if a line needs multi-line continuation
    fn needs_multiline(&self, line: &str) -> bool {
        // Check for explicit continuation with backslash
        if line.ends_with('\\') {
            return true;
        }

        // Check for incomplete parentheses/brackets
        let open_parens = line.chars().filter(|&c| c == '(').count();
        let close_parens = line.chars().filter(|&c| c == ')').count();

        if open_parens > close_parens {
            return true;
        }

        // Check for incomplete string literals
        let single_quotes = line.chars().filter(|&c| c == '\'').count();
        let double_quotes = line.chars().filter(|&c| c == '"').count();

        if single_quotes % 2 != 0 || double_quotes % 2 != 0 {
            return true;
        }

        // Check for common multi-line patterns
        let lower_line = line.to_lowercase();
        if lower_line.ends_with(".where(")
            || lower_line.ends_with(".select(")
            || lower_line.ends_with(".all(")
            || lower_line.ends_with(".any(")
            || lower_line.ends_with(".repeat(")
            || lower_line.ends_with("aggregate(")
        {
            return true;
        }

        false
    }

    /// Check if a multi-line expression is complete
    fn is_expression_complete(&self, expression: &str) -> bool {
        // Remove explicit continuation markers
        let cleaned = expression.replace(" \\", "").replace("\\", "");

        // Check for balanced parentheses/brackets
        let open_parens = cleaned.chars().filter(|&c| c == '(').count();
        let close_parens = cleaned.chars().filter(|&c| c == ')').count();

        if open_parens != close_parens {
            return false;
        }

        // Check for complete string literals
        let single_quotes = cleaned.chars().filter(|&c| c == '\'').count();
        let double_quotes = cleaned.chars().filter(|&c| c == '"').count();

        if single_quotes % 2 != 0 || double_quotes % 2 != 0 {
            return false;
        }

        true
    }

    /// Format an error with Ariadne diagnostics
    fn format_error_with_ariadne(&mut self, expression: &str, error: &anyhow::Error) -> String {
        // Check if we can parse the expression to get detailed diagnostics
        let parse_result = parse_with_analysis(expression);

        if !parse_result.diagnostics.is_empty() {
            // Use parser diagnostics for better error reporting
            self.format_parser_diagnostics(expression, &parse_result.diagnostics)
        } else {
            // Fallback to simple error formatting
            self.formatter.format_error(&error)
        }
    }

    /// Format analyzer diagnostics with Ariadne (unified with parser diagnostics)
    fn format_analyzer_diagnostics(
        &mut self,
        expression: &str,
        analyzer_diagnostics: &[octofhir_fhirpath::diagnostics::Diagnostic],
    ) -> String {
        // Reuse the same formatting logic as parser diagnostics
        self.format_parser_diagnostics(expression, analyzer_diagnostics)
    }

    /// Format enhanced Ariadne diagnostics from PropertyValidator
    fn format_enhanced_ariadne_diagnostics(
        &mut self,
        expression: &str,
        ariadne_diagnostics: &[octofhir_fhirpath::diagnostics::AriadneDiagnostic],
    ) -> String {
        let mut output = Vec::new();

        // Add the expression as a source
        let source_id = self.diagnostic_engine.add_source("expression", expression);

        // Use the DiagnosticEngine to emit a beautiful unified report
        match self.diagnostic_engine.emit_unified_report(
            ariadne_diagnostics,
            source_id,
            &mut output,
        ) {
            Ok(_) => {
                // Add header if colors are enabled
                let mut result = if self.config.color_output {
                    format!("{} Enhanced Property Validation:\n", "🔍".bright_cyan())
                } else {
                    "Enhanced Property Validation:\n".to_string()
                };

                result.push_str(
                    &String::from_utf8(output)
                        .unwrap_or_else(|_| "Encoding error in diagnostics".to_string()),
                );
                result
            }
            Err(e) => {
                // Fallback to simple diagnostic listing
                if ariadne_diagnostics.is_empty() {
                    return "No enhanced diagnostics available".to_string();
                }

                let mut result = String::new();
                for (i, diagnostic) in ariadne_diagnostics.iter().enumerate() {
                    if i > 0 {
                        result.push('\n');
                    }

                    let severity_marker = if self.config.color_output {
                        match diagnostic.severity {
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error => {
                                "❌".bright_red()
                            }
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Warning => {
                                "⚠️".bright_yellow()
                            }
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Info => {
                                "ℹ️".bright_blue()
                            }
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Hint => {
                                "💡".bright_cyan()
                            }
                        }
                    } else {
                        match diagnostic.severity {
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error => {
                                "✗".normal()
                            }
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Warning => {
                                "⚠".normal()
                            }
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Info => {
                                "i".normal()
                            }
                            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Hint => {
                                "?".normal()
                            }
                        }
                    };

                    result.push_str(&format!(
                        "{} [{}] {}",
                        severity_marker,
                        diagnostic.error_code.code_str(),
                        diagnostic.message
                    ));

                    // Add help text if available
                    if let Some(help) = &diagnostic.help {
                        result.push_str(&format!("\n  {}", help));
                    }

                    // Add note if available
                    if let Some(note) = &diagnostic.note {
                        result.push_str(&format!("\n  {}", note));
                    }
                }

                format!("Fallback diagnostic formatting ({}): {}", e, result)
            }
        }
    }

    /// Calculate span from diagnostic message by finding tokens in expression (same as CLI)
    fn calculate_span_from_message(
        &self,
        message: &str,
        expression: &str,
    ) -> Option<std::ops::Range<usize>> {
        // Extract resource type or property name from message
        if message.contains("Unknown resource type: '") {
            // Extract resource type name from message like "Unknown resource type: 'Pat'"
            let start_marker = "Unknown resource type: '";
            let end_marker = "'";
            if let Some(start) = message.find(start_marker) {
                let name_start = start + start_marker.len();
                if let Some(name_end) = message[name_start..].find(end_marker) {
                    let resource_name = &message[name_start..name_start + name_end];
                    // Find this resource name in the expression
                    if let Some(pos) = expression.find(resource_name) {
                        return Some(pos..pos + resource_name.len());
                    }
                }
            }
        } else if message.contains("Cannot validate property '") {
            // Extract property name from message like "Cannot validate property 'name' on unknown type"
            let start_marker = "Cannot validate property '";
            let end_marker = "'";
            if let Some(start) = message.find(start_marker) {
                let name_start = start + start_marker.len();
                if let Some(name_end) = message[name_start..].find(end_marker) {
                    let property_name = &message[name_start..name_start + name_end];
                    // Find this property name in the expression (usually after a dot)
                    if let Some(pos) = expression.find(&format!(".{}", property_name)) {
                        return Some(pos + 1..pos + 1 + property_name.len());
                    }
                }
            }
        }

        None
    }

    /// Format parser diagnostics with Ariadne
    fn format_parser_diagnostics(
        &mut self,
        expression: &str,
        diagnostics: &[octofhir_fhirpath::diagnostics::Diagnostic],
    ) -> String {
        use octofhir_fhirpath::core::error_code::ErrorCode;

        let mut output = Vec::new();

        // Add the expression as a source
        let source_id = self.diagnostic_engine.add_source("expression", expression);

        let mut ariadne_diagnostics = Vec::new();

        for diagnostic in diagnostics {
            // Convert location to span range
            let span = if let Some(location) = &diagnostic.location {
                location.offset..(location.offset + location.length)
            } else {
                // If no location, highlight the entire expression
                0..expression.len()
            };

            // Create AriadneDiagnostic using the engine's factory method
            let ariadne_diagnostic = self.diagnostic_engine.create_diagnostic(
                // Parse the error code or use a default
                ErrorCode::new(
                    diagnostic
                        .code
                        .code
                        .strip_prefix("FP")
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(1), // Default to FP0001 if parsing fails
                ),
                diagnostic.severity.clone(),
                span,
                diagnostic.message.clone(),
            );

            ariadne_diagnostics.push(ariadne_diagnostic);
        }

        // Emit the unified report using the engine's method
        match self.diagnostic_engine.emit_unified_report(
            &ariadne_diagnostics,
            source_id,
            &mut output,
        ) {
            Ok(_) => String::from_utf8(output)
                .unwrap_or_else(|_| format!("Encoding error in diagnostics")),
            Err(_) => {
                // Fallback to simple message
                format!("Error: {}", diagnostics[0].message)
            }
        }
    }
}
