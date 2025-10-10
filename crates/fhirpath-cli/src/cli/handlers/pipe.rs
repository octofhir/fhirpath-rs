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

//! Pipe mode handler for Unix-style workflows

use crate::EmbeddedModelProvider;
use crate::cli::context::{CliContext, EngineBuilder};
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use serde_json::Value as JsonValue;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

/// Handle pipe mode: read NDJSON from stdin, evaluate, output to stdout
pub async fn handle_pipe_mode(
    expression: &str,
    variables: &[String],
    context: &CliContext,
    model_provider: &Arc<EmbeddedModelProvider>,
) -> anyhow::Result<()> {
    // Create FHIRPath engine once (reuse for all inputs)
    let engine = EngineBuilder::new()
        .with_model_provider(model_provider.clone())
        .build()
        .await?;

    // Parse expression once
    let parse_result = parse_with_mode(expression, ParsingMode::Fast);
    if !parse_result.success {
        // In pipe mode, write errors to stderr
        eprintln!("Error parsing expression: {}", expression);
        for diag in &parse_result.diagnostics {
            eprintln!("  {}", diag.message);
        }
        return Err(anyhow::anyhow!("Failed to parse expression"));
    }

    // Parse variables
    let mut env = std::collections::HashMap::new();
    for var in variables {
        if let Some((name, value)) = var.split_once('=') {
            env.insert(name.to_string(), value.to_string());
        }
    }

    // Read from stdin line by line (NDJSON format)
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut line_number = 0;
    let mut success_count = 0;
    let mut error_count = 0;

    for line_result in stdin.lock().lines() {
        line_number += 1;

        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line {}: {}", line_number, e);
                error_count += 1;
                continue;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON
        let resource: JsonValue = match serde_json::from_str(&line) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Error parsing JSON on line {}: {}", line_number, e);
                error_count += 1;
                continue;
            }
        };

        // Create context collection
        let model_provider_arc =
            model_provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>;

        let context_collection = match octofhir_fhirpath::Collection::from_json_resource(
            resource.clone(),
            Some(model_provider_arc.clone()),
        )
        .await
        {
            Ok(collection) => collection,
            Err(_) => {
                // Fallback to untyped resource
                octofhir_fhirpath::Collection::single(octofhir_fhirpath::FhirPathValue::resource(
                    resource,
                ))
            }
        };

        let eval_context = octofhir_fhirpath::EvaluationContext::new(
            context_collection,
            model_provider_arc,
            engine.get_terminology_provider(),
            engine.get_validation_provider(),
            None,
        )
        .await;

        // Evaluate
        let result = match engine
            .evaluate_with_metadata(expression, &eval_context)
            .await
        {
            Ok(eval_result) => eval_result.result.value,
            Err(e) => {
                eprintln!("Error evaluating line {}: {}", line_number, e);
                error_count += 1;
                continue;
            }
        };

        // Convert result to JSON and output
        let result_json = result.to_json_value();
        match serde_json::to_string(&result_json) {
            Ok(json_str) => {
                if let Err(e) = writeln!(stdout_lock, "{}", json_str) {
                    eprintln!("Error writing output for line {}: {}", line_number, e);
                    error_count += 1;
                } else {
                    success_count += 1;
                }
            }
            Err(e) => {
                eprintln!("Error serializing result for line {}: {}", line_number, e);
                error_count += 1;
            }
        }
    }

    // Write summary to stderr if not quiet
    if !context.quiet {
        eprintln!();
        eprintln!(
            "Pipe mode complete: {} succeeded, {} failed",
            success_count, error_count
        );
    }

    if error_count > 0 {
        Err(anyhow::anyhow!(
            "Pipe mode completed with {} errors",
            error_count
        ))
    } else {
        Ok(())
    }
}

/// Check if stdin is a pipe (not a terminal)
pub fn is_stdin_pipe() -> bool {
    use std::io::IsTerminal;
    !io::stdin().is_terminal()
}

/// Check if stdout is a pipe (not a terminal)
pub fn is_stdout_pipe() -> bool {
    use std::io::IsTerminal;
    !io::stdout().is_terminal()
}
