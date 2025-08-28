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

//! Interactive Read-Eval-Print Loop (REPL) for FHIRPath expressions

pub mod commands;
pub mod completion;
pub mod display;
pub mod session;
pub mod help;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use serde_json::Value as JsonValue;

use crate::model::provider::ModelProvider;

pub use session::ReplSession;
pub use commands::ReplCommand;

/// Configuration for the REPL session
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Prompt string to display
    pub prompt: String,
    /// Maximum number of history entries
    pub history_size: usize,
    /// Whether to auto-save history
    pub auto_save_history: bool,
    /// Whether to use colored output
    pub color_output: bool,
    /// Whether to show type information
    pub show_types: bool,
    /// History file path
    pub history_file: Option<PathBuf>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "fhirpath> ".to_string(),
            history_size: 1000,
            auto_save_history: true,
            color_output: !std::env::var("NO_COLOR").is_ok(),
            show_types: false,
            history_file: None,
        }
    }
}

/// Start the REPL with the given configuration
pub async fn start_repl(
    model_provider: Arc<dyn ModelProvider>,
    config: ReplConfig,
    initial_resource: Option<JsonValue>,
    initial_variables: Vec<(String, String)>,
) -> Result<()> {
    use crate::FhirPathEngine;
    use crate::registry::create_standard_registry;
    
    // Create the engine asynchronously
    let registry = Arc::new(create_standard_registry().await);
    let engine = FhirPathEngine::new(registry, model_provider);
    
    let mut session = ReplSession::with_engine(engine, config)?;

    // Load initial resource if provided
    if let Some(resource) = initial_resource {
        session.load_resource_from_json(resource)?;
    }

    // Set initial variables
    for (name, value) in initial_variables {
        session.set_variable(&name, &value)?;
    }

    session.run().await
}