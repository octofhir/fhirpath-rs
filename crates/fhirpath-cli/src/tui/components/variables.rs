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

//! Variables Panel Component

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

/// Variables panel for displaying and managing variables
pub struct VariablesPanel {
    list_state: ListState,
    scroll_state: ScrollState,
}

impl VariablesPanel {
    /// Create a new variables panel
    pub async fn new(_config: &TuiConfig) -> Result<Self> {
        Ok(Self {
            list_state: ListState::default(),
            scroll_state: ScrollState::new(),
        })
    }
}

impl TuiComponent for VariablesPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect, state: &AppState, theme: &TuiTheme) {
        let is_focused = state.focused_panel == PanelType::Variables;
        let block = utils::create_panel_block("Variables", PanelType::Variables, is_focused, theme);

        let items: Vec<ListItem> = if state.variables.is_empty() {
            vec![ListItem::new("No variables defined")]
        } else {
            state
                .variables
                .iter()
                .map(|(name, value)| {
                    let item_text = format!("%{} = {:?}", name, value);
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

        let var_names: Vec<_> = state.variables.keys().cloned().collect();

        match key.code {
            KeyCode::Up => {
                self.scroll_state.select_previous(var_names.len());
                self.list_state.select(self.scroll_state.selected_index);
                ComponentResult::Handled
            }
            KeyCode::Down => {
                self.scroll_state.select_next(var_names.len());
                self.list_state.select(self.scroll_state.selected_index);
                ComponentResult::Handled
            }
            KeyCode::Delete => {
                if let Some(selected) = self.scroll_state.selected_index {
                    if let Some(var_name) = var_names.get(selected) {
                        return ComponentResult::UnsetVariable(var_name.clone());
                    }
                }
                ComponentResult::Handled
            }
            KeyCode::Enter => {
                if let Some(selected) = self.scroll_state.selected_index {
                    if let Some(var_name) = var_names.get(selected) {
                        return ComponentResult::EditVariable(var_name.clone());
                    }
                }
                ComponentResult::Handled
            }
            _ => ComponentResult::NotHandled,
        }
    }

    fn update(&mut self, _state: &mut AppState) -> ComponentResult {
        ComponentResult::Handled
    }

    fn size_constraints(&self) -> SizeConstraints {
        SizeConstraints {
            min_width: Some(20),
            preferred_width: Some(30),
            max_width: None,
            min_height: Some(5),
            preferred_height: Some(10),
            max_height: None,
        }
    }
}
