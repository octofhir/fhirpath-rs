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

//! Terminal User Interface (TUI) for FHIRPath REPL
//! 
//! This module provides a rich, multi-panel terminal interface using Ratatui
//! that replaces the simple rustyline-based REPL with advanced features:
//! 
//! - Multi-panel layout with input, output, diagnostics, variables, and history
//! - Real-time syntax highlighting and validation
//! - Interactive auto-completion with context awareness  
//! - Professional visual design with configurable themes
//! - Mouse and keyboard navigation support
//! 
//! ## Architecture Overview
//! 
//! The TUI is built around several core concepts:
//! 
//! - **Application State**: Centralized state management for all panels
//! - **Component System**: Modular, reusable UI components  
//! - **Event Handling**: Unified event processing with key bindings
//! - **Layout Management**: Flexible, configurable panel layouts
//! - **Theme System**: Customizable color schemes and visual styles

pub mod app;
pub mod components;
pub mod config;
pub mod events;
pub mod layout;
pub mod themes;
pub mod utils;

pub use app::{TuiApp, AppState, AppMode};
pub use config::{TuiConfig, FeatureFlags};
pub use events::{EventHandler, KeyBindings, TuiAction};
pub use layout::{LayoutManager, PanelType, PanelLayout, LayoutConfig};
pub use themes::{TuiTheme, ColorScheme};

use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use serde_json::Value as JsonValue;

use octofhir_fhirpath::core::ModelProvider;

/// Main entry point for starting the TUI REPL
pub async fn start_tui(
    model_provider: Arc<dyn ModelProvider>,
    config: TuiConfig,
    initial_resource: Option<JsonValue>,
    initial_variables: Vec<(String, String)>,
) -> Result<()> {
    // Initialize terminal
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Create and run the TUI application
    let mut app = TuiApp::new(model_provider, config, terminal).await?;

    // Load initial resource if provided
    if let Some(resource) = initial_resource {
        app.load_resource_from_json(resource)?;
    }

    // Set initial variables
    for (name, value) in initial_variables {
        app.set_variable(&name, &value).await?;
    }

    // Run the main event loop
    let result = app.run().await;

    // Cleanup terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    io::stdout().execute(DisableMouseCapture)?;

    result
}

/// Utility function to check terminal capabilities
pub fn check_terminal_capabilities() -> Result<()> {
    use crossterm::terminal::{size, supports_keyboard_enhancement};
    
    let (width, height) = size().context("Failed to get terminal size")?;
    
    if width < 80 || height < 24 {
        anyhow::bail!(
            "Terminal too small for TUI. Minimum size is 80x24, current size is {}x{}",
            width, height
        );
    }

    // Check for advanced features
    let keyboard_enhanced = supports_keyboard_enhancement().unwrap_or(false);
    
    if !keyboard_enhanced {
        eprintln!("Warning: Terminal does not support keyboard enhancement. Some key combinations may not work.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_check() {
        // This test may fail in CI environments with small terminals
        // It's mainly for local development validation
        match check_terminal_capabilities() {
            Ok(_) => println!("Terminal capabilities check passed"),
            Err(e) => println!("Terminal capabilities check failed: {}", e),
        }
    }
}