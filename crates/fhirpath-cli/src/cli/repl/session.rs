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
use crate::FhirPathEngine;
use crate::analyzer::{AnalyzerConfig, FhirPathAnalyzer};
use crate::model::value::FhirPathValue;

/// Main REPL session that handles user interaction
pub struct ReplSession {
    engine: FhirPathEngine,
    analyzer: FhirPathAnalyzer,
    editor: Editor<FhirPathCompleter, FileHistory>,
    current_resource: Option<FhirPathValue>,
    variables: HashMap<String, FhirPathValue>,
    config: ReplConfig,
    formatter: DisplayFormatter,
    help_system: HelpSystem,
    interrupt_count: u32,
    // Multi-line expression support
    multiline_buffer: String,
    in_multiline_mode: bool,
}

impl ReplSession {
    /// Create a new REPL session with a pre-created engine
    pub async fn with_engine(engine: FhirPathEngine, config: ReplConfig) -> Result<Self> {
        // Create analyzer
        let analyzer_config = AnalyzerConfig::default();
        let analyzer =
            FhirPathAnalyzer::with_config(engine.model_provider().clone(), analyzer_config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create analyzer: {}", e))?;

        // Initialize editor with history
        let mut editor = Editor::<FhirPathCompleter, FileHistory>::new()
            .context("Failed to create readline editor")?;

        // Set up completer
        let completer = FhirPathCompleter::with_registry(
            engine.model_provider().clone(),
            Some(engine.registry().clone()),
        );
        editor.set_helper(Some(completer));

        // Load history if file is specified
        if let Some(history_path) = &config.history_file {
            if history_path.exists() {
                let _ = editor.load_history(history_path);
            }
        }

        let formatter = DisplayFormatter::new(config.color_output);
        let help_system = HelpSystem::with_registry(engine.registry().clone());

        Ok(Self {
            engine,
            analyzer,
            editor,
            current_resource: None,
            variables: HashMap::new(),
            config,
            formatter,
            help_system,
            interrupt_count: 0,
            multiline_buffer: String::new(),
            in_multiline_mode: false,
        })
    }

    /// Start the main REPL loop
    pub async fn run(&mut self) -> Result<()> {
        self.print_welcome();

        // Cache function names for autocomplete
        self.cache_function_names().await;

        loop {
            // Use different prompt for multi-line mode
            let current_prompt = if self.in_multiline_mode {
                "... "
            } else {
                &self.config.prompt
            };

            match self.editor.readline(current_prompt) {
                Ok(line) => {
                    let line = line.trim();

                    // Handle empty lines
                    if line.is_empty() {
                        if self.in_multiline_mode {
                            // Empty line in multi-line mode - try to evaluate the buffer
                            self.try_evaluate_multiline().await;
                        }
                        continue;
                    }

                    // Handle multi-line continuation
                    if self.in_multiline_mode {
                        self.multiline_buffer.push(' ');
                        self.multiline_buffer.push_str(line);

                        // Check if this completes the expression
                        if self.is_expression_complete(&self.multiline_buffer) {
                            self.try_evaluate_multiline().await;
                        }
                        continue;
                    }

                    // Regular single line processing
                    if self.needs_multiline(line) {
                        // Start multi-line mode
                        self.start_multiline(line);
                        continue;
                    }

                    // Add to history and process normally
                    self.editor
                        .add_history_entry(line)
                        .context("Failed to add history entry")?;

                    match self.process_input(line).await {
                        Ok(Some(output)) => {
                            println!("{}", output);
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
                    if self.in_multiline_mode {
                        // Cancel multi-line mode
                        if self.config.color_output {
                            println!("\n🚫 Multi-line mode cancelled.");
                        } else {
                            println!("\nMulti-line mode cancelled.");
                        }
                        self.reset_multiline();
                        continue;
                    }

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
            ReplCommand::Quit => {
                std::process::exit(0);
            }
        }
    }

    /// Evaluate a FHIRPath expression with enhanced feedback
    async fn evaluate_expression(&self, expression: &str) -> Result<String> {
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
                FhirPathValue::Resource(res) => res.as_json(),
                FhirPathValue::JsonValue(json) => json.as_inner().clone(),
                _ => serde_json::json!({}),
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
            serde_json::json!({})
        };

        // Add timing information for complex expressions
        let start = std::time::Instant::now();

        let result = if self.variables.is_empty() {
            self.engine
                .evaluate(expression, input_json)
                .await
                .with_context(|| format!("Failed to evaluate expression: '{}'", expression))?
        } else {
            // Convert our variables to the format expected by the engine
            let variables: std::collections::HashMap<String, FhirPathValue> = self
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            self.engine
                .evaluate_with_variables(expression, input_json, variables)
                .await
                .with_context(|| {
                    format!(
                        "Failed to evaluate expression with variables: '{}'",
                        expression
                    )
                })?
        };

        let duration = start.elapsed();
        let mut output = self
            .formatter
            .format_result(&result, self.config.show_types);

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
        let resource = FhirPathValue::resource_from_json(json);
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
    async fn parse_variable_value(&self, value: &str) -> Result<(FhirPathValue, String)> {
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
                        let fhir_val = FhirPathValue::from(string_val.clone());
                        Ok((fhir_val, format!("\"{}\"", string_val)))
                    } else if value.starts_with('\'') && value.ends_with('\'') {
                        // FHIRPath string
                        let string_val = value[1..value.len() - 1].to_string();
                        let fhir_val = FhirPathValue::from(string_val.clone());
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
                let fhir_val = FhirPathValue::resource_from_json(json_val.clone());
                let display =
                    serde_json::to_string(&json_val).unwrap_or_else(|_| value.to_string());
                Ok((fhir_val, display))
            } else {
                // Treat as string literal
                let fhir_val = FhirPathValue::from(value);
                Ok((fhir_val, format!("'{}'", value)))
            }
        }
    }

    /// Try to evaluate a string as a FHIRPath expression
    async fn try_evaluate_as_expression(&self, expression: &str) -> Result<FhirPathValue> {
        let input_json = if let Some(resource) = &self.current_resource {
            match resource {
                FhirPathValue::Resource(res) => res.as_json(),
                FhirPathValue::JsonValue(json) => json.as_inner().clone(),
                _ => serde_json::json!({}),
            }
        } else {
            // For expressions that don't need context like literals or today/now
            serde_json::json!({})
        };

        if self.variables.is_empty() {
            self.engine
                .evaluate(expression, input_json)
                .await
                .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))
        } else {
            let variables: std::collections::HashMap<String, FhirPathValue> = self
                .variables
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            self.engine
                .evaluate_with_variables(expression, input_json, variables)
                .await
                .map_err(|e| anyhow::anyhow!("Evaluation error: {}", e))
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
        match self.analyzer.analyze(expression).await {
            Ok(analysis) => {
                if analysis.type_annotations.is_empty() {
                    Ok(format!(
                        "No type information available for '{}'",
                        expression
                    ))
                } else {
                    let types: Vec<String> = analysis
                        .type_annotations
                        .values()
                        .filter_map(|info| info.fhir_path_type.clone())
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
    async fn explain_expression(&self, expression: &str) -> Result<String> {
        // Try to parse and analyze the expression to provide insights
        match self.analyzer.analyze(expression).await {
            Ok(analysis) => {
                let mut explanation = vec![
                    format!("Expression analysis for: {}", expression),
                    "─".repeat(50),
                ];

                if !analysis.type_annotations.is_empty() {
                    let types: Vec<String> = analysis
                        .type_annotations
                        .values()
                        .filter_map(|info| info.fhir_path_type.clone())
                        .collect();
                    if !types.is_empty() {
                        explanation.push(format!("Return type(s): {}", types.join(" | ")));
                    }
                }

                if !analysis.validation_errors.is_empty() {
                    explanation.push("Validation issues:".to_string());
                    for error in &analysis.validation_errors {
                        explanation.push(format!("  ⚠️  {}", error.message));
                    }
                }

                if !analysis.function_calls.is_empty() {
                    explanation.push("Function calls found:".to_string());
                    for func in &analysis.function_calls {
                        explanation.push(format!("  📞  {}", func.function_name));
                    }
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
        let function_names = self.engine.registry().function_names().await;
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

    /// Start multi-line mode with initial line
    fn start_multiline(&mut self, initial_line: &str) {
        self.in_multiline_mode = true;
        self.multiline_buffer = if initial_line.ends_with('\\') {
            initial_line[..initial_line.len() - 1].to_string()
        } else {
            initial_line.to_string()
        };

        if self.config.color_output {
            println!(
                "📝 Multi-line mode started. Press Enter on empty line to evaluate, or Ctrl+C to cancel."
            );
        } else {
            println!(
                "Multi-line mode started. Press Enter on empty line to evaluate, or Ctrl+C to cancel."
            );
        }
    }

    /// Try to evaluate the multi-line buffer
    async fn try_evaluate_multiline(&mut self) {
        if !self.multiline_buffer.is_empty() {
            let expression = self.multiline_buffer.clone();

            // Add to history
            if let Err(e) = self.editor.add_history_entry(&expression) {
                eprintln!("Warning: Failed to add history entry: {}", e);
            }

            // Evaluate the expression
            match self.process_input(&expression).await {
                Ok(Some(output)) => {
                    println!("{}", output);
                }
                Ok(None) => {
                    // Command handled, no output
                }
                Err(e) => {
                    println!("{}", self.formatter.format_error(&e));
                }
            }
        }

        // Reset multi-line state
        self.reset_multiline();
    }

    /// Reset multi-line state
    fn reset_multiline(&mut self) {
        self.in_multiline_mode = false;
        self.multiline_buffer.clear();
        self.interrupt_count = 0; // Reset interrupt count when exiting multi-line
    }
}
