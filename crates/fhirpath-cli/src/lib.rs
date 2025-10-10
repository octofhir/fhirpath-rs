#![allow(clippy::uninlined_format_args)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::new_without_default)]
#![allow(clippy::manual_strip)]
#![allow(clippy::len_zero)]
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

//! FHIRPath CLI Library
//!
//! This library provides command-line tools for FHIRPath expression evaluation.
//!
//! ## Features
//!
//! - `cli` - Core CLI functionality (default)
//! - `repl` - Interactive REPL (default)
//! - `tui` - Terminal User Interface
//! - `server` - HTTP API server
//! - `watch` - File watching for auto-reload
//! - `all` - All features
//!
//! ## Example
//!
//! ```no_run
//! use fhirpath_cli::{EmbeddedModelProvider, cli::context::CliContext};
//! use octofhir_fhir_model::provider::FhirVersion;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let model_provider = Arc::new(EmbeddedModelProvider::new(FhirVersion::R4));
//!     // Use the CLI programmatically
//!     Ok(())
//! }
//! ```

pub mod cli;

#[cfg(feature = "tui")]
pub mod tui;

// Re-export CLI functionality (avoiding ambiguous glob re-exports)
pub use cli::{Cli, Commands};

// Re-export TUI functionality (only when tui feature is enabled)
#[cfg(feature = "tui")]
pub use tui::{TuiConfig, check_terminal_capabilities, start_tui};

// Re-export model providers
pub use octofhir_fhir_model::EmptyModelProvider;
pub use octofhir_fhirschema::EmbeddedSchemaProvider as EmbeddedModelProvider;
