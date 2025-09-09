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

//! Diagnostics Panel Component

use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::{Frame, text::Text};

use super::{ComponentResult, ScrollState, SizeConstraints, TuiComponent, utils};
use crate::tui::app::AppState;
use crate::tui::config::TuiConfig;
use crate::tui::layout::PanelType;
use crate::tui::themes::TuiTheme;

/// Diagnostics panel for displaying errors and warnings
pub struct DiagnosticsPanel {
    list_state: ListState,
    scroll_state: ScrollState,
}

impl DiagnosticsPanel {
    /// Create a new diagnostics panel
    pub async fn new(_config: &TuiConfig) -> Result<Self> {
        Ok(Self {
            list_state: ListState::default(),
            scroll_state: ScrollState::new(),
        })
    }
}

impl TuiComponent for DiagnosticsPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        let is_focused = state.focused_panel == PanelType::Diagnostics;
        let block =
            utils::create_panel_block("Diagnostics", PanelType::Diagnostics, is_focused, theme);

        let items: Vec<ListItem> = if state.diagnostics.is_empty() {
            vec![ListItem::new("No diagnostics")]
        } else {
            state
                .diagnostics
                .iter()
                .map(|diagnostic| {
                    let severity_str = format!("{:?}", diagnostic.severity).to_uppercase();
                    let item_text = format!("{}: {}", severity_str, diagnostic.message);
                    ListItem::new(Text::from(item_text))
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(theme.styles.selected_item);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: &mut AppState) -> ComponentResult {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                self.scroll_state.select_previous(state.diagnostics.len());
                self.list_state.select(self.scroll_state.selected_index);
                ComponentResult::Handled
            }
            KeyCode::Down => {
                self.scroll_state.select_next(state.diagnostics.len());
                self.list_state.select(self.scroll_state.selected_index);
                ComponentResult::Handled
            }
            KeyCode::Enter => ComponentResult::ToggleDiagnosticDetails,
            _ => ComponentResult::NotHandled,
        }
    }

    fn update(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }

    fn size_constraints(&self) -> SizeConstraints {
        SizeConstraints {
            min_height: Some(3),
            preferred_height: Some(8),
            ..Default::default()
        }
    }
}
