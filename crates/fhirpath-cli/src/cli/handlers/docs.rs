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

//! Handler for the docs command

use crate::cli::context::CliContext;
use colored::Colorize;
use octofhir_fhirpath::core::error_code::ErrorCode;
use std::io::Write;
use std::process;

/// Handle the docs command
pub fn handle_docs(error_code: &str, context: &CliContext) {
    let code_num = if error_code.starts_with("FP") || error_code.starts_with("fp") {
        error_code[2..].parse::<u16>()
    } else {
        error_code.parse::<u16>()
    };

    match code_num {
        Ok(num) => {
            let error_code_obj = ErrorCode::new(num);
            let error_info = error_code_obj.info();
            let docs_url = error_code_obj.docs_url();

            // Display error information
            if context.output_format != crate::cli::output::OutputFormat::Json {
                if context.use_colors() {
                    // Colored output (Rust-like style)
                    println!(
                        "{}: {}",
                        format!("error[{}]", error_code_obj.code_str()).red().bold(),
                        error_info.title.bold()
                    );

                    println!("\n{}", "Description:".cyan().bold());
                    println!("  {}", error_info.description);

                    println!("\n{}", "Help:".cyan().bold());
                    println!("  {}", error_info.help);

                    println!("\n{}", "Category:".cyan().bold());
                    println!("  {:?} errors", error_code_obj.category());

                    println!(
                        "\n{} {}",
                        "Online documentation:".green().bold(),
                        docs_url.underline().blue()
                    );
                } else {
                    // Non-colored output
                    println!("error[{}]: {}", error_code_obj.code_str(), error_info.title);

                    println!("\nDescription:");
                    println!("  {}", error_info.description);

                    println!("\nHelp:");
                    println!("  {}", error_info.help);

                    println!("\nCategory:");
                    println!("  {:?} errors", error_code_obj.category());

                    println!("\nOnline documentation: {docs_url}");
                }

                // Ask if user wants to open browser
                if !context.quiet {
                    println!("\nWould you like to open the online documentation? [y/N]");
                    std::io::stdout().flush().unwrap();

                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        let input = input.trim().to_lowercase();
                        if input == "y" || input == "yes" {
                            open_browser(&docs_url);
                        }
                    }
                }
            } else {
                // JSON output format
                use serde_json::json;
                let json_output = json!({
                    "error_code": error_code_obj.code_str(),
                    "title": error_info.title,
                    "description": error_info.description,
                    "help": error_info.help,
                    "category": format!("{:?}", error_code_obj.category()),
                    "docs_url": docs_url
                });
                println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
            }
        }
        Err(_) => {
            if context.use_colors() {
                eprintln!(
                    "{}: Invalid error code format: '{}'",
                    "error".red().bold(),
                    error_code
                );
                eprintln!("{}: Expected format: FP0001 or 1", "help".cyan().bold());
            } else {
                eprintln!("error: Invalid error code format: '{error_code}'");
                eprintln!("help: Expected format: FP0001 or 1");
            }
            process::exit(1);
        }
    }
}

fn open_browser(url: &str) {
    use std::process::Command;

    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", url]).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    };

    match result {
        Ok(_) => {
            println!("Opened documentation in your default browser.");
        }
        Err(e) => {
            eprintln!("Failed to open browser: {e}. Please visit: {url}");
        }
    }
}
