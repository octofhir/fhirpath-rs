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

//! CLI module for FHIRPath evaluation and analysis

pub mod ast;
pub mod diagnostics;
pub mod output;
pub mod repl;
// pub mod server; // Commented out temporarily - will return later

use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser, Clone)]
#[command(name = "octofhir-fhirpath")]
#[command(about = "OctoFHIR FHIRPath CLI")]
#[command(version)]
#[command(author = "OctoFHIR Team <funyloony@gmail.com>")]
pub struct Cli {
    /// FHIR version to use (r4, r4b, r5)
    #[arg(long, value_name = "VERSION", default_value = "r4")]
    pub fhir_version: String,

    /// Additional FHIR packages to load (format: package@version)
    #[arg(long = "package", value_name = "PACKAGE")]
    pub packages: Vec<String>,

    /// Output format
    #[arg(long, short = 'o', value_enum, default_value = "pretty")]
    pub output_format: OutputFormat,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Suppress informational messages
    #[arg(long, short)]
    pub quiet: bool,

    /// Verbose output with additional details
    #[arg(long, short)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Evaluate FHIRPath expression against a FHIR resource
    Evaluate {
        /// FHIRPath expression to evaluate
        expression: String,
        /// JSON file containing FHIR resource, or JSON string directly (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        /// Initial variables to set in format var=value (can be used multiple times)
        #[arg(long = "var", short = 'V')]
        variables: Vec<String>,
        /// Pretty-print JSON output (only applies to raw format)
        #[arg(short, long)]
        pretty: bool,
        /// Output format
        #[arg(long, short = 'o', value_enum)]
        output_format: Option<OutputFormat>,
        /// Disable colored output
        #[arg(long)]
        no_color: bool,
        /// Suppress informational messages
        #[arg(long, short = 'q')]
        quiet: bool,
        /// Verbose output with additional details
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Validate FHIRPath expression syntax (alias for parse)
    Validate {
        /// FHIRPath expression to validate
        expression: String,
        /// Output format
        #[arg(long, short = 'o', value_enum)]
        output_format: Option<OutputFormat>,
        /// Disable colored output
        #[arg(long)]
        no_color: bool,
        /// Suppress informational messages
        #[arg(long, short = 'q')]
        quiet: bool,
        /// Verbose output with additional details
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Analyze FHIRPath expressions with comprehensive FHIR field validation
    Analyze {
        /// FHIRPath expression to analyze
        expression: String,
        /// Initial variables to set in format var=value (can be used multiple times)
        #[arg(long = "var", short = 'V')]
        variables: Vec<String>,
        /// Only validate, don't analyze types
        #[arg(long)]
        validate_only: bool,
        /// Disable type inference
        #[arg(long)]
        no_inference: bool,
        /// Output format
        #[arg(long, short = 'o', value_enum)]
        output_format: Option<OutputFormat>,
        /// Disable colored output
        #[arg(long)]
        no_color: bool,
        /// Suppress informational messages
        #[arg(long, short = 'q')]
        quiet: bool,
        /// Verbose output with additional details
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Open documentation for FHIRPath error codes
    Docs {
        /// Error code to get documentation for (e.g., FP0001, FP0055)
        error_code: String,
    },
    // /// Start interactive FHIRPath REPL (commented out during Phase 1)
    // Repl {
    //     /// JSON file containing FHIR resource to load initially
    //     #[arg(short, long)]
    //     input: Option<String>,
    //     /// Initial variables to set in format var=value (can be used multiple times)
    //     #[arg(short, long = "variable")]
    //     variables: Vec<String>,
    //     /// History file to use (default: ~/.fhirpath_history)
    //     #[arg(long)]
    //     history_file: Option<String>,
    //     /// Maximum number of history entries (default: 1000)
    //     #[arg(long, default_value = "1000")]
    //     history_size: usize,
    // },
    /// Registry information for functions and operators
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    // /// Start HTTP server with web interface
    // Server {
    //     /// Port to bind the server to
    //     #[arg(short, long, default_value = "8084")]
    //     port: u16,
    //     /// Directory for JSON file storage
    //     #[arg(short, long, default_value = "./storage")]
    //     storage: std::path::PathBuf,
    //     /// Host to bind to
    //     #[arg(long, default_value = "127.0.0.1")]
    //     host: String,
    //     /// Enable CORS for all origins (development mode)
    //     #[arg(long)]
    //     cors_all: bool,
    //     /// Maximum request body size in MB
    //     #[arg(long, default_value = "60")]
    //     max_body_size: u64,
    //     /// Expression execution timeout in seconds
    //     #[arg(long, default_value = "30")]
    //     timeout: u64,
    //     /// Rate limit: requests per minute per IP
    //     #[arg(long, default_value = "100")]
    //     rate_limit: u32,
    //     /// Run server without web UI (API-only mode)
    //     #[arg(long)]
    //     no_ui: bool,
    // },
    // /// Start Terminal User Interface (TUI) - Advanced multi-panel REPL
    // Tui {
    //     /// JSON file containing FHIR resource to load initially
    //     #[arg(short, long)]
    //     input: Option<String>,
    //     /// Initial variables to set in format var=value (can be used multiple times)
    //     #[arg(short, long = "variable")]
    //     variables: Vec<String>,
    //     /// Configuration file path (default: ~/.config/fhirpath-tui/config.toml)
    //     #[arg(long)]
    //     config: Option<String>,
    //     /// Theme to use (dark, light, high_contrast)
    //     #[arg(long, default_value = "dark")]
    //     theme: String,
    //     /// Disable mouse support
    //     #[arg(long)]
    //     no_mouse: bool,
    //     /// Disable syntax highlighting
    //     #[arg(long)]
    //     no_syntax_highlighting: bool,
    //     /// Disable auto-completion
    //     #[arg(long)]
    //     no_auto_completion: bool,
    //     /// Enable performance monitoring
    //     #[arg(long)]
    //     performance_monitoring: bool,
    //     /// Check terminal capabilities and exit
    //     #[arg(long)]
    //     check_terminal: bool,
    // },
}

#[derive(Subcommand, Clone)]
pub enum RegistryCommands {
    /// List available functions or operators
    List {
        /// What to list: functions or operators
        #[arg(value_enum)]
        target: RegistryTarget,
        /// Filter by category (e.g., Collection, Math, Logical)
        #[arg(long)]
        category: Option<String>,
        /// Search pattern to filter by name or description
        #[arg(long)]
        search: Option<String>,
        /// Output format
        #[arg(long, short = 'o', value_enum)]
        output_format: Option<OutputFormat>,
        /// Disable colored output
        #[arg(long)]
        no_color: bool,
        /// Suppress informational messages
        #[arg(long, short = 'q')]
        quiet: bool,
        /// Verbose output with additional details
        #[arg(long, short = 'v')]
        verbose: bool,
    },
    /// Show detailed information about a specific function or operator
    Show {
        /// Name of the function or operator to show
        name: String,
        /// Whether to show function or operator details
        #[arg(long, value_enum, default_value = "auto")]
        target: RegistryShowTarget,
        /// Output format
        #[arg(long, short = 'o', value_enum)]
        output_format: Option<OutputFormat>,
        /// Disable colored output
        #[arg(long)]
        no_color: bool,
        /// Suppress informational messages
        #[arg(long, short = 'q')]
        quiet: bool,
        /// Verbose output with additional details
        #[arg(long, short = 'v')]
        verbose: bool,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum RegistryTarget {
    Functions,
    Operators,
}

#[derive(clap::ValueEnum, Clone)]
pub enum RegistryShowTarget {
    Auto,
    Function,
    Operator,
}
