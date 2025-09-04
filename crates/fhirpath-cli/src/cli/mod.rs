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

pub mod diagnostics;
pub mod diagnostic_demo;
pub mod output;
// TODO: Re-enable after improving implementation
// pub mod repl;
// pub mod server;

use clap::{Parser, Subcommand};
use output::OutputFormat;

#[derive(Parser)]
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
    #[arg(
        long,
        short = 'o',
        value_enum,
        default_value = "raw"
    )]
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

#[derive(Subcommand)]
pub enum Commands {
    /// Evaluate FHIRPath expression against a FHIR resource
    Evaluate {
        /// FHIRPath expression to evaluate
        expression: String,
        /// JSON file containing FHIR resource, or JSON string directly (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
        /// Initial variables to set in format var=value (can be used multiple times)
        #[arg(short, long = "variable")]
        variables: Vec<String>,
        /// Pretty-print JSON output (only applies to raw format)
        #[arg(short, long)]
        pretty: bool,
    },
    /// Parse and validate FHIRPath expression syntax
    Parse {
        /// FHIRPath expression to parse
        expression: String,
    },
    /// Validate FHIRPath expression syntax (alias for parse)
    Validate {
        /// FHIRPath expression to validate
        expression: String,
    },
    /// Analyze FHIRPath expressions with comprehensive FHIR field validation
    Analyze {
        /// FHIRPath expression to analyze
        expression: String,
        /// Initial variables to set in format var=value (can be used multiple times)
        #[arg(short, long = "variable")]
        variables: Vec<String>,
        /// Only validate, don't analyze types
        #[arg(long)]
        validate_only: bool,
        /// Disable type inference
        #[arg(long)]
        no_inference: bool,
    },
    // TODO: Re-enable REPL subcommand after improving implementation
    // /// Start interactive FHIRPath REPL
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
    // TODO: Re-enable server subcommand after fixing dependencies
    // /// Start HTTP server with web interface
    // Server {
    //     /// Port to bind the server to
    //     #[arg(short, long, default_value = "8080")]
    //     port: u16,
    //     /// Directory for JSON file storage
    //     #[arg(short, long, default_value = "./storage")]
    //     storage: PathBuf,
    //     /// Host to bind to
    //     #[arg(long, default_value = "127.0.0.1")]
    //     host: String,
    //     /// Enable CORS for all origins (development mode)
    //     #[arg(long)]
    //     cors_all: bool,
    // },
    /// Demonstrate Ariadne diagnostic integration
    DiagnosticDemo {
        /// FHIRPath expression to parse with diagnostics (optional)
        expression: Option<String>,
        /// Show different diagnostic types
        #[arg(long)]
        show_types: bool,
    },
}
