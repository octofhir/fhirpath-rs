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

//! Help Panel Component

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{Frame, text::Text};
use std::sync::Arc;

use super::{ComponentResult, ScrollState, TuiComponent, utils};
use crate::tui::app::AppState;
use crate::tui::config::TuiConfig;
use crate::tui::layout::PanelType;
use crate::tui::themes::TuiTheme;

use octofhir_fhirpath::registry::FunctionRegistry;

/// Help panel for displaying documentation and keybindings
pub struct HelpPanel {
    scroll_state: ScrollState,
    _registry: Option<Arc<FunctionRegistry>>,
}

impl HelpPanel {
    /// Create a new help panel
    pub async fn new(_config: &TuiConfig, _registry: Arc<FunctionRegistry>) -> Result<Self> {
        Ok(Self {
            scroll_state: ScrollState::new(),
            _registry: None,
        })
    }
}

impl TuiComponent for HelpPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        let is_focused = state.focused_panel == PanelType::Help;
        let block = utils::create_panel_block("Help", PanelType::Help, is_focused, theme);

        let help_text = r#"FHIRPath TUI Help

KEYBOARD SHORTCUTS:
  F1           - Toggle this help panel
  Esc          - Exit application
  Tab          - Next panel
  Shift+Tab    - Previous panel

INPUT PANEL:
  Enter        - Execute expression
  Ctrl+Space   - Show completions
  Ctrl+C       - Clear input

OUTPUT PANEL:
  ↑/↓          - Scroll up/down
  PgUp/PgDn    - Page up/down

DIAGNOSTICS PANEL:
  ↑/↓          - Navigate diagnostics
  Enter        - Show details

VARIABLES PANEL:
  ↑/↓          - Navigate variables
  Enter        - Edit variable
  Delete       - Remove variable

HISTORY PANEL:
  ↑/↓          - Navigate history
  Enter        - Load expression
  Delete       - Clear history

EXAMPLES:
  Patient.name.given.first()
  Patient.birthDate > @1990-01-01
  Bundle.entry.resource.ofType(Patient)
"#;

        let paragraph = Paragraph::new(Text::from(help_text))
            .block(block)
            .scroll((self.scroll_state.offset as u16, 0));

        frame.render_widget(paragraph, area);

        // Render scrollbar if focused
        if is_focused {
            let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);
            let mut scrollbar_state = ScrollbarState::default();
            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent, _state: &mut AppState) -> ComponentResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                self.scroll_state.scroll_up();
                ComponentResult::Handled
            }
            KeyCode::Down => {
                self.scroll_state.scroll_down(30, 10); // Approximate content size
                ComponentResult::Handled
            }
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_state.scroll_up();
                }
                ComponentResult::Handled
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_state.scroll_down(30, 10);
                }
                ComponentResult::Handled
            }
            _ => ComponentResult::NotHandled,
        }
    }

    fn update(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }
}
