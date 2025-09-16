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

//! TUI Application State Management
//!
//! This module defines the main application state structure and provides
//! centralized management for all TUI components and their interactions.

use std::collections::HashMap;
use std::io::Stdout;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{self, Event, MouseEvent};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Frame, Terminal};
use rust_decimal::Decimal;
use serde_json::Value as JsonValue;

use super::components::*;
use super::config::TuiConfig;
use super::events::{EventHandler, KeyBindings, TuiAction};
use super::layout::{LayoutManager, PanelType};
use super::themes::TuiTheme;

// use octofhir_fhirpath::analyzer::StaticAnalyzer; // Removed
use octofhir_fhirpath::core::ModelProvider;
use octofhir_fhirpath::diagnostics::{AriadneDiagnostic, DiagnosticEngine};
use octofhir_fhirpath::{FhirPathEngine, FhirPathValue};

/// Main TUI application state and event loop manager
pub struct TuiApp {
    /// Current application mode
    mode: AppMode,

    /// Track Ctrl+C presses for double-press exit
    ctrl_c_count: u8,
    last_ctrl_c: Instant,

    /// Panel layout and focus management
    layout: LayoutManager,

    /// UI components for each panel
    components: ComponentManager,

    /// Event handling and key bindings
    event_handler: EventHandler,

    /// FHIRPath evaluation engine
    engine: FhirPathEngine,

    /// Static analyzer for real-time validation
    // analyzer: StaticAnalyzer, // Removed

    /// Diagnostic engine for error formatting
    diagnostic_engine: DiagnosticEngine,

    /// Terminal interface
    terminal: Terminal<CrosstermBackend<Stdout>>,

    /// Current application state
    state: AppState,

    /// Configuration
    config: TuiConfig,

    /// Theme for visual styling
    theme: TuiTheme,

    /// Should the application exit
    should_quit: bool,

    /// Last render time for performance tracking
    last_render: Instant,
}

/// Current application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMode {
    /// Normal REPL operation
    Normal,
    /// Main command menu
    Menu,
    /// Help system display
    Help,
    /// Settings/configuration panel
    Settings,
    /// Command history browser
    History,
    /// Export functionality
    Export,
    /// Full-screen diagnostics view
    DiagnosticsDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuCommand {
    Evaluate,
    Analyze,
    CheckSyntax,
    Documentation,
    LoadFile,
    Exit,
}

/// Centralized application state
#[derive(Debug)]
pub struct AppState {
    /// Currently focused panel
    pub focused_panel: PanelType,

    /// Current FHIR resource loaded in context
    pub current_resource: Option<FhirPathValue>,

    /// Variable definitions and values
    pub variables: HashMap<String, FhirPathValue>,

    /// Current expression being edited
    pub current_expression: String,

    /// Most recent evaluation result
    pub last_result: Option<octofhir_fhirpath::Collection>,

    /// Current diagnostics from analysis
    pub diagnostics: Vec<AriadneDiagnostic>,

    /// Expression evaluation history
    pub evaluation_history: Vec<HistoryEntry>,

    /// Auto-completion suggestions
    pub completions: Vec<CompletionItem>,

    /// Whether real-time validation is enabled
    pub live_validation: bool,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// Currently selected menu command (when in Menu mode)
    pub selected_menu_command: MenuCommand,
}

/// Historical evaluation entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: Instant,
    pub expression: String,
    pub result: Result<octofhir_fhirpath::Collection, String>,
    pub execution_time: Duration,
}

/// Auto-completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub text: String,
    pub display: String,
    pub kind: CompletionKind,
    pub documentation: Option<String>,
    pub insert_range: Option<(usize, usize)>,
}

/// Type of completion suggestion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    Function,
    Property,
    ResourceType,
    Variable,
    Keyword,
    Operator,
}

/// Performance tracking metrics
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub last_evaluation_time: Option<Duration>,
    pub last_analysis_time: Option<Duration>,
    pub last_render_time: Option<Duration>,
    pub total_evaluations: u64,
    pub average_evaluation_time: Duration,
}

impl TuiApp {
    /// Create a new TUI application
    pub async fn new(
        model_provider: Arc<dyn ModelProvider>,
        config: TuiConfig,
        terminal: Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<Self> {
        // Create engine and analyzer
        let registry = Arc::new(octofhir_fhirpath::create_empty_registry());
        let engine = FhirPathEngine::new(registry.clone(), model_provider.clone()).await?;
        // let analyzer = StaticAnalyzer::new(registry, model_provider); // Removed

        // Create diagnostic engine with theme support
        let diagnostic_engine =
            DiagnosticEngine::with_colors(config.theme.diagnostic_colors.clone());

        // Initialize layout and components
        let layout = LayoutManager::new(config.layout.clone());
        let components = ComponentManager::new(&config, &engine /*, &analyzer*/).await?;

        // Set up event handling
        let event_handler = EventHandler::new(KeyBindings::default());

        // Initialize application state
        let state = AppState {
            focused_panel: PanelType::Input,
            current_resource: None,
            variables: HashMap::new(),
            current_expression: String::new(),
            last_result: None,
            diagnostics: Vec::new(),
            evaluation_history: Vec::new(),
            completions: Vec::new(),
            live_validation: config.features.real_time_validation,
            performance: PerformanceMetrics::default(),
            selected_menu_command: MenuCommand::Evaluate,
        };

        Ok(Self {
            mode: AppMode::Normal,
            ctrl_c_count: 0,
            last_ctrl_c: Instant::now(),
            layout,
            components,
            event_handler,
            engine,
            // analyzer,
            diagnostic_engine,
            terminal,
            state,
            config: config.clone(),
            theme: config.theme,
            should_quit: false,
            last_render: Instant::now(),
        })
    }

    /// Run the main event loop
    pub async fn run(&mut self) -> Result<()> {
        // Initial render
        self.render_frame()?;

        loop {
            // Handle events with timeout for responsiveness
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        // Handle Ctrl+C for exit (double press within 2 seconds)
                        if key_event.code == crossterm::event::KeyCode::Char('c')
                            && key_event.modifiers == crossterm::event::KeyModifiers::CONTROL
                        {
                            let now = Instant::now();
                            if now.duration_since(self.last_ctrl_c) < Duration::from_secs(2) {
                                self.ctrl_c_count += 1;
                            } else {
                                self.ctrl_c_count = 1;
                            }
                            self.last_ctrl_c = now;

                            if self.ctrl_c_count >= 2 {
                                break; // Exit the application
                            }
                            // Show exit message on first Ctrl+C
                            continue;
                        }

                        // Handle special keys based on mode
                        match self.mode {
                            AppMode::Menu => {
                                if self.handle_menu_key(key_event).await? {
                                    break;
                                }
                            }
                            _ => {
                                // Check for global keys that work in all modes
                                if self.handle_global_key(key_event).await? {
                                    break;
                                }

                                // First check for global key bindings
                                let action =
                                    self.event_handler.handle_key_event(key_event, &self.state);

                                // If no action was matched, forward the event to the focused component
                                if matches!(action, TuiAction::NoAction) {
                                    let component_result = self
                                        .components
                                        .handle_key_event(key_event, &mut self.state);
                                    if self.handle_component_result(component_result).await? {
                                        break;
                                    }
                                } else {
                                    // Handle the mapped action
                                    if self.handle_action(action).await? {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        self.handle_mouse_event(mouse_event)?;
                    }
                    Event::Resize(width, height) => {
                        self.handle_resize(width, height)?;
                    }
                    _ => {}
                }
            }

            // Update components and state
            self.update_components().await?;

            // Render if needed
            if self.should_render() {
                self.render_frame()?;
                self.last_render = Instant::now();
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle component result
    async fn handle_component_result(&mut self, result: ComponentResult) -> Result<bool> {
        match result {
            ComponentResult::Handled => Ok(false),
            ComponentResult::NotHandled => Ok(false),
            ComponentResult::SwitchFocus(panel) => {
                let old_panel = self.state.focused_panel;
                self.state.focused_panel = panel;
                self.components
                    .handle_focus_change(old_panel, panel, &mut self.state);
                Ok(false)
            }
            ComponentResult::ExitApp => Ok(true),
            ComponentResult::ExecuteExpression => {
                self.execute_current_expression().await?;
                self.analyze_current_expression().await?;
                Ok(false)
            }
            ComponentResult::UpdateExpression(expr) => {
                self.state.current_expression = expr;
                if self.state.live_validation {
                    self.analyze_current_expression().await?;
                }
                Ok(false)
            }
            ComponentResult::ShowCompletions => {
                // TODO: Implement completions
                Ok(false)
            }
            ComponentResult::LoadResource(path) => {
                self.load_resource_from_file(&path).await?;
                Ok(false)
            }
            ComponentResult::SetVariable(name, value) => {
                // Parse the value as FhirPathValue
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&value) {
                    let fhirpath_value = match json_value {
                        serde_json::Value::String(s) => FhirPathValue::String(s.into(), None, None),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                FhirPathValue::Integer(i, None, None)
                            } else if let Some(f) = n.as_f64() {
                                FhirPathValue::Decimal(
                                    Decimal::try_from(f).unwrap_or_else(|_| Decimal::from(0)), None, None
                                )
                            } else {
                                FhirPathValue::String(value.into(), None, None)
                            }
                        }
                        serde_json::Value::Bool(b) => FhirPathValue::Boolean(b, None, None),
                        _ => FhirPathValue::String(value.into(), None, None),
                    };
                    self.state.variables.insert(name, fhirpath_value);
                } else {
                    // Fallback to string value
                    self.state
                        .variables
                        .insert(name, FhirPathValue::String(value.into(), None, None));
                }
                Ok(false)
            }
            ComponentResult::UnsetVariable(name) => {
                self.state.variables.remove(&name);
                Ok(false)
            }
            ComponentResult::ToggleDiagnosticDetails => {
                self.mode = if self.mode == AppMode::DiagnosticsDetail {
                    AppMode::Normal
                } else {
                    AppMode::DiagnosticsDetail
                };
                Ok(false)
            }
            ComponentResult::EditVariable(_name) => {
                // TODO: Implement variable editing
                Ok(false)
            }
            ComponentResult::ClearHistory => {
                self.state.evaluation_history.clear();
                Ok(false)
            }
            ComponentResult::LoadFromHistory(index) => {
                if let Some(entry) = self.state.evaluation_history.get(index) {
                    self.state.current_expression = entry.expression.clone();
                }
                Ok(false)
            }
        }
    }

    /// Handle a TUI action
    async fn handle_action(&mut self, action: TuiAction) -> Result<bool> {
        match action {
            TuiAction::Quit => {
                self.should_quit = true;
                Ok(true)
            }
            TuiAction::FocusPanel(panel) => {
                self.state.focused_panel = panel;
                self.layout.set_focused_panel(panel);
                Ok(false)
            }
            TuiAction::ExecuteExpression => {
                self.execute_current_expression().await?;
                Ok(false)
            }
            TuiAction::ToggleMode(mode) => {
                self.mode = if self.mode == mode {
                    AppMode::Normal
                } else {
                    mode
                };
                Ok(false)
            }
            TuiAction::UpdateExpression(expr) => {
                self.state.current_expression = expr;
                if self.state.live_validation {
                    self.analyze_current_expression().await?;
                }
                Ok(false)
            }
            TuiAction::ShowCompletions => {
                self.update_completions().await?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Execute the current expression
    async fn execute_current_expression(&mut self) -> Result<()> {
        let expression = self.state.current_expression.clone();
        if expression.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();

        // Evaluate expression
        let result = match &self.state.current_resource {
            Some(resource) => {
                // Create evaluation context with resource
                let context_collection = octofhir_fhirpath::Collection::single(resource.clone());
                let embedded_provider = octofhir_fhir_model::EmptyModelProvider;
                let mut eval_context =
                    octofhir_fhirpath::EvaluationContext::new(context_collection, std::sync::Arc::new(embedded_provider), None).await;

                // Set variables if any
                for (name, value) in &self.state.variables {
                    eval_context.set_variable(name.clone(), value.clone());
                }

                // Parse and evaluate
                let parse_result = octofhir_fhirpath::parser::parse_with_mode(
                    &expression,
                    octofhir_fhirpath::parser::ParsingMode::Analysis,
                );
                if parse_result.success && parse_result.ast.is_some() {
                    let ast = parse_result.ast.unwrap();
                    self.engine.evaluate_ast(&ast, &eval_context).await
                } else {
                    Err(octofhir_fhirpath::core::FhirPathError::parse_error(
                        octofhir_fhirpath::core::error_code::FP0001,
                        "Parse error",
                        &expression,
                        None,
                    ))
                }
            }
            None => {
                // Create empty evaluation context
                let context_collection = octofhir_fhirpath::Collection::empty();
                let embedded_provider = octofhir_fhir_model::EmptyModelProvider;
                let mut eval_context =
                    octofhir_fhirpath::EvaluationContext::new(context_collection, std::sync::Arc::new(embedded_provider), None).await;

                // Set variables if any
                for (name, value) in &self.state.variables {
                    eval_context.set_variable(name.clone(), value.clone());
                }

                // Parse and evaluate
                let parse_result = octofhir_fhirpath::parser::parse_with_mode(
                    &expression,
                    octofhir_fhirpath::parser::ParsingMode::Analysis,
                );
                if parse_result.success && parse_result.ast.is_some() {
                    let ast = parse_result.ast.unwrap();
                    self.engine.evaluate_ast(&ast, &eval_context).await
                } else {
                    Err(octofhir_fhirpath::core::FhirPathError::parse_error(
                        octofhir_fhirpath::core::error_code::FP0001,
                        "Parse error",
                        &expression,
                        None,
                    ))
                }
            }
        };

        let execution_time = start_time.elapsed();

        // Update performance metrics
        self.state.performance.last_evaluation_time = Some(execution_time);
        self.state.performance.total_evaluations += 1;
        self.state.performance.average_evaluation_time = Duration::from_nanos(
            (self.state.performance.average_evaluation_time.as_nanos() as u64
                * (self.state.performance.total_evaluations - 1)
                + execution_time.as_nanos() as u64)
                / self.state.performance.total_evaluations,
        );

        // Store result and add to history
        let history_entry = match &result {
            Ok(value) => {
                let collection_result = value.value.clone();
                self.state.last_result = Some(collection_result.clone());
                HistoryEntry {
                    timestamp: Instant::now(),
                    expression: expression.clone(),
                    result: Ok(collection_result),
                    execution_time,
                }
            }
            Err(err) => HistoryEntry {
                timestamp: Instant::now(),
                expression: expression.clone(),
                result: Err(err.to_string()),
                execution_time,
            },
        };

        self.state.evaluation_history.push(history_entry);

        // Clear the current expression after successful execution
        if result.is_ok() {
            self.state.current_expression.clear();
        }

        Ok(())
    }

    /// Analyze the current expression for diagnostics
    async fn analyze_current_expression(&mut self) -> Result<()> {
        if self.state.current_expression.is_empty() {
            self.state.diagnostics.clear();
            return Ok(());
        }

        let start_time = Instant::now();

        // Parse first to get AST for analysis
        let parse_result = octofhir_fhirpath::parser::parse_with_mode(
            &self.state.current_expression,
            octofhir_fhirpath::parser::ParsingMode::Analysis,
        );
        // Static analysis not available (StaticAnalyzer removed)
        let analysis_result: Result<(), octofhir_fhirpath::core::FhirPathError> = Ok(());

        self.state.performance.last_analysis_time = Some(start_time.elapsed());

        // Convert analysis results to diagnostics
        self.state.diagnostics = match analysis_result {
            Ok(_) => {
                // Analysis succeeded, no diagnostics
                Vec::new()
            }
            Err(_) => {
                // Parse error - create basic AriadneDiagnostic
                vec![octofhir_fhirpath::diagnostics::AriadneDiagnostic {
                    severity: octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error,
                    error_code: octofhir_fhirpath::core::error_code::FP0001,
                    message: "Syntax error in expression".to_string(),
                    span: 0..self.state.current_expression.len(),
                    help: Some("Check FHIRPath syntax".to_string()),
                    note: None,
                    related: Vec::new(),
                }]
            }
        };

        Ok(())
    }

    /// Load a resource from a file
    async fn load_resource_from_file(&mut self, path: &str) -> Result<()> {
        use anyhow::Context;

        // Read file content
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read file")?;

        // Parse as JSON
        let json_value: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse JSON")?;

        // Convert to FhirPathValue
        let fhirpath_value = self.json_to_fhirpath_value(&json_value);

        // Store as current resource
        self.state.current_resource = Some(fhirpath_value);

        Ok(())
    }

    /// Convert JSON value to FhirPathValue
    fn json_to_fhirpath_value(&self, json: &serde_json::Value) -> FhirPathValue {
        match json {
            serde_json::Value::Null => FhirPathValue::String("null".to_string().into(), None, None),
            serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b, None, None),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    FhirPathValue::Integer(i, None, None)
                } else if let Some(f) = n.as_f64() {
                    FhirPathValue::Decimal(
                        Decimal::try_from(f).unwrap_or_else(|_| Decimal::from(0)), None, None
                    )
                } else {
                    FhirPathValue::String(n.to_string().into(), None, None)
                }
            }
            serde_json::Value::String(s) => FhirPathValue::String(s.clone().into(), None, None),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                // For complex objects, create a Resource value
                FhirPathValue::Resource(std::sync::Arc::new(json.clone()), None, None)
            }
        }
    }

    /// Handle global key events that work in all modes
    async fn handle_global_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // F2 or Alt+M to open menu
            (KeyCode::F(2), KeyModifiers::NONE) | (KeyCode::Char('m'), KeyModifiers::ALT) => {
                self.mode = AppMode::Menu;
                Ok(false)
            }
            // F1 for help
            (KeyCode::F(1), KeyModifiers::NONE) => {
                self.mode = if self.mode == AppMode::Help {
                    AppMode::Normal
                } else {
                    AppMode::Help
                };
                Ok(false)
            }
            // Esc to return to normal mode
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.mode = AppMode::Normal;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Handle menu navigation keys
    async fn handle_menu_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.code, key.modifiers) {
            // Arrow keys for menu navigation
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.state.selected_menu_command = match self.state.selected_menu_command {
                    MenuCommand::Evaluate => MenuCommand::Exit,
                    MenuCommand::Analyze => MenuCommand::Evaluate,
                    MenuCommand::CheckSyntax => MenuCommand::Analyze,
                    MenuCommand::Documentation => MenuCommand::CheckSyntax,
                    MenuCommand::LoadFile => MenuCommand::Documentation,
                    MenuCommand::Exit => MenuCommand::LoadFile,
                };
                Ok(false)
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                self.state.selected_menu_command = match self.state.selected_menu_command {
                    MenuCommand::Evaluate => MenuCommand::Analyze,
                    MenuCommand::Analyze => MenuCommand::CheckSyntax,
                    MenuCommand::CheckSyntax => MenuCommand::Documentation,
                    MenuCommand::Documentation => MenuCommand::LoadFile,
                    MenuCommand::LoadFile => MenuCommand::Exit,
                    MenuCommand::Exit => MenuCommand::Evaluate,
                };
                Ok(false)
            }
            // Enter to execute selected command
            (KeyCode::Enter, KeyModifiers::NONE) => {
                match self.state.selected_menu_command {
                    MenuCommand::Evaluate => {
                        self.mode = AppMode::Normal;
                        self.state.focused_panel = PanelType::Input;
                    }
                    MenuCommand::Analyze => {
                        self.mode = AppMode::Normal;
                        if !self.state.current_expression.is_empty() {
                            self.analyze_current_expression().await?;
                        }
                        self.state.focused_panel = PanelType::Diagnostics;
                    }
                    MenuCommand::CheckSyntax => {
                        self.mode = AppMode::Normal;
                        if !self.state.current_expression.is_empty() {
                            self.analyze_current_expression().await?;
                        }
                        self.state.focused_panel = PanelType::Diagnostics;
                    }
                    MenuCommand::Documentation => {
                        self.mode = AppMode::Help;
                        self.state.focused_panel = PanelType::Help;
                    }
                    MenuCommand::LoadFile => {
                        // TODO: Implement file picker or input dialog
                        self.mode = AppMode::Normal;
                    }
                    MenuCommand::Exit => {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            // Number keys for direct selection
            (KeyCode::Char('1'), KeyModifiers::NONE) => {
                self.state.selected_menu_command = MenuCommand::Evaluate;
                Ok(false)
            }
            (KeyCode::Char('2'), KeyModifiers::NONE) => {
                self.state.selected_menu_command = MenuCommand::Analyze;
                Ok(false)
            }
            (KeyCode::Char('3'), KeyModifiers::NONE) => {
                self.state.selected_menu_command = MenuCommand::CheckSyntax;
                Ok(false)
            }
            (KeyCode::Char('4'), KeyModifiers::NONE) => {
                self.state.selected_menu_command = MenuCommand::Documentation;
                Ok(false)
            }
            (KeyCode::Char('5'), KeyModifiers::NONE) => {
                self.state.selected_menu_command = MenuCommand::LoadFile;
                Ok(false)
            }
            (KeyCode::Char('6'), KeyModifiers::NONE) => {
                self.state.selected_menu_command = MenuCommand::Exit;
                Ok(false)
            }
            // Esc to close menu
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.mode = AppMode::Normal;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Update auto-completion suggestions
    async fn update_completions(&mut self) -> Result<()> {
        // This would integrate with the existing completion system
        // For now, just clear completions
        self.state.completions.clear();
        Ok(())
    }

    /// Render a frame
    fn render_frame(&mut self) -> Result<()> {
        let render_start = Instant::now();

        // Calculate layout inside the draw closure

        // Pre-calculate what we need before the closure
        let layout = &mut self.layout;
        let components = &mut self.components;
        let state = &self.state;
        let theme = &self.theme;
        let mode = self.mode;

        self.terminal.draw(|frame| {
            // Calculate layout inside the closure using the frame area
            let chunks = layout.calculate_layout(frame.area());

            // Render all components in their assigned areas
            components.render_all(frame, &chunks, state, theme);

            // Render status line manually here to avoid borrowing issues
            Self::render_status_line_static(frame, chunks.status_line, mode, state, theme);

            // Render menu overlay if in menu mode
            if mode == AppMode::Menu {
                Self::render_menu_overlay(frame, state, theme);
            }
        })?;

        self.state.performance.last_render_time = Some(render_start.elapsed());
        Ok(())
    }

    /// Render the status line (static version for use in closures)
    fn render_status_line_static(
        frame: &mut Frame,
        area: Rect,
        mode: AppMode,
        state: &AppState,
        theme: &TuiTheme,
    ) {
        use ratatui::style::{Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Paragraph};

        let mode_text = match mode {
            AppMode::Normal => "NORMAL",
            AppMode::Menu => "MENU",
            AppMode::Help => "HELP",
            AppMode::Settings => "SETTINGS",
            AppMode::History => "HISTORY",
            AppMode::Export => "EXPORT",
            AppMode::DiagnosticsDetail => "DIAGNOSTICS",
        };

        let panel_text = format!("{:?}", state.focused_panel);

        let key_hints = match state.focused_panel {
            PanelType::Input => "Enter:Execute | Ctrl+Space:Complete | Tab:Next Panel",
            PanelType::Output => "↑↓:Scroll | Tab:Next Panel",
            PanelType::Diagnostics => "↑↓:Select | Enter:Details | Tab:Next Panel",
            PanelType::Variables => "↑↓:Select | Enter:Edit | Del:Remove | Tab:Next Panel",
            PanelType::History => "↑↓:Select | Enter:Load | Del:Remove | Tab:Next Panel",
            PanelType::Help => "↑↓:Scroll | Esc:Close | Tab:Next Panel",
        };

        let status_line = Line::from(vec![
            Span::styled(
                format!(" {} ", mode_text),
                Style::default()
                    .bg(theme.colors.focused_border)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled(
                format!("Panel: {} ", panel_text),
                Style::default().fg(theme.colors.normal_text),
            ),
            Span::raw(" | "),
            Span::styled(
                key_hints.to_string(),
                Style::default().fg(theme.colors.normal_text),
            ),
        ]);

        let paragraph = Paragraph::new(status_line).block(Block::default().borders(Borders::TOP));

        frame.render_widget(paragraph, area);
    }

    /// Render the status line (instance method for compatibility)
    fn render_status_line(&self, frame: &mut Frame, area: Rect) {
        Self::render_status_line_static(frame, area, self.mode, &self.state, &self.theme);
    }

    /// Render the menu overlay
    fn render_menu_overlay(frame: &mut Frame, state: &AppState, theme: &TuiTheme) {
        use ratatui::layout::Alignment;
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

        // Create a centered popup area
        let area = frame.area();
        let popup_area = centered_rect(60, 40, area);

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        // Create menu items
        let menu_items = [
            ("1", "Evaluate Expression", MenuCommand::Evaluate),
            ("2", "Analyze Expression", MenuCommand::Analyze),
            ("3", "Check Syntax", MenuCommand::CheckSyntax),
            ("4", "Documentation", MenuCommand::Documentation),
            ("5", "Load File", MenuCommand::LoadFile),
            ("6", "Exit", MenuCommand::Exit),
        ];

        let list_items: Vec<ListItem> = menu_items
            .iter()
            .map(|(key, label, command)| {
                let style = if *command == state.selected_menu_command {
                    Style::default()
                        .bg(theme.colors.focused_border)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.colors.normal_text)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", key),
                        Style::default()
                            .fg(theme.colors.highlight_text)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*label, style),
                ]))
            })
            .collect();

        let menu_list = List::new(list_items).block(
            Block::default()
                .title(" FHIRPath Commands ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.colors.focused_border)),
        );

        frame.render_widget(menu_list, popup_area);

        // Add instructions at the bottom
        let instructions =
            Paragraph::new("↑↓: Navigate | Enter: Select | Esc: Close | 1-6: Quick select")
                .style(Style::default().fg(theme.colors.normal_text))
                .alignment(Alignment::Center);

        let instruction_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height.saturating_sub(1),
            width: popup_area.width,
            height: 1,
        };

        frame.render_widget(instructions, instruction_area);
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl TuiApp {
    /// Handle terminal resize
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.layout.handle_resize(width, height);
        Ok(())
    }

    /// Update all components
    async fn update_components(&mut self) -> Result<()> {
        self.components.update_all(&mut self.state).await
    }

    /// Check if we should render this frame
    fn should_render(&self) -> bool {
        // Render at most 60 FPS
        self.last_render.elapsed() > Duration::from_millis(16)
    }

    /// Load a FHIR resource from JSON
    pub fn load_resource_from_json(&mut self, json: JsonValue) -> Result<()> {
        let resource = octofhir_fhirpath::FhirPathValue::resource(json);
        self.state.current_resource = Some(resource);
        Ok(())
    }

    /// Set a variable value
    pub async fn set_variable(&mut self, name: &str, value: &str) -> Result<()> {
        // Try to parse as JSON first, otherwise treat as string
        let parsed_value = match serde_json::from_str::<JsonValue>(value) {
            Ok(json_value) => octofhir_fhirpath::FhirPathValue::resource(json_value),
            Err(_) => octofhir_fhirpath::FhirPathValue::String(value.to_string().into()),
        };
        self.state.variables.insert(name.to_string(), parsed_value);
        Ok(())
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        use crossterm::event::{MouseButton, MouseEventKind};

        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Determine which panel was clicked based on coordinates
                let click_result = self
                    .layout
                    .get_panel_at_position(mouse_event.column, mouse_event.row);
                if let Some(panel_type) = click_result {
                    // Switch focus to clicked panel
                    self.state.focused_panel = panel_type;
                }
            }
            MouseEventKind::ScrollUp => {
                // Scroll up in the currently focused panel
                let _component_result = self.components.handle_scroll_up(&mut self.state)?;
                // Note: Scroll results are handled immediately, no async processing needed
            }
            MouseEventKind::ScrollDown => {
                // Scroll down in the currently focused panel
                let _component_result = self.components.handle_scroll_down(&mut self.state)?;
                // Note: Scroll results are handled immediately, no async processing needed
            }
            _ => {
                // Ignore other mouse events like drag, move, etc.
            }
        }

        Ok(())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            focused_panel: PanelType::Input,
            current_resource: None,
            variables: HashMap::new(),
            current_expression: String::new(),
            last_result: None,
            diagnostics: Vec::new(),
            evaluation_history: Vec::new(),
            completions: Vec::new(),
            live_validation: true,
            performance: PerformanceMetrics::default(),
            selected_menu_command: MenuCommand::Evaluate,
        }
    }
}
