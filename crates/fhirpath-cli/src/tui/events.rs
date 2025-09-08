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

//! Event Handling System
//! 
//! This module provides a comprehensive event handling system with customizable
//! key bindings, context-aware actions, and efficient event routing.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

/// Wrapper for KeyEvent that implements Hash and Eq for use in HashMap
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyEventWrapper {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<KeyEvent> for KeyEventWrapper {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

impl From<KeyEventWrapper> for KeyEvent {
    fn from(wrapper: KeyEventWrapper) -> Self {
        KeyEvent::new(wrapper.code, wrapper.modifiers)
    }
}

use super::app::{AppMode, AppState};
use super::layout::PanelType;

/// Main event handler that routes events to appropriate handlers
pub struct EventHandler {
    key_bindings: KeyBindings,
}

impl EventHandler {
    /// Create a new event handler with default key bindings
    pub fn new(key_bindings: KeyBindings) -> Self {
        Self { key_bindings }
    }
    
    /// Handle a key event and return the appropriate action
    pub fn handle_key_event(&self, key: KeyEvent, state: &AppState) -> TuiAction {
        // First check for global key bindings
        if let Some(action) = self.key_bindings.get_global_action(&key) {
            return action;
        }
        
        // Then check for panel-specific bindings
        if let Some(action) = self.key_bindings.get_panel_action(&key, state.focused_panel) {
            return action;
        }
        
        // Finally check for mode-specific bindings
        self.key_bindings.get_mode_action(&key, AppMode::Normal)
            .unwrap_or(TuiAction::NoAction)
    }
    
    /// Update key bindings configuration
    pub fn set_key_bindings(&mut self, bindings: KeyBindings) {
        self.key_bindings = bindings;
    }
}

/// TUI actions that can be triggered by events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiAction {
    /// No action (event not handled)
    NoAction,
    
    /// Exit the application
    Quit,
    
    // Navigation actions
    /// Focus a specific panel
    FocusPanel(PanelType),
    /// Move focus to next panel
    NextPanel,
    /// Move focus to previous panel
    PreviousPanel,
    
    // Input actions
    /// Execute the current expression
    ExecuteExpression,
    /// Update the current expression
    UpdateExpression(String),
    /// Clear the current input
    ClearInput,
    /// Show auto-completion suggestions
    ShowCompletions,
    /// Insert text at cursor
    InsertText(String),
    /// Move cursor left
    CursorLeft,
    /// Move cursor right
    CursorRight,
    /// Move cursor to start of line
    CursorHome,
    /// Move cursor to end of line
    CursorEnd,
    /// Delete character at cursor
    DeleteChar,
    /// Delete character before cursor
    Backspace,
    /// Delete word at cursor
    DeleteWord,
    /// Delete to end of line
    DeleteToEnd,
    
    // History actions
    /// Navigate to previous history item
    HistoryPrevious,
    /// Navigate to next history item
    HistoryNext,
    /// Load expression from history
    LoadFromHistory(usize),
    /// Clear history
    ClearHistory,
    
    // Selection and scrolling
    /// Scroll up in current panel
    ScrollUp,
    /// Scroll down in current panel
    ScrollDown,
    /// Page up in current panel
    PageUp,
    /// Page down in current panel
    PageDown,
    /// Select previous item
    SelectPrevious,
    /// Select next item
    SelectNext,
    /// Activate/enter selected item
    ActivateSelected,
    
    // Variable management
    /// Set a variable
    SetVariable(String, String),
    /// Unset a variable
    UnsetVariable(String),
    /// Edit selected variable
    EditVariable(String),
    
    // Resource management
    /// Load resource from file
    LoadResource(String),
    /// Clear current resource
    ClearResource,
    
    // Mode toggles
    /// Toggle application mode
    ToggleMode(AppMode),
    /// Toggle help panel
    ToggleHelp,
    /// Toggle settings panel
    ToggleSettings,
    /// Toggle diagnostic details
    ToggleDiagnosticDetails,
    
    // Panel-specific actions
    /// Copy result to clipboard
    CopyResult,
    /// Export results to file
    ExportResults(String),
    /// Change output format
    ChangeOutputFormat(OutputFormat),
    
    // Configuration actions
    /// Save current configuration
    SaveConfiguration,
    /// Reset to default configuration
    ResetConfiguration,
    /// Toggle feature flag
    ToggleFeature(String),
}

/// Output format options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Raw,
    Pretty,
    Json,
    Table,
}

/// Key binding system with support for different contexts
#[derive(Debug, Clone)]
pub struct KeyBindings {
    /// Global key bindings (active in all contexts)
    global: HashMap<KeyEventWrapper, TuiAction>,
    /// Panel-specific key bindings
    panel_bindings: HashMap<PanelType, HashMap<KeyEventWrapper, TuiAction>>,
    /// Mode-specific key bindings
    mode_bindings: HashMap<AppMode, HashMap<KeyEventWrapper, TuiAction>>,
}

impl KeyBindings {
    /// Create a new key bindings instance
    pub fn new() -> Self {
        Self {
            global: HashMap::new(),
            panel_bindings: HashMap::new(),
            mode_bindings: HashMap::new(),
        }
    }
    
    /// Get global action for key event
    pub fn get_global_action(&self, key: &KeyEvent) -> Option<TuiAction> {
        let wrapper = KeyEventWrapper::from(key.clone());
        self.global.get(&wrapper).cloned()
    }
    
    /// Get panel-specific action for key event
    pub fn get_panel_action(&self, key: &KeyEvent, panel: PanelType) -> Option<TuiAction> {
        let wrapper = KeyEventWrapper::from(key.clone());
        self.panel_bindings
            .get(&panel)
            .and_then(|bindings| bindings.get(&wrapper))
            .cloned()
    }
    
    /// Get mode-specific action for key event
    pub fn get_mode_action(&self, key: &KeyEvent, mode: AppMode) -> Option<TuiAction> {
        let wrapper = KeyEventWrapper::from(key.clone());
        self.mode_bindings
            .get(&mode)
            .and_then(|bindings| bindings.get(&wrapper))
            .cloned()
    }
    
    /// Add a global key binding
    pub fn bind_global(&mut self, key: KeyEvent, action: TuiAction) {
        let wrapper = KeyEventWrapper::from(key);
        self.global.insert(wrapper, action);
    }
    
    /// Add a panel-specific key binding
    pub fn bind_panel(&mut self, panel: PanelType, key: KeyEvent, action: TuiAction) {
        let wrapper = KeyEventWrapper::from(key);
        self.panel_bindings
            .entry(panel)
            .or_insert_with(HashMap::new)
            .insert(wrapper, action);
    }
    
    /// Add a mode-specific key binding
    pub fn bind_mode(&mut self, mode: AppMode, key: KeyEvent, action: TuiAction) {
        let wrapper = KeyEventWrapper::from(key);
        self.mode_bindings
            .entry(mode)
            .or_insert_with(HashMap::new)
            .insert(wrapper, action);
    }
    
    /// Load key bindings from configuration
    pub fn from_config(config: &HashMap<String, String>) -> anyhow::Result<Self> {
        let mut bindings = Self::default();
        
        for (key_str, action_str) in config {
            let key = parse_key_string(key_str)?;
            let action = parse_action_string(action_str)?;
            
            // Determine context from action string prefix
            if action_str.starts_with("global:") {
                bindings.bind_global(key, action);
            } else if let Some(panel_str) = action_str.strip_prefix("panel:") {
                if let Some((panel_name, _)) = panel_str.split_once(':') {
                    if let Ok(panel) = panel_name.parse::<PanelType>() {
                        bindings.bind_panel(panel, key, action);
                    }
                }
            }
        }
        
        Ok(bindings)
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut bindings = Self::new();
        
        // Global key bindings
        bindings.bind_global(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            TuiAction::Quit,
        );
        bindings.bind_global(
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            TuiAction::Quit,
        );
        bindings.bind_global(
            KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
            TuiAction::NextPanel,
        );
        bindings.bind_global(
            KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
            TuiAction::PreviousPanel,
        );
        bindings.bind_global(
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
            TuiAction::ToggleHelp,
        );
        bindings.bind_global(
            KeyEvent::new(KeyCode::F(2), KeyModifiers::NONE),
            TuiAction::ToggleSettings,
        );
        
        // Input panel bindings
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            TuiAction::ExecuteExpression,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::CONTROL),
            TuiAction::ShowCompletions,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            TuiAction::ClearInput,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Up, KeyModifiers::ALT),
            TuiAction::HistoryPrevious,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Down, KeyModifiers::ALT),
            TuiAction::HistoryNext,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            TuiAction::CursorLeft,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            TuiAction::CursorRight,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Home, KeyModifiers::NONE),
            TuiAction::CursorHome,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::End, KeyModifiers::NONE),
            TuiAction::CursorEnd,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
            TuiAction::DeleteChar,
        );
        bindings.bind_panel(
            PanelType::Input,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            TuiAction::Backspace,
        );
        
        // Output panel bindings
        bindings.bind_panel(
            PanelType::Output,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            TuiAction::ScrollUp,
        );
        bindings.bind_panel(
            PanelType::Output,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            TuiAction::ScrollDown,
        );
        bindings.bind_panel(
            PanelType::Output,
            KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            TuiAction::PageUp,
        );
        bindings.bind_panel(
            PanelType::Output,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            TuiAction::PageDown,
        );
        bindings.bind_panel(
            PanelType::Output,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            TuiAction::CopyResult,
        );
        
        // Diagnostics panel bindings
        bindings.bind_panel(
            PanelType::Diagnostics,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            TuiAction::SelectPrevious,
        );
        bindings.bind_panel(
            PanelType::Diagnostics,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            TuiAction::SelectNext,
        );
        bindings.bind_panel(
            PanelType::Diagnostics,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            TuiAction::ToggleDiagnosticDetails,
        );
        
        // Variables panel bindings
        bindings.bind_panel(
            PanelType::Variables,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            TuiAction::SelectPrevious,
        );
        bindings.bind_panel(
            PanelType::Variables,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            TuiAction::SelectNext,
        );
        bindings.bind_panel(
            PanelType::Variables,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            TuiAction::ActivateSelected,
        );
        bindings.bind_panel(
            PanelType::Variables,
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
            TuiAction::UnsetVariable("".to_string()), // Will be filled by handler
        );
        
        // History panel bindings
        bindings.bind_panel(
            PanelType::History,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            TuiAction::SelectPrevious,
        );
        bindings.bind_panel(
            PanelType::History,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            TuiAction::SelectNext,
        );
        bindings.bind_panel(
            PanelType::History,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            TuiAction::ActivateSelected,
        );
        bindings.bind_panel(
            PanelType::History,
            KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE),
            TuiAction::ClearHistory,
        );
        
        // Help panel bindings
        bindings.bind_panel(
            PanelType::Help,
            KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
            TuiAction::ScrollUp,
        );
        bindings.bind_panel(
            PanelType::Help,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
            TuiAction::ScrollDown,
        );
        bindings.bind_panel(
            PanelType::Help,
            KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE),
            TuiAction::PageUp,
        );
        bindings.bind_panel(
            PanelType::Help,
            KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE),
            TuiAction::PageDown,
        );
        
        bindings
    }
}

/// Parse a key string like "Ctrl+C" into a KeyEvent
fn parse_key_string(key_str: &str) -> anyhow::Result<KeyEvent> {
    let mut modifiers = KeyModifiers::NONE;
    let mut key_code = KeyCode::Char(' ');
    
    let parts: Vec<&str> = key_str.split('+').collect();
    
    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            _ => anyhow::bail!("Unknown modifier: {}", part),
        }
    }
    
    let key_part = parts.last().unwrap();
    key_code = match key_part.to_lowercase().as_str() {
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "space" => KeyCode::Char(' '),
        "esc" | "escape" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        key if key.len() == 1 => KeyCode::Char(key.chars().next().unwrap()),
        _ => anyhow::bail!("Unknown key: {}", key_part),
    };
    
    Ok(KeyEvent::new(key_code, modifiers))
}

/// Parse an action string into a TuiAction
fn parse_action_string(action_str: &str) -> anyhow::Result<TuiAction> {
    // Strip context prefixes
    let action_str = action_str
        .strip_prefix("global:")
        .or_else(|| action_str.strip_prefix("panel:"))
        .map(|s| s.split_once(':').map(|(_, action)| action).unwrap_or(s))
        .unwrap_or(action_str);
    
    match action_str {
        "quit" => Ok(TuiAction::Quit),
        "next_panel" => Ok(TuiAction::NextPanel),
        "previous_panel" => Ok(TuiAction::PreviousPanel),
        "execute_expression" => Ok(TuiAction::ExecuteExpression),
        "clear_input" => Ok(TuiAction::ClearInput),
        "show_completions" => Ok(TuiAction::ShowCompletions),
        "history_previous" => Ok(TuiAction::HistoryPrevious),
        "history_next" => Ok(TuiAction::HistoryNext),
        "cursor_left" => Ok(TuiAction::CursorLeft),
        "cursor_right" => Ok(TuiAction::CursorRight),
        "cursor_home" => Ok(TuiAction::CursorHome),
        "cursor_end" => Ok(TuiAction::CursorEnd),
        "delete_char" => Ok(TuiAction::DeleteChar),
        "backspace" => Ok(TuiAction::Backspace),
        "scroll_up" => Ok(TuiAction::ScrollUp),
        "scroll_down" => Ok(TuiAction::ScrollDown),
        "page_up" => Ok(TuiAction::PageUp),
        "page_down" => Ok(TuiAction::PageDown),
        "select_previous" => Ok(TuiAction::SelectPrevious),
        "select_next" => Ok(TuiAction::SelectNext),
        "activate_selected" => Ok(TuiAction::ActivateSelected),
        "toggle_help" => Ok(TuiAction::ToggleHelp),
        "toggle_settings" => Ok(TuiAction::ToggleSettings),
        "copy_result" => Ok(TuiAction::CopyResult),
        _ => anyhow::bail!("Unknown action: {}", action_str),
    }
}

impl std::str::FromStr for PanelType {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "input" => Ok(PanelType::Input),
            "output" => Ok(PanelType::Output),
            "diagnostics" => Ok(PanelType::Diagnostics),
            "variables" => Ok(PanelType::Variables),
            "history" => Ok(PanelType::History),
            "help" => Ok(PanelType::Help),
            _ => anyhow::bail!("Unknown panel type: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_key_string() {
        assert_eq!(
            parse_key_string("Ctrl+C").unwrap(),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
        );
        
        assert_eq!(
            parse_key_string("Alt+Enter").unwrap(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)
        );
        
        assert_eq!(
            parse_key_string("F1").unwrap(),
            KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE)
        );
    }
    
    #[test]
    fn test_parse_action_string() {
        assert_eq!(
            parse_action_string("quit").unwrap(),
            TuiAction::Quit
        );
        
        assert_eq!(
            parse_action_string("global:next_panel").unwrap(),
            TuiAction::NextPanel
        );
        
        assert_eq!(
            parse_action_string("panel:input:execute_expression").unwrap(),
            TuiAction::ExecuteExpression
        );
    }
    
    #[test]
    fn test_key_bindings() {
        let bindings = KeyBindings::default();
        
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(bindings.get_global_action(&quit_key), Some(TuiAction::Quit));
        
        let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(
            bindings.get_panel_action(&enter_key, PanelType::Input),
            Some(TuiAction::ExecuteExpression)
        );
    }
}