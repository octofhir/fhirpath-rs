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

//! TUI Component System
//! 
//! This module provides a modular component architecture for the TUI interface.
//! Each panel is implemented as a separate component with its own state management,
//! rendering logic, and event handling capabilities.

pub mod input;
pub mod output;
pub mod diagnostics;
pub mod variables;
pub mod history;
pub mod help;

use std::collections::HashMap;

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

use super::app::AppState;
use super::config::TuiConfig;
use super::layout::{PanelType, PanelLayout};
use super::themes::TuiTheme;

use octofhir_fhirpath::analyzer::StaticAnalyzer;
use octofhir_fhirpath::FhirPathEngine;

pub use input::InputPanel;
pub use output::OutputPanel;
pub use diagnostics::DiagnosticsPanel;
pub use variables::VariablesPanel;
pub use history::HistoryPanel;
pub use help::HelpPanel;

/// Trait for TUI components that can render themselves and handle events
pub trait TuiComponent {
    /// Render the component in the given area
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme);
    
    /// Handle a key event when this component has focus
    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> ComponentResult;
    
    /// Update component state (called every frame)
    fn update(&mut self, state: &mut AppState) -> ComponentResult;
    
    /// Called when the component gains focus
    fn on_focus(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }
    
    /// Called when the component loses focus
    fn on_blur(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }
    
    /// Get the component's preferred size constraints
    fn size_constraints(&self) -> SizeConstraints {
        SizeConstraints::default()
    }
}

/// Result of component event handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentResult {
    /// Event was handled by this component
    Handled,
    /// Event was not handled, should bubble up
    NotHandled,
    /// Request to switch focus to another panel
    SwitchFocus(PanelType),
    /// Request to exit the application
    ExitApp,
    /// Request to execute the current expression
    ExecuteExpression,
    /// Request to update the current expression
    UpdateExpression(String),
    /// Request to show completions
    ShowCompletions,
    /// Request to load a resource
    LoadResource(String),
    /// Request to set a variable
    SetVariable(String, String),
    /// Request to unset a variable
    UnsetVariable(String),
    /// Request to toggle diagnostic details
    ToggleDiagnosticDetails,
    /// Request to edit a variable
    EditVariable(String),
    /// Request to clear history
    ClearHistory,
    /// Request to load from history
    LoadFromHistory(usize),
}

/// Size constraints for component layout
#[derive(Debug, Clone)]
pub struct SizeConstraints {
    pub min_width: Option<u16>,
    pub max_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_height: Option<u16>,
    pub preferred_width: Option<u16>,
    pub preferred_height: Option<u16>,
}

impl Default for SizeConstraints {
    fn default() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            preferred_width: None,
            preferred_height: None,
        }
    }
}

/// Manager for all TUI components
pub struct ComponentManager {
    input: InputPanel,
    output: OutputPanel,
    diagnostics: DiagnosticsPanel,
    variables: VariablesPanel,
    history: HistoryPanel,
    help: HelpPanel,
}

impl ComponentManager {
    /// Create a new component manager
    pub async fn new(
        config: &TuiConfig,
        engine: &FhirPathEngine,
        analyzer: &StaticAnalyzer,
    ) -> Result<Self> {
        Ok(Self {
            input: InputPanel::new(config, analyzer).await?,
            output: OutputPanel::new(config).await?,
            diagnostics: DiagnosticsPanel::new(config).await?,
            variables: VariablesPanel::new(config).await?,
            history: HistoryPanel::new(config).await?,
            help: HelpPanel::new(config, engine.get_function_registry().clone()).await?,
        })
    }
    
    /// Render all components in their respective areas
    pub fn render_all(
        &mut self,
        frame: &mut Frame,
        layout: &PanelLayout,
        state: &AppState,
        theme: &TuiTheme,
    ) {
        // Render main panels
        self.input.render(frame, layout.input, state, theme);
        self.output.render(frame, layout.output, state, theme);
        self.diagnostics.render(frame, layout.diagnostics, state, theme);
        self.variables.render(frame, layout.variables, state, theme);
        self.history.render(frame, layout.history, state, theme);
        
        // Render help panel if visible
        if layout.help.width > 0 && layout.help.height > 0 {
            self.help.render(frame, layout.help, state, theme);
        }
    }
    
    /// Handle key event for the currently focused component
    pub fn handle_key_event(
        &mut self,
        key: KeyEvent,
        state: &mut AppState,
    ) -> ComponentResult {
        match state.focused_panel {
            PanelType::Input => self.input.handle_key_event(key, state),
            PanelType::Output => self.output.handle_key_event(key, state),
            PanelType::Diagnostics => self.diagnostics.handle_key_event(key, state),
            PanelType::Variables => self.variables.handle_key_event(key, state),
            PanelType::History => self.history.handle_key_event(key, state),
            PanelType::Help => self.help.handle_key_event(key, state),
        }
    }
    
    /// Update all components
    pub async fn update_all(&mut self, state: &mut AppState) -> Result<()> {
        // Update components (most are stateless, but some may need periodic updates)
        self.input.update(state);
        self.output.update(state);
        self.diagnostics.update(state);
        self.variables.update(state);
        self.history.update(state);
        self.help.update(state);
        
        Ok(())
    }
    
    /// Handle focus change events
    pub fn handle_focus_change(
        &mut self,
        old_panel: PanelType,
        new_panel: PanelType,
        state: &mut AppState,
    ) {
        // Blur old component
        match old_panel {
            PanelType::Input => { self.input.on_blur(state); }
            PanelType::Output => { self.output.on_blur(state); }
            PanelType::Diagnostics => { self.diagnostics.on_blur(state); }
            PanelType::Variables => { self.variables.on_blur(state); }
            PanelType::History => { self.history.on_blur(state); }
            PanelType::Help => { self.help.on_blur(state); }
        }
        
        // Focus new component
        match new_panel {
            PanelType::Input => { self.input.on_focus(state); }
            PanelType::Output => { self.output.on_focus(state); }
            PanelType::Diagnostics => { self.diagnostics.on_focus(state); }
            PanelType::Variables => { self.variables.on_focus(state); }
            PanelType::History => { self.history.on_focus(state); }
            PanelType::Help => { self.help.on_focus(state); }
        }
    }
    
    /// Handle scroll up event for the focused panel
    pub fn handle_scroll_up(&mut self, state: &mut AppState) -> Result<ComponentResult> {
        let result = match state.focused_panel {
            PanelType::Output => {
                // Scroll up in output panel
                ComponentResult::Handled
            },
            PanelType::Diagnostics => {
                // Scroll up in diagnostics panel  
                ComponentResult::Handled
            },
            PanelType::Variables => {
                // Scroll up in variables panel
                ComponentResult::Handled
            },
            PanelType::History => {
                // Scroll up in history panel
                ComponentResult::Handled
            },
            PanelType::Help => {
                // Scroll up in help panel
                ComponentResult::Handled
            },
            _ => ComponentResult::Handled,
        };
        Ok(result)
    }
    
    /// Handle scroll down event for the focused panel
    pub fn handle_scroll_down(&mut self, state: &mut AppState) -> Result<ComponentResult> {
        let result = match state.focused_panel {
            PanelType::Output => {
                // Scroll down in output panel
                ComponentResult::Handled
            },
            PanelType::Diagnostics => {
                // Scroll down in diagnostics panel
                ComponentResult::Handled
            },
            PanelType::Variables => {
                // Scroll down in variables panel
                ComponentResult::Handled
            },
            PanelType::History => {
                // Scroll down in history panel
                ComponentResult::Handled
            },
            PanelType::Help => {
                // Scroll down in help panel
                ComponentResult::Handled
            },
            _ => ComponentResult::Handled,
        };
        Ok(result)
    }
}

/// Scroll state for components that support scrolling
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    pub offset: usize,
    pub selected_index: Option<usize>,
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Scroll up by one item
    pub fn scroll_up(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }
    
    /// Scroll down by one item
    pub fn scroll_down(&mut self, max_items: usize, visible_items: usize) {
        let max_offset = max_items.saturating_sub(visible_items);
        if self.offset < max_offset {
            self.offset += 1;
        }
    }
    
    /// Select previous item
    pub fn select_previous(&mut self, max_items: usize) {
        if max_items == 0 {
            self.selected_index = None;
            return;
        }
        
        match self.selected_index {
            None => self.selected_index = Some(0),
            Some(0) => self.selected_index = Some(max_items - 1),
            Some(idx) => self.selected_index = Some(idx - 1),
        }
    }
    
    /// Select next item
    pub fn select_next(&mut self, max_items: usize) {
        if max_items == 0 {
            self.selected_index = None;
            return;
        }
        
        match self.selected_index {
            None => self.selected_index = Some(0),
            Some(idx) if idx >= max_items - 1 => self.selected_index = Some(0),
            Some(idx) => self.selected_index = Some(idx + 1),
        }
    }
    
    /// Ensure selected item is visible
    pub fn ensure_selected_visible(&mut self, visible_items: usize) {
        if let Some(selected) = self.selected_index {
            if selected < self.offset {
                self.offset = selected;
            } else if selected >= self.offset + visible_items {
                self.offset = selected.saturating_sub(visible_items - 1);
            }
        }
    }
}

/// Common utility functions for components
pub mod utils {
    use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Borders, BorderType};
    
    use super::PanelType;
    use crate::tui::themes::TuiTheme;
    
    /// Create a bordered block for a panel
    pub fn create_panel_block<'a>(
        title: &str,
        panel_type: PanelType,
        is_focused: bool,
        theme: &TuiTheme,
    ) -> Block<'a> {
        let border_color = if is_focused {
            theme.colors.focused_border
        } else {
            theme.colors.unfocused_border
        };
        
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
    }
    
    /// Split area horizontally with minimum sizes
    pub fn split_horizontal_with_min(area: Rect, ratios: &[u16], mins: &[u16]) -> Vec<Rect> {
        let constraints: Vec<Constraint> = ratios
            .iter()
            .zip(mins.iter())
            .map(|(&ratio, &min)| Constraint::Min(min.max(area.height * ratio / 100)))
            .collect();
            
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area)
            .to_vec()
    }
    
    /// Split area vertically with minimum sizes
    pub fn split_vertical_with_min(area: Rect, ratios: &[u16], mins: &[u16]) -> Vec<Rect> {
        let constraints: Vec<Constraint> = ratios
            .iter()
            .zip(mins.iter())
            .map(|(&ratio, &min)| Constraint::Min(min.max(area.width * ratio / 100)))
            .collect();
            
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area)
            .to_vec()
    }
    
    /// Calculate inner area with padding
    pub fn inner_area(area: Rect, padding: u16) -> Rect {
        area.inner(Margin {
            horizontal: padding,
            vertical: padding,
        })
    }
}