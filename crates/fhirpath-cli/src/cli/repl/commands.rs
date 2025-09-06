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

//! REPL command parsing and definitions

use anyhow::{Result, anyhow};

/// Available REPL commands
#[derive(Debug, Clone)]
pub enum ReplCommand {
    /// Load a FHIR resource from file
    Load { path: String },
    /// Set a variable
    Set { name: String, value: String },
    /// Unset a variable
    Unset { name: String },
    /// List all variables
    Vars,
    /// Show current resource
    Resource,
    /// Show type information for expression
    Type { expression: String },
    /// Explain expression evaluation
    Explain { expression: String },
    /// Show help
    Help { function: Option<String> },
    /// Show command history
    History,
    /// Analyze expression with diagnostics
    Analyze { expression: String },
    /// Validate expression syntax
    Validate { expression: String },
    /// Exit REPL
    Quit,
}

impl ReplCommand {
    /// Parse a command line starting with ':'
    pub fn parse(line: &str) -> Result<Option<Self>> {
        if !line.starts_with(':') {
            return Ok(None);
        }

        let line = &line[1..]; // Remove the ':'
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            return Err(anyhow!("Empty command"));
        }

        let command = match parts[0] {
            "load" => {
                if parts.len() != 2 {
                    return Err(anyhow!("Usage: :load <file>"));
                }
                ReplCommand::Load {
                    path: parts[1].to_string(),
                }
            }
            "set" => {
                if parts.len() < 3 {
                    return Err(anyhow!("Usage: :set <name> <value>"));
                }
                let name = parts[1].to_string();
                let value = parts[2..].join(" ");
                ReplCommand::Set { name, value }
            }
            "unset" => {
                if parts.len() != 2 {
                    return Err(anyhow!("Usage: :unset <name>"));
                }
                ReplCommand::Unset {
                    name: parts[1].to_string(),
                }
            }
            "vars" => {
                if parts.len() != 1 {
                    return Err(anyhow!("Usage: :vars"));
                }
                ReplCommand::Vars
            }
            "resource" => {
                if parts.len() != 1 {
                    return Err(anyhow!("Usage: :resource"));
                }
                ReplCommand::Resource
            }
            "type" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: :type <expression>"));
                }
                let expression = parts[1..].join(" ");
                ReplCommand::Type { expression }
            }
            "explain" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: :explain <expression>"));
                }
                let expression = parts[1..].join(" ");
                ReplCommand::Explain { expression }
            }
            "help" | "h" => {
                let function = if parts.len() > 1 {
                    Some(parts[1].to_string())
                } else {
                    None
                };
                ReplCommand::Help { function }
            }
            "history" | "hist" => {
                if parts.len() != 1 {
                    return Err(anyhow!("Usage: :history"));
                }
                ReplCommand::History
            }
            "analyze" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: :analyze <expression>"));
                }
                let expression = parts[1..].join(" ");
                ReplCommand::Analyze { expression }
            }
            "validate" => {
                if parts.len() < 2 {
                    return Err(anyhow!("Usage: :validate <expression>"));
                }
                let expression = parts[1..].join(" ");
                ReplCommand::Validate { expression }
            }
            "quit" | "q" | "exit" => {
                if parts.len() != 1 {
                    return Err(anyhow!("Usage: :quit"));
                }
                ReplCommand::Quit
            }
            _ => {
                return Err(anyhow!("Unknown command: {}", parts[0]));
            }
        };

        Ok(Some(command))
    }
}
